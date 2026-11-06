//! Target compact, ASERT, kerja blok, chainwork, dan aturan cap waktu.
//!
//! Sesuai `docs/reference/spesifikasi-validasi-v0.1.md` Bagian 3 sampai 6. Semua angka bilangan bulat,
//! dan pembagian dibulatkan ke bawah.

use zevrigo_pow::Target;
use zevrigo_primitives::BlockHeight;
use zevrigo_primitives::params::{
    ASERT_HALF_LIFE_BLOCKS, BLOCK_TIME_SECS, MAX_FUTURE_BLOCK_TIME_SECS, MEDIAN_TIME_SPAN,
};

use crate::uint::U256;

/// Waktu paruh ASERT dalam detik (`720 × 120 = 86_400`).
pub const ASERT_HALF_LIFE_SECS: u64 = ASERT_HALF_LIFE_BLOCKS * BLOCK_TIME_SECS;

/// Basis pecahan eksponen ASERT (`2^16`).
const RADIX: i128 = 65_536;

/// Jumlah bit basis pecahan (`16`).
const RADIX_BITS: i128 = 16;

/// Pembulatan polinomial faktor ASERT (`2^47`).
const FACTOR_ROUNDING: u128 = 1 << 47;

/// Galat perhitungan target dan kesulitan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum DifficultyError {
    /// Encoding target compact tidak sah (Spesifikasi Validasi Bagian 3).
    #[error("target compact {compact:#010x} tidak sah")]
    InvalidCompact {
        /// Nilai compact yang ditolak.
        compact: u32,
    },
    /// Target bernilai nol tidak punya encoding compact.
    #[error("target bernilai nol")]
    ZeroTarget,
    /// Luapan aritmetika. Tidak terjadi untuk masukan `u64`, tetapi tetap dijaga dengan `checked_*`.
    #[error("luapan aritmetika saat menghitung target")]
    ArithmeticOverflow,
}

/// Mengubah target compact `u32` di header menjadi [`Target`] 256-bit (Spesifikasi Validasi Bagian 3).
///
/// Byte teratas adalah jumlah byte target `E` (1 sampai 32), tiga byte berikutnya mantisa `M` dengan byte
/// teratas bukan nol. Tidak ada bit tanda. Untuk `E < 3`, bit mantisa yang terbuang wajib nol.
///
/// # Errors
///
/// [`DifficultyError::InvalidCompact`] jika salah satu syarat di atas dilanggar.
pub fn decode_compact(compact: u32) -> Result<Target, DifficultyError> {
    let invalid = DifficultyError::InvalidCompact { compact };
    let [size, m_hi, m_mid, m_lo] = compact.to_be_bytes();
    if !(1..=32).contains(&size) || m_hi == 0 {
        return Err(invalid);
    }
    let start = 32_usize.checked_sub(usize::from(size)).ok_or(invalid)?;
    let mut out = [0_u8; 32];
    for (offset, byte) in [m_hi, m_mid, m_lo].into_iter().enumerate() {
        match start
            .checked_add(offset)
            .and_then(|index| out.get_mut(index))
        {
            Some(slot) => *slot = byte,
            None if byte == 0 => {}
            None => return Err(invalid),
        }
    }
    Ok(Target::from_be_bytes(out))
}

/// Mengubah [`Target`] menjadi encoding compact `u32` kanonis (Spesifikasi Validasi Bagian 3).
///
/// Mantisa adalah tiga byte teratas target, dibulatkan ke bawah.
///
/// # Errors
///
/// [`DifficultyError::ZeroTarget`] jika target bernilai nol.
pub fn encode_compact(target: &Target) -> Result<u32, DifficultyError> {
    let bytes = target.to_be_bytes();
    let first = bytes
        .iter()
        .position(|byte| *byte != 0)
        .ok_or(DifficultyError::ZeroTarget)?;
    let byte_at = |offset: usize| {
        first
            .checked_add(offset)
            .and_then(|index| bytes.get(index))
            .copied()
            .unwrap_or(0)
    };
    // `first` paling besar 31, jadi `32 - first` tidak pernah jenuh dan selalu muat di u8.
    let size = u8::try_from(32_usize.saturating_sub(first))
        .map_err(|_| DifficultyError::ArithmeticOverflow)?;
    Ok(u32::from_be_bytes([
        size,
        byte_at(0),
        byte_at(1),
        byte_at(2),
    ]))
}

