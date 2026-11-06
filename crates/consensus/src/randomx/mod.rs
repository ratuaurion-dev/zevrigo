//! V3 RandomX untuk testnet dan mainnet (DEC-01, TSK-M06 bagian B): sumber hash dan tinggi blok kunci.
//!
//! Hash RandomX dihitung MOD-03 di satu utas pemilik cache ([`randomx_hasher`]). Validasi hanya memakai
//! [`RandomxHasher`] lewat [`crate::check_header_pow`] dan [`crate::check_block_pow`], sehingga test boleh memakai
//! penghitung lain. Kunci blok setinggi `h` adalah `block_id` leluhurnya di [`randomx_key_height`] pada chain blok itu
//! sendiri; sampai tinggi 2_112 leluhur itu genesis, sehingga kuncinya `network_id`.

use std::sync::Arc;

use zevrigo_pow::{RandomxService, key_height};
use zevrigo_primitives::{BlockHeight, Hash32, Header};

/// Penghitung hash PoW RandomX untuk V3 testnet dan mainnet.
pub trait RandomxHasher {
    /// Hash RandomX 96 byte `header` dengan kunci `key`, atau `None` jika pustaka atau utasnya gagal.
    fn randomx_hash(&self, key: &Hash32, header: &Header) -> Option<Hash32>;
}

impl RandomxHasher for RandomxService {
    fn randomx_hash(&self, key: &Hash32, header: &Header) -> Option<Hash32> {
        self.hash(*key, header).ok()
    }
}

/// Galat memulai layanan RandomX.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("layanan RandomX tidak bisa dimulai: {message}")]
pub struct RandomxStartError {
    /// Keterangan galat.
    pub message: String,
}

/// Memulai layanan RandomX MOD-03: satu utas yang memiliki cache mode ringan untuk dua kunci terakhir (masing-masing
/// 256 MiB, dibuat saat kunci itu pertama kali diminta). Dipakai node testnet dan mainnet.
///
/// # Errors
///
/// [`RandomxStartError`] jika utas tidak bisa dibuat.
pub fn randomx_hasher() -> Result<Arc<dyn RandomxHasher + Send + Sync>, RandomxStartError> {
    let service = RandomxService::start().map_err(|error| RandomxStartError {
        message: error.to_string(),
    })?;
    Ok(Arc::new(service))
}

/// Tinggi blok kunci RandomX untuk blok setinggi `height` (Konsep Bagian 5.10): 0 sampai tinggi 2_112, sesudahnya
/// kelipatan 2_048 terbesar yang tidak lebih dari `height − 65`.
#[must_use]
pub const fn randomx_key_height(height: BlockHeight) -> BlockHeight {
    key_height(height)
}

#[cfg(test)]
mod tests;
