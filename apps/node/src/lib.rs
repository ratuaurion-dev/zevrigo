//! MOD-11: Pustaka node Homeostasis Coin (`homeostasis-node`): logika chain aktif, blok cabang dan reorg (TSK-M27),
//! sinkronisasi header lalu blok dan siaran P2P (TSK-M28), backend RPC, dan daemon (TSK-M18).
//!
//! Biner `homeostasis-node` (`src/main.rs`) hanya membaca argumen lalu memanggil [`daemon::run`].

#![forbid(unsafe_code)]

pub mod chain;
pub mod daemon;
pub mod p2p;

pub use chain::{
    BlockReceipt, Clock, HeadersOutcome, INVALID_ANCESTOR_RULE, MAX_LOCATOR_IDS, Node, NodeError,
    NodeLimits, TipSummary, TxOutcome, block_rule, locator_heights, mempool_reason,
    transaction_result,
};
pub use daemon::{ArgError, Command, Config, DaemonError, parse_args};
pub use p2p::{Announcement, P2pService, Relay, block_ack, local_status, tx_ack};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-11";
