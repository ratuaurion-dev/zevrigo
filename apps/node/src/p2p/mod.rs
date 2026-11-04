//! P2P di daemon (TSK-M28, Spesifikasi P2P v0.1 Bagian 4 sampai 7): menghubungkan [`Network`] MOD-06 dengan
//! [`Node`].
//!
//! Saat terhubung, ujung chain ditukar lewat `status` (MOD-06). Jika chainwork rekan lebih besar, node meminta header
//! dengan locator, lalu badan blok 16 sekali minta. Blok yang menjadi ujung chain aktif baru dan transaksi baru di
//! mempool disiarkan ke rekan lain, termasuk yang datang lewat RPC ([`Relay`]). Rekan yang mengirim header, blok,
//! atau transaksi tidak sah disanksi (putus dan blokir 24 jam, MOD-06). Setiap 10 detik node mencoba lagi sinkronisasi
//! yang belum selesai. Jika indeks header penuh sebelum cabang rekan terbukti lebih berat, node menjalankan presync
//! header dengan rekan itu (TSK-M35 bagian B, [`Presync`]).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::Duration;

use homeostasis_consensus::TxError;
use homeostasis_crypto::block_id;
use homeostasis_mempool::MempoolError;
use homeostasis_network::message::{MAX_BLOCK_IDS, MAX_HEADERS};
use homeostasis_network::{
    Ack, Blocks, GetBlocks, GetHeaders, Headers, Multiaddr, NetEvent, Network, PeerId,
    ProtocolKind, Reply, RequestId, Status,
};
use homeostasis_primitives::{Block, BlockHeight, Decode, Encode, Hash32, Transaction};
use homeostasis_rpc::{
    BlockInfo, BlockTemplate, ChainInfo, GetBlockTemplateParams, Hex32, MempoolInfo, MempoolScan,
    RpcBackend, RpcError, SendRawTransactionResult, SubmitBlockResult, SubmitStatus, UtxoInfo,
};
use homeostasis_storage::ChainStore;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::MissedTickBehavior;

use crate::chain::{
    HeadersOutcome, MAX_LOCATOR_IDS, Node, Presync, PresyncOutcome, TxOutcome, transaction_result,
};

/// Selang percobaan ulang sinkronisasi (10 detik).
pub const SYNC_INTERVAL: Duration = Duration::from_secs(10);

/// ID blok yang diingat per rekan (diketahui sudah dimiliki rekan itu) sebelum daftarnya dikosongkan.
const MAX_KNOWN_PER_PEER: usize = 4_096;

/// Blok atau transaksi baru dari RPC yang perlu disiarkan ke rekan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Announcement {
    /// Blok yang menjadi ujung chain aktif baru.
    Block(Box<Block>),
    /// Transaksi yang baru masuk mempool.
    Tx(Transaction),
}

/// Backend RPC yang meneruskan semua metode ke [`Node`], lalu mengumumkan blok yang menjadi ujung chain aktif baru dan
/// transaksi yang baru masuk mempool ke layanan P2P.
pub struct Relay<S> {
    node: Arc<Node<S>>,
    sender: UnboundedSender<Announcement>,
}

impl<S> Relay<S> {
    /// Membungkus `node`; pengumuman dikirim ke `sender`.
    #[must_use]
    pub const fn new(node: Arc<Node<S>>, sender: UnboundedSender<Announcement>) -> Self {
        Self { node, sender }
    }
}

