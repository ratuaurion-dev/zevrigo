//! MOD-11: Pustaka node Homeostasis Coin (`homeostasis-node`): logika chain aktif dan backend RPC (TSK-M18).
//!
//! Biner `homeostasis-node` (`src/main.rs`) memakai pustaka ini mulai TSK-M18 bagian B (daemon).

#![forbid(unsafe_code)]

pub mod chain;

pub use chain::{Clock, Node, NodeError, block_rule, mempool_reason};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-11";
