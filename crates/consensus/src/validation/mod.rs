//! Validasi transaksi dan blok (Spesifikasi Validasi v0.1 Bagian 7 sampai 9, DEC-09, DEC-17).
//!
//! Tiga tahap, dipanggil berurutan oleh node:
//! 1. Bebas konteks, V1–V4: [`decode_block`] dan [`check_block_context_free`].
//! 2. Konteks header, V5–V7: [`check_header_context`]. V8 terpisah di [`check_future_timestamp`], karena hanya
//!    menunda blok baru dan tidak dipakai saat memeriksa ulang chain yang tersimpan.
//! 3. Konteks UTXO, V9–V10: [`check_block_transactions`], dengan T1–T6 per transaksi di [`check_transaction`].
//!
//! MOD-04 tidak membaca penyimpanan maupun jam sistem. Output yang dibelanjakan diberikan pemanggil lewat
//! [`UtxoLookup`], dan `now` untuk V8 juga diberikan pemanggil.

use std::collections::{BTreeMap, BTreeSet};

use homeostasis_crypto::{
    PublicKey, SighashError, Signature, block_merkle_root, condition_hash, sighash_v1, txid, verify,
};
use homeostasis_pow::verify_regtest_pow;
use homeostasis_primitives::params::COINBASE_MATURITY;
use homeostasis_primitives::{
    Amount, Block, BlockHeight, Decode, DecodeError, Hash32, Header, Network, OutPoint, Output,
    OutputKind, Transaction, TxBody, UtxoEntry, Witness,
};

use crate::chain_params::ChainParams;
use crate::difficulty::{
    DifficultyError, asert_target, decode_compact, encode_compact, median_time_past,
    timestamp_within_future_limit,
};
use crate::supply::{SupplyError, block_subsidy};

/// Galat validasi satu transaksi biasa (Spesifikasi Validasi Bagian 7, T1–T6).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TxError {
    /// Coinbase tidak diperiksa dengan aturan transaksi biasa.
    #[error("bukan transaksi biasa")]
    NotRegular,
    /// Jumlah witness tidak sama dengan jumlah input (dijamin decode, tetap dijaga).
    #[error("jumlah witness tidak sama dengan jumlah input")]
    WitnessCountMismatch,
    /// Indeks input tidak muat di `u32`.
    #[error("indeks input melebihi u32")]
    IndexOverflow,
    /// T1: outpoint yang sama dibelanjakan dua kali di dalam satu transaksi.
    #[error("input ganda dalam satu transaksi: {outpoint:?}")]
    DuplicateInput {
        /// Outpoint yang dirujuk lebih dari sekali.
        outpoint: OutPoint,
    },
    /// T1: output yang dibelanjakan tidak ada di view UTXO.
    #[error("output yang dibelanjakan tidak ada: {outpoint:?}")]
    MissingUtxo {
        /// Outpoint yang tidak ditemukan.
        outpoint: OutPoint,
    },
    /// T2: output coinbase dibelanjakan sebelum 100 blok.
    #[error("output coinbase belum matang: {outpoint:?}")]
    ImmatureCoinbase {
        /// Outpoint coinbase yang dibelanjakan terlalu cepat.
        outpoint: OutPoint,
    },
    /// T3: jenis witness tidak sama dengan jenis output yang dibelanjakan.
    #[error("jenis witness input {input_index} tidak sama dengan jenis output")]
    WitnessKindMismatch {
        /// Indeks input.
        input_index: u32,
    },
    /// T4: hash kondisi yang diungkapkan tidak sama dengan payload output.
    #[error("kondisi input {input_index} tidak cocok dengan output")]
    ConditionHashMismatch {
        /// Indeks input.
        input_index: u32,
    },
    /// T4: kunci waktu kondisi belum terpenuhi.
    #[error("kunci waktu input {input_index} belum terpenuhi")]
    ConditionLocked {
        /// Indeks input.
        input_index: u32,
    },
    /// T5: penjumlahan jumlah koin meluap.
    #[error("luapan saat menjumlahkan jumlah koin")]
    AmountOverflow,
    /// T5: total output lebih besar dari total input.
    #[error("total output melebihi total input")]
    OutputsExceedInputs,
    /// Pesan sighash tidak bisa disusun.
    #[error("sighash tidak bisa disusun: {0}")]
    Sighash(#[from] SighashError),
    /// T6: tanda tangan atau kunci tidak sah.
    #[error("tanda tangan input {input_index} tidak sah")]
    InvalidSignature {
        /// Indeks input.
        input_index: u32,
    },
}

