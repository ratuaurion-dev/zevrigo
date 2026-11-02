//! MOD-14: Pustaka explorer Homeostasis Coin (`homeostasis-explorer`), sesuai Spesifikasi Explorer v0.1 (DEC-25).
//!
//! - [`source`]: sumber data chain untuk indexer; [`rpc_source`]: node lewat RPC v0.1.
//! - [`db`]: skema SQLite dan encoding nilai tersimpan.
//! - [`index`]: [`Indexer`], sinkronisasi tahan reorg dan pemeriksaan I1 sampai I4.
//! - [`query`]: bacaan indeks (blok, transaksi, alamat, pasokan, alokasi genesis, mempool).
//! - [`api`]: API HTTP baca saja sebagai fungsi murni.
//! - [`cli`] dan [`server`]: argumen, server axum, sinkronisasi berkala, dan program `homeostasis-explorer`.
//! - [`web`]: penyajian file antarmuka web (hasil build `apps/explorer/web`) dan kebijakan keamanan konten.
//!
//! Explorer bukan konsensus: blok dari node tidak divalidasi ulang, tetapi block_id, txid, input, dan jumlah koin
//! setiap blok diperiksa sendiri, sehingga pasokan dan alokasi genesis bisa diperiksa tanpa percaya pada pencipta.

#![forbid(unsafe_code)]

pub mod api;
pub mod cli;
pub mod db;
pub mod index;
pub mod query;
pub mod rpc_source;
pub mod server;
pub mod source;
pub mod web;

pub use api::{ApiResponse, NodeView, SyncStatus, respond};
pub use cli::{
    ArgError, Command, Config, USAGE, db_path, default_http_port, default_rpc_url, parse_args,
};
pub use db::{DB_FILE, SCHEMA_VERSION, amount_from_blob, amount_to_blob};
pub use index::{IndexError, Indexer, SyncReport};
pub use query::{
    AddressSummary, AddressTx, AllocationInfo, BlockDetail, BlockRow, OutputRow, SpentBy,
    SupplyInfo, TxDetail, TxRow, confirmations,
};
pub use rpc_source::RpcSource;
pub use server::{ExplorerError, SYNC_INTERVAL, Shared, init_logging, router, run};
pub use source::{ChainSource, SourceBlock, SourceError};
pub use web::{CONTENT_SECURITY_POLICY, INDEX_FILE, WebResponse};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-14";

#[cfg(test)]
mod test_support;
