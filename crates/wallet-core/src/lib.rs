//! MOD-09: Wallet inti Homeostasis Coin (`HSIS`): frasa pemulihan BIP39, turunan kunci BIP32, kunci akun publik
//! untuk watch-only, dan alamat bech32m (TSK-M15, Spesifikasi Wallet v0.1).
//!
//! Pemilihan koin, penyusunan, dan tanda tangan transaksi menyusul di TSK-M16. Crate ini bukan konsensus: chain
//! hanya menyimpan `kind` dan `payload` output. Penjumlahan skalar dan titik secp256k1 memakai MOD-02; SHA-256,
//! HMAC-SHA512, dan PBKDF2 memakai `sha2`, `hmac`, dan `pbkdf2` (DEC-10 bagian 4).

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

pub mod address;
pub mod bip32;
pub mod mnemonic;

pub use address::{AddressError, PAYLOAD_LEN, decode_address, encode_address};
pub use bip32::{
    ACCOUNT, ACCOUNT_KEY_HEX_LEN, Bip32Error, CHANGE_BRANCH, ExtendedPrivKey, ExtendedPubKey,
    HARDENED, PURPOSE, RECEIVE_BRANCH, TESTNET_COIN_TYPE, account_key, account_path, coin_type,
    hardened,
};
pub use mnemonic::{
    Mnemonic, MnemonicError, NEW_WALLET_ENTROPY_LEN, SEED_ITERATIONS, SEED_LEN, Seed, WORDLIST_LEN,
};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-09";

#[cfg(test)]
mod tests;