/// Galat validasi blok (Spesifikasi Validasi Bagian 9, V1–V10).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum BlockError {
    /// V1: decode kanonis gagal.
    #[error("blok tidak bisa di-decode: {0}")]
    Decode(#[from] DecodeError),
    /// V2 atau V6: target compact tidak sah, atau target ASERT tidak bisa dihitung.
    #[error("target tidak sah: {0}")]
    Difficulty(#[from] DifficultyError),
    /// V2: target header di atas `pow_limit` jaringan.
    #[error("target header di atas pow_limit")]
    TargetAboveLimit,
    /// V3: PoW jaringan ini belum ada (DEC-01). Hash PoW regtest tidak pernah diterima di sini (DEC-09).
    #[error("PoW untuk jaringan {network:?} belum tersedia")]
    PowUnavailable {
        /// Jaringan yang PoW-nya belum ada.
        network: Network,
    },
    /// V3: hash PoW di atas target.
    #[error("hash PoW di atas target")]
    HighHash,
    /// V4: Merkle root header tidak sama dengan isi blok.
    #[error("merkle root header tidak sama dengan isi blok")]
    MerkleRootMismatch,
    /// V5: `prev_block_id` bukan ID blok induk.
    #[error("prev_block_id bukan ID blok induk")]
    PrevBlockMismatch,
    /// V5: tinggi bukan tinggi induk + 1.
    #[error("tinggi blok {actual} tidak menyambung ke induk {parent}")]
    HeightMismatch {
        /// Tinggi blok induk.
        parent: BlockHeight,
        /// Tinggi di header.
        actual: BlockHeight,
    },
    /// V6: target header bukan target ASERT.
    #[error("target header {actual:#010x}, seharusnya {expected:#010x}")]
    TargetMismatch {
        /// Target ASERT dalam format compact.
        expected: u32,
        /// Target di header.
        actual: u32,
    },
    /// Konteks induk tidak memuat cap waktu.
    #[error("cap waktu blok induk tidak diberikan")]
    MissingParentTimestamps,
    /// V7: cap waktu tidak lebih besar dari median 11 blok terakhir.
    #[error("cap waktu {timestamp} tidak lebih besar dari median {median}")]
    TimestampNotAfterMedian {
        /// Cap waktu header.
        timestamp: u64,
        /// Median cap waktu blok sebelumnya.
        median: u64,
    },
    /// V8: cap waktu lebih dari 1 jam di depan jam lokal. Blok ditunda, bukan ditandai tidak sah.
    #[error("cap waktu {timestamp} lebih dari 1 jam di depan jam lokal {now}")]
    TimestampTooFarInFuture {
        /// Cap waktu header.
        timestamp: u64,
        /// Jam lokal node.
        now: u64,
    },
    /// Genesis tidak divalidasi dengan aturan ini; node membandingkan `block_id`-nya dengan genesis jaringan.
    #[error("genesis tidak divalidasi dengan aturan blok biasa")]
    Genesis,
    /// V9: transaksi di posisi `index` (coinbase = 0) tidak sah.
    #[error("transaksi ke-{index} tidak sah: {error}")]
    Transaction {
        /// Posisi transaksi di blok.
        index: usize,
        /// Aturan yang dilanggar.
        error: TxError,
    },
    /// Outpoint baru sudah ada di view blok.
    #[error("outpoint baru sudah ada: {outpoint:?}")]
    DuplicateOutpoint {
        /// Outpoint yang duplikat.
        outpoint: OutPoint,
    },
    /// Indeks output atau posisi transaksi tidak muat di tipenya.
    #[error("indeks output melebihi batas")]
    OutputIndexOverflow,
    /// Penjumlahan biaya atau output coinbase meluap.
    #[error("luapan saat menjumlahkan jumlah koin")]
    AmountOverflow,
    /// Subsidy tidak bisa dihitung.
    #[error("subsidy tidak bisa dihitung: {0}")]
    Subsidy(#[from] SupplyError),
    /// V10 (C1, DEC-17): output coinbase tidak tepat sama dengan subsidy + total biaya.
    #[error("output coinbase {actual:?}, seharusnya tepat {expected:?}")]
    CoinbaseAmountMismatch {
        /// Subsidy + total biaya.
        expected: Amount,
        /// Total output coinbase.
        actual: Amount,
    },
}

/// Data blok induk untuk tahap konteks header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParentContext<'a> {
    /// ID blok induk.
    pub block_id: Hash32,
    /// Tinggi blok induk.
    pub height: BlockHeight,
    /// Cap waktu blok terakhir sampai induk, urut dari lama ke baru; elemen terakhir milik induk.
    /// Cukup 11 terakhir; yang lebih lama diabaikan.
    pub timestamps: &'a [u64],
}

/// Ringkasan blok yang lolos tahap konteks UTXO.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockSummary {
    /// Subsidy tambang blok ini.
    pub subsidy: Amount,
    /// Total biaya semua transaksi biasa di blok ini.
    pub fees: Amount,
}

