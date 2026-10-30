//! Bilangan bulat tak bertanda 256-bit ([`U256`]) untuk target, kerja blok, dan chainwork.
//!
//! Ditulis sendiri tanpa crate bignum (Spesifikasi Validasi Bagian 11). Tidak ada operasi yang bisa panic.

use core::cmp::Ordering;

/// Jumlah limb `u64` di dalam [`U256`].
const LIMBS: usize = 4;

/// Jumlah limb hasil perkalian `U256 × u64`.
const WIDE: usize = 5;

/// Jumlah bit [`U256`].
const BITS: u32 = 256;

/// Bilangan bulat tak bertanda 256-bit.
///
/// Disimpan sebagai empat limb `u64`, limb pertama paling tidak signifikan. Urutan [`Ord`] adalah urutan numerik.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct U256([u64; LIMBS]);

impl U256 {
    /// Nilai `0`.
    pub const ZERO: Self = Self([0; LIMBS]);

    /// Nilai `1`.
    pub const ONE: Self = Self([1, 0, 0, 0]);

    /// Nilai terbesar, `2^256 - 1`.
    pub const MAX: Self = Self([u64::MAX; LIMBS]);

    /// Membentuk [`U256`] dari `u128`.
    #[must_use]
    pub const fn from_u128(value: u128) -> Self {
        Self([low_u64(value), low_u64(value.wrapping_shr(64)), 0, 0])
    }

    /// Membentuk [`U256`] dari 32 byte big-endian.
    #[must_use]
    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        let mut limbs = [0_u64; LIMBS];
        let (chunks, _) = bytes.as_slice().as_chunks::<8>();
        for (limb, chunk) in limbs.iter_mut().rev().zip(chunks) {
            *limb = u64::from_be_bytes(*chunk);
        }
        Self(limbs)
    }

    /// Mengembalikan 32 byte big-endian.
    #[must_use]
    pub fn to_be_bytes(&self) -> [u8; 32] {
        let mut out = [0_u8; 32];
        let (chunks, _) = out.as_mut_slice().as_chunks_mut::<8>();
        for (chunk, limb) in chunks.iter_mut().zip(self.0.iter().rev()) {
            *chunk = limb.to_be_bytes();
        }
        out
    }

    /// Mengembalikan `true` jika nilainya `0`.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == [0; LIMBS]
    }

    /// Jumlah bit bermakna: `0` untuk nilai `0`, `256` jika bit teratas menyala.
    #[must_use]
    pub fn bits(&self) -> u32 {
        let mut top = BITS;
        for limb in self.0.iter().rev() {
            if *limb != 0 {
                return top.saturating_sub(limb.leading_zeros());
            }
            // `top` turun dari 256 ke 64 dengan langkah 64, sehingga tidak pernah jenuh.
            top = top.saturating_sub(64);
        }
        0
    }

    /// Penjumlahan. Mengembalikan `None` jika hasilnya tidak muat di 256 bit.
    #[must_use]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        let mut out = [0_u64; LIMBS];
        let mut carry = false;
        for ((slot, a), b) in out.iter_mut().zip(self.0).zip(rhs.0) {
            let (sum, carry_a) = a.overflowing_add(b);
            let (sum, carry_b) = sum.overflowing_add(u64::from(carry));
            *slot = sum;
            carry = carry_a || carry_b;
        }
        if carry { None } else { Some(Self(out)) }
    }

    /// Pengurangan. Mengembalikan `None` jika `rhs > self`.
    #[must_use]
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        let (diff, borrow) = self.overflowing_sub(rhs);
        if borrow { None } else { Some(diff) }
    }

    /// Pembagian bilangan bulat, dibulatkan ke bawah. Mengembalikan `None` jika `rhs = 0`.
    #[must_use]
    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.is_zero() {
            return None;
        }
        let mut quotient = Self::ZERO;
        let mut remainder = Self::ZERO;
        for index in (0..BITS).rev() {
            // remainder < rhs, jadi 2 × remainder + 1 < 2^257: cukup satu bit carry.
            let (shifted, carry) = remainder.shl1(self.bit(index));
            let (next_quotient, _) = quotient.shl1(false);
            if carry || shifted >= rhs {
                // Nilai sebenarnya (carry × 2^256 + shifted) ≥ rhs dan selisihnya < rhs,
                // sehingga pengurangan yang wrap memberi hasil yang tepat.
                remainder = shifted.overflowing_sub(rhs).0;
                quotient = next_quotient.with_low_bit();
            } else {
                remainder = shifted;
                quotient = next_quotient;
            }
        }
        Some(quotient)
    }

    /// `floor(self × factor ÷ 2^shift)`. Mengembalikan `None` jika hasilnya tidak muat di 256 bit.
    pub(crate) fn mul_u64_shr(self, factor: u64, shift: u32) -> Option<Self> {
        from_wide(shr_limbs(self.mul_wide(factor), shift))
    }

    /// `self × factor × 2^shift`. Mengembalikan `None` jika hasilnya tidak muat di 256 bit.
    pub(crate) fn checked_mul_u64_shl(self, factor: u64, shift: u32) -> Option<Self> {
        let product = from_wide(self.mul_wide(factor))?;
        if product.is_zero() {
            return Some(Self::ZERO);
        }
        if product.bits().saturating_add(shift) > BITS {
            return None;
        }
        Some(Self(shl_limbs(product.0, shift)))
    }

    fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
        let mut out = [0_u64; LIMBS];
        let mut borrow = false;
        for ((slot, a), b) in out.iter_mut().zip(self.0).zip(rhs.0) {
            let (diff, borrow_a) = a.overflowing_sub(b);
            let (diff, borrow_b) = diff.overflowing_sub(u64::from(borrow));
            *slot = diff;
            borrow = borrow_a || borrow_b;
        }
        (Self(out), borrow)
    }

    fn mul_wide(self, factor: u64) -> [u64; WIDE] {
        let mut out = [0_u64; WIDE];
        let mut carry = 0_u64;
        for (slot, limb) in out.iter_mut().zip(self.0) {
            // limb × factor + carry ≤ (2^64 - 1)^2 + (2^64 - 1) < 2^128, jadi tidak pernah wrap.
            let wide = u128::from(limb)
                .wrapping_mul(u128::from(factor))
                .wrapping_add(u128::from(carry));
            *slot = low_u64(wide);
            carry = low_u64(wide.wrapping_shr(64));
        }
        if let Some(top) = out.last_mut() {
            *top = carry;
        }
        out
    }

    fn shl1(self, low_bit: bool) -> (Self, bool) {
        let mut out = [0_u64; LIMBS];
        let mut carry = u64::from(low_bit);
        for (slot, limb) in out.iter_mut().zip(self.0) {
            *slot = limb.wrapping_shl(1) | carry;
            carry = limb.wrapping_shr(63);
        }
        (Self(out), carry != 0)
    }

    fn bit(&self, index: u32) -> bool {
        let limb = usize::try_from(index.wrapping_shr(6))
            .ok()
            .and_then(|position| self.0.get(position))
            .copied()
            .unwrap_or(0);
        (limb.wrapping_shr(index & 63) & 1) == 1
    }

    fn with_low_bit(self) -> Self {
        let [l0, l1, l2, l3] = self.0;
        Self([l0 | 1, l1, l2, l3])
    }
}

