//! Tipe jumlah koin (`Amount`) dalam satuan terkecil quanta (`u128`).
//!
//! Satu koin `HSIS` setara dengan `10^18` quanta ([`QUANTA_PER_HSIS`]).
//! Seluruh operasi aritmetika pada [`Amount`] bersifat eksplisit (`checked_*` atau
//! `saturating_sub`) tanpa implementasi operator `+` atau `-` agar tidak ada luapan diam-diam.

use std::fmt;

/// Jumlah quanta dalam tepat `1 HSIS` (`10^18` quanta).
pub const QUANTA_PER_HSIS: u128 = 1_000_000_000_000_000_000;

/// Jumlah digit pecahan maksimum untuk format teks `HSIS` (`10^18` -> 18 digit).
const MAX_FRACTIONAL_DIGITS: usize = 18;

/// Galat saat mengurai teks angka `HSIS` melalui [`Amount::from_hsis_str`].
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AmountParseError {
    /// Format teks melanggar tata bahasa angka `HSIS` (kosong, spasi, tanda, nol depan, titik gantung, atau karakter non-digit).
    #[error("format jumlah HSIS tidak valid")]
    InvalidFormat,
    /// Bagian pecahan setelah titik memiliki lebih dari 18 digit desimal.
    #[error("jumlah digit desimal melebihi batas maksimum 18 digit")]
    TooManyDecimals,
    /// Nilai yang diurai melebihi kapasitas maksimum `u128` quanta.
    #[error("jumlah HSIS melampaui kapasitas maksimum u128 quanta")]
    Overflow,
}

/// Jumlah koin Homeostasis yang disimpan sebagai bilangan bulat tak bertanda `u128` dalam quanta.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Amount(u128);

impl Amount {
    /// Nilai nol (`0` quanta).
    pub const ZERO: Self = Self(0);

    /// Membentuk [`Amount`] langsung dari nilai quanta (`u128`).
    #[must_use]
    pub const fn from_quanta(quanta: u128) -> Self {
        Self(quanta)
    }

    /// Mengembalikan nilai mentah dalam quanta (`u128`).
    #[must_use]
    pub const fn as_quanta(self) -> u128 {
        self.0
    }