/// Sumber output yang belum dibelanjakan untuk validasi.
///
/// Untuk blok, ini set UTXO di ujung induk. Pemanggil (node) bisa mengambil dulu entri yang dirujuk input
/// blok dari penyimpanan ke `BTreeMap`, sehingga validasi tidak bergantung pada MOD-05.
pub trait UtxoLookup {
    /// Mengembalikan entri UTXO untuk `outpoint`, atau `None` jika tidak ada.
    fn lookup(&self, outpoint: &OutPoint) -> Option<UtxoEntry>;
}

impl UtxoLookup for BTreeMap<OutPoint, UtxoEntry> {
    fn lookup(&self, outpoint: &OutPoint) -> Option<UtxoEntry> {
        self.get(outpoint).cloned()
    }
}

/// V1: decode kanonis blok, termasuk ukuran ≤ 100_000 byte dan batas struktur Spesifikasi Format Bagian 10.
///
/// # Errors
///
/// [`BlockError::Decode`] jika byte tidak sah.
pub fn decode_block(bytes: &[u8]) -> Result<Block, BlockError> {
    Ok(Block::from_bytes(bytes)?)
}

/// V2–V4: target sah dan ≤ `pow_limit`, PoW, dan Merkle root.
///
/// PoW regtest hanya diterima jika `params.network()` adalah regtest. Testnet dan mainnet selalu
/// [`BlockError::PowUnavailable`] sampai DEC-01 (DEC-09).
///
/// # Errors
///
/// Galat aturan pertama yang dilanggar, berurutan V2, V3, V4.
pub fn check_block_context_free(block: &Block, params: &ChainParams) -> Result<(), BlockError> {
    let header = block.header();
    let target = decode_compact(header.target())?;
    if target > params.pow_limit() {
        return Err(BlockError::TargetAboveLimit);
    }
    match params.network() {
        Network::Regtest => {
            verify_regtest_pow(header, &target).map_err(|_| BlockError::HighHash)?;
        }
        network @ (Network::Testnet | Network::Mainnet) => {
            return Err(BlockError::PowUnavailable { network });
        }
    }
    if block_merkle_root(block) != Some(header.merkle_root()) {
        return Err(BlockError::MerkleRootMismatch);
    }
    Ok(())
}

/// V5–V7: menyambung ke induk, target sama dengan ASERT, dan cap waktu lebih besar dari median.
///
/// # Errors
///
/// Galat aturan pertama yang dilanggar, berurutan V5, V6, V7.
pub fn check_header_context(
    header: &Header,
    parent: &ParentContext<'_>,
    params: &ChainParams,
) -> Result<(), BlockError> {
    if header.prev_block_id() != parent.block_id {
        return Err(BlockError::PrevBlockMismatch);
    }
    if parent.height.next() != Some(header.height()) {
        return Err(BlockError::HeightMismatch {
            parent: parent.height,
            actual: header.height(),
        });
    }
    let parent_timestamp = parent
        .timestamps
        .last()
        .copied()
        .ok_or(BlockError::MissingParentTimestamps)?;
    let next = asert_target(
        &params.genesis_target(),
        params.genesis_timestamp(),
        parent.height,
        parent_timestamp,
        &params.pow_limit(),
    )?;
    let expected = encode_compact(&next)?;
    if header.target() != expected {
        return Err(BlockError::TargetMismatch {
            expected,
            actual: header.target(),
        });
    }
    let median = median_time_past(parent.timestamps).ok_or(BlockError::MissingParentTimestamps)?;
    if header.timestamp() <= median {
        return Err(BlockError::TimestampNotAfterMedian {
            timestamp: header.timestamp(),
            median,
        });
    }
    Ok(())
}

