//! Tipe `Target` kesulitan Proof-of-Work 256-bit big-endian.

use zevrigo_primitives::Hash32;

/// Target kesulitan Proof-of-Work sebagai bilangan 256-bit big-endian.
///
/// Urutan perbandingan derived `Ord` pada representasi array byte `[u8; 32]`
/// identik dengan urutan numerik bilangan bulat 256-bit big-endian (byte indeks 0
/// adalah byte paling signifikan / MSB).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Target([u8; 32]);

impl Target {
    /// Nilai target terendah (semua 32 byte bernilai 0x00).
    pub const ZERO: Self = Self([0x00; 32]);

    /// Nilai target tertinggi (semua 32 byte bernilai 0xff).
    pub const MAX: Self = Self([0xff; 32]);

    /// Membuat [`Target`] dari array 32 byte dalam urutan big-endian.
    #[must_use]
    pub const fn from_be_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Mengembalikan representasi 32 byte big-endian dari [`Target`].
    #[must_use]
    pub const fn to_be_bytes(&self) -> [u8; 32] {
        self.0
    }

    /// Memeriksa apakah `pow_hash` memenuhi target ini (`hash ≤ target`).
    ///
    /// Perbandingan dilakukan secara numerik 256-bit big-endian.
    /// Nilai hash yang sama persis dengan target dianggap memenuhi.
    #[must_use]
    pub fn is_met_by(&self, pow_hash: &Hash32) -> bool {
        pow_hash.as_bytes() <= &self.0
    }
}

#[cfg(test)]
mod tests;
