//! MOD-08: Tipe pesan dan encoding nilai JSON-RPC sesuai Spesifikasi RPC v0.1 (TSK-M12 bagian A).
//!
//! Server dan klien `jsonrpsee` menyusul di TSK-M12 bagian B. Crate ini bukan konsensus.

#![forbid(unsafe_code)]

pub mod error;
pub mod transport;
pub mod types;
pub mod value;

pub use error::RejectData;
pub use types::{
    BlockInfo, BlockTemplate, ChainInfo, GetBlockHashParams, GetBlockParams,
    GetBlockTemplateParams, MAX_SCAN_PAYLOADS, METHODS, MempoolInfo, PayloadList, ScanUtxosParams,
    SendRawTransactionParams, SendRawTransactionResult, SubmitBlockParams, SubmitBlockResult,
    SubmitStatus, UtxoInfo,
};
pub use value::{
    CompactHex, DecimalU64, Hex32, HexBytes, KindNumber, NetworkName, Quanta, ValueError,
};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-08";

#[cfg(test)]
mod tests;
