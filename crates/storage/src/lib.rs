//! MOD-05: Penyimpanan blok, set UTXO, pruning.
//!
//! Belum diimplementasikan (kerangka R0).

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-05";

#[cfg(test)]
mod tests;
