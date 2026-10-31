//! MOD-07: Mempool, antrean transaksi belum terkonfirmasi di memori (TSK-M13).
//!
//! Setiap transaksi diperiksa dengan aturan konsensus MOD-04 (`check_transaction`, T1 sampai T6) terhadap set UTXO
//! ujung chain ditambah output transaksi lain di mempool. Mempool tidak menghitung txid; nilainya diberikan
//! pemanggil (dihitung dengan MOD-02), sama seperti MOD-05. Aturan antrean (batas byte, tanpa penggantian
//! transaksi) bukan konsensus dan bisa berubah tanpa memecah chain.

#![forbid(unsafe_code)]

pub mod pool;

pub use pool::{DEFAULT_MAX_BYTES, Mempool, MempoolEntry, MempoolError, rule_name};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-07";

#[cfg(test)]
mod tests;
