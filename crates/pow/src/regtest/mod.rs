//! Implementasi Proof-of-Work khusus pengembangan di jaringan `regtest` (`DEC-09`).

use homeostasis_primitives::block::Header;
use homeostasis_primitives::hash::Hash32;

use crate::target::Target;

/// Kesalahan verifikasi Proof-of-Work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PowError {
    /// Hash PoW berada di atas target kesulitan.
    #[error("hash PoW regtest di atas target")]
    AboveTarget {
        /// Nilai hash PoW yang dihasilkan.
        pow_hash: Hash32,
        /// Target kesulitan yang harus dipenuhi.
        target: Target,
    },
}

/// Menghitung hash Proof-of-Work regtest dari 96 byte header blok.
///
/// PoW ini hanya sah di jaringan regtest (`DEC-09`), dan penolakannya
/// di testnet dan mainnet dilakukan oleh MOD-04 (TSK-M09).
#[must_use]
pub fn regtest_pow_hash(header: &Header) -> Hash32 {
    homeostasis_crypto::hash(homeostasis_crypto::Domain::RegtestPow, &header.to_array())
}

/// Memverifikasi bahwa header blok memenuhi target kesulitan PoW regtest.
///
/// Mengembalikan `Ok(())` jika hash PoW header ≤ target kesulitan.
///
/// PoW ini hanya sah di jaringan regtest (`DEC-09`), dan penolakannya
/// di testnet dan mainnet dilakukan oleh MOD-04 (TSK-M09).
pub fn verify_regtest_pow(header: &Header, target: &Target) -> Result<(), PowError> {
    let pow_hash = regtest_pow_hash(header);
    if target.is_met_by(&pow_hash) {
        Ok(())
    } else {
        Err(PowError::AboveTarget {
            pow_hash,
            target: *target,
        })
    }
}

/// Mencari nonce yang memenuhi target PoW dev regtest (`DEC-09`).
///
/// Melakukan iterasi dari `header.nonce()` sampai paling banyak `max_attempts` percobaan
/// dengan `checked_add`. Mengembalikan `None` jika `max_attempts == 0`, semua percobaan
/// gagal, atau terjadi overflow `u64`.
///
/// PoW ini hanya sah di jaringan regtest (`DEC-09`), dan penolakannya
/// di testnet dan mainnet dilakukan oleh MOD-04 (TSK-M09).
#[must_use]
pub fn mine_regtest(header: &Header, target: &Target, max_attempts: u64) -> Option<Header> {
    if max_attempts == 0 {
        return None;
    }
    let start_nonce = header.nonce();
    for i in 0..max_attempts {
        let nonce = start_nonce.checked_add(i)?;
        let candidate = Header::new(
            header.height(),
            header.prev_block_id(),
            header.merkle_root(),
            header.timestamp(),
            header.target(),
            nonce,
        );
        if verify_regtest_pow(&candidate, target).is_ok() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests;
