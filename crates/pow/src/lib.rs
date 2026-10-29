//! MOD-03: Proof of Work. Berisi Target 256-bit dan PoW dev regtest (DEC-09); PoW mainnet menunggu DEC-01.

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

pub mod regtest;
pub mod target;

pub use regtest::{PowError, mine_regtest, regtest_pow_hash, verify_regtest_pow};
pub use target::Target;

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-03";

#[cfg(test)]
mod tests;
