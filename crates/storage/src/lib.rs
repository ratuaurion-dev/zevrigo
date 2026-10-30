//! MOD-05: Penyimpanan blok, set UTXO, dan undo data. Berisi trait ChainStore, MemoryStore (TSK-M10), RedbStore berbasis redb (TSK-M11, DEC-03), dan pencarian UTXO per payload (TSK-M26).

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

mod codec;
pub mod memory;
pub mod redb_store;
pub mod store;

pub use memory::MemoryStore;
pub use redb_store::RedbStore;
pub use store::{BlockUndo, ChainStore, StoreError};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-05";

#[cfg(test)]
mod tests;
