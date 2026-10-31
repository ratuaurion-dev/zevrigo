//! MOD-11: Pustaka node Homeostasis Coin (`homeostasis-node`): logika chain aktif, backend RPC, dan daemon (TSK-M18).
//!
//! Biner `homeostasis-node` (`src/main.rs`) hanya membaca argumen lalu memanggil [`daemon::run`].

#![forbid(unsafe_code)]

pub mod chain;
pub mod daemon;

pub use chain::{Clock, Node, NodeError, block_rule, mempool_reason};
pub use daemon::{ArgError, Command, Config, DaemonError, parse_args};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-11";
