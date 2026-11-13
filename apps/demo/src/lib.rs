//! MOD-18: Pustaka layanan jaringan demo regtest Zevrigo Coin (`zevrigo-demo`), sesuai Rancangan Ekosistem Demo v0.1
//! (DEC-34, DEC-36).
//!
//! Hanya untuk jaringan demo regtest. Satu program dengan beberapa mode, masing-masing dijalankan sebagai proses sendiri di
//! belakang Cloudflare Tunnel dengan hostname sendiri:
//! - `rpc` ([`gateway`], TSK-M53): gerbang RPC untuk wallet peserta. Hanya metode aman yang diteruskan ke node, dengan
//!   batas laju per pengunjung ([`limit`]) tanpa menyimpan alamat IP.
//! - `upacara` ([`upacara`], TSK-M62): alat operator sekali pakai, upacara genesis regtest (alokasi genesis ke kantong
//!   treasury). Bukan layanan dan tidak dibuka ke publik.
//! - `faucet` ([`faucet`], TSK-M54 bagian A): faucet demo, 100 ZVRG per pengunjung per hari dari wallet faucet, dengan
//!   batas per alamat dan batas harian. Tanpa menyimpan alamat IP.
//! - `daftar` (TSK-M55): belum tersedia.
//!
//! [`cli`] membaca argumen program.

#![forbid(unsafe_code)]

pub mod cli;
pub mod faucet;
pub mod gateway;
pub mod limit;
pub mod upacara;

pub use cli::{
    ArgError, Command, DEFAULT_DAILY_LIMIT, DEFAULT_FAUCET_AMOUNT, DEFAULT_FAUCET_PORT,
    DEFAULT_FAUCET_RATE, DEFAULT_NODE_URL, DEFAULT_ORIGIN, DEFAULT_RATE, DEFAULT_RPC_PORT,
    FaucetConfig, RpcConfig, USAGE, UpacaraConfig, parse_args, parse_pocket,
};
pub use faucet::{
    FAUCET_PATH, Faucet, FaucetError, FaucetStatus, Payout, RouterOptions, Settings,
    faucet_client_ip, run_faucet,
};
pub use gateway::{
    ALLOWED_METHODS, CLIENT_IP_HEADER, Call, DemoError, MAX_BODY_BYTES, client_ip, init_logging,
    parse_call, router, run_rpc,
};
pub use limit::{Limits, MAX_CLIENTS, Rate, RateLimiter};
pub use upacara::{
    GENESIS_PAYLOAD, MAX_POCKETS, Pocket, REGTEST_NETWORK_ID, UpacaraError, build_ceremony,
    run_command as run_upacara,
};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-18";