/// Target ASERT untuk blok di tinggi `parent_height + 1` (Spesifikasi Validasi Bagian 4).
///
/// Blok acuan adalah genesis: `anchor_target` dan `anchor_timestamp` adalah target dan cap waktu genesis.
/// Hasilnya paling kecil 1 dan paling besar `pow_limit`.
///
/// # Errors
///
/// [`DifficultyError::ArithmeticOverflow`] jika aritmetika meluap (tidak terjadi untuk masukan `u64`).
pub fn asert_target(
    anchor_target: &Target,
    anchor_timestamp: u64,
    parent_height: BlockHeight,
    parent_timestamp: u64,
    pow_limit: &Target,
) -> Result<Target, DifficultyError> {
    let overflow = DifficultyError::ArithmeticOverflow;
    let time_delta = i128::from(parent_timestamp)
        .checked_sub(i128::from(anchor_timestamp))
        .ok_or(overflow)?;
    let ideal = i128::from(BLOCK_TIME_SECS)
        .checked_mul(i128::from(parent_height.get()))
        .ok_or(overflow)?;
    let exponent = time_delta
        .checked_sub(ideal)
        .and_then(|drift| drift.checked_mul(RADIX))
        .and_then(|scaled| scaled.checked_div_euclid(i128::from(ASERT_HALF_LIFE_SECS)))
        .ok_or(overflow)?;
    let shifts = exponent.checked_div_euclid(RADIX).ok_or(overflow)?;
    let frac = exponent.checked_rem_euclid(RADIX).ok_or(overflow)?;
    let factor = u128::try_from(frac)
        .ok()
        .and_then(asert_factor)
        .ok_or(overflow)?;
    let shift = shifts.checked_sub(RADIX_BITS).ok_or(overflow)?;

    let anchor = to_u256(anchor_target);
    let limit = to_u256(pow_limit);
    let next = if shift < 0 {
        let right = u32::try_from(shift.checked_neg().ok_or(overflow)?).unwrap_or(u32::MAX);
        anchor.mul_u64_shr(factor, right)
    } else {
        let left = u32::try_from(shift).unwrap_or(u32::MAX);
        anchor.checked_mul_u64_shl(factor, left)
    };
    // `None` berarti hasilnya ≥ 2^256, jadi pasti di atas `pow_limit`.
    let next = next.unwrap_or(limit);
    let next = if next.is_zero() { U256::ONE } else { next };
    Ok(to_target(next.min(limit)))
}

/// Kerja satu blok: `floor(2^256 ÷ (T + 1))` (Spesifikasi Validasi Bagian 5).
///
/// Mengembalikan `None` hanya untuk target nol, karena hasilnya `2^256` tidak muat di 256 bit.
#[must_use]
pub fn block_work(target: &Target) -> Option<U256> {
    let value = to_u256(target);
    if value.is_zero() {
        return None;
    }
    if value == U256::MAX {
        return Some(U256::ONE);
    }
    // 2^256 ÷ (T + 1) = (2^256 - 1 - T) ÷ (T + 1) + 1, dihitung tanpa bilangan 257 bit.
    let divisor = value.checked_add(U256::ONE)?;
    let complement = U256::MAX.checked_sub(value)?;
    complement.checked_div(divisor)?.checked_add(U256::ONE)
}

/// Chainwork: jumlah [`block_work`] semua target, dari genesis sampai ujung (Spesifikasi Validasi Bagian 5).
///
/// Mengembalikan `None` jika ada target nol atau jumlahnya tidak muat di 256 bit.
#[must_use]
pub fn chain_work(targets: &[Target]) -> Option<U256> {
    targets.iter().try_fold(U256::ZERO, |total, target| {
        total.checked_add(block_work(target)?)
    })
}

/// Median cap waktu blok sebelumnya (Spesifikasi Validasi Bagian 6, V7).
///
/// `prev_timestamps` urut dari blok lama ke blok baru. Yang dipakai paling banyak 11 terakhir;
/// hasilnya elemen ke-`k ÷ 2` setelah diurutkan naik. Mengembalikan `None` jika kosong.
#[must_use]
pub fn median_time_past(prev_timestamps: &[u64]) -> Option<u64> {
    let start = prev_timestamps.len().saturating_sub(MEDIAN_TIME_SPAN);
    let mut window = prev_timestamps.get(start..)?.to_vec();
    window.sort_unstable();
    let middle = window.len().checked_div(2)?;
    window.get(middle).copied()
}

/// Aturan V7: cap waktu wajib lebih besar dari [`median_time_past`].
///
/// Tanpa blok sebelumnya (genesis), aturan ini tidak berlaku dan hasilnya `true`.
#[must_use]
pub fn timestamp_after_median(timestamp: u64, prev_timestamps: &[u64]) -> bool {
    median_time_past(prev_timestamps).is_none_or(|median| timestamp > median)
}

/// Aturan V8: cap waktu wajib ≤ `now + 3_600`, dengan `now` jam lokal node yang diberikan pemanggil.
///
/// Blok yang gagal aturan ini ditunda, bukan tidak sah. Penjumlahan sengaja saturating: `now` di dekat
/// `u64::MAX` tidak mungkin terjadi pada jam nyata.
#[must_use]
pub fn timestamp_within_future_limit(timestamp: u64, now: u64) -> bool {
    timestamp <= now.saturating_add(MAX_FUTURE_BLOCK_TIME_SECS)
}

/// Faktor `2^(frac / 65536)` dalam basis `2^16`: polinomial aserti3-2d, hasil 65_536 sampai 131_071.
fn asert_factor(frac: u128) -> Option<u64> {
    let frac2 = frac.checked_mul(frac)?;
    let frac3 = frac2.checked_mul(frac)?;
    let poly = 195_766_423_245_049_u128
        .checked_mul(frac)?
        .checked_add(971_821_376_u128.checked_mul(frac2)?)?
        .checked_add(5_127_u128.checked_mul(frac3)?)?
        .checked_add(FACTOR_ROUNDING)?;
    let factor = 65_536_u128.checked_add(poly.wrapping_shr(48))?;
    u64::try_from(factor).ok()
}

fn to_u256(target: &Target) -> U256 {
    U256::from_be_bytes(target.to_be_bytes())
}

fn to_target(value: U256) -> Target {
    Target::from_be_bytes(value.to_be_bytes())
}

#[cfg(test)]
mod tests;
