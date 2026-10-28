//! MOD-02: Primitif kriptografi Homeostasis Coin (`HSIS`): hash BLAKE3 berpemisah domain, Merkle root, Schnorr BIP340, dan MuSig2.

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

pub mod hash;
pub mod merkle;
pub mod schnorr;

pub use hash::{Domain, DomainHasher, hash};
pub use merkle::merkle_root;
pub use schnorr::{PublicKey, SchnorrError, SecretKey, Signature, sign, verify};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-02";

#[cfg(test)]
mod tests;
