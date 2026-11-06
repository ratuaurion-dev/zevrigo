//! MOD-04: Satu-satunya tempat aturan konsensus Homeostasis Coin (`HSIS`): pasokan, target, ASERT, chainwork,
//! cap waktu, parameter jaringan, validasi, dan pemakaian PoW RandomX (kunci, penghitung).

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

pub mod chain_params;
pub mod difficulty;
pub mod randomx;
pub mod supply;
pub mod uint;
pub mod validation;

pub use chain_params::{
    ChainParams, ParamsError, REGTEST_GENESIS_KEY, REGTEST_GENESIS_TARGET_COMPACT,
    REGTEST_GENESIS_TIMESTAMP, REGTEST_POW_LIMIT_COMPACT, TESTNET_GENESIS_EXTRA,
    TESTNET_GENESIS_KEY, TESTNET_GENESIS_TARGET_COMPACT, TESTNET_GENESIS_TIMESTAMP,
    TESTNET_POW_LIMIT_COMPACT, known_genesis, pow_limit, pow_limit_compact, regtest_genesis,
    regtest_network_id, testnet_genesis, testnet_network_id,
};
pub use difficulty::{
    ASERT_HALF_LIFE_SECS, DifficultyError, asert_target, block_work, chain_work, decode_compact,
    encode_compact, median_time_past, timestamp_after_median, timestamp_within_future_limit,
};
pub use randomx::{RandomxHasher, RandomxStartError, randomx_hasher, randomx_key_height};
pub use supply::{
    ScheduleEntry, SupplyError, SupplySchedule, base_reward, block_subsidy, total_supply_after,
};
pub use uint::U256;
pub use validation::{
    BlockError, BlockSummary, ParentContext, TxError, UtxoLookup, check_block_context_free,
    check_block_pow, check_block_transactions, check_future_timestamp, check_header_context,
    check_header_context_free, check_header_pow, check_transaction, decode_block,
};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-04";

#[cfg(test)]
mod tests;
