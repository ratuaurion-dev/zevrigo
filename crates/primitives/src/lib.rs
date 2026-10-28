//! MOD-01: Tipe dasar Homeostasis Coin (`HSIS`): jumlah (`u128` quanta), tinggi blok, hash 32 byte, dan parameter chain.

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

pub mod amount;
pub mod hash;
pub mod height;
pub mod params;

pub use amount::{Amount, AmountParseError, QUANTA_PER_HSIS};
pub use hash::{Hash32, HashParseError};
pub use height::BlockHeight;

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-01";

#[cfg(test)]
mod tests;
