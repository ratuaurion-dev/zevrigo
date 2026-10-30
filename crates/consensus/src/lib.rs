//! MOD-04: Satu-satunya tempat aturan konsensus Homeostasis Coin (`HSIS`): pasokan, target, ASERT, chainwork,
//! cap waktu, parameter jaringan, dan validasi.

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

pub mod chain_params;
pub mod difficulty;
pub mod supply;
pub mod uint;

pub use chain_params::{
    REGTEST_GENESIS_KEY, REGTEST_GENESIS_TARGET_COMPACT, REGTEST_GENESIS_TIMESTAMP,
    REGTEST_POW_LIMIT_COMPACT, pow_limit, pow_limit_compact, regtest_genesis, regtest_network_id,
};
pub use difficulty::{
    ASERT_HALF_LIFE_SECS, DifficultyError, asert_target, block_work, chain_work, decode_compact,
    encode_compact, median_time_past, timestamp_after_median, timestamp_within_future_limit,
};
pub use supply::{
    ScheduleEntry, SupplyError, SupplySchedule, base_reward, block_subsidy, total_supply_after,
};
pub use uint::U256;

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-04";

#[cfg(test)]
mod tests;
