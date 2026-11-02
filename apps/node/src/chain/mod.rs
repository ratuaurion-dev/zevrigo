//! Logika chain node (TSK-M18 bagian A, TSK-M27): chain aktif di atas [`ChainStore`], blok cabang, pemilihan chain
//! berdasarkan chainwork, reorg, mempool, dan implementasi [`RpcBackend`] untuk 8 metode Spesifikasi RPC v0.1.
//!
//! Aturan konsensus tidak ditulis di sini. Blok diperiksa dengan fungsi MOD-04 berurutan V1 sampai V10
//! (Spesifikasi Validasi Bagian 9), transaksi dengan mempool MOD-07 (T1 sampai T6).
//!
//! Pemilihan chain (Spesifikasi P2P v0.1 Bagian 5): blok yang induknya dikenal tetapi bukan ujung chain aktif disimpan
//! sebagai blok cabang setelah lolos V1 sampai V8. Jika chainwork sebuah cabang melebihi chainwork ujung chain aktif,
//! node melakukan reorg tanpa batas kedalaman: `undo_tip` sampai titik cabang, lalu setiap blok cabang diperiksa V9 dan
//! V10 dan diterapkan. Jika satu blok cabang tidak sah, blok itu dan keturunannya ditandai tidak sah dan chain lama
//! dipulihkan. Jika chainwork sama, chain yang lebih dulu diterima tetap aktif. Blok cabang disimpan di memori node
//! (v0.1), bukan di penyimpanan; setelah node dibuka ulang, cabang diterima lagi dari rekan. Jumlah dan ukuran blok
//! cabang, indeks header, dan catatan blok tidak sah dibatasi [`NodeLimits`] (TSK-M34).
//!
//! Blok yang induknya belum dikenal dilaporkan `orphan` dan tidak disimpan. Blok yang gagal V8 dilaporkan `deferred`
//! dan tidak disimpan; pengirim boleh mengirim ulang.
//!
//! Sinkronisasi P2P (TSK-M28) memakai indeks header tanpa badan blok, locator, dan jawaban `GetHeaders`/`GetBlocks`
//! di submodul `sync`; [`Node::receive_block`] dan [`Node::receive_transaction`] melaporkan juga apakah ujung chain
//! berubah atau transaksi baru masuk mempool, untuk siaran ke rekan.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Display;
use std::sync::{Mutex, MutexGuard};

use homeostasis_consensus::{
    BlockError, ChainParams, ParentContext, U256, asert_target, block_subsidy, block_work,
    check_block_context_free, check_block_transactions, check_future_timestamp,
    check_header_context, decode_block, decode_compact, encode_compact, median_time_past,
    regtest_genesis, total_supply_after,
};
use homeostasis_crypto::{block_id, block_merkle_root, txid};
use homeostasis_mempool::{Mempool, MempoolEntry, MempoolError, rule_name};
use homeostasis_primitives::params::{MAX_BLOCK_SIZE, MEDIAN_TIME_SPAN};
use homeostasis_primitives::{
    Amount, Block, BlockHeight, Decode, Encode, Hash32, Header, OutPoint, Output, Transaction,
    TxBody, UtxoEntry,
};
use homeostasis_rpc::{
    BlockInfo, BlockTemplate, ChainInfo, CompactHex, GetBlockTemplateParams, Hex32, HexBytes,
    MempoolInfo, NetworkName, Quanta, RpcBackend, RpcError, SendRawTransactionResult,
    SubmitBlockResult, SubmitStatus, UtxoInfo,
};
use homeostasis_storage::{ChainStore, StoreError};
use thiserror::Error;

mod sync;

pub use sync::{HeadersOutcome, MAX_LOCATOR_IDS, locator_heights};

/// Nama aturan untuk blok yang induk atau leluhurnya tidak sah: blok itu tidak punya induk sah (V5).
pub const INVALID_ANCESTOR_RULE: &str = "V5";

/// Ruang yang disisihkan dari ukuran blok saat memilih transaksi mempool untuk template: header 96 byte, jumlah
/// transaksi, dan coinbase (sekitar 70 byte), dengan cadangan.
const TEMPLATE_RESERVE_BYTES: usize = 1_000;

/// Jam lokal node dalam detik Unix, untuk aturan V8 dan cap waktu template. Disuntikkan agar test deterministik.
pub type Clock = Box<dyn Fn() -> u64 + Send + Sync>;

/// Batas memori node untuk data yang hanya disimpan di memori (Spesifikasi P2P v0.1 Bagian 9): blok cabang, indeks
/// header tanpa badan blok, dan catatan blok tidak sah. Kebijakan node, bukan aturan konsensus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeLimits {
    /// Jumlah blok cabang paling banyak.
    pub max_side_blocks: usize,
    /// Total ukuran encoding blok cabang paling banyak, dalam byte.
    pub max_side_bytes: usize,
    /// Jumlah header tanpa badan blok paling banyak.
    pub max_headers: usize,
    /// Jumlah catatan blok tidak sah paling banyak; catatan tertua dilupakan lebih dulu.
    pub max_invalid: usize,
}

impl NodeLimits {
    /// Batas bawaan: 10_000 blok cabang dan 64 MiB, 100_000 header (sekitar 20 MB), 10_000 catatan blok tidak sah.
    pub const DEFAULT: Self = Self {
        max_side_blocks: 10_000,
        max_side_bytes: 64 * 1024 * 1024,
        max_headers: 100_000,
        max_invalid: 10_000,
    };
}

