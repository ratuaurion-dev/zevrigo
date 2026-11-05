//! MOD-03: Proof of Work. Berisi Target 256-bit, PoW dev regtest (DEC-09), dan PoW RandomX untuk testnet dan mainnet
//! (DEC-01, TSK-M06).

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

pub mod randomx;
pub mod regtest;
pub mod target;

pub use randomx::{RandomxError, RandomxVerifier, key_height};
pub use regtest::{PowError, mine_regtest, regtest_pow_hash, verify_regtest_pow};
pub use target::Target;

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-03";

#[cfg(test)]
mod tests;
