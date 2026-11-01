//! MOD-14: Pustaka explorer Homeostasis Coin (`homeostasis-explorer`), sesuai Spesifikasi Explorer v0.1 (DEC-25).
//!
//! - [`source`]: sumber data chain (node lewat RPC v0.1; adaptornya di TSK-M30).
//! - [`db`]: skema SQLite dan encoding nilai tersimpan.
//! - [`index`]: [`Indexer`], sinkronisasi tahan reorg dan pemeriksaan I1 sampai I4.
//! - [`query`]: bacaan indeks untuk API (blok, transaksi, alamat, pasokan, alokasi genesis, mempool).
//!
//! Explorer bukan konsensus: blok dari node tidak divalidasi ulang, tetapi block_id, txid, input, dan jumlah koin
//! setiap blok diperiksa sendiri, sehingga pasokan dan alokasi genesis bisa diperiksa tanpa percaya pada pencipta.

#![forbid(unsafe_code)]

pub mod db;
pub mod index;
pub mod query;
pub mod source;

pub use db::{DB_FILE, SCHEMA_VERSION, amount_from_blob, amount_to_blob};
pub use index::{IndexError, Indexer, SyncReport};
pub use query::{
    AddressSummary, AddressTx, AllocationInfo, BlockDetail, BlockRow, OutputRow, SpentBy,
    SupplyInfo, TxDetail, TxRow, confirmations,
};
pub use source::{ChainSource, SourceBlock, SourceError};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-14";

#[cfg(test)]
mod test_support;
