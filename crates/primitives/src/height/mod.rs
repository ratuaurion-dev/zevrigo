//! Tipe tinggi blok (`BlockHeight`) berbasis `u64` dengan aritmetika terperiksa.

use std::fmt;

/// Tinggi blok di dalam rantai Homeostasis, dimulai dari [`BlockHeight::GENESIS`] (`0`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BlockHeight(u64);

impl BlockHeight {
    /// Tinggi blok genesis (`0`).
    pub const GENESIS: Self = Self(0);

    /// Membentuk [`BlockHeight`] dari nilai mentah `u64`.
    #[must_use]
    pub const fn new(height: u64) -> Self {
        Self(height)
    }

    /// Mengembalikan nilai tinggi blok sebagai `u64`.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Menambah tinggi blok sebesar `delta`, mengembalikan `None` jika meluap melewati `u64::MAX`.
    #[must_use]
    pub const fn checked_add(self, delta: u64) -> Option<Self> {
        match self.0.checked_add(delta) {
            Some(next_height) => Some(Self(next_height)),
            None => None,
        }
    }

    /// Mengurangi tinggi blok sebesar `delta`, mengembalikan `None` jika `self < delta`.
    #[must_use]
    pub const fn checked_sub(self, delta: u64) -> Option<Self> {
        match self.0.checked_sub(delta) {
            Some(prev_height) => Some(Self(prev_height)),
            None => None,
        }
    }

    /// Mengembalikan tinggi blok berikutnya (`self + 1`), atau `None` jika sudah berada di `u64::MAX`.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        self.checked_add(1)
    }
}

impl fmt::Display for BlockHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests;
