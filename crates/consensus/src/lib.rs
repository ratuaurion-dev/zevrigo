//! MOD-04: Satu-satunya tempat aturan konsensus Homeostasis Coin (`HSIS`): pasokan, validasi, ASERT, dan cap waktu.

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

pub mod supply;

pub use supply::{
    ScheduleEntry, SupplyError, SupplySchedule, base_reward, block_subsidy, total_supply_after,
};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-04";

#[cfg(test)]
mod tests;
