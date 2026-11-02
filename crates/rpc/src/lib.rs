//! MOD-08: Tipe pesan, encoding nilai, server, dan klien JSON-RPC sesuai Spesifikasi RPC v0.1 (TSK-M12).
//!
//! Server dan klien memakai `jsonrpsee` 0.26 lewat HTTP lokal. Data node disediakan lewat trait [`RpcBackend`],
//! sehingga crate ini tetap hanya bergantung pada MOD-01. Crate ini bukan konsensus.

#![forbid(unsafe_code)]

pub mod client;
pub mod error;
pub mod server;
pub mod transport;
pub mod types;
pub mod value;

pub use client::{RpcClient, error_code, reject_reason};
pub use error::{RejectData, RpcError};
pub use server::{RpcBackend, RpcServer, ServerError, rpc_module};
pub use types::{
    BlockInfo, BlockTemplate, ChainInfo, GetBlockHashParams, GetBlockParams,
    GetBlockTemplateParams, MAX_SCAN_PAYLOADS, METHODS, MempoolInfo, MempoolOutput, MempoolScan,
    MempoolSpend, PayloadList, ScanMempoolParams, ScanUtxosParams, SendRawTransactionParams,
    SendRawTransactionResult, SubmitBlockParams, SubmitBlockResult, SubmitStatus, UtxoInfo,
};
pub use value::{
    CompactHex, DecimalU64, Hex32, HexBytes, KindNumber, NetworkName, Quanta, ValueError,
};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-08";

#[cfg(test)]
mod tests;
