//! Logika chain node (TSK-M18 bagian A): chain aktif di atas [`ChainStore`], mempool, dan implementasi
//! [`RpcBackend`] untuk 8 metode Spesifikasi RPC v0.1.
//!
//! Aturan konsensus tidak ditulis di sini. Blok diperiksa dengan fungsi MOD-04 berurutan V1 sampai V10
//! (Spesifikasi Validasi Bagian 9), transaksi dengan mempool MOD-07 (T1 sampai T6). Node ini mengikuti satu chain:
//! blok yang induknya bukan ujung chain aktif dilaporkan `orphan` dan tidak disimpan sampai TSK-F04 (multi-node dan
//! reorg). Blok yang gagal V8 dilaporkan `deferred` dan tidak disimpan; pengirim boleh mengirim ulang.

use std::collections::BTreeMap;
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

/// Ruang yang disisihkan dari ukuran blok saat memilih transaksi mempool untuk template: header 96 byte, jumlah
/// transaksi, dan coinbase (sekitar 70 byte), dengan cadangan.
const TEMPLATE_RESERVE_BYTES: usize = 1_000;

/// Jam lokal node dalam detik Unix, untuk aturan V8 dan cap waktu template. Disuntikkan agar test deterministik.
pub type Clock = Box<dyn Fn() -> u64 + Send + Sync>;

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

/// Isi node yang dijaga satu mutex: penyimpanan, mempool, dan ringkasan chain aktif.
struct ChainState<S> {
    store: S,
    mempool: Mempool,
    metas: Vec<BlockMeta>,
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
        mut store: S,
        params: ChainParams,
        mempool_max_bytes: usize,
        clock: Clock,
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
        Ok(Self {
            params,
            clock,
            state: Mutex::new(ChainState {
                store,
                mempool: Mempool::new(mempool_max_bytes),
                metas,
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
        // V1.
        let block = match decode_block(bytes) {
            Ok(block) => block,
            Err(error) => return Ok(invalid(None, block_rule(&error, "V2"))),
        };
        let header = *block.header();
        let id = block_id(&header);
        let mut guard = self.lock()?;
        let state = &mut *guard;
        if state.store.block(&id).map_err(internal)?.is_some() {
            return Ok(outcome(SubmitStatus::Duplicate, id));
        }
        // V2 sampai V4.
        if let Err(error) = check_block_context_free(&block, &self.params) {
            return Ok(invalid(Some(id), block_rule(&error, "V2")));
        }
        let (tip_height, tip) = state.tip()?;
        if header.prev_block_id() != tip.id {
            return Ok(outcome(SubmitStatus::Orphan, id));
        }
        // V5 sampai V7.
        let timestamps = state.recent_timestamps();
        let parent = ParentContext {
            block_id: tip.id,
            height: BlockHeight::new(tip_height),
            timestamps: &timestamps,
        };
        if let Err(error) = check_header_context(&header, &parent, &self.params) {
            return Ok(invalid(Some(id), block_rule(&error, "V6")));
        }
        // V8: ditunda, bukan tidak sah.
        if check_future_timestamp(&header, (self.clock)()).is_err() {
            return Ok(outcome(SubmitStatus::Deferred, id));
        }
        // V9 dan V10 terhadap set UTXO ujung chain.
        let utxos = prefetch(&state.store, block.txs()).map_err(internal)?;
        let summary = match check_block_transactions(&block, &self.params, tip.mined_total, &utxos)
        {
            Ok(summary) => summary,
            Err(error) => return Ok(invalid(Some(id), block_rule(&error, "V9"))),
        };
        let work = header_work(&header).ok_or_else(|| internal("target blok tidak sah"))?;
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
            .apply_block(&block, id, &txids)
            .map_err(internal)?;
        state.metas.push(meta);
        self.revalidate_mempool(state)?;
        Ok(outcome(SubmitStatus::Accepted, id))
    }

    fn send_raw_transaction(&self, bytes: &[u8]) -> Result<SendRawTransactionResult, RpcError> {
        let tx = Transaction::from_bytes(bytes).map_err(|_| RpcError::InvalidEncoding)?;
        let id = txid(&tx);
        let mut guard = self.lock()?;
        let state = &mut *guard;
        let (tip_height, _) = state.tip()?;
        let height = BlockHeight::new(tip_height)
            .next()
            .ok_or_else(|| internal("tinggi blok meluap"))?;
        let utxos = prefetch(&state.store, [&tx]).map_err(internal)?;
        match state
            .mempool
            .add(tx, id, height, &self.params.network_id(), &utxos)
        {
            // Duplikat dianggap berhasil: transaksinya memang sudah di mempool.
            Ok(_) | Err(MempoolError::Duplicate { .. }) => Ok(SendRawTransactionResult {
                txid: Hex32::from(id),
            }),
            Err(error) => Err(RpcError::InvalidTransaction {
                reason: mempool_reason(&error).to_owned(),
            }),
        }
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
