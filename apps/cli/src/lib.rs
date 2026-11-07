//! MOD-12: Pustaka alat baris perintah Zevrigo Coin (`zevrigo-cli`), TSK-M19.
//!
//! Perintah node (`info`) dan wallet regtest atau testnet (`wallet-create`, `wallet-restore`, `address`, `balance`,
//! `history`, `send`) sesuai Spesifikasi Wallet v0.1 Bagian 5 sampai 7. Frasa, kunci, alamat, pemilihan koin, dan tanda tangan
//! dari MOD-09 (DEC-21); komunikasi ke node lewat klien RPC MOD-08. CLI tidak memvalidasi konsensus.

#![forbid(unsafe_code)]

pub mod args;
pub mod commands;
pub mod history;
pub mod scan;
pub mod wallet_file;

pub use args::{
    Action, ArgError, Command, Config, DEFAULT_FEE_RATE, USAGE, default_rpc_url, parse_args,
};
pub use commands::{
    Balance, Cli, CliError, MIN_PASSWORD_CHARS, PASSWORD_ENV, PasswordPrompt, Protection, Sent,
    check_new_password, format_balance, format_history, format_info, format_sent, next_address,
    os_random, run, show_phrase, terminal_password, wallet_create, wallet_encrypt, wallet_restore,
};
pub use history::{
    History, HistoryEntry, HistoryError, HistoryState, HistoryTip, WalletKeys, history_path,
};
pub use scan::{Found, GAP_LIMIT, ScanResult, scan_wallet, wallet_payloads};
pub use wallet_file::{
    EncryptedPhrase, NextIndex, WalletFile, WalletFileError, load_network, network_name,
    wallet_path,
};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-12";