/// V8: cap waktu ≤ `now + 3_600`. Hanya untuk blok baru; blok yang gagal ditunda, bukan ditandai tidak sah.
///
/// # Errors
///
/// [`BlockError::TimestampTooFarInFuture`] jika cap waktu terlalu jauh di depan.
pub fn check_future_timestamp(header: &Header, now: u64) -> Result<(), BlockError> {
    if timestamp_within_future_limit(header.timestamp(), now) {
        Ok(())
    } else {
        Err(BlockError::TimestampTooFarInFuture {
            timestamp: header.timestamp(),
            now,
        })
    }
}

/// T1–T6 untuk satu transaksi biasa yang dimuat di blok setinggi `height`. Mengembalikan biayanya.
///
/// Dipakai juga oleh mempool (MOD-07) dengan `height` = tinggi blok berikutnya.
///
/// # Errors
///
/// [`TxError`] untuk aturan pertama yang dilanggar, berurutan T1–T2 per input, T3–T4 per input, T5, lalu T6.
pub fn check_transaction<U: UtxoLookup>(
    tx: &Transaction,
    height: BlockHeight,
    network_id: &Hash32,
    utxos: &U,
) -> Result<Amount, TxError> {
    let TxBody::Regular(body) = tx.body() else {
        return Err(TxError::NotRegular);
    };
    let witnesses = tx.witnesses();
    if witnesses.len() != body.inputs().len() {
        return Err(TxError::WitnessCountMismatch);
    }

    // T1 dan T2.
    let mut seen = BTreeSet::new();
    let mut spent: Vec<Output> = Vec::with_capacity(body.inputs().len());
    for input in body.inputs() {
        let outpoint = OutPoint::from_input(input);
        if !seen.insert(outpoint) {
            return Err(TxError::DuplicateInput { outpoint });
        }
        let entry = utxos
            .lookup(&outpoint)
            .ok_or(TxError::MissingUtxo { outpoint })?;
        if entry.is_coinbase() {
            let mature_at = entry
                .height()
                .checked_add(COINBASE_MATURITY)
                .ok_or(TxError::ImmatureCoinbase { outpoint })?;
            if height < mature_at {
                return Err(TxError::ImmatureCoinbase { outpoint });
            }
        }
        spent.push(entry.output().clone());
    }

    // T3 dan T4.
    for (index, (witness, output)) in witnesses.iter().zip(&spent).enumerate() {
        let input_index = input_index(index)?;
        match (witness, output.kind()) {
            (Witness::SingleKey(_), OutputKind::SingleKey) => {}
            (Witness::Condition(revealed), OutputKind::ConditionHash) => {
                if condition_hash(revealed.condition()).as_bytes() != output.payload() {
                    return Err(TxError::ConditionHashMismatch { input_index });
                }
                if height < revealed.condition().lock_height() {
                    return Err(TxError::ConditionLocked { input_index });
                }
            }
            _ => return Err(TxError::WitnessKindMismatch { input_index }),
        }
    }

    // T5.
    let total_in =
        Amount::checked_sum(spent.iter().map(Output::amount)).ok_or(TxError::AmountOverflow)?;
    let total_out = Amount::checked_sum(body.outputs().iter().map(Output::amount))
        .ok_or(TxError::AmountOverflow)?;
    let fee = total_in
        .checked_sub(total_out)
        .ok_or(TxError::OutputsExceedInputs)?;

    // T6, terakhir karena paling mahal.
    for (index, (witness, output)) in witnesses.iter().zip(&spent).enumerate() {
        let input_index = input_index(index)?;
        let msg = sighash_v1(network_id, tx, input_index, &spent)?;
        let valid = match witness {
            Witness::SingleKey(sig) => verify_key(output.payload(), msg.as_bytes(), sig),
            Witness::Condition(revealed) => revealed.sigs().iter().all(|entry| {
                revealed
                    .condition()
                    .keys()
                    .get(usize::from(entry.key_index()))
                    .is_some_and(|key| verify_key(key, msg.as_bytes(), entry.sig()))
            }),
        };
        if !valid {
            return Err(TxError::InvalidSignature { input_index });
        }
    }
    Ok(fee)
}

