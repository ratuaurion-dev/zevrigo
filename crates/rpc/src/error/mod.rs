//! Kode error JSON-RPC (Spesifikasi RPC v0.1 Bagian 4) dan isi `data` untuk transaksi yang ditolak.

use serde::{Deserialize, Serialize};

/// `-32700`: JSON rusak.
pub const PARSE_ERROR: i32 = -32_700;
/// `-32600`: request tidak sah.
pub const INVALID_REQUEST: i32 = -32_600;
/// `-32601`: metode tidak ada.
pub const METHOD_NOT_FOUND: i32 = -32_601;
/// `-32602`: parameter tidak sah, termasuk nilai yang melanggar encoding Bagian 2.
pub const INVALID_PARAMS: i32 = -32_602;
/// `-32603`: galat internal node.
pub const INTERNAL_ERROR: i32 = -32_603;
/// `1001`: blok atau data yang diminta tidak ditemukan.
pub const NOT_FOUND: i32 = 1_001;
/// `1002`: transaksi tidak sah; `data` berisi [`RejectData`].
pub const INVALID_TRANSACTION: i32 = 1_002;
/// `1003`: hex atau encoding kanonis tidak sah.
pub const INVALID_ENCODING: i32 = 1_003;

/// Isi field `data` pada error `1002`: nama aturan yang dilanggar, misalnya `T6`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectData {
    /// Nama aturan MOD-04 yang dilanggar (`V1` sampai `V10`, `T1` sampai `T6`).
    pub reason: String,
}
