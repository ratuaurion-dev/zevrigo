//! Kode error JSON-RPC (Spesifikasi RPC v0.1 Bagian 4), isi `data` untuk transaksi yang ditolak, dan
//! [`RpcError`] yang dikembalikan backend node.

use jsonrpsee::server::types::ErrorObject;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

/// Error dari backend node, dipetakan ke kode JSON-RPC oleh server.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum RpcError {
    /// Blok atau data yang diminta tidak ada di chain aktif (`1001`).
    #[error("tidak ditemukan")]
    NotFound,
    /// Transaksi tidak sah (`1002`), dengan nama aturan yang dilanggar.
    #[error("transaksi tidak sah: {reason}")]
    InvalidTransaction {
        /// Nama aturan MOD-04 yang dilanggar, misalnya `T6`.
        reason: String,
    },
    /// Byte bukan encoding kanonis yang sah (`1003`).
    #[error("hex atau encoding kanonis tidak sah")]
    InvalidEncoding,
    /// Galat internal node (`-32603`), misalnya kegagalan penyimpanan.
    #[error("galat internal: {message}")]
    Internal {
        /// Keterangan galat.
        message: String,
    },
}

impl RpcError {
    /// Kode JSON-RPC untuk error ini.
    #[must_use]
    pub const fn code(&self) -> i32 {
        match self {
            Self::NotFound => NOT_FOUND,
            Self::InvalidTransaction { .. } => INVALID_TRANSACTION,
            Self::InvalidEncoding => INVALID_ENCODING,
            Self::Internal { .. } => INTERNAL_ERROR,
        }
    }
}

impl From<RpcError> for ErrorObject<'static> {
    fn from(err: RpcError) -> Self {
        let code = err.code();
        let message = err.to_string();
        let data = match err {
            RpcError::InvalidTransaction { reason } => Some(RejectData { reason }),
            RpcError::NotFound | RpcError::InvalidEncoding | RpcError::Internal { .. } => None,
        };
        ErrorObject::owned(code, message, data)
    }
}

#[cfg(test)]
mod tests;
