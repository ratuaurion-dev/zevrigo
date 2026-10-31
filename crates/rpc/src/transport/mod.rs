//! Parameter transpor HTTP JSON-RPC (Spesifikasi RPC v0.1 Bagian 1).

use homeostasis_primitives::params::Network;

/// Alamat bawaan server RPC: hanya lokal.
pub const DEFAULT_HOST: &str = "127.0.0.1";

/// Ukuran request maksimum dalam byte (1 MiB, cukup untuk blok 100_000 byte dalam hex).
pub const MAX_REQUEST_BYTES: u32 = 1_048_576;

/// Port bawaan server RPC per jaringan: regtest 28_801, testnet 18_801, mainnet 8_801.
#[must_use]
pub const fn default_port(network: Network) -> u16 {
    match network {
        Network::Mainnet => 8_801,
        Network::Testnet => 18_801,
        Network::Regtest => 28_801,
    }
}
