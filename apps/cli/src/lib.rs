//! MOD-12: Pustaka alat baris perintah Homeostasis Coin (`homeostasis-cli`), TSK-M19.
//!
//! Perintah node (`info`) dan wallet regtest (`wallet-create`, `wallet-restore`, `address`, `balance`, `send`)
//! sesuai Spesifikasi Wallet v0.1 Bagian 5 sampai 7. Frasa, kunci, alamat, pemilihan koin, dan tanda tangan dari
//! MOD-09 (DEC-21); komunikasi ke node lewat klien RPC MOD-08. CLI tidak memvalidasi konsensus.

#![forbid(unsafe_code)]

pub mod args;
pub mod commands;
pub mod scan;
pub mod wallet_file;

pub use args::{
    Action, ArgError, Command, Config, DEFAULT_FEE_RATE, USAGE, default_rpc_url, parse_args,
};
pub use commands::{
    Balance, Cli, CliError, Sent, format_balance, format_info, format_sent, next_address,
    os_random, run, wallet_create, wallet_restore,
};
pub use scan::{Found, GAP_LIMIT, ScanResult, scan_wallet};
pub use wallet_file::{NextIndex, WalletFile, WalletFileError, wallet_path};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-12";
