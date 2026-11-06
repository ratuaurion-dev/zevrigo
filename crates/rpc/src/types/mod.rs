//! Nama metode, parameter request, dan hasil 9 metode RPC (Spesifikasi RPC v0.1 Bagian 3).
//!
//! Parameter dikirim berdasarkan nama (objek JSON); field yang tidak dikenal ditolak. Hasil tidak menolak field
//! tambahan agar klien lama tetap bisa membaca hasil versi berikutnya.

use serde::{Deserialize, Serialize};
use zevrigo_primitives::block::Block;
use zevrigo_primitives::encoding::Encode;
use zevrigo_primitives::hash::Hash32;
use zevrigo_primitives::params::COINBASE_MATURITY;
use zevrigo_primitives::utxo::{OutPoint, UtxoEntry};

use crate::value::{
    CompactHex, DecimalU64, Hex32, HexBytes, KindNumber, NetworkName, Quanta, ValueError,
};

/// Nama metode `get_chain_info`.
pub const METHOD_GET_CHAIN_INFO: &str = "get_chain_info";
/// Nama metode `get_block_hash`.
pub const METHOD_GET_BLOCK_HASH: &str = "get_block_hash";
/// Nama metode `get_block`.
pub const METHOD_GET_BLOCK: &str = "get_block";
/// Nama metode `get_block_template`.
pub const METHOD_GET_BLOCK_TEMPLATE: &str = "get_block_template";
/// Nama metode `submit_block`.
pub const METHOD_SUBMIT_BLOCK: &str = "submit_block";
/// Nama metode `send_raw_transaction`.
pub const METHOD_SEND_RAW_TRANSACTION: &str = "send_raw_transaction";
/// Nama metode `get_mempool`.
pub const METHOD_GET_MEMPOOL: &str = "get_mempool";
/// Nama metode `scan_utxos`.
pub const METHOD_SCAN_UTXOS: &str = "scan_utxos";
/// Nama metode `scan_mempool`.
pub const METHOD_SCAN_MEMPOOL: &str = "scan_mempool";

/// Kesembilan metode RPC v0.1, urut seperti di spesifikasi.
pub const METHODS: [&str; 9] = [
    METHOD_GET_CHAIN_INFO,
    METHOD_GET_BLOCK_HASH,
    METHOD_GET_BLOCK,
    METHOD_GET_BLOCK_TEMPLATE,
    METHOD_SUBMIT_BLOCK,
    METHOD_SEND_RAW_TRANSACTION,
    METHOD_GET_MEMPOOL,
    METHOD_SCAN_UTXOS,
    METHOD_SCAN_MEMPOOL,
];

/// Jumlah payload maksimum untuk `scan_utxos` dan `scan_mempool`.
pub const MAX_SCAN_PAYLOADS: usize = 64;

/// Parameter `get_block_hash`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetBlockHashParams {
    /// Tinggi blok di chain aktif.
    pub height: u64,
}

/// Parameter `get_block`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetBlockParams {
    /// ID blok di chain aktif.
    pub block_id: Hex32,
}

/// Parameter `get_block_template`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetBlockTemplateParams {
    /// Jenis output penerima hadiah blok.
    pub payout_kind: KindNumber,
    /// Payload output penerima hadiah blok.
    pub payout_payload: Hex32,
}

/// Parameter `submit_block`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitBlockParams {
    /// Encoding kanonis blok.
    pub hex: HexBytes,
}

/// Parameter `send_raw_transaction`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SendRawTransactionParams {
    /// Encoding kanonis transaksi lengkap (dengan witness).
    pub hex: HexBytes,
}

/// Daftar 1 sampai [`MAX_SCAN_PAYLOADS`] payload untuk `scan_utxos` dan `scan_mempool`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<Hex32>", into = "Vec<Hex32>")]
pub struct PayloadList(Vec<Hex32>);

impl PayloadList {
    /// Mengembalikan payload di dalamnya.
    #[must_use]
    pub fn as_slice(&self) -> &[Hex32] {
        &self.0
    }
}

impl TryFrom<Vec<Hex32>> for PayloadList {
    type Error = ValueError;

