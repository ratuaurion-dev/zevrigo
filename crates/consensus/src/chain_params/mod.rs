//! Parameter per jaringan dan blok genesis regtest.
//!
//! Sesuai `docs/reference/spesifikasi-validasi-v0.1.md` Bagian 2 dan 10. Nilai testnet dan mainnet baru
//! ditetapkan bersama DEC-01 dan pengukuran testnet.

use homeostasis_crypto::{merkle_root, network_id, wtxid};
use homeostasis_pow::Target;
use homeostasis_primitives::params::GENESIS_ALLOCATION;
use homeostasis_primitives::{
    Block, BlockHeight, DecodeError, GENESIS_EXTRA_LEN, Hash32, Header, Network, Output,
    OutputKind, Transaction,
};

use crate::difficulty::{DifficultyError, decode_compact};

/// `pow_limit` regtest dalam format compact (`0x7fffff × 2^232`).
pub const REGTEST_POW_LIMIT_COMPACT: u32 = 0x207f_ffff;

/// Target genesis regtest dalam format compact (sama dengan `pow_limit` regtest).
pub const REGTEST_GENESIS_TARGET_COMPACT: u32 = REGTEST_POW_LIMIT_COMPACT;

/// Cap waktu genesis regtest: 28 Oktober 2026 00.00 UTC.
pub const REGTEST_GENESIS_TIMESTAMP: u64 = 1_793_145_600;

/// Kunci x-only penerima alokasi genesis regtest: kunci publik dari kunci rahasia `1`.
///
/// Sengaja publik, karena koin regtest tidak bernilai dan test perlu membelanjakan alokasi genesis.
pub const REGTEST_GENESIS_KEY: [u8; 32] = [
    0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b, 0x07,
    0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98,
];

/// `pow_limit` jaringan dalam format compact.
///
/// Testnet dan mainnet memakai placeholder `0`, yang sengaja tidak sah sehingga [`pow_limit`] gagal.
#[must_use]
pub const fn pow_limit_compact(network: Network) -> u32 {
    match network {
        Network::Regtest => REGTEST_POW_LIMIT_COMPACT,
        // TODO(param): pow_limit testnet, ditetapkan dari pengukuran testnet bersama DEC-01.
        Network::Testnet => 0,
        // TODO(param): pow_limit mainnet, ditetapkan di spesifikasi genesis yang diumumkan (L1).
        Network::Mainnet => 0,
    }
}

/// `pow_limit` jaringan sebagai [`Target`] 256-bit.
///
/// # Errors
///
/// [`DifficultyError::InvalidCompact`] untuk testnet dan mainnet sampai `pow_limit`-nya ditetapkan.
pub fn pow_limit(network: Network) -> Result<Target, DifficultyError> {
    decode_compact(pow_limit_compact(network))
}

/// Blok genesis regtest (Spesifikasi Validasi Bagian 10).
///
/// Header: tinggi 0, `prev_block_id` nol, cap waktu [`REGTEST_GENESIS_TIMESTAMP`], target
/// [`REGTEST_GENESIS_TARGET_COMPACT`], nonce 0. Coinbase: `extra` 32 byte nol dan satu output alokasi
/// pencipta `G` ke [`REGTEST_GENESIS_KEY`]. PoW genesis tidak diperiksa.
///
/// # Errors
///
/// [`DecodeError`] dari konstruktor MOD-01. Tidak terjadi untuk nilai tetap ini, dan dibuktikan oleh test.
pub fn regtest_genesis() -> Result<Block, DecodeError> {
    let allocation = Output::new(
        GENESIS_ALLOCATION,
        OutputKind::SingleKey,
        REGTEST_GENESIS_KEY,
    )?;
    let coinbase = Transaction::coinbase(
        BlockHeight::GENESIS,
        vec![0_u8; GENESIS_EXTRA_LEN],
        vec![allocation],
    )?;
    let root = merkle_root(&[wtxid(&coinbase)]).ok_or(DecodeError::EmptyBlock)?;
    let header = Header::new(
        BlockHeight::GENESIS,
        Hash32::ZERO,
        root,
        REGTEST_GENESIS_TIMESTAMP,
        REGTEST_GENESIS_TARGET_COMPACT,
        0,
    );
    Block::new(header, vec![coinbase])
}

/// ID jaringan regtest: `block_id` genesis regtest.
///
/// # Errors
///
/// Sama dengan [`regtest_genesis`].
pub fn regtest_network_id() -> Result<Hash32, DecodeError> {
    Ok(network_id(regtest_genesis()?.header()))
}

#[cfg(test)]
mod tests;
