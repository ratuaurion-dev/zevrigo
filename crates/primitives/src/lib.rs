//! MOD-01: Tipe dasar: jumlah (u128 quanta), tinggi blok, hash, parameter chain, struktur blok dan transaksi.
//!
//! Belum diimplementasikan (kerangka R0).

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-01";

#[cfg(test)]
mod tests;
