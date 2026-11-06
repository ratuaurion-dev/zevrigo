//! Fungsi hash BLAKE3 dengan pemisah domain wajib (mode `derive_key`).
//!
//! Sesuai `docs/reference/spesifikasi-format-v0.1.md` Bagian 3, setiap perhitungan hash
//! di protokol Zevrigo Coin wajib menyertakan [`Domain`] eksplisit sehingga keluaran
//! hash untuk kegunaan yang berbeda tidak pernah bisa saling menggantikan.

use zevrigo_primitives::Hash32;

/// Pemisah domain untuk setiap penggunaan hash BLAKE3 di dalam protokol Zevrigo Coin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Domain {
    /// Identitas badan transaksi tanpa witness (`txid`).
    Txid,
    /// Identitas transaksi lengkap beserta witness (`wtxid`).
    Wtxid,
    /// Daun pohon Merkle transaksi.
    MerkleLeaf,
    /// Simpul dalam (internal node) pohon Merkle transaksi.
    MerkleNode,
    /// Identitas header blok (`block_id`).
    BlockId,
    /// Hash skrip kondisi `Condition` pada output jenis 1.
    Condition,
    /// Pesan 32 byte yang ditandatangani pada skema `sighash v1`.
    Sighash,
    /// Hash Proof-of-Work khusus pengembangan di jaringan `regtest` (`DEC-09`).
    RegtestPow,
}

impl Domain {
    /// Mengembalikan string konteks kanonis untuk mode BLAKE3 `derive_key`.
    #[must_use]
    pub const fn context(self) -> &'static str {
        match self {
            Self::Txid => "zevrigo 2026-10-28 txid",
            Self::Wtxid => "zevrigo 2026-10-28 wtxid",
            Self::MerkleLeaf => "zevrigo 2026-10-28 merkle leaf",
            Self::MerkleNode => "zevrigo 2026-10-28 merkle node",
            Self::BlockId => "zevrigo 2026-10-28 block id",
            Self::Condition => "zevrigo 2026-10-28 condition",
            Self::Sighash => "zevrigo 2026-10-28 sighash v1",
            Self::RegtestPow => "zevrigo 2026-10-28 regtest pow",
        }
    }
}

/// Menghitung hash BLAKE3 `derive_key` 32 byte atas `data` di bawah `domain` tertentu.
#[must_use]
pub fn hash(domain: Domain, data: &[u8]) -> Hash32 {
    Hash32::from_bytes(blake3::derive_key(domain.context(), data))
}

/// Penghitung hash BLAKE3 bertahap (*streaming*) yang terikat pada satu [`Domain`].
#[derive(Clone, Debug)]
pub struct DomainHasher {
    inner: blake3::Hasher,
}

impl DomainHasher {
    /// Membuat [`DomainHasher`] baru dengan string konteks milik `domain`.
    #[must_use]
    pub fn new(domain: Domain) -> Self {
        Self {
            inner: blake3::Hasher::new_derive_key(domain.context()),
        }
    }

    /// Menambahkan potongan byte `data` ke dalam state hash.
    pub fn update(&mut self, data: &[u8]) -> &mut Self {
        self.inner.update(data);
        self
    }

    /// Menyelesaikan perhitungan dan mengembalikan nilai [`Hash32`] 32 byte.
    #[must_use]
    pub fn finalize(&self) -> Hash32 {
        Hash32::from_bytes(*self.inner.finalize().as_bytes())
    }
}

#[cfg(test)]
mod tests;