impl<S: ChainStore + Send + 'static> RpcBackend for Relay<S> {
    fn chain_info(&self) -> Result<ChainInfo, RpcError> {
        self.node.chain_info()
    }

    fn block_hash(&self, height: u64) -> Result<Hex32, RpcError> {
        self.node.block_hash(height)
    }

    fn block(&self, block_id: Hex32) -> Result<BlockInfo, RpcError> {
        self.node.block(block_id)
    }

    fn block_template(&self, params: &GetBlockTemplateParams) -> Result<BlockTemplate, RpcError> {
        self.node.block_template(params)
    }

    fn submit_block(&self, bytes: &[u8]) -> Result<SubmitBlockResult, RpcError> {
        let receipt = self.node.receive_block(bytes)?;
        if receipt.new_tip
            && let Ok(block) = Block::from_bytes(bytes)
        {
            // Saluran tertutup berarti P2P sudah berhenti; blok tetap diterima node.
            let _ = self.sender.send(Announcement::Block(Box::new(block)));
        }
        Ok(receipt.result)
    }

    fn send_raw_transaction(&self, bytes: &[u8]) -> Result<SendRawTransactionResult, RpcError> {
        let tx = Transaction::from_bytes(bytes).map_err(|_| RpcError::InvalidEncoding)?;
        let (id, outcome) = self.node.receive_transaction(tx.clone())?;
        if outcome == TxOutcome::Added {
            let _ = self.sender.send(Announcement::Tx(tx));
        }
        transaction_result(id, &outcome)
    }

    fn mempool(&self) -> Result<MempoolInfo, RpcError> {
        self.node.mempool()
    }

    fn scan_utxos(&self, payloads: &[Hex32]) -> Result<Vec<UtxoInfo>, RpcError> {
        self.node.scan_utxos(payloads)
    }

    fn scan_mempool(&self, payloads: &[Hex32]) -> Result<MempoolScan, RpcError> {
        self.node.scan_mempool(payloads)
    }
}

/// Jawaban `Ack` untuk blok dari rekan (Spesifikasi P2P Bagian 6).
#[must_use]
pub const fn block_ack(status: SubmitStatus) -> Ack {
    match status {
        SubmitStatus::Accepted | SubmitStatus::Duplicate => Ack::Accepted,
        SubmitStatus::Orphan => Ack::UnknownParent,
        SubmitStatus::Deferred => Ack::Deferred,
        SubmitStatus::Invalid => Ack::Invalid,
    }
}

/// Jawaban `Ack` untuk transaksi dari rekan (Spesifikasi P2P Bagian 6).
///
/// Penolakan yang bergantung pada view chain dan mempool penerima atau pada waktu (input belum ada atau sudah
/// dibelanjakan, coinbase belum matang, kunci waktu kondisi) dan penolakan kebijakan antrean dijawab `4` tanpa sanksi.
/// Pelanggaran lain atas T1 sampai T6 dan struktur dijawab `3`, dan pengirimnya disanksi.
#[must_use]
pub const fn tx_ack(outcome: &TxOutcome) -> Ack {
    match outcome {
        TxOutcome::Added | TxOutcome::Duplicate => Ack::Accepted,
        TxOutcome::Rejected(MempoolError::Invalid(
            TxError::MissingUtxo { .. }
            | TxError::ImmatureCoinbase { .. }
            | TxError::ConditionLocked { .. },
        )) => Ack::Rejected,
        TxOutcome::Rejected(MempoolError::Invalid(_)) => Ack::Invalid,
        TxOutcome::Rejected(_) => Ack::Rejected,
    }
}

/// Pesan `Status` dari ujung chain aktif `node`.
///
/// # Errors
///
/// [`RpcError::Internal`] jika state node rusak.
pub fn local_status<S: ChainStore>(node: &Node<S>) -> Result<Status, RpcError> {
    let tip = node.tip_summary()?;
    Ok(Status::new(
        node.params().network_id(),
        BlockHeight::new(tip.height),
        tip.id,
        tip.chainwork,
    ))
}

/// Yang diketahui node tentang satu rekan yang terhubung.
#[derive(Debug, Default)]
struct PeerState {
    /// Chainwork ujung rekan menurut `status` terakhir.
    chainwork: [u8; 32],
    /// ID blok yang diketahui sudah dimiliki rekan.
    known: BTreeSet<Hash32>,
    /// Permintaan `GetHeaders` ke rekan ini sedang berjalan.
    headers_pending: bool,
    /// ID blok yang dijawab `Entry 00` atau gagal diambil dari rekan ini; dikosongkan setiap selang sinkronisasi.
    missing_bodies: BTreeSet<Hash32>,
    /// Presync header yang sedang berjalan dengan rekan ini (TSK-M35 bagian B). Selama ada, jawaban `Headers` dari rekan
    /// ini diproses presync, dan sinkronisasi header biasa dengan rekan ini tidak dimulai.
    presync: Option<Presync>,
}