impl Default for NodeLimits {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Galat saat membuka node di atas sebuah penyimpanan.
#[derive(Debug, Error)]
pub enum NodeError {
    /// Penyimpanan gagal.
    #[error("penyimpanan gagal: {0}")]
    Store(#[from] StoreError),
    /// Genesis jaringan tidak tersedia atau tidak bisa dibentuk.
    #[error("genesis jaringan tidak tersedia: {message}")]
    Genesis {
        /// Keterangan galat.
        message: String,
    },
    /// Penyimpanan berisi chain dengan genesis lain (jaringan lain atau folder data yang salah).
    #[error("penyimpanan berisi genesis {found:?}, seharusnya {expected:?}")]
    WrongGenesis {
        /// ID genesis jaringan ini.
        expected: Hash32,
        /// ID blok tinggi 0 di penyimpanan.
        found: Option<Hash32>,
    },
    /// Isi penyimpanan tidak konsisten.
    #[error("isi penyimpanan tidak konsisten di tinggi {height}: {message}")]
    Corrupt {
        /// Tinggi blok yang bermasalah.
        height: u64,
        /// Keterangan galat.
        message: String,
    },
}

/// Nama aturan untuk `reason` hasil `submit_block` (`V1` sampai `V10`, atau `T1` sampai `T6` untuk transaksi).
///
/// `difficulty_rule` dipakai untuk [`BlockError::Difficulty`], yang bisa berasal dari V2 (target header) atau V6
/// (target ASERT), tergantung tahap pemeriksaan.
#[must_use]
pub fn block_rule(error: &BlockError, difficulty_rule: &'static str) -> &'static str {
    match error {
        BlockError::Decode(_) => "V1",
        BlockError::Difficulty(_) => difficulty_rule,
        BlockError::TargetAboveLimit => "V2",
        BlockError::PowUnavailable { .. } | BlockError::HighHash => "V3",
        BlockError::MerkleRootMismatch => "V4",
        BlockError::PrevBlockMismatch | BlockError::HeightMismatch { .. } => "V5",
        BlockError::TargetMismatch { .. } => "V6",
        BlockError::MissingParentTimestamps | BlockError::TimestampNotAfterMedian { .. } => "V7",
        BlockError::TimestampTooFarInFuture { .. } => "V8",
        BlockError::Transaction { error, .. } => match rule_name(error) {
            "struktur" => "V9",
            name => name,
        },
        BlockError::Genesis
        | BlockError::DuplicateOutpoint { .. }
        | BlockError::OutputIndexOverflow => "V9",
        BlockError::AmountOverflow
        | BlockError::Subsidy(_)
        | BlockError::CoinbaseAmountMismatch { .. } => "V10",
    }
}

/// Nilai `data.reason` error RPC `1002` untuk transaksi yang ditolak mempool.
///
/// Aturan konsensus: `T1` sampai `T6` atau `struktur`. Penolakan antrean yang bukan konsensus: `terlalu_besar` dan
/// `mempool_penuh` (Spesifikasi RPC v0.1 belum punya kode khusus; dicatat untuk v0.2).
#[must_use]
pub const fn mempool_reason(error: &MempoolError) -> &'static str {
    match error {
        MempoolError::TooLarge { .. } => "terlalu_besar",
        MempoolError::Full { .. } => "mempool_penuh",
        MempoolError::Duplicate { .. } => "duplikat",
        MempoolError::Conflict { .. } => "T1",
        MempoolError::Invalid(tx_error) => rule_name(tx_error),
    }
}

/// Error internal node untuk RPC (`-32603`).
fn internal(error: impl Display) -> RpcError {
    RpcError::Internal {
        message: error.to_string(),
    }
}

/// Hasil `submit_block` tanpa `reason`.
fn outcome(status: SubmitStatus, id: Hash32) -> SubmitBlockResult {
    SubmitBlockResult {
        status,
        block_id: Some(Hex32::from(id)),
        reason: None,
    }
}

/// Hasil `submit_block` berstatus `invalid` dengan nama aturan.
fn invalid(id: Option<Hash32>, rule: &str) -> SubmitBlockResult {
    SubmitBlockResult {
        status: SubmitStatus::Invalid,
        block_id: id.map(Hex32::from),
        reason: Some(rule.to_owned()),
    }
}

/// Outpoint yang dibelanjakan transaksi biasa; coinbase tidak punya input.
fn regular_inputs(tx: &Transaction) -> Vec<OutPoint> {
    match tx.body() {
        TxBody::Regular(body) => body.inputs().iter().map(OutPoint::from_input).collect(),
        TxBody::Coinbase(_) => Vec::new(),
    }
}

/// Mengambil dari penyimpanan entri UTXO yang dirujuk input `txs`, agar validasi MOD-04 dan MOD-07 tidak bergantung
/// pada MOD-05. Outpoint yang tidak ada dilewati (validasi yang menolaknya dengan T1).
fn prefetch<'a, S: ChainStore>(
    store: &S,
    txs: impl IntoIterator<Item = &'a Transaction>,
) -> Result<BTreeMap<OutPoint, UtxoEntry>, StoreError> {
    let mut utxos = BTreeMap::new();
    for tx in txs {
        for outpoint in regular_inputs(tx) {
            if let Some(entry) = store.utxo(&outpoint)? {
                utxos.insert(outpoint, entry);
            }
        }
    }
    Ok(utxos)
}

/// Ringkasan satu blok di chain aktif yang dipakai berulang.
#[derive(Clone, Copy, Debug)]
struct BlockMeta {
    id: Hash32,
    timestamp: u64,
    chainwork: U256,
    mined_total: Amount,
}

/// Kerja satu blok dari target compact header.
fn header_work(header: &Header) -> Option<U256> {
    decode_compact(header.target())
        .ok()
        .and_then(|target| block_work(&target))
}

/// Menyusun ulang ringkasan chain aktif dari penyimpanan, dari genesis sampai ujung.
fn rebuild_metas<S: ChainStore>(store: &S) -> Result<Vec<BlockMeta>, NodeError> {
    let Some((tip, _)) = store.tip()? else {
        return Ok(Vec::new());
    };
    let mut metas: Vec<BlockMeta> = Vec::new();
    for height in 0..=tip.get() {
        let corrupt = |message: &str| NodeError::Corrupt {
            height,
            message: message.to_owned(),
        };
        let id = store
            .block_id_at(BlockHeight::new(height))?
            .ok_or_else(|| corrupt("ID blok tidak ada"))?;
        let block = store.block(&id)?.ok_or_else(|| corrupt("blok tidak ada"))?;
        let header = block.header();
        let work = header_work(header).ok_or_else(|| corrupt("target tidak sah"))?;
        let meta = match metas.last() {
            None => BlockMeta {
                id,
                timestamp: header.timestamp(),
                chainwork: work,
                mined_total: Amount::ZERO,
            },
            Some(prev) => {
                let subsidy = block_subsidy(header.height(), prev.mined_total)
                    .map_err(|e| corrupt(&e.to_string()))?;
                BlockMeta {
                    id,
                    timestamp: header.timestamp(),
                    chainwork: prev
                        .chainwork
                        .checked_add(work)
                        .ok_or_else(|| corrupt("chainwork meluap"))?,
                    mined_total: prev
                        .mined_total
                        .checked_add(subsidy)
                        .ok_or_else(|| corrupt("total tambang meluap"))?,
                }
            }
        };
        metas.push(meta);
    }
    Ok(metas)
}

/// Tinggi setiap blok chain aktif, berdasarkan ID.
fn index_heights(metas: &[BlockMeta]) -> Result<BTreeMap<Hash32, u64>, NodeError> {
    let mut heights = BTreeMap::new();
    for (height, meta) in (0_u64..).zip(metas) {
        heights.insert(meta.id, height);
    }
    if heights.len() != metas.len() {
        return Err(NodeError::Corrupt {
            height: 0,
            message: "ID blok ganda di chain aktif".to_owned(),
        });
    }
    Ok(heights)
}

/// Blok cabang: lolos V1 sampai V8 terhadap induknya, tetapi tidak di chain aktif. V9 dan V10 diperiksa saat blok
/// itu disambungkan ke chain aktif lewat reorg.
#[derive(Clone, Debug)]
struct SideBlock {
    block: Block,
    chainwork: U256,
    sequence: u64,
    /// Ukuran encoding blok, untuk `NodeLimits::max_side_bytes`.
    size: usize,
}

/// Header yang lolos V2, V3, dan V5 sampai V8 terhadap induknya, tetapi badan bloknya belum diterima (sinkronisasi
/// header lalu blok, TSK-M28).
#[derive(Clone, Copy, Debug)]
struct HeaderEntry {
    header: Header,
    chainwork: U256,
    sequence: u64,
}

/// Data induk untuk V5 sampai V7 dan chainwork blok baru.
struct Parent {
    height: u64,
    chainwork: U256,
    timestamps: Vec<u64>,
}

/// Hasil menyambungkan satu blok ke ujung chain aktif.
enum Connect {
    /// Lolos V9 dan V10, sudah diterapkan ke penyimpanan.
    Applied,
    /// Tidak sah; berisi nama aturan.
    Invalid(&'static str),
}

/// Isi node yang dijaga satu mutex: penyimpanan, mempool, ringkasan chain aktif, blok cabang, indeks header, dan blok
/// tidak sah.
struct ChainState<S> {
    store: S,
    mempool: Mempool,
    metas: Vec<BlockMeta>,
    heights: BTreeMap<Hash32, u64>,
    side: BTreeMap<Hash32, SideBlock>,
    headers: BTreeMap<Hash32, HeaderEntry>,
    invalid: BTreeMap<Hash32, &'static str>,
    /// ID di `invalid` urut dicatat, untuk melupakan catatan tertua lebih dulu.
    invalid_order: VecDeque<Hash32>,
    next_sequence: u64,
    limits: NodeLimits,
}

impl<S> ChainState<S> {
    /// Tinggi dan ringkasan ujung chain aktif.
    fn tip(&self) -> Result<(u64, BlockMeta), RpcError> {
        let meta = *self.metas.last().ok_or_else(|| internal("chain kosong"))?;
        let height = u64::try_from(self.metas.len().saturating_sub(1)).map_err(internal)?;
        Ok((height, meta))
    }

