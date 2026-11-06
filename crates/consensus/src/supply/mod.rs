//! Aturan konsensus jadwal penerbitan dan pasokan koin Zevrigo (`ZVRG`).
//!
//! Seluruh parameter numerik diambil langsung dari [`zevrigo_primitives::params`]
//! tanpa menyalin ulang konstanta protokol.

use std::iter::FusedIterator;

use zevrigo_primitives::params::{
    GENESIS_ALLOCATION, HALF_LIFE_DIVISOR, MAIN_SUPPLY, SLOW_START_BLOCKS, TAIL_REWARD,
};
use zevrigo_primitives::{Amount, BlockHeight};

/// Galat perhitungan jadwal pasokan koin konsensus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SupplyError {
    /// Terjadi luapan bilangan bulat saat menghitung subsidi blok.
    #[error("luapan aritmetika saat menghitung subsidi blok")]
    ArithmeticOverflow,
}

/// Menghitung hadiah dasar `base(M) = max((S - M) / D, T)` dalam quanta.
///
/// Sesuai aturan protokol:
/// - `S` adalah [`MAIN_SUPPLY`], `D` adalah [`HALF_LIFE_DIVISOR`], dan `T` adalah [`TAIL_REWARD`].
/// - Jika `mined_so_far >= MAIN_SUPPLY`, selisih `(S - M)` dianggap `0` melalui [`Amount::saturating_sub`].
/// - Pembagian bilangan bulat dibulatkan ke bawah dan tidak pernah panic untuk nilai [`Amount`] apa pun.
#[must_use]
pub fn base_reward(mined_so_far: Amount) -> Amount {
    // Aturan protokol: jika M >= S, maka (S - M) ditetapkan menjadi 0 secara saturasi.
    let remaining = MAIN_SUPPLY.saturating_sub(mined_so_far);
    // HALF_LIFE_DIVISOR bernilai konstanta protokol tak-nol, sehingga checked_div selalu menghasilkan Some.
    let decayed_quanta = remaining
        .as_quanta()
        .checked_div(HALF_LIFE_DIVISOR)
        .unwrap_or(0);
    core::cmp::max(Amount::from_quanta(decayed_quanta), TAIL_REWARD)
}

/// Menghitung subsidi tambang pada tinggi blok `height` dengan akumulasi tambang sebelumnya `mined_so_far` (`M`).
///
/// Aturan konsensus:
/// - Genesis (`h == 0`): subsidi tambang `0` (blok genesis hanya memuat alokasi pencipta `G` di luar `S`).
/// - *Slow start* (`1 <= h < SLOW_START_BLOCKS`): `(base_reward(M) * h) / SLOW_START_BLOCKS` (pembagian bulat ke bawah).
/// - Pasca-*slow start* (`h >= SLOW_START_BLOCKS`): `base_reward(M)`.
///
/// # Errors
///
/// Mengembalikan [`SupplyError::ArithmeticOverflow`] bila perkalian `base_reward(M) * h` meluap melewati `u128::MAX`.
pub fn block_subsidy(height: BlockHeight, mined_so_far: Amount) -> Result<Amount, SupplyError> {
    if height == BlockHeight::GENESIS {
        return Ok(Amount::ZERO);
    }

    let base = base_reward(mined_so_far);
    let raw_height = height.get();

    if raw_height < SLOW_START_BLOCKS {
        let numerator = base
            .as_quanta()
            .checked_mul(u128::from(raw_height))
            .ok_or(SupplyError::ArithmeticOverflow)?;
        let denominator = u128::from(SLOW_START_BLOCKS);
        let subsidy_quanta = numerator
            .checked_div(denominator)
            .ok_or(SupplyError::ArithmeticOverflow)?;
        return Ok(Amount::from_quanta(subsidy_quanta));
    }

    Ok(base)
}

/// Menghitung total pasokan koin keseluruhan (`G + mined_total`) sesudah blok dengan akumulasi tambang `mined_total`.
///
/// Mengembalikan `None` jika penjumlahan [`GENESIS_ALLOCATION`] dan `mined_total` meluap melewati `u128::MAX`.
#[must_use]
pub fn total_supply_after(mined_total: Amount) -> Option<Amount> {
    GENESIS_ALLOCATION.checked_add(mined_total)
}

/// Satu langkah keluaran jadwal pasokan pada tinggi blok tertentu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduleEntry {
    /// Tinggi blok entri ini.
    pub height: BlockHeight,
    /// Subsidi tambang baru yang diterbitkan pada blok `height`.
    pub subsidy: Amount,
    /// Akumulasi total hasil tambang dari genesis sampai `height` inklusif (`mined_total_after(h)`).
    pub mined_total_after: Amount,
}

/// Iterator deterministik berbiaya `O(1)` per blok untuk jadwal penerbitan koin mulai dari genesis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupplySchedule {
    next_height: Option<BlockHeight>,
    mined_so_far: Amount,
}

impl SupplySchedule {
    /// Membentuk [`SupplySchedule`] baru yang dimulai dari blok genesis (`h = 0`, `M = 0`).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            next_height: Some(BlockHeight::GENESIS),
            mined_so_far: Amount::ZERO,
        }
    }

    /// Membentuk [`SupplySchedule`] yang melanjutkan dari tinggi `next_height` dan akumulasi tambang `mined_so_far`.
    #[must_use]
    pub const fn from_state(next_height: BlockHeight, mined_so_far: Amount) -> Self {
        Self {
            next_height: Some(next_height),
            mined_so_far,
        }
    }
}

impl Default for SupplySchedule {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for SupplySchedule {
    type Item = ScheduleEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let height = self.next_height?;
        let Ok(subsidy) = block_subsidy(height, self.mined_so_far) else {
            self.next_height = None;
            return None;
        };
        let Some(mined_total_after) = self.mined_so_far.checked_add(subsidy) else {
            self.next_height = None;
            return None;
        };

        self.mined_so_far = mined_total_after;
        self.next_height = height.next();

        Some(ScheduleEntry {
            height,
            subsidy,
            mined_total_after,
        })
    }
}

impl FusedIterator for SupplySchedule {}

#[cfg(test)]
mod tests;