    /// Menjumlahkan dua [`Amount`], mengembalikan `None` jika terjadi luapan `u128`.
    #[must_use]
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        match self.0.checked_add(rhs.0) {
            Some(sum) => Some(Self(sum)),
            None => None,
        }
    }

    /// Mengurangkan `rhs` dari `self`, mengembalikan `None` jika `self < rhs`.
    #[must_use]
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        match self.0.checked_sub(rhs.0) {
            Some(diff) => Some(Self(diff)),
            None => None,
        }
    }

    /// Mengurangkan `rhs` dari `self` dengan batas bawah [`Amount::ZERO`].
    ///
    /// Operasi saturasi ini sengaja disediakan untuk rumus yang memang memotong selisih negatif
    /// menjadi nol secara eksplisit (misalnya sisa pasokan `(S - M)` yang dianggap `0` bila `M >= S`).
    #[must_use]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        // Sengaja memakai saturating_sub: jika rhs > self, hasil berhenti tepat di 0 quanta.
        Self(self.0.saturating_sub(rhs.0))
    }

    /// Menjumlahkan seluruh elemen iterator secara terperiksa, mengembalikan `None` saat terjadi luapan.
    #[must_use]
    pub fn checked_sum<I: IntoIterator<Item = Self>>(iter: I) -> Option<Self> {
        let mut total = Self::ZERO;
        for item in iter {
            total = total.checked_add(item)?;
        }
        Some(total)
    }

    /// Memformat nilai [`Amount`] menjadi teks angka `HSIS` kanonis (tanpa akhiran satuan).
    ///
    /// Bagian bulat ditulis penuh; jika bagian pecahan tidak nol, diikuti titik dan maksimal
    /// 18 digit pecahan tanpa nol di belakang. Jika bagian pecahan nol, titik ditiadakan.
    #[must_use]
    pub fn to_hsis_string(&self) -> String {
        // QUANTA_PER_HSIS bernilai konstanta tak-nol (10^18), sehingga pembagian selalu berhasil.
        let whole = self.0.checked_div(QUANTA_PER_HSIS).unwrap_or(0);
        let frac = self.0.checked_rem(QUANTA_PER_HSIS).unwrap_or(0);

        if frac == 0 {
            return whole.to_string();
        }

        let frac_padded = format!("{frac:018}");
        let frac_trimmed = frac_padded.trim_end_matches('0');
        format!("{whole}.{frac_trimmed}")
    }

    /// Mengurai teks angka `HSIS` menjadi [`Amount`] dengan aturan ketat.
    ///
    /// Aturan validasi:
    /// - Hanya digit ASCII (`0`–`9`) dan paling banyak satu titik (`.`).
    /// - Bagian bulat wajib ada dan tidak boleh diawali `'0'` kecuali tepat `"0"`.
    /// - Jika ada titik, wajib diikuti `1` sampai `18` digit ASCII.
    /// - Tanpa tanda `+`/`-`, spasi, notasi eksponen, pemisah ribuan, atau titik di akhir.
    ///
    /// # Errors
    ///
    /// Mengembalikan:
    /// - [`AmountParseError::InvalidFormat`] jika struktur atau karakter melanggar aturan tata bahasa.
    /// - [`AmountParseError::TooManyDecimals`] jika terdapat lebih dari 18 digit sesudah titik.
    /// - [`AmountParseError::Overflow`] jika nilai quanta melampaui `u128::MAX`.
    pub fn from_hsis_str(s: &str) -> Result<Self, AmountParseError> {
        let (int_part, frac_part_opt) = match s.split_once('.') {
            Some((int_part, rest)) => {
                if rest.contains('.') {
                    return Err(AmountParseError::InvalidFormat);
                }
                (int_part, Some(rest))
            }
            None => (s, None),
        };

        if int_part.is_empty() {
            return Err(AmountParseError::InvalidFormat);
        }
        if !int_part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(AmountParseError::InvalidFormat);
        }
        if int_part.len() > 1 && int_part.starts_with('0') {
            return Err(AmountParseError::InvalidFormat);
        }

        if let Some(frac_part) = frac_part_opt {
            if frac_part.is_empty() {
                return Err(AmountParseError::InvalidFormat);
            }
            if !frac_part.bytes().all(|b| b.is_ascii_digit()) {
                return Err(AmountParseError::InvalidFormat);
            }
            if frac_part.len() > MAX_FRACTIONAL_DIGITS {
                return Err(AmountParseError::TooManyDecimals);
            }
        }

        let mut whole: u128 = 0;
        for ch in int_part.chars() {
            let digit = u128::from(ch.to_digit(10).ok_or(AmountParseError::InvalidFormat)?);
            whole = whole
                .checked_mul(10)
                .and_then(|acc| acc.checked_add(digit))
                .ok_or(AmountParseError::Overflow)?;
        }
        let whole_quanta = whole
            .checked_mul(QUANTA_PER_HSIS)
            .ok_or(AmountParseError::Overflow)?;

        let frac_quanta = match frac_part_opt {
            None => 0,
            Some(frac_part) => {
                let mut frac: u128 = 0;
                for ch in frac_part.chars() {
                    let digit = u128::from(ch.to_digit(10).ok_or(AmountParseError::InvalidFormat)?);
                    frac = frac
                        .checked_mul(10)
                        .and_then(|acc| acc.checked_add(digit))
                        .ok_or(AmountParseError::Overflow)?;
                }
                let pad_zeros = MAX_FRACTIONAL_DIGITS
                    .checked_sub(frac_part.len())
                    .ok_or(AmountParseError::TooManyDecimals)?;
                for _ in 0..pad_zeros {
                    frac = frac.checked_mul(10).ok_or(AmountParseError::Overflow)?;
                }
                frac
            }
        };

        let total = whole_quanta
            .checked_add(frac_quanta)
            .ok_or(AmountParseError::Overflow)?;

        Ok(Self(total))
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} HSIS", self.to_hsis_string())
    }
}

#[cfg(test)]
mod tests;
