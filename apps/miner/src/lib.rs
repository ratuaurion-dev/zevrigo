//! MOD-13: Pustaka penambang CPU Homeostasis Coin (`homeostasis-miner`), TSK-M20.
//!
//! Miner mengambil template blok dari node lewat RPC (MOD-08), mencari nonce dengan PoW dev regtest (MOD-03,
//! DEC-09), lalu mengirim blok yang ditemukan. Miner tidak memvalidasi chain; node yang memeriksa V1 sampai V10.

#![forbid(unsafe_code)]

pub mod cli;
pub mod mining;

pub use cli::{ArgError, Command, MinerConfig, USAGE, parse_args};
pub use mining::{ATTEMPTS_PER_TEMPLATE, Miner, MinerError, Round};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-13";