impl Ord for U256 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.iter().rev().cmp(other.0.iter().rev())
    }
}

impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// 64 bit terbawah dari `value`. Pemotongan ini disengaja.
const fn low_u64(value: u128) -> u64 {
    value as u64
}

fn from_wide(wide: [u64; WIDE]) -> Option<U256> {
    let [l0, l1, l2, l3, l4] = wide;
    if l4 == 0 {
        Some(U256([l0, l1, l2, l3]))
    } else {
        None
    }
}

/// Geser kiri `shift` bit. Bit yang keluar dari limb teratas dibuang.
fn shl_limbs<const N: usize>(limbs: [u64; N], shift: u32) -> [u64; N] {
    let limb_shift = usize::try_from(shift.wrapping_shr(6)).unwrap_or(usize::MAX);
    let bit_shift = shift & 63;
    let mut out = [0_u64; N];
    for (index, slot) in out.iter_mut().enumerate() {
        let Some(source) = index.checked_sub(limb_shift) else {
            continue;
        };
        let current = limbs.get(source).copied().unwrap_or(0);
        let below = source
            .checked_sub(1)
            .and_then(|position| limbs.get(position))
            .copied()
            .unwrap_or(0);
        *slot = if bit_shift == 0 {
            current
        } else {
            current.wrapping_shl(bit_shift) | below.wrapping_shr(64_u32.wrapping_sub(bit_shift))
        };
    }
    out
}

/// Geser kanan `shift` bit (dibulatkan ke bawah). Geser ≥ lebar total menghasilkan nol.
fn shr_limbs<const N: usize>(limbs: [u64; N], shift: u32) -> [u64; N] {
    let limb_shift = usize::try_from(shift.wrapping_shr(6)).unwrap_or(usize::MAX);
    let bit_shift = shift & 63;
    let mut out = [0_u64; N];
    for (index, slot) in out.iter_mut().enumerate() {
        let Some(source) = index.checked_add(limb_shift) else {
            continue;
        };
        let current = limbs.get(source).copied().unwrap_or(0);
        let above = source
            .checked_add(1)
            .and_then(|position| limbs.get(position))
            .copied()
            .unwrap_or(0);
        *slot = if bit_shift == 0 {
            current
        } else {
            current.wrapping_shr(bit_shift) | above.wrapping_shl(64_u32.wrapping_sub(bit_shift))
        };
    }
    out
}

#[cfg(test)]
mod tests;
