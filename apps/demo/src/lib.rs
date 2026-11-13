//! MOD-18: Pustaka layanan jaringan demo regtest Zevrigo Coin (`zevrigo-demo`), sesuai Rancangan Ekosistem Demo v0.1
//! (DEC-34, DEC-36).
//!
//! Hanya untuk jaringan demo regtest. Satu program dengan tiga mode, masing-masing dijalankan sebagai proses sendiri di
//! belakang Cloudflare Tunnel dengan hostname sendiri:
//! - `rpc` ([`gateway`], TSK-M53): gerbang RPC untuk wallet peserta. Hanya metode aman yang diteruskan ke node, dengan
//!   batas laju per pengunjung ([`limit`]) tanpa menyimpan alamat IP.
//! - `faucet` (TSK-M54) dan `daftar` (TSK-M55): belum tersedia.
//!
//! [`cli`] membaca argumen program.

#![forbid(unsafe_code)]

pub mod cli;
pub mod gateway;
pub mod limit;

pub use cli::{
    ArgError, Command, DEFAULT_NODE_URL, DEFAULT_RATE, DEFAULT_RPC_PORT, RpcConfig, USAGE,
    parse_args,
};
pub use gateway::{
    ALLOWED_METHODS, CLIENT_IP_HEADER, Call, DemoError, MAX_BODY_BYTES, client_ip, init_logging,
    parse_call, router, run_rpc,
};
pub use limit::{Limits, MAX_CLIENTS, Rate, RateLimiter};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-18";
