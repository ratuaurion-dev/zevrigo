//! MOD-05: Penyimpanan blok, set UTXO, dan undo data. Berisi trait ChainStore dan MemoryStore (TSK-M10); RedbStore menyusul (TSK-M11, DEC-03).

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

pub mod memory;
pub mod store;

pub use memory::MemoryStore;
pub use store::{BlockUndo, ChainStore, StoreError};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-05";

#[cfg(test)]
mod tests;