    /// Cap waktu 11 blok terakhir sampai ujung, urut dari lama ke baru.
    fn recent_timestamps(&self) -> Vec<u64> {
        let start = self.metas.len().saturating_sub(MEDIAN_TIME_SPAN);
        self.metas
            .iter()
            .skip(start)
            .map(|meta| meta.timestamp)
            .collect()
    }

    /// Menambah ringkasan blok baru di ujung chain aktif.
    fn push_meta(&mut self, meta: BlockMeta) -> Result<(), RpcError> {
        let height = u64::try_from(self.metas.len()).map_err(internal)?;
        self.heights.insert(meta.id, height);
        self.metas.push(meta);
        Ok(())
    }

    /// Melepas ringkasan blok ujung chain aktif.
    fn pop_meta(&mut self) -> Option<BlockMeta> {
        let meta = self.metas.pop()?;
        self.heights.remove(&meta.id);
        Some(meta)
    }

    /// Apakah blok `id` sudah dikenal (di chain aktif atau sebagai blok cabang).
    fn known(&self, id: &Hash32) -> bool {
        self.heights.contains_key(id) || self.side.contains_key(id)
    }

    /// Cap waktu blok chain aktif dari tinggi `height` mundur, paling banyak `count`, urut dari baru ke lama.
    fn active_timestamps_back(&self, height: u64, count: usize) -> Result<Vec<u64>, RpcError> {
        let end = usize::try_from(height)
            .ok()
            .and_then(|index| index.checked_add(1))
            .ok_or_else(|| internal("tinggi blok di luar jangkauan"))?;
        let start = end.saturating_sub(count);
        let slice = self
            .metas
            .get(start..end)
            .ok_or_else(|| internal("ringkasan chain tidak lengkap"))?;
        Ok(slice.iter().rev().map(|meta| meta.timestamp).collect())
    }

