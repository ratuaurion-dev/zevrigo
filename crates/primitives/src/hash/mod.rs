//! Tipe nilai hash 32 byte (`Hash32`) beserta pengodean heksadesimal mandiri.

use std::fmt::{self, Write as _};

/// Panjang byte [`Hash32`].
const HASH_BYTE_LEN: usize = 32;

/// Panjang karakter heksadesimal untuk [`Hash32`] (`32 × 2`).
const HASH_HEX_LEN: usize = 64;

/// Galat saat mengurai string heksadesimal menjadi [`Hash32`].
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum HashParseError {
    /// Panjang string bukan tepat 64 karakter.
    #[error("panjang hex harus tepat 64 karakter, ditemukan {actual}")]
    InvalidLength {
        /// Jumlah karakter aktual pada input.
        actual: usize,
    },
    /// Terdapat karakter bukan heksadesimal ASCII pada indeks karakter tertentu.
    #[error("karakter hex tidak valid pada indeks {index}")]
    InvalidHexChar {
        /// Indeks karakter berbasis nol tempat galat ditemukan.
        index: usize,
    },
}

/// Nilai hash 32 byte (256-bit) berukuran tetap.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Hash32([u8; HASH_BYTE_LEN]);

impl Hash32 {
    /// Nilai hash berisi 32 byte nol (`0x00`).
    pub const ZERO: Self = Self([0_u8; HASH_BYTE_LEN]);

    /// Membentuk [`Hash32`] dari array 32 byte.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; HASH_BYTE_LEN]) -> Self {
        Self(bytes)
    }

    /// Mengembalikan referensi ke array 32 byte di dalamnya.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; HASH_BYTE_LEN] {
        &self.0
    }

    /// Mengurai string heksadesimal sepanjang tepat 64 karakter (huruf besar maupun kecil diterima).
    ///
    /// # Errors
    ///
    /// Mengembalikan:
    /// - [`HashParseError::InvalidLength`] jika jumlah karakter input bukan `64`.
    /// - [`HashParseError::InvalidHexChar`] jika ada karakter di luar `0-9`, `a-f`, `A-F`.
    pub fn from_hex(s: &str) -> Result<Self, HashParseError> {
        let char_count = s.chars().count();
        if char_count != HASH_HEX_LEN {
            return Err(HashParseError::InvalidLength { actual: char_count });
        }

        let mut out = [0_u8; HASH_BYTE_LEN];
        let mut chars = s.chars().enumerate();

        for slot in &mut out {
            let (hi_idx, hi_ch) = chars
                .next()
                .ok_or(HashParseError::InvalidLength { actual: char_count })?;
            let hi =
                decode_hex_nibble(hi_ch).ok_or(HashParseError::InvalidHexChar { index: hi_idx })?;

            let (lo_idx, lo_ch) = chars
                .next()
                .ok_or(HashParseError::InvalidLength { actual: char_count })?;
            let lo =
                decode_hex_nibble(lo_ch).ok_or(HashParseError::InvalidHexChar { index: lo_idx })?;

            *slot = hi.wrapping_shl(4) | lo;
        }

        Ok(Self(out))
    }
}

const fn nibble_to_lower_hex(nibble: u8) -> char {
    match nibble {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        10 => 'a',
        11 => 'b',
        12 => 'c',
        13 => 'd',
        14 => 'e',
        _ => 'f',
    }
}

fn decode_hex_nibble(ch: char) -> Option<u8> {
    ch.to_digit(16).and_then(|v| u8::try_from(v).ok())
}

impl fmt::Display for Hash32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in &self.0 {
            f.write_char(nibble_to_lower_hex(byte.wrapping_shr(4)))?;
            f.write_char(nibble_to_lower_hex(byte & 0x0f))?;
        }
        Ok(())
    }
}

impl fmt::Debug for Hash32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests;
