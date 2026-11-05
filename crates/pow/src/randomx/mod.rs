//! PoW RandomX untuk testnet dan mainnet (DEC-01): mode ringan (cache 256 MiB) untuk verifikasi, dan rumus tinggi kunci.
//!
//! Hash dihitung pustaka C++ resmi RandomX lewat crate `randomx-rs` (DEC-10 bagian 10). `unsafe` dan floating point
//! hanya ada di dalam pustaka itu; modul ini tetap tanpa keduanya. Hash PoW dari 96 byte header dibaca sebagai bilangan
//! 256-bit big-endian dan harus ≤ target, sama dengan PoW regtest.
//!
//! Salt Argon2 masih nilai resmi RandomX. Salt sendiri chain ini (Konsep Bagian 5.10) ditetapkan bersama parameter
//! testnet (TSK-M06 bagian C). Pemilihan jaringan dan sumber kunci adalah aturan konsensus MOD-04 (TSK-M06 bagian B).

use homeostasis_primitives::block::Header;
use homeostasis_primitives::hash::Hash32;
use homeostasis_primitives::height::BlockHeight;
use homeostasis_primitives::params::{RANDOMX_KEY_INTERVAL, RANDOMX_KEY_LAG};
use randomx_rs::{RandomXCache, RandomXError, RandomXFlag, RandomXVM};

use crate::target::Target;

/// Tinggi terakhir yang masih memakai kunci genesis (`2_048 + 64`).
const LAST_GENESIS_KEY_HEIGHT: u64 = RANDOMX_KEY_INTERVAL + RANDOMX_KEY_LAG;

/// Jarak dari blok yang diperiksa ke batas jeda: jeda `64` blok dihitung dari induk (`64 + 1`).
const LAG_FROM_BLOCK: u64 = RANDOMX_KEY_LAG + 1;

/// Topeng kelipatan interval kunci (`!(2_048 - 1)`).
const INTERVAL_MASK: u64 = !(RANDOMX_KEY_INTERVAL - 1);

/// Galat PoW RandomX.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RandomxError {
    /// Pustaka RandomX gagal (alokasi cache atau VM, atau input kosong).
    #[error("pustaka RandomX gagal: {0}")]
    Library(String),
    /// Hash dari pustaka tidak 32 byte.
    #[error("hash RandomX tidak 32 byte")]
    HashLength,
    /// Hash PoW berada di atas target kesulitan.
    #[error("hash PoW RandomX di atas target")]
    AboveTarget {
        /// Nilai hash PoW yang dihasilkan.
        pow_hash: Hash32,
        /// Target kesulitan yang harus dipenuhi.
        target: Target,
    },
}

fn library(error: RandomXError) -> RandomxError {
    RandomxError::Library(error.to_string())
}

/// Tinggi blok kunci RandomX untuk blok di tinggi `height`.
///
/// Sampai tinggi `2_112` kuncinya blok genesis (tinggi 0). Sesudahnya, kelipatan `2_048` terbesar yang tidak lebih dari
/// `height − 65`, sehingga blok kunci sudah tertimbun paling sedikit 64 blok sampai induk. Rumus sama dengan Monero:
/// kunci berganti setiap 2_048 blok dengan jeda 64 blok (Konsep Bagian 5.10).
#[must_use]
pub const fn key_height(height: BlockHeight) -> BlockHeight {
    let value = height.get();
    if value <= LAST_GENESIS_KEY_HEIGHT {
        return BlockHeight::GENESIS;
    }
    match value.checked_sub(LAG_FROM_BLOCK) {
        Some(base) => BlockHeight::new(base & INTERVAL_MASK),
        None => BlockHeight::GENESIS,
    }
}

/// Pemeriksa PoW RandomX mode ringan untuk satu kunci.
///
/// Kunci adalah `block_id` blok di [`key_height`]. Membuat pemeriksa menyiapkan cache 256 MiB (Argon2d), jadi satu
/// pemeriksa dipakai untuk semua blok dengan kunci yang sama. Tipe ini tidak `Send`: pemiliknya satu utas.
pub struct RandomxVerifier {
    key: Hash32,
    vm: RandomXVM,
}

impl RandomxVerifier {
    /// Menyiapkan cache dan VM mode ringan untuk `key`, dengan flag yang dianjurkan pustaka untuk CPU ini (JIT dan
    /// AES perangkat keras jika ada; hasil hash sama untuk semua flag).
    ///
    /// # Errors
    ///
    /// [`RandomxError::Library`] jika cache atau VM tidak bisa dialokasikan.
    pub fn new(key: Hash32) -> Result<Self, RandomxError> {
        let vm = light_vm(key.as_bytes())?;
        Ok(Self { key, vm })
    }

    /// Kunci pemeriksa ini.
    #[must_use]
    pub const fn key(&self) -> Hash32 {
        self.key
    }

    /// Hash PoW RandomX dari 96 byte header.
    ///
    /// # Errors
    ///
    /// [`RandomxError::Library`] atau [`RandomxError::HashLength`] jika pustaka gagal.
    pub fn hash(&self, header: &Header) -> Result<Hash32, RandomxError> {
        hash_with(&self.vm, &header.to_array())
    }

    /// Memeriksa bahwa hash PoW RandomX header ≤ `target`.
    ///
    /// # Errors
    ///
    /// [`RandomxError::AboveTarget`] jika hash di atas target, atau galat dari [`RandomxVerifier::hash`].
    pub fn verify(&self, header: &Header, target: &Target) -> Result<(), RandomxError> {
        let pow_hash = self.hash(header)?;
        if target.is_met_by(&pow_hash) {
            Ok(())
        } else {
            Err(RandomxError::AboveTarget {
                pow_hash,
                target: *target,
            })
        }
    }
}

/// Cache dan VM mode ringan untuk kunci sembarang panjang (vektor uji resmi memakai kunci teks).
pub(crate) fn light_vm(key: &[u8]) -> Result<RandomXVM, RandomxError> {
    let flags = RandomXFlag::get_recommended_flags();
    let cache = RandomXCache::new(flags, key).map_err(library)?;
    RandomXVM::new(flags, Some(cache), None).map_err(library)
}

/// Hash RandomX dari `input` sembarang dengan `vm`.
pub(crate) fn hash_with(vm: &RandomXVM, input: &[u8]) -> Result<Hash32, RandomxError> {
    let output = vm.calculate_hash(input).map_err(library)?;
    let bytes: [u8; 32] = output
        .as_slice()
        .try_into()
        .map_err(|_| RandomxError::HashLength)?;
    Ok(Hash32::from_bytes(bytes))
}

#[cfg(test)]
mod tests;