/// V9–V10: semua transaksi biasa lolos T1–T6 berurutan, lalu coinbase mengklaim tepat subsidy + biaya (DEC-17).
///
/// `utxos` adalah set UTXO di ujung induk. `mined_before` adalah `M_h` = `mined_total_after(h − 1)` dari
/// [`crate::SupplySchedule`].
///
/// # Errors
///
/// [`BlockError::Genesis`] untuk tinggi 0, [`BlockError::Transaction`] untuk transaksi pertama yang tidak sah,
/// lalu [`BlockError::CoinbaseAmountMismatch`].
pub fn check_block_transactions<U: UtxoLookup>(
    block: &Block,
    params: &ChainParams,
    mined_before: Amount,
    utxos: &U,
) -> Result<BlockSummary, BlockError> {
    let height = block.header().height();
    if height == BlockHeight::GENESIS {
        return Err(BlockError::Genesis);
    }
    let (coinbase, regular) = block
        .txs()
        .split_first()
        .ok_or(BlockError::Decode(DecodeError::EmptyBlock))?;
    let network_id = params.network_id();
    let mut view = BlockView {
        parent: utxos,
        created: BTreeMap::new(),
        spent: BTreeSet::new(),
    };
    view.add_outputs(coinbase, height, true)?;

    let mut fees = Amount::ZERO;
    for (position, tx) in regular.iter().enumerate() {
        let index = position
            .checked_add(1)
            .ok_or(BlockError::OutputIndexOverflow)?;
        let fee = check_transaction(tx, height, &network_id, &view)
            .map_err(|error| BlockError::Transaction { index, error })?;
        fees = fees.checked_add(fee).ok_or(BlockError::AmountOverflow)?;
        view.spend_inputs(tx);
        view.add_outputs(tx, height, false)?;
    }

    let subsidy = block_subsidy(height, mined_before)?;
    let expected = subsidy
        .checked_add(fees)
        .ok_or(BlockError::AmountOverflow)?;
    let actual = Amount::checked_sum(coinbase.body().outputs().iter().map(Output::amount))
        .ok_or(BlockError::AmountOverflow)?;
    if actual != expected {
        return Err(BlockError::CoinbaseAmountMismatch { expected, actual });
    }
    Ok(BlockSummary { subsidy, fees })
}

/// View UTXO selama satu blok: set induk + output yang dibuat di blok − yang sudah dibelanjakan di blok.
struct BlockView<'a, U> {
    parent: &'a U,
    created: BTreeMap<OutPoint, UtxoEntry>,
    spent: BTreeSet<OutPoint>,
}

impl<U: UtxoLookup> BlockView<'_, U> {
    fn add_outputs(
        &mut self,
        tx: &Transaction,
        height: BlockHeight,
        is_coinbase: bool,
    ) -> Result<(), BlockError> {
        let id = txid(tx);
        for (index, output) in tx.body().outputs().iter().enumerate() {
            let index = u32::try_from(index).map_err(|_| BlockError::OutputIndexOverflow)?;
            let outpoint = OutPoint::new(id, index);
            let entry = UtxoEntry::new(output.clone(), height, is_coinbase);
            if self.created.insert(outpoint, entry).is_some() {
                return Err(BlockError::DuplicateOutpoint { outpoint });
            }
        }
        Ok(())
    }

    fn spend_inputs(&mut self, tx: &Transaction) {
        if let TxBody::Regular(body) = tx.body() {
            for input in body.inputs() {
                let outpoint = OutPoint::from_input(input);
                self.created.remove(&outpoint);
                self.spent.insert(outpoint);
            }
        }
    }
}

impl<U: UtxoLookup> UtxoLookup for BlockView<'_, U> {
    fn lookup(&self, outpoint: &OutPoint) -> Option<UtxoEntry> {
        if self.spent.contains(outpoint) {
            return None;
        }
        self.created
            .get(outpoint)
            .cloned()
            .or_else(|| self.parent.lookup(outpoint))
    }
}

fn input_index(index: usize) -> Result<u32, TxError> {
    u32::try_from(index).map_err(|_| TxError::IndexOverflow)
}

/// Kunci x-only tidak sah dianggap tanda tangan tidak sah (koin di output itu hangus).
fn verify_key(key: &[u8; 32], msg: &[u8; 32], sig: &[u8; 64]) -> bool {
    PublicKey::from_bytes(key).is_ok_and(|public| verify(&public, msg, &Signature::from_bytes(sig)))
}

#[cfg(test)]
mod tests;