    /// Data induk `parent_id` jika induk itu dikenal: tinggi, chainwork, dan cap waktu sampai 11 blok terakhir
    /// sampai induk (urut dari lama ke baru), mengikuti cabang induk itu.
    fn parent_info(&self, parent_id: &Hash32) -> Result<Option<Parent>, RpcError> {
        let (height, chainwork) = if let Some(&height) = self.heights.get(parent_id) {
            let index = usize::try_from(height).map_err(internal)?;
            let meta = self
                .metas
                .get(index)
                .ok_or_else(|| internal("ringkasan chain tidak lengkap"))?;
            (height, meta.chainwork)
        } else if let Some(entry) = self.side.get(parent_id) {
            (entry.block.header().height().get(), entry.chainwork)
        } else {
            return Ok(None);
        };
        let mut newest_first: Vec<u64> = Vec::with_capacity(MEDIAN_TIME_SPAN);
        let mut cursor = *parent_id;
        while newest_first.len() < MEDIAN_TIME_SPAN {
            if let Some(entry) = self.side.get(&cursor) {
                newest_first.push(entry.block.header().timestamp());
                cursor = entry.block.header().prev_block_id();
                continue;
            }
            let active = *self
                .heights
                .get(&cursor)
                .ok_or_else(|| internal("leluhur blok cabang tidak dikenal"))?;
            let need = MEDIAN_TIME_SPAN.saturating_sub(newest_first.len());
            newest_first.extend(self.active_timestamps_back(active, need)?);
            break;
        }
        newest_first.reverse();
        Ok(Some(Parent {
            height,
            chainwork,
            timestamps: newest_first,
        }))
    }

    /// Menyimpan blok cabang dengan nomor urut kedatangan berikutnya.
    fn store_side(&mut self, id: Hash32, block: Block, chainwork: U256) {
        let sequence = self.next_sequence;
        // Sengaja saturating: 2^64 blok cabang tidak tercapai; jika pun tercapai, urutan tetap tidak turun.
        self.next_sequence = sequence.saturating_add(1);
        let size = block.to_bytes().len();
        self.side.insert(
            id,
            SideBlock {
                block,
                chainwork,
                sequence,
                size,
            },
        );
    }

    /// Blok cabang dengan chainwork terbanyak; jika sama, yang lebih dulu diterima.
    fn best_side(&self) -> Option<(Hash32, U256)> {
        let mut best: Option<(Hash32, U256, u64)> = None;
        for (id, entry) in &self.side {
            let better = best.is_none_or(|(_, work, sequence)| {
                entry.chainwork > work || (entry.chainwork == work && entry.sequence < sequence)
            });
            if better {
                best = Some((*id, entry.chainwork, entry.sequence));
            }
        }
        best.map(|(id, work, _)| (id, work))
    }

    /// ID blok cabang dari sesudah titik cabang di chain aktif sampai `tip`, urut naik, beserta tinggi titik cabang.
    fn branch_to(&self, tip: Hash32) -> Result<(u64, Vec<Hash32>), RpcError> {
        let mut ids = Vec::new();
        let mut cursor = tip;
        loop {
            if let Some(&height) = self.heights.get(&cursor) {
                ids.reverse();
                return Ok((height, ids));
            }
            let entry = self
                .side
                .get(&cursor)
                .ok_or_else(|| internal("leluhur blok cabang tidak dikenal"))?;
            ids.push(cursor);
            cursor = entry.block.header().prev_block_id();
        }
    }

    /// Mencatat blok `id` tidak sah dengan aturan `rule`. Jika catatan melebihi `NodeLimits::max_invalid`, catatan
    /// tertua dilupakan; blok yang terlupa diperiksa ulang jika datang lagi.
    fn note_invalid(&mut self, id: Hash32, rule: &'static str) {
        if self.invalid.insert(id, rule).is_none() {
            self.invalid_order.push_back(id);
        }
        while self.invalid.len() > self.limits.max_invalid {
            let Some(oldest) = self.invalid_order.pop_front() else {
                return;
            };
            self.invalid.remove(&oldest);
        }
    }

    /// ID semua blok cabang dan header keturunan `roots` (tanpa `roots` sendiri), lewat satu peta induk ke anak.
    fn descendants(&self, roots: &[Hash32]) -> Vec<Hash32> {
        let mut children: BTreeMap<Hash32, Vec<Hash32>> = BTreeMap::new();
        for (id, entry) in &self.side {
            children
                .entry(entry.block.header().prev_block_id())
                .or_default()
                .push(*id);
        }
        for (id, entry) in &self.headers {
            children
                .entry(entry.header.prev_block_id())
                .or_default()
                .push(*id);
        }
        let mut found = Vec::new();
        let mut queue: Vec<Hash32> = roots.to_vec();
        while let Some(parent) = queue.pop() {
            if let Some(kids) = children.remove(&parent) {
                found.extend(kids.iter().copied());
                queue.extend(kids);
            }
        }
        found
    }

    /// Menandai blok `id` tidak sah, lalu membuang semua blok cabang dan header keturunannya dengan aturan
    /// [`INVALID_ANCESTOR_RULE`].
    fn mark_invalid(&mut self, id: Hash32, rule: &'static str) {
        let descendants = self.descendants(&[id]);
        self.side.remove(&id);
        self.headers.remove(&id);
        self.note_invalid(id, rule);
        for child in descendants {
            self.side.remove(&child);
            self.headers.remove(&child);
            self.note_invalid(child, INVALID_ANCESTOR_RULE);
        }
    }

    /// Blok cabang di jalur menuju header dengan chainwork terbanyak, jika chainwork header itu melebihi `tip_work`:
    /// cabang yang sedang diunduh dan lebih berat daripada chain aktif.
    fn protected_side(&self, tip_work: U256) -> BTreeSet<Hash32> {
        let mut protected = BTreeSet::new();
        let Some((best, work)) = self.best_header() else {
            return protected;
        };
        if work <= tip_work {
            return protected;
        }
        let mut cursor = best;
        while let Some(entry) = self.headers.get(&cursor) {
            cursor = entry.header.prev_block_id();
        }
        while let Some(entry) = self.side.get(&cursor) {
            protected.insert(cursor);
            cursor = entry.block.header().prev_block_id();
        }
        protected
    }