impl PeerState {
    fn remember(&mut self, id: Hash32) {
        if self.known.len() >= MAX_KNOWN_PER_PEER {
            self.known.clear();
        }
        self.known.insert(id);
    }
}

/// Permintaan `GetBlocks` yang sedang berjalan (paling banyak satu).
struct BodyRequest {
    request: RequestId,
    peer: PeerId,
    ids: Vec<Hash32>,
}

/// Layanan P2P satu node: memproses event [`Network`] dan pengumuman [`Relay`].
pub struct P2pService<S> {
    node: Arc<Node<S>>,
    net: Network,
    peers: BTreeMap<PeerId, PeerState>,
    bodies: Option<BodyRequest>,
    printed: bool,
}

impl<S: ChainStore + Send + 'static> P2pService<S> {
    /// Layanan di atas `node` dan `net`.
    #[must_use]
    pub const fn new(node: Arc<Node<S>>, net: Network) -> Self {
        Self {
            node,
            net,
            peers: BTreeMap::new(),
            bodies: None,
            printed: false,
        }
    }

    /// Berjalan terus: event jaringan, pengumuman dari RPC, dan percobaan ulang setiap [`SYNC_INTERVAL`]. Alamat dengar
    /// pertama dicetak ke stdout sebagai `p2p: <alamat>/p2p/<PeerId>`.
    pub async fn run(mut self, mut announcements: UnboundedReceiver<Announcement>) {
        let mut tick = tokio::time::interval(SYNC_INTERVAL);
        tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                event = self.net.next_event() => self.on_event(event),
                Some(announcement) = announcements.recv() => self.on_announcement(announcement),
                _ = tick.tick() => self.on_tick(),
            }
        }
    }

    fn on_event(&mut self, event: NetEvent) {
        match event {
            NetEvent::Listening { address } => self.on_listening(&address),
            NetEvent::PeerConnected { peer, address } => {
                tracing::info!(%peer, %address, "rekan terhubung");
                self.peers.insert(peer, PeerState::default());
            }
            NetEvent::PeerDisconnected { peer } => {
                tracing::info!(%peer, "rekan terputus");
                self.peers.remove(&peer);
                if self
                    .bodies
                    .as_ref()
                    .is_some_and(|pending| pending.peer == peer)
                {
                    self.bodies = None;
                }
                self.fetch_bodies();
                self.sync_from_best_peer();
            }
            NetEvent::PeerStatus { peer, status } => self.on_status(peer, &status),
            NetEvent::GetHeaders { request, reply, .. } => {
                let headers = self
                    .node
                    .headers_after(&request.locator, &request.stop, MAX_HEADERS)
                    .unwrap_or_else(|error| {
                        tracing::warn!(%error, "header untuk rekan gagal dibaca");
                        Vec::new()
                    });
                let _ = self.net.reply_headers(reply, Headers { headers });
            }
            NetEvent::GetBlocks { request, reply, .. } => {
                let entries = self
                    .node
                    .blocks_by_id(&request.ids)
                    .unwrap_or_else(|error| {
                        tracing::warn!(%error, "blok untuk rekan gagal dibaca");
                        vec![None; request.ids.len()]
                    });
                let _ = self.net.reply_blocks(reply, Blocks { entries });
            }
            NetEvent::Block { peer, block, reply } => self.on_block(peer, &block, reply),
            NetEvent::Tx { peer, tx, reply } => self.on_tx(peer, tx, reply),
            NetEvent::Headers { peer, headers, .. } => self.on_headers(peer, &headers),
            NetEvent::Blocks {
                peer,
                request,
                blocks,
            } => self.on_bodies(peer, request, blocks),
            NetEvent::BlockAck { .. } | NetEvent::TxAck { .. } => {}
            NetEvent::RequestFailed {
                peer,
                kind,
                request,
                ..
            } => self.on_failure(peer, kind, request),
            NetEvent::PeerPunished { peer, offense } => {
                tracing::warn!(%peer, ?offense, "rekan disanksi: diputus dan diblokir 24 jam");
            }
        }
    }

    fn on_listening(&mut self, address: &Multiaddr) {
        let full = format!("{address}/p2p/{}", self.net.local_peer_id());
        tracing::info!(address = %full, "P2P mendengar");
        if !self.printed {
            println!("p2p: {full}");
            self.printed = true;
        }
    }

    fn local_chainwork(&self) -> [u8; 32] {
        self.node.tip_summary().map_or([0; 32], |tip| tip.chainwork)
    }

    fn refresh_status(&mut self) {
        match local_status(&self.node) {
            Ok(status) => self.net.set_local_status(status),
            Err(error) => tracing::warn!(%error, "status lokal gagal dibaca"),
        }
    }

    fn on_status(&mut self, peer: PeerId, status: &Status) {
        if let Some(state) = self.peers.get_mut(&peer) {
            state.chainwork = status.chainwork;
        }
        if status.chainwork > self.local_chainwork() {
            self.request_headers(peer, None);
        }
    }

    /// Meminta header ke `peer` dengan locator chain aktif, didahului `after` (header terakhir batch sebelumnya).
    fn request_headers(&mut self, peer: PeerId, after: Option<Hash32>) {
        let mut locator = match self.node.locator() {
            Ok(locator) => locator,
            Err(error) => {
                tracing::warn!(%error, "locator gagal disusun");
                return;
            }
        };
        if let Some(last) = after {
            locator.insert(0, last);
            locator.truncate(MAX_LOCATOR_IDS);
        }
        let Some(state) = self.peers.get_mut(&peer) else {
            return;
        };
        if after.is_none() && (state.headers_pending || state.presync.is_some()) {
            return;
        }
        state.headers_pending = true;
        let _ = self.net.request_headers(
            &peer,
            GetHeaders {
                locator,
                stop: Hash32::ZERO,
            },
        );
    }

    fn sync_from_best_peer(&mut self) {
        let local = self.local_chainwork();
        let best = self
            .peers
            .iter()
            .filter(|(_, state)| {
                state.chainwork > local && !state.headers_pending && state.presync.is_none()
            })
            .max_by_key(|(_, state)| state.chainwork)
            .map(|(peer, _)| *peer);
        if let Some(peer) = best {
            self.request_headers(peer, None);
        }
    }

    fn on_headers(&mut self, peer: PeerId, headers: &Headers) {
        let presync = self.peers.get_mut(&peer).and_then(|state| {
            state.headers_pending = false;
            state.presync.take()
        });
        if let Some(presync) = presync {
            self.on_presync_headers(peer, presync, headers);
            return;
        }
        let outcome = match self.node.accept_headers(&headers.headers) {
            Ok(outcome) => outcome,
            Err(error) => {
                tracing::warn!(%error, "header gagal diproses");
                return;
            }
        };
        match &outcome {
            HeadersOutcome::Accepted {
                last: Some(last), ..
            } if headers.headers.len() >= MAX_HEADERS => self.request_headers(peer, Some(*last)),
            HeadersOutcome::Full { .. } => {
                tracing::info!(%peer, "indeks header penuh; presync header dengan rekan dimulai");
                self.start_presync(peer);
                self.fetch_bodies();
            }
            HeadersOutcome::Accepted { .. } | HeadersOutcome::Deferred { .. } => {
                self.fetch_bodies();
            }
            HeadersOutcome::Disconnected | HeadersOutcome::Invalid { .. } => {
                tracing::warn!(%peer, ?outcome, "header tidak sah dari rekan");
                self.net.punish(&peer);
            }
        }
    }

    /// Memulai presync header dengan `peer`: header diminta dengan locator chain aktif, dan presync dicatat hanya jika
    /// permintaannya terkirim.
    fn start_presync(&mut self, peer: PeerId) {
        self.request_headers(peer, None);
        if let Some(state) = self.peers.get_mut(&peer)
            && state.headers_pending
        {
            state.presync = Some(Presync::default());
        }
    }

    /// Meminta header sesudah `after` untuk presync `presync`; presync disimpan hanya jika permintaannya terkirim.
    fn continue_presync(&mut self, peer: PeerId, presync: Presync, after: Hash32) {
        self.request_headers(peer, Some(after));
        if let Some(state) = self.peers.get_mut(&peer)
            && state.headers_pending
        {
            state.presync = Some(presync);
        }
    }

    /// Memproses jawaban `Headers` dari rekan yang sedang presync (Spesifikasi P2P Bagian 5).
    fn on_presync_headers(&mut self, peer: PeerId, mut presync: Presync, headers: &Headers) {
        let outcome = match self.node.presync_headers(&mut presync, &headers.headers) {
            Ok(outcome) => outcome,
            Err(error) => {
                tracing::warn!(%error, "presync header gagal diproses");
                return;
            }
        };
        match &outcome {
            PresyncOutcome::Continue { after } if headers.headers.len() >= MAX_HEADERS => {
                self.continue_presync(peer, presync, *after);
            }
            PresyncOutcome::Redownload { fork } => {
                tracing::info!(%peer, "cabang rekan terbukti lebih berat; header diunduh ulang");
                self.continue_presync(peer, presync, *fork);
            }
            PresyncOutcome::Done { added } => {
                tracing::info!(%peer, added = *added, "presync header selesai");
                self.fetch_bodies();
            }
            PresyncOutcome::Continue { .. }
            | PresyncOutcome::NotHeavier
            | PresyncOutcome::Mismatch
            | PresyncOutcome::Deferred => {
                tracing::info!(%peer, ?outcome, "presync header berhenti tanpa sanksi");
            }
            PresyncOutcome::Disconnected | PresyncOutcome::Invalid { .. } => {
                tracing::warn!(%peer, ?outcome, "header presync tidak sah dari rekan");
                self.net.punish(&peer);
            }
        }
    }

    /// Meminta badan blok berikutnya yang dibutuhkan, jika belum ada permintaan yang berjalan.
    fn fetch_bodies(&mut self) {
        if self.bodies.is_some() {
            return;
        }
        let ids = match self.node.missing_blocks(MAX_BLOCK_IDS) {
            Ok(ids) => ids,
            Err(error) => {
                tracing::warn!(%error, "daftar blok yang dibutuhkan gagal disusun");
                return;
            }
        };
        let Some(first) = ids.first().copied() else {
            return;
        };
        let Some(peer) = self
            .peers
            .iter()
            .filter(|(_, state)| !state.missing_bodies.contains(&first))
            .max_by_key(|(_, state)| state.chainwork)
            .map(|(peer, _)| *peer)
        else {
            return;
        };
        let request = self
            .net
            .request_blocks(&peer, GetBlocks { ids: ids.clone() });
        self.bodies = Some(BodyRequest { request, peer, ids });
    }

    fn on_bodies(&mut self, peer: PeerId, request: RequestId, blocks: Blocks) {
        let Some(pending) = self
            .bodies
            .take_if(|pending| pending.request == request && pending.peer == peer)
        else {
            return;
        };
        let mut new_tip: Option<Block> = None;
        let mut punished = false;
        for (id, entry) in pending.ids.iter().zip(blocks.entries) {
            let Some(block) = entry else {
                // Entry 00: blok diminta dari rekan lain.
                if let Some(state) = self.peers.get_mut(&peer) {
                    state.missing_bodies.insert(*id);
                }
                break;
            };
            if block_id(block.header()) != *id {
                tracing::warn!(%peer, "rekan mengirim blok yang tidak diminta");
                punished = true;
                break;
            }
            if let Some(state) = self.peers.get_mut(&peer) {
                state.remember(*id);
            }
            match self.node.receive_block(&block.to_bytes()) {
                Ok(receipt) if receipt.result.status == SubmitStatus::Invalid => {
                    tracing::warn!(%peer, result = ?receipt.result, "blok tidak sah dari rekan");
                    punished = true;
                    break;
                }
                Ok(receipt) => {
                    if receipt.new_tip {
                        new_tip = Some(block);
                    }
                }
                Err(error) => {
                    tracing::warn!(%error, "blok dari rekan gagal diproses");
                    break;
                }
            }
        }
        if let Some(block) = new_tip {
            self.broadcast_block(&block, Some(peer));
            self.refresh_status();
        }
        if punished {
            self.net.punish(&peer);
        }
        self.fetch_bodies();
    }

    fn on_block(&mut self, peer: PeerId, block: &Block, reply: Reply<Ack>) {
        let id = block_id(block.header());
        if let Some(state) = self.peers.get_mut(&peer) {
            state.remember(id);
        }
        let receipt = match self.node.receive_block(&block.to_bytes()) {
            Ok(receipt) => receipt,
            Err(error) => {
                tracing::warn!(%error, "blok dari rekan gagal diproses");
                return;
            }
        };
        let ack = block_ack(receipt.result.status);
        let _ = self.net.reply_ack(reply, ack);
        if receipt.new_tip {
            self.broadcast_block(block, Some(peer));
            self.refresh_status();
        }
        match ack {
            Ack::UnknownParent => self.request_headers(peer, None),
            Ack::Invalid => {
                tracing::warn!(%peer, result = ?receipt.result, "blok tidak sah dari rekan");
                self.net.punish(&peer);
            }
            Ack::Accepted | Ack::Deferred | Ack::Rejected => {}
        }
    }

    fn on_tx(&mut self, peer: PeerId, tx: Transaction, reply: Reply<Ack>) {
        let outcome = match self.node.receive_transaction(tx.clone()) {
            Ok((_, outcome)) => outcome,
            Err(error) => {
                tracing::warn!(%error, "transaksi dari rekan gagal diproses");
                return;
            }
        };
        let ack = tx_ack(&outcome);
        let _ = self.net.reply_ack(reply, ack);
        if outcome == TxOutcome::Added {
            self.broadcast_tx(&tx, Some(peer));
        }
        if ack == Ack::Invalid {
            tracing::warn!(%peer, ?outcome, "transaksi tidak sah dari rekan");
            self.net.punish(&peer);
        }
    }

    fn on_failure(&mut self, peer: PeerId, kind: ProtocolKind, request: RequestId) {
        match kind {
            ProtocolKind::Headers => {
                if let Some(state) = self.peers.get_mut(&peer) {
                    state.headers_pending = false;
                    state.presync = None;
                }
            }
            ProtocolKind::Blocks => {
                if let Some(pending) = self
                    .bodies
                    .take_if(|pending| pending.request == request && pending.peer == peer)
                    && let (Some(first), Some(state)) =
                        (pending.ids.first(), self.peers.get_mut(&peer))
                {
                    state.missing_bodies.insert(*first);
                }
                self.fetch_bodies();
            }
            ProtocolKind::Status | ProtocolKind::Block | ProtocolKind::Tx => {}
        }
    }

    fn on_announcement(&mut self, announcement: Announcement) {
        match announcement {
            Announcement::Block(block) => {
                self.broadcast_block(&block, None);
                self.refresh_status();
            }
            Announcement::Tx(tx) => self.broadcast_tx(&tx, None),
        }
    }

    fn on_tick(&mut self) {
        for state in self.peers.values_mut() {
            state.missing_bodies.clear();
        }
        self.sync_from_best_peer();
        self.fetch_bodies();
    }

    /// Mengirim `block` ke semua rekan yang belum diketahui punya blok itu, kecuali `except`.
    fn broadcast_block(&mut self, block: &Block, except: Option<PeerId>) {
        let id = block_id(block.header());
        let mut targets = Vec::new();
        for (peer, state) in &mut self.peers {
            if Some(*peer) == except || state.known.contains(&id) {
                continue;
            }
            state.remember(id);
            targets.push(*peer);
        }
        for peer in targets {
            let _ = self.net.send_block(&peer, block.clone());
        }
    }

    /// Mengirim `tx` ke semua rekan kecuali `except`.
    fn broadcast_tx(&mut self, tx: &Transaction, except: Option<PeerId>) {
        let targets: Vec<PeerId> = self
            .peers
            .keys()
            .copied()
            .filter(|peer| Some(*peer) != except)
            .collect();
        for peer in targets {
            let _ = self.net.send_tx(&peer, tx.clone());
        }
    }
}

#[cfg(test)]
mod tests;
