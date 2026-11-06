//! MOD-09: Wallet inti Zevrigo Coin (`ZVRG`): frasa pemulihan BIP39, turunan kunci BIP32, kunci akun publik
//! untuk watch-only, alamat bech32m (TSK-M15), serta pemilihan koin, penyusunan, dan tanda tangan transaksi kirim
//! (TSK-M16), sesuai Spesifikasi Wallet v0.1.
//!
//! Crate ini bukan konsensus: chain hanya menyimpan `kind` dan `payload` output. Penjumlahan skalar dan titik
//! secp256k1, `sighash_v1`, dan BIP340 memakai MOD-02; SHA-256, HMAC-SHA512, dan PBKDF2 memakai `sha2`, `hmac`, dan
//! `pbkdf2` (DEC-10 bagian 4). Enkripsi frasa dengan password (Argon2id + XChaCha20-Poly1305) di [`vault`] (DEC-27);
//! frasa, seed, dan kunci rahasia dihapus dari memori saat dibuang (`zeroize`).

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

pub mod address;
pub mod bip32;
pub mod mnemonic;
pub mod send;
pub mod vault;

pub use address::{AddressError, PAYLOAD_LEN, decode_address, encode_address};
pub use bip32::{
    ACCOUNT, ACCOUNT_KEY_HEX_LEN, Bip32Error, CHANGE_BRANCH, ExtendedPrivKey, ExtendedPubKey,
    HARDENED, PURPOSE, RECEIVE_BRANCH, TESTNET_COIN_TYPE, account_key, account_path, coin_type,
    hardened,
};
pub use mnemonic::{
    Mnemonic, MnemonicError, NEW_WALLET_ENTROPY_LEN, SEED_ITERATIONS, SEED_LEN, Seed, WORDLIST_LEN,
};
pub use send::{
    Coin, Plan, SINGLE_KEY_WITNESS_SIZE, SendError, SendRequest, build_transaction, plan, sign_plan,
};
pub use vault::{KdfParams, NONCE_LEN, SALT_LEN, Sealed, VaultError, seal, unseal};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-09";

#[cfg(test)]
mod tests;