    /// Membuang blok cabang sampai jumlah dan ukurannya dalam `NodeLimits`. Yang dibuang selalu daun (blok cabang tanpa
    /// anak cabang) dengan chainwork paling kecil, jika sama yang paling akhir diterima, sehingga setiap cabang yang
    /// tersisa tetap utuh sampai chain aktif. Blok cabang di jalur menuju header yang lebih berat daripada chain aktif
    /// tidak pernah dibuang, sehingga cabang itu selalu bisa disambungkan (tanpa batas kedalaman reorg).
    fn trim_side(&mut self) -> Result<(), RpcError> {
        let (_, tip) = self.tip()?;
        let protected = self.protected_side(tip.chainwork);
        loop {
            let bytes = self
                .side
                .values()
                .fold(0_usize, |total, entry| total.saturating_add(entry.size));
            if self.side.len() <= self.limits.max_side_blocks && bytes <= self.limits.max_side_bytes
            {
                return Ok(());
            }
            let parents: BTreeSet<Hash32> = self
                .side
                .values()
                .map(|entry| entry.block.header().prev_block_id())
                .collect();
            let victim = self
                .side
                .iter()
                .filter(|(id, _)| !parents.contains(*id) && !protected.contains(*id))
                .min_by(|(_, a), (_, b)| {
                    a.chainwork
                        .cmp(&b.chainwork)
                        .then(b.sequence.cmp(&a.sequence))
                })
                .map(|(id, _)| *id);
            let Some(victim) = victim else {
                return Ok(());
            };
            self.side.remove(&victim);
        }
    }
}

impl<S: ChainStore> ChainState<S> {
    /// Melepas blok ujung chain aktif (`undo_tip`) beserta ringkasannya.
    fn disconnect_tip(&mut self) -> Result<(Hash32, Block, BlockMeta), RpcError> {
        let (id, block) = self.store.undo_tip().map_err(internal)?;
        let meta = self
            .pop_meta()
            .ok_or_else(|| internal("ringkasan chain kosong"))?;
        if meta.id != id {
            return Err(internal("ringkasan chain tidak sama dengan penyimpanan"));
        }
        Ok((id, block, meta))
    }

    /// Menerapkan kembali blok yang sudah pernah sah di posisi yang sama, beserta ringkasannya.
    fn reattach(&mut self, id: Hash32, block: &Block, meta: BlockMeta) -> Result<(), RpcError> {
        let txids: Vec<Hash32> = block.txs().iter().map(txid).collect();
        self.store
            .apply_block(block, id, &txids)
            .map_err(internal)?;
        self.push_meta(meta)
    }
}

/// Hasil menerima satu blok (RPC atau P2P): hasil `submit_block` dan apakah ujung chain aktif berubah.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockReceipt {
    /// Hasil seperti `submit_block` (Spesifikasi RPC 3.5).
    pub result: SubmitBlockResult,
    /// `true` jika ujung chain aktif berubah karena blok ini (disambung ke ujung atau reorg ke cabangnya).
    pub new_tip: bool,
}

/// Hasil memasukkan transaksi ke mempool.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TxOutcome {
    /// Transaksi baru masuk mempool.
    Added,
    /// Transaksi sudah ada di mempool.
    Duplicate,
    /// Ditolak mempool: aturan konsensus (T1 sampai T6, struktur) atau kebijakan antrean.
    Rejected(MempoolError),
}

/// Ujung chain aktif untuk pesan `Status` P2P.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TipSummary {
    /// Tinggi ujung.
    pub height: u64,
    /// ID blok ujung.
    pub id: Hash32,
    /// Chainwork kumulatif ujung, 256 bit big-endian.
    pub chainwork: [u8; 32],
}

/// Hasil RPC `send_raw_transaction` dari [`TxOutcome`]. Duplikat dianggap berhasil, karena transaksinya memang sudah
/// di mempool.
///
/// # Errors
///
/// [`RpcError::InvalidTransaction`] dengan nama aturan dari [`mempool_reason`] jika transaksi ditolak.
pub fn transaction_result(
    id: Hash32,
    outcome: &TxOutcome,
) -> Result<SendRawTransactionResult, RpcError> {
    match outcome {
        TxOutcome::Added | TxOutcome::Duplicate => Ok(SendRawTransactionResult {
            txid: Hex32::from(id),
        }),
        TxOutcome::Rejected(error) => Err(RpcError::InvalidTransaction {
            reason: mempool_reason(error).to_owned(),
        }),
    }
}

/// Node satu chain di atas penyimpanan `S`. Semua akses lewat satu mutex, sehingga aman dipakai server RPC.
pub struct Node<S> {
    params: ChainParams,
    clock: Clock,
    state: Mutex<ChainState<S>>,
}

impl<S: ChainStore> Node<S> {
    /// Membuka node. Penyimpanan kosong diisi genesis jaringan; penyimpanan berisi wajib bergenesis sama. Ringkasan
    /// chain (chainwork, total tambang) disusun ulang dari penyimpanan.
    ///
    /// # Errors
    ///
    /// [`NodeError::Genesis`] jika genesis jaringan belum tersedia (saat ini hanya regtest),
    /// [`NodeError::WrongGenesis`], [`NodeError::Store`], atau [`NodeError::Corrupt`].
    pub fn open(
        store: S,
        params: ChainParams,
        mempool_max_bytes: usize,
        clock: Clock,
    ) -> Result<Self, NodeError> {
        Self::open_with_limits(store, params, mempool_max_bytes, clock, NodeLimits::DEFAULT)
    }

    /// Seperti [`Node::open`], dengan batas memori `limits` untuk blok cabang, indeks header, dan catatan blok tidak
    /// sah.
    ///
    /// # Errors
    ///
    /// Sama dengan [`Node::open`].
    pub fn open_with_limits(
        mut store: S,
        params: ChainParams,
        mempool_max_bytes: usize,
        clock: Clock,
        limits: NodeLimits,
    ) -> Result<Self, NodeError> {
        let genesis = regtest_genesis().map_err(|e| NodeError::Genesis {
            message: e.to_string(),
        })?;
        let genesis_id = block_id(genesis.header());
        if genesis_id != params.network_id() {
            return Err(NodeError::Genesis {
                message: "hanya genesis regtest yang tersedia".to_owned(),
            });
        }
        if store.tip()?.is_none() {
            let txids: Vec<Hash32> = genesis.txs().iter().map(txid).collect();
            store.apply_block(&genesis, genesis_id, &txids)?;
        }
        let found = store.block_id_at(BlockHeight::GENESIS)?;
        if found != Some(genesis_id) {
            return Err(NodeError::WrongGenesis {
                expected: genesis_id,
                found,
            });
        }
        let metas = rebuild_metas(&store)?;
        let heights = index_heights(&metas)?;
        Ok(Self {
            params,
            clock,
            state: Mutex::new(ChainState {
                store,
                mempool: Mempool::new(mempool_max_bytes),
                metas,
                heights,
                side: BTreeMap::new(),
                headers: BTreeMap::new(),
                invalid: BTreeMap::new(),
                invalid_order: VecDeque::new(),
                next_sequence: 0,
                limits,
            }),
        })
    }

