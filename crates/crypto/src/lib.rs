//! MOD-02: Primitif kriptografi Homeostasis Coin (`HSIS`): hash BLAKE3 berpemisah domain, Merkle root, Schnorr BIP340, ID data chain, sighash v1, dan penjumlahan skalar/titik secp256k1 untuk turunan kunci BIP32 wallet.

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

pub mod hash;
pub mod ids;
pub mod merkle;
pub mod schnorr;
pub mod sighash;
pub mod tweak;

pub use hash::{Domain, DomainHasher, hash};
pub use ids::{block_id, block_merkle_root, condition_hash, network_id, txid, wtxid};
pub use merkle::merkle_root;
pub use schnorr::{PublicKey, SchnorrError, SecretKey, Signature, sign, verify};
pub use sighash::{SighashError, sighash_v1};
pub use tweak::{
    COMPRESSED_PUBLIC_KEY_LEN, TweakError, is_valid_public_key, public_key_compressed,
    public_tweak_add, secret_tweak_add,
};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-02";

#[cfg(test)]
mod tests;