    fn try_from(payloads: Vec<Hex32>) -> Result<Self, Self::Error> {
        if (1..=MAX_SCAN_PAYLOADS).contains(&payloads.len()) {
            Ok(Self(payloads))
        } else {
            Err(ValueError::PayloadCount {
                actual: payloads.len(),
            })
        }
    }
}

impl From<PayloadList> for Vec<Hex32> {
    fn from(list: PayloadList) -> Self {
        list.0
    }
}

/// Parameter `scan_utxos`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScanUtxosParams {
    /// Payload yang dicari.
    pub payloads: PayloadList,
}

/// Parameter `scan_mempool`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScanMempoolParams {
    /// Payload yang dicari.
    pub payloads: PayloadList,
}

/// Hasil `get_chain_info`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainInfo {
    /// Jaringan node.
    pub network: NetworkName,
    /// Tinggi ujung chain aktif.
    pub height: u64,
    /// ID blok ujung chain aktif.
    pub best_block_id: Hex32,
    /// Chainwork ujung (256-bit big-endian).
    pub chainwork: Hex32,
    /// Target wajib blok berikutnya (ASERT).
    pub next_target: CompactHex,
    /// Median time past ujung; cap waktu blok berikutnya wajib lebih besar.
    pub median_time_past: u64,
    /// Total hasil tambang setelah blok ujung, tanpa alokasi genesis.
    pub mined_total: Quanta,
    /// Alokasi genesis ditambah `mined_total`.
    pub total_supply: Quanta,
    /// Jumlah transaksi di mempool.
    pub mempool_count: u64,
}

/// Hasil `get_block`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockInfo {
    /// ID blok.
    pub block_id: Hex32,
    /// Tinggi blok.
    pub height: u64,
    /// ID blok induk.
    pub prev_block_id: Hex32,
    /// Merkle root.
    pub merkle_root: Hex32,
    /// Cap waktu (detik Unix).
    pub timestamp: u64,
    /// Target compact.
    pub target: CompactHex,
    /// Nonce.
    pub nonce: DecimalU64,
    /// Ukuran encoding blok dalam byte.
    pub size: u64,
    /// Txid semua transaksi, urut seperti di blok.
    pub txids: Vec<Hex32>,
    /// Encoding kanonis blok.
    pub hex: HexBytes,
}

impl BlockInfo {
    /// Menyusun hasil `get_block` dari blok, ID-nya, dan txid semua transaksinya (dihitung pemanggil dengan MOD-02,
    /// urut seperti di blok).
    #[must_use]
    pub fn from_block(block: &Block, block_id: Hash32, txids: &[Hash32]) -> Self {
        let header = block.header();
        let bytes = block.to_bytes();
        Self {
            block_id: Hex32::from(block_id),
            height: header.height().get(),
            prev_block_id: Hex32::from(header.prev_block_id()),
            merkle_root: Hex32::from(header.merkle_root()),
            timestamp: header.timestamp(),
            target: CompactHex::new(header.target()),
            nonce: DecimalU64::new(header.nonce()),
            size: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
            txids: txids.iter().copied().map(Hex32::from).collect(),
            hex: HexBytes::new(bytes),
        }
    }
}

/// Hasil `get_block_template`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockTemplate {
    /// Tinggi blok berikutnya.
    pub height: u64,
    /// ID blok induk (ujung chain aktif).
    pub prev_block_id: Hex32,
    /// Target wajib blok berikutnya.
    pub target: CompactHex,
    /// Target yang sama dalam bentuk 256-bit big-endian (hasil decode `target` oleh node), agar miner tidak perlu
    /// menulis ulang aturan decode compact MOD-04 (TSK-M20).
    pub full_target: Hex32,
    /// Kunci RandomX blok berikutnya (testnet dan mainnet, TSK-M06 bagian C3): `block_id` leluhur di tinggi kunci
    /// (Spesifikasi Validasi Bagian 2), dihitung node agar miner tidak menulis ulang aturan kunci. Regtest (PoW dev):
    /// 32 byte nol.
    pub pow_key: Hex32,
    /// Cap waktu minimum (`median_time_past + 1`).
    pub min_timestamp: u64,
    /// Subsidy blok.
    pub subsidy: Quanta,
    /// Total biaya transaksi yang dimasukkan.
    pub fees: Quanta,
    /// Blok lengkap siap ditambang dengan `nonce = 0`.
    pub hex: HexBytes,
}