    /// Parameter jaringan node.
    #[must_use]
    pub const fn params(&self) -> &ChainParams {
        &self.params
    }

    /// Menutup node dan mengembalikan penyimpanannya.
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika mutex rusak karena thread lain panik.
    pub fn into_store(self) -> Result<S, RpcError> {
        self.state
            .into_inner()
            .map(|state| state.store)
            .map_err(|_| internal("state node rusak"))
    }

    /// Mengunci state node.
    fn lock(&self) -> Result<MutexGuard<'_, ChainState<S>>, RpcError> {
        self.state.lock().map_err(|_| internal("state node rusak"))
    }

    /// Target compact wajib untuk blok setelah induk di `parent_height` bercap waktu `parent_timestamp` (ASERT,
    /// argumen sama dengan V6 di MOD-04).
    fn next_target(&self, parent_height: u64, parent_timestamp: u64) -> Result<u32, RpcError> {
        let target = asert_target(
            &self.params.genesis_target(),
            self.params.genesis_timestamp(),
            BlockHeight::new(parent_height),
            parent_timestamp,
            &self.params.pow_limit(),
        )
        .map_err(internal)?;
        encode_compact(&target).map_err(internal)
    }

    /// Memeriksa ulang mempool terhadap ujung chain yang baru.
    fn revalidate_mempool(&self, state: &mut ChainState<S>) -> Result<(), RpcError> {
        let (tip_height, _) = state.tip()?;
        let next = BlockHeight::new(tip_height)
            .next()
            .ok_or_else(|| internal("tinggi blok meluap"))?;
        let utxos = prefetch(
            &state.store,
            state.mempool.in_arrival_order().map(MempoolEntry::tx),
        )
        .map_err(internal)?;
        state
            .mempool
            .revalidate(next, &self.params.network_id(), &utxos);
        Ok(())
    }

    /// Menyusun ulang mempool setelah reorg: transaksi biasa dari blok yang dilepas (urut seperti di chain lama),
    /// lalu isi mempool lama sesuai urutan kedatangan, masing-masing diperiksa terhadap ujung chain yang baru.
    /// Transaksi yang tidak sah lagi dibuang.
    fn rebuild_mempool(
        &self,
        state: &mut ChainState<S>,
        returned: Vec<Transaction>,
    ) -> Result<(), RpcError> {
        let (tip_height, _) = state.tip()?;
        let next = BlockHeight::new(tip_height)
            .next()
            .ok_or_else(|| internal("tinggi blok meluap"))?;
        let max_bytes = state.mempool.max_bytes();
        let old = std::mem::replace(&mut state.mempool, Mempool::new(max_bytes));
        let mut candidates = returned;
        candidates.extend(old.in_arrival_order().map(|entry| entry.tx().clone()));
        let utxos = prefetch(&state.store, &candidates).map_err(internal)?;
        let network_id = self.params.network_id();
        for tx in candidates {
            let id = txid(&tx);
            // Penolakan berarti transaksi tidak sah lagi di chain baru (misalnya inputnya sudah dibelanjakan).
            let _ = state.mempool.add(tx, id, next, &network_id, &utxos);
        }
        Ok(())
    }

    /// V9 dan V10 terhadap set UTXO ujung chain aktif, lalu menerapkan blok yang induknya ujung chain aktif.
    fn connect(
        &self,
        state: &mut ChainState<S>,
        block: &Block,
        id: Hash32,
    ) -> Result<Connect, RpcError> {
        let (_, tip) = state.tip()?;
        let utxos = prefetch(&state.store, block.txs()).map_err(internal)?;
        let summary = match check_block_transactions(block, &self.params, tip.mined_total, &utxos) {
            Ok(summary) => summary,
            Err(error) => return Ok(Connect::Invalid(block_rule(&error, "V9"))),
        };
        let header = block.header();
        let work = header_work(header).ok_or_else(|| internal("target blok tidak sah"))?;
        let meta = BlockMeta {
            id,
            timestamp: header.timestamp(),
            chainwork: tip
                .chainwork
                .checked_add(work)
                .ok_or_else(|| internal("chainwork meluap"))?,
            mined_total: tip
                .mined_total
                .checked_add(summary.subsidy)
                .ok_or_else(|| internal("total tambang meluap"))?,
        };
        let txids: Vec<Hash32> = block.txs().iter().map(txid).collect();
        state
            .store
            .apply_block(block, id, &txids)
            .map_err(internal)?;
        state.push_meta(meta)?;
        Ok(Connect::Applied)
    }

    /// Reorg ke blok cabang `new_tip`: lepas chain aktif sampai titik cabang, lalu sambungkan cabang. Jika satu blok
    /// cabang tidak sah, blok itu dan keturunannya ditandai tidak sah, dan chain lama dipulihkan.
    fn reorg_to(&self, state: &mut ChainState<S>, new_tip: Hash32) -> Result<(), RpcError> {
        let (fork_height, branch) = state.branch_to(new_tip)?;
        let mut detached: Vec<(Hash32, Block, BlockMeta)> = Vec::new();
        while state.tip()?.0 > fork_height {
            detached.push(state.disconnect_tip()?);
        }
        detached.reverse();

        let mut failure: Option<(Hash32, &'static str)> = None;
        for id in &branch {
            let entry = state
                .side
                .get(id)
                .cloned()
                .ok_or_else(|| internal("blok cabang hilang saat reorg"))?;
            match self.connect(state, &entry.block, *id)? {
                Connect::Applied => {
                    state.side.remove(id);
                }
                Connect::Invalid(rule) => {
                    failure = Some((*id, rule));
                    break;
                }
            }
        }

        if let Some((bad, rule)) = failure {
            state.mark_invalid(bad, rule);
            while state.tip()?.0 > fork_height {
                let (id, block, meta) = state.disconnect_tip()?;
                state.store_side(id, block, meta.chainwork);
            }
            for (id, block, meta) in &detached {
                state.reattach(*id, block, *meta)?;
            }
            return Ok(());
        }

        let mut returned = Vec::new();
        for (id, block, meta) in detached {
            returned.extend(block.txs().iter().filter(|tx| tx.is_regular()).cloned());
            state.store_side(id, block, meta.chainwork);
        }
        self.rebuild_mempool(state, returned)
    }

    /// Selama ada blok cabang dengan chainwork lebih besar daripada ujung chain aktif, reorg ke sana. Setiap
    /// percobaan yang gagal membuang paling sedikit satu blok cabang, sehingga perulangan ini pasti berhenti.
    fn activate_best(&self, state: &mut ChainState<S>) -> Result<(), RpcError> {
        loop {
            let (_, tip) = state.tip()?;
            let Some((candidate, work)) = state.best_side() else {
                return Ok(());
            };
            if work <= tip.chainwork {
                return Ok(());
            }
            self.reorg_to(state, candidate)?;
        }
    }

    /// Ujung chain aktif untuk pesan `Status` P2P.
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika state node rusak.
    pub fn tip_summary(&self) -> Result<TipSummary, RpcError> {
        let state = self.lock()?;
        let (height, tip) = state.tip()?;
        Ok(TipSummary {
            height,
            id: tip.id,
            chainwork: tip.chainwork.to_be_bytes(),
        })
    }

    /// Menerima satu blok (RPC `submit_block` atau P2P), berurutan V1 sampai V10 seperti `submit_block`, dan
    /// melaporkan apakah ujung chain aktif berubah.
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika node gagal memproses blok.
    pub fn receive_block(&self, bytes: &[u8]) -> Result<BlockReceipt, RpcError> {
        // V1.
        let block = match decode_block(bytes) {
            Ok(block) => block,
            Err(error) => {
                return Ok(BlockReceipt {
                    result: invalid(None, block_rule(&error, "V2")),
                    new_tip: false,
                });
            }
        };
        let mut guard = self.lock()?;
        let state = &mut *guard;
        let (_, before) = state.tip()?;
        let result = self.submit_decoded(state, block)?;
        let (_, after) = state.tip()?;
        Ok(BlockReceipt {
            result,
            new_tip: after.id != before.id,
        })
    }

    /// `submit_block` untuk blok yang sudah lolos V1, dengan state node terkunci.
    fn submit_decoded(
        &self,
        state: &mut ChainState<S>,
        block: Block,
    ) -> Result<SubmitBlockResult, RpcError> {
        let header = *block.header();
        let id = block_id(&header);
        if state.known(&id) {
            return Ok(outcome(SubmitStatus::Duplicate, id));
        }
        if let Some(rule) = state.invalid.get(&id) {
            return Ok(invalid(Some(id), rule));
        }
        // V2 sampai V4. Tidak dicatat sebagai tidak sah: blok seperti ini murah dibuat.
        if let Err(error) = check_block_context_free(&block, &self.params) {
            return Ok(invalid(Some(id), block_rule(&error, "V2")));
        }
        let prev = header.prev_block_id();
        if state.invalid.contains_key(&prev) {
            state.note_invalid(id, INVALID_ANCESTOR_RULE);
            return Ok(invalid(Some(id), INVALID_ANCESTOR_RULE));
        }
        let Some(parent) = state.parent_info(&prev)? else {
            return Ok(outcome(SubmitStatus::Orphan, id));
        };
        // V5 sampai V7 terhadap induk, di chain aktif maupun cabang.
        let context = ParentContext {
            block_id: prev,
            height: BlockHeight::new(parent.height),
            timestamps: &parent.timestamps,
        };
        if let Err(error) = check_header_context(&header, &context, &self.params) {
            let rule = block_rule(&error, "V6");
            state.note_invalid(id, rule);
            return Ok(invalid(Some(id), rule));
        }
        // V8: ditunda, bukan tidak sah.
        if check_future_timestamp(&header, (self.clock)()).is_err() {
            return Ok(outcome(SubmitStatus::Deferred, id));
        }
        let (_, tip) = state.tip()?;
        if prev == tip.id {
            // V9 dan V10 terhadap set UTXO ujung chain.
            return match self.connect(state, &block, id)? {
                Connect::Applied => {
                    state.headers.remove(&id);
                    self.revalidate_mempool(state)?;
                    Ok(outcome(SubmitStatus::Accepted, id))
                }
                Connect::Invalid(rule) => {
                    state.note_invalid(id, rule);
                    Ok(invalid(Some(id), rule))
                }
            };
        }
        // Blok cabang: V9 dan V10 menunggu sampai cabangnya disambungkan.
        let work = header_work(&header).ok_or_else(|| internal("target blok tidak sah"))?;
        let chainwork = parent
            .chainwork
            .checked_add(work)
            .ok_or_else(|| internal("chainwork meluap"))?;
        state.store_side(id, block, chainwork);
        state.headers.remove(&id);
        self.activate_best(state)?;
        state.trim_side()?;
        if let Some(rule) = state.invalid.get(&id) {
            return Ok(invalid(Some(id), rule));
        }
        Ok(outcome(SubmitStatus::Accepted, id))
    }

    /// Memasukkan transaksi ke mempool (RPC `send_raw_transaction` atau P2P), diperiksa T1 sampai T6 terhadap ujung
    /// chain aktif dan isi mempool. Mengembalikan txid dan hasilnya.
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika state node rusak atau penyimpanan gagal dibaca.
    pub fn receive_transaction(&self, tx: Transaction) -> Result<(Hash32, TxOutcome), RpcError> {
        let id = txid(&tx);
        let mut guard = self.lock()?;
        let state = &mut *guard;
        let (tip_height, _) = state.tip()?;
        let height = BlockHeight::new(tip_height)
            .next()
            .ok_or_else(|| internal("tinggi blok meluap"))?;
        let utxos = prefetch(&state.store, [&tx]).map_err(internal)?;
        let outcome = match state
            .mempool
            .add(tx, id, height, &self.params.network_id(), &utxos)
        {
            Ok(_) => TxOutcome::Added,
            Err(MempoolError::Duplicate { .. }) => TxOutcome::Duplicate,
            Err(error) => TxOutcome::Rejected(error),
        };
        Ok((id, outcome))
    }
}

