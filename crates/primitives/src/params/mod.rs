//! Konstanta parameter protokol Homeostasis Coin (`HSIS`) sesuai sumber kebenaran di `AGENTS.md`.
//!
//! Nilai di modul ini bersifat tetap dan diperiksa konsistensinya pada waktu kompilasi.
//! Parameter yang baru ditetapkan saat peluncuran testnet sengaja belum dimasukkan di sini.

use crate::amount::{Amount, QUANTA_PER_HSIS};

/// Waktu antar-blok target dalam satuan detik (`120` detik = 2 menit).
pub const BLOCK_TIME_SECS: u64 = 120;

/// Jumlah blok konstan per tahun protokol (`365 × 24 × 3600 / 120 = 262_800` blok).
pub const BLOCKS_PER_YEAR: u64 = 262_800;

/// Pasokan utama `S` yang berasal dari penambangan (`43_800_000 HSIS` dalam quanta).
pub const MAIN_SUPPLY: Amount = Amount::from_quanta(43_800_000 * QUANTA_PER_HSIS);

/// Alokasi genesis pencipta `G` (`4_380_000 HSIS` dalam quanta), hanya di blok genesis, di luar `S`, tanpa kunci waktu.
pub const GENESIS_ALLOCATION: Amount = Amount::from_quanta(4_380_000 * QUANTA_PER_HSIS);

/// Konstanta pembagi peluruhan emisi `D` (`2_274_842`) pada rumus hadiah dasar `base(M) = max((S - M) / D, T)`.
pub const HALF_LIFE_DIVISOR: u128 = 2_274_842;

/// Hadiah ekor minimum `T` per blok (`1 HSIS = 10^18` quanta).
pub const TAIL_REWARD: Amount = Amount::from_quanta(QUANTA_PER_HSIS);

/// Ambang tinggi blok fase *slow start* (`10_080` blok; untuk `1 <= h < 10_080`, `subsidy = base(M) * h / 10_080`).
pub const SLOW_START_BLOCKS: u64 = 10_080;

/// Jumlah konfirmasi blok sebelum output coinbase boleh dibelanjakan (`100` blok).
pub const COINBASE_MATURITY: u64 = 100;

/// Ukuran blok terenkode maksimum dalam satuan byte (`100_000` byte).
pub const MAX_BLOCK_SIZE: usize = 100_000;

/// Interval pergantian kunci dataset PoW RandomX dalam jumlah blok (`2_048` blok).
pub const RANDOMX_KEY_INTERVAL: u64 = 2_048;

/// Jeda tinggi blok kunci RandomX sebelum kunci baru aktif (`64` blok).
pub const RANDOMX_KEY_LAG: u64 = 64;

/// Waktu paruh penyesuaian kesulitan ASERT per blok dalam satuan blok (`720` blok).
pub const ASERT_HALF_LIFE_BLOCKS: u64 = 720;

/// Jumlah blok terakhir yang dipakai untuk menghitung batas bawah cap waktu median (`11` blok).
pub const MEDIAN_TIME_SPAN: usize = 11;

/// Toleransi maksimum cap waktu blok di masa depan relatif terhadap waktu lokal node (`3_600` detik = 1 jam).
pub const MAX_FUTURE_BLOCK_TIME_SECS: u64 = 3_600;

/// Jaringan Homeostasis Coin (Spesifikasi Validasi Bagian 2).
///
/// Aturan konsensus sama di semua jaringan. Yang berbeda hanya fungsi PoW, `pow_limit`, dan genesis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Network {
    /// Jaringan utama.
    Mainnet,
    /// Jaringan uji publik.
    Testnet,
    /// Jaringan uji lokal dengan PoW dev (DEC-09).
    Regtest,
}

impl Network {
    /// Ketiga jaringan, berurutan.
    pub const ALL: [Self; 3] = [Self::Mainnet, Self::Testnet, Self::Regtest];

    /// Awalan alamat bech32m (HRP) jaringan ini (Spesifikasi Format Bagian 8).
    #[must_use]
    pub const fn hrp(self) -> &'static str {
        match self {
            Self::Mainnet => "hsis",
            Self::Testnet => "thsis",
            Self::Regtest => "rhsis",
        }
    }
}

// Pemeriksaan invarian parameter protokol pada saat kompilasi:
// 1. Hadiah ekor tepat 0,6% dari S per tahun: T == S * 6 / 1000 / BLOCKS_PER_YEAR
const _: () = assert!(
    TAIL_REWARD.as_quanta() == MAIN_SUPPLY.as_quanta() * 6 / 1_000 / (BLOCKS_PER_YEAR as u128)
);

// 2. Alokasi genesis pencipta tepat 10% dari pasokan utama S: G == S / 10
const _: () = assert!(GENESIS_ALLOCATION.as_quanta() == MAIN_SUPPLY.as_quanta() / 10);

// 3. Hadiah ekor tepat 1 HSIS
const _: () = assert!(TAIL_REWARD.as_quanta() == QUANTA_PER_HSIS);

#[cfg(test)]
mod tests;