/// Status hasil `submit_block`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubmitStatus {
    /// Masuk chain aktif atau cabang yang dikenal.
    Accepted,
    /// Sudah dikenal.
    Duplicate,
    /// Induk belum dikenal.
    Orphan,
    /// Gagal V8 (cap waktu terlalu jauh di depan), diperiksa ulang nanti.
    Deferred,
    /// Tidak sah.
    Invalid,
}

/// Hasil `submit_block`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitBlockResult {
    /// Status blok.
    pub status: SubmitStatus,
    /// ID blok, jika decode berhasil.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_id: Option<Hex32>,
    /// Untuk status `invalid`: nama aturan yang dilanggar, misalnya `V6`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Hasil `send_raw_transaction`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendRawTransactionResult {
    /// Txid transaksi yang diterima mempool.
    pub txid: Hex32,
}

/// Hasil `get_mempool`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MempoolInfo {
    /// Txid transaksi di mempool, urut naik bytewise.
    pub txids: Vec<Hex32>,
}

/// Satu elemen hasil `scan_utxos`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UtxoInfo {
    /// Txid outpoint.
    pub txid: Hex32,
    /// Indeks output outpoint.
    pub index: u32,
    /// Jumlah koin output.
    pub amount: Quanta,
    /// Jenis output.
    pub kind: KindNumber,
    /// Payload output.
    pub payload: Hex32,
    /// Tinggi blok pembuat output.
    pub height: u64,
    /// Apakah output berasal dari coinbase.
    pub is_coinbase: bool,
    /// `tinggi ujung - height + 1`.
    pub confirmations: u64,
    /// Tinggi blok paling awal yang boleh membelanjakannya: `height + 100` untuk coinbase, `height + 1` untuk
    /// lainnya. Kunci waktu kondisi tidak terlihat dari output.
    pub spendable_at: u64,
}

/// UTXO di ujung chain aktif milik salah satu payload yang dicari, yang sudah dibelanjakan transaksi di mempool.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MempoolSpend {
    /// Txid outpoint.
    pub txid: Hex32,
    /// Indeks output outpoint.
    pub index: u32,
    /// Txid transaksi mempool yang membelanjakannya.
    pub spent_by: Hex32,
}

/// Output transaksi di mempool ke salah satu payload yang dicari.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MempoolOutput {
    /// Txid transaksi mempool.
    pub txid: Hex32,
    /// Indeks output.
    pub index: u32,
    /// Jumlah koin output.
    pub amount: Quanta,
    /// Jenis output.
    pub kind: KindNumber,
    /// Payload output.
    pub payload: Hex32,
}

/// Hasil `scan_mempool`: transaksi tertunda yang menyentuh payload yang dicari.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MempoolScan {
    /// UTXO yang sedang dibelanjakan, urut `(txid, index)`.
    pub spent: Vec<MempoolSpend>,
    /// Output tertunda, urut `(txid, index)`.
    pub outputs: Vec<MempoolOutput>,
}

impl UtxoInfo {
    /// Menyusun satu elemen `scan_utxos` dari entri set UTXO dan tinggi ujung chain aktif.
    ///
    /// Mengembalikan `None` jika tinggi output di atas `tip_height` atau `spendable_at` melebihi `u64`.
    #[must_use]
    pub fn from_entry(outpoint: &OutPoint, entry: &UtxoEntry, tip_height: u64) -> Option<Self> {
        let height = entry.height().get();
        let confirmations = tip_height.checked_sub(height)?.checked_add(1)?;
        let delay = if entry.is_coinbase() {
            COINBASE_MATURITY
        } else {
            1
        };
        let spendable_at = height.checked_add(delay)?;
        let output = entry.output();
        Some(Self {
            txid: Hex32::from(outpoint.txid()),
            index: outpoint.index(),
            amount: Quanta::new(output.amount()),
            kind: KindNumber::new(output.kind()),
            payload: Hex32::new(*output.payload()),
            height,
            is_coinbase: entry.is_coinbase(),
            confirmations,
            spendable_at,
        })
    }
}

#[cfg(test)]
mod tests;