impl<S: ChainStore + Send + 'static> RpcBackend for Node<S> {
    fn chain_info(&self) -> Result<ChainInfo, RpcError> {
        let state = self.lock()?;
        let (height, tip) = state.tip()?;
        let median = median_time_past(&state.recent_timestamps())
            .ok_or_else(|| internal("cap waktu chain kosong"))?;
        let next_target = self.next_target(height, tip.timestamp)?;
        let total_supply =
            total_supply_after(tip.mined_total).ok_or_else(|| internal("total pasokan meluap"))?;
        let mempool_count = u64::try_from(state.mempool.len()).map_err(internal)?;
        Ok(ChainInfo {
            network: NetworkName::new(self.params.network()),
            height,
            best_block_id: Hex32::from(tip.id),
            chainwork: Hex32::new(tip.chainwork.to_be_bytes()),
            next_target: CompactHex::new(next_target),
            median_time_past: median,
            mined_total: Quanta::new(tip.mined_total),
            total_supply: Quanta::new(total_supply),
            mempool_count,
        })
    }

    fn block_hash(&self, height: u64) -> Result<Hex32, RpcError> {
        let state = self.lock()?;
        state
            .store
            .block_id_at(BlockHeight::new(height))
            .map_err(internal)?
            .map(Hex32::from)
            .ok_or(RpcError::NotFound)
    }

    fn block(&self, id: Hex32) -> Result<BlockInfo, RpcError> {
        let state = self.lock()?;
        let id = id.to_hash32();
        let block = state
            .store
            .block(&id)
            .map_err(internal)?
            .ok_or(RpcError::NotFound)?;
        let txids: Vec<Hash32> = block.txs().iter().map(txid).collect();
        Ok(BlockInfo::from_block(&block, id, &txids))
    }

    fn block_template(&self, params: &GetBlockTemplateParams) -> Result<BlockTemplate, RpcError> {
        let state = self.lock()?;
        let (tip_height, tip) = state.tip()?;
        let height = BlockHeight::new(tip_height)
            .next()
            .ok_or_else(|| internal("tinggi blok meluap"))?;
        let target = self.next_target(tip_height, tip.timestamp)?;
        let full_target = decode_compact(target).map_err(internal)?.to_be_bytes();
        let median = median_time_past(&state.recent_timestamps())
            .ok_or_else(|| internal("cap waktu chain kosong"))?;
        let min_timestamp = median
            .checked_add(1)
            .ok_or_else(|| internal("cap waktu meluap"))?;
        let timestamp = (self.clock)().max(min_timestamp);
        let subsidy = block_subsidy(height, tip.mined_total).map_err(internal)?;

        let chosen = state
            .mempool
            .select_for_block(MAX_BLOCK_SIZE.saturating_sub(TEMPLATE_RESERVE_BYTES));
        let fees = Amount::checked_sum(chosen.iter().map(|entry| entry.fee()))
            .ok_or_else(|| internal("total biaya meluap"))?;
        let reward = subsidy
            .checked_add(fees)
            .ok_or_else(|| internal("hadiah blok meluap"))?;
        let payout = Output::new(
            reward,
            params.payout_kind.kind(),
            *params.payout_payload.bytes(),
        )
        .map_err(internal)?;
        let coinbase = Transaction::coinbase(height, Vec::new(), vec![payout]).map_err(internal)?;
        let mut txs = vec![coinbase];
        txs.extend(chosen.iter().map(|entry| entry.tx().clone()));

        let draft = Block::new(
            Header::new(height, tip.id, Hash32::ZERO, timestamp, target, 0),
            txs.clone(),
        )
        .map_err(internal)?;
        let merkle = block_merkle_root(&draft).ok_or_else(|| internal("merkle root kosong"))?;
        let block = Block::new(
            Header::new(height, tip.id, merkle, timestamp, target, 0),
            txs,
        )
        .map_err(internal)?;
        Ok(BlockTemplate {
            height: height.get(),
            prev_block_id: Hex32::from(tip.id),
            target: CompactHex::new(target),
            full_target: Hex32::new(full_target),
            min_timestamp,
            subsidy: Quanta::new(subsidy),
            fees: Quanta::new(fees),
            hex: HexBytes::new(block.to_bytes()),
        })
    }

    fn submit_block(&self, bytes: &[u8]) -> Result<SubmitBlockResult, RpcError> {
        self.receive_block(bytes).map(|receipt| receipt.result)
    }

    fn send_raw_transaction(&self, bytes: &[u8]) -> Result<SendRawTransactionResult, RpcError> {
        let tx = Transaction::from_bytes(bytes).map_err(|_| RpcError::InvalidEncoding)?;
        let (id, outcome) = self.receive_transaction(tx)?;
        transaction_result(id, &outcome)
    }

    fn mempool(&self) -> Result<MempoolInfo, RpcError> {
        let state = self.lock()?;
        Ok(MempoolInfo {
            txids: state.mempool.txids().into_iter().map(Hex32::from).collect(),
        })
    }

    fn scan_utxos(&self, payloads: &[Hex32]) -> Result<Vec<UtxoInfo>, RpcError> {
        let wanted: Vec<[u8; 32]> = payloads.iter().map(|payload| *payload.bytes()).collect();
        let state = self.lock()?;
        let (tip_height, _) = state.tip()?;
        let found = state.store.utxos_by_payloads(&wanted).map_err(internal)?;
        found
            .iter()
            .map(|(outpoint, entry)| {
                UtxoInfo::from_entry(outpoint, entry, tip_height)
                    .ok_or_else(|| internal("tinggi UTXO di atas ujung chain"))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
