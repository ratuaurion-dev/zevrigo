//! MOD-06: P2P Zevrigo Coin di atas rust-libp2p 0.57 (DEC-24), sesuai Spesifikasi P2P v0.1.
//!
//! - [`message`]: enam pesan P2P dan encoding kanonisnya (Bagian 3).
//! - [`frame`]: bingkai `varint<32>(panjang) ‖ isi`, paling banyak 2 MiB (Bagian 3).
//! - [`protocol`]: nama protokol `/zvrg/<net>/<nama>/1` (Bagian 2).
//! - [`key`]: kunci identitas rekan Ed25519 di `<datadir>/<jaringan>/p2p_key` (Bagian 1).
//! - [`ban`]: blokir 24 jam per `PeerId` dan per alamat IP (Bagian 7).
//! - [`service`]: swarm libp2p (TCP, Noise, Yamux), lima protokol request-response, batas koneksi, sambung ulang,
//!   pertukaran `status`, dan sanksi.
//!
//! Keputusan sinkronisasi, pemilihan chain, dan validasi blok atau transaksi ada di MOD-11 dan MOD-04 (TSK-M28).

#![forbid(unsafe_code)]

pub mod ban;
mod codec;
pub mod frame;
pub mod key;
pub mod message;
pub mod protocol;
pub mod service;

pub use ban::{BAN_DURATION, BanList};
pub use key::{KeyError, P2P_KEY_FILE, keypair_from_secret, load_or_create};
pub use libp2p::identity::Keypair;
pub use libp2p::{Multiaddr, PeerId};
pub use message::{
    Ack, Blocks, GetBlocks, GetHeaders, Headers, MAX_MESSAGE_SIZE, MessageError, Status, Wire,
};
pub use protocol::{ProtocolKind, protocol_name};
pub use service::{NetEvent, Network, NetworkConfig, NetworkError, Offense, Reply, RequestId};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-06";

#[cfg(test)]
mod test_support;

#[cfg(test)]
mod tests;
