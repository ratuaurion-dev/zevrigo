//! Batas laju API explorer (Spesifikasi Explorer v0.1 Bagian 6, TSK-M40): ember token per klien dan satu ember untuk
//! semua klien bersama, sebagai fungsi murni dengan waktu dari pemanggil (milidetik). Semua hitungan bilangan bulat
//! dalam satuan mili-permintaan.
//!
//! Privasi: alamat IP klien tidak disimpan. Kunci ember adalah hash 64-bit alamat IP dengan kunci acak per proses
//! ([`RandomState`]), hanya di memori, tidak pernah ditulis ke log, disk, atau jawaban. Paling banyak [`MAX_CLIENTS`]
//! ember klien; bila penuh, ember yang sudah terisi penuh kembali (klien yang diam) dibuang lebih dulu, dan klien baru
//! sesudah itu berbagi satu ember limpahan. Permintaan tanpa alamat klien juga memakai ember limpahan.

use std::collections::HashMap;
use std::hash::{BuildHasher, RandomState};
use std::net::IpAddr;

/// Satu permintaan dalam satuan mili-permintaan.
const COST: u64 = 1_000;

/// Jumlah ember klien paling banyak di memori.
pub const MAX_CLIENTS: usize = 4_096;

/// Laju satu ember: isi kembali per detik dan kapasitas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rate {
    /// Permintaan yang terisi kembali setiap detik.
    pub per_second: u32,
    /// Permintaan paling banyak sekaligus (kapasitas ember).
    pub burst: u32,
}

/// Batas laju explorer: per klien dan untuk semua klien bersama.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Ember setiap klien (alamat IP).
    pub client: Rate,
    /// Ember bersama semua klien.
    pub global: Rate,
}

impl Limits {
    /// Batas dari opsi `--api-rate <n>`: setiap klien `n` permintaan per detik (kapasitas `4n`), semua klien bersama
    /// `20n` per detik (kapasitas `40n`). `None` untuk `0` (tanpa batas).
    #[must_use]
    pub const fn from_rate(rate: u32) -> Option<Self> {
        if rate == 0 {
            return None;
        }
        Some(Self {
            client: Rate {
                per_second: rate,
                burst: rate.saturating_mul(4),
            },
            global: Rate {
                per_second: rate.saturating_mul(20),
                burst: rate.saturating_mul(40),
            },
        })
    }
}

/// Kapasitas ember dalam mili-permintaan.
fn capacity(rate: Rate) -> u64 {
    u64::from(rate.burst).saturating_mul(COST)
}

/// Satu ember token.
#[derive(Clone, Copy, Debug)]
struct Bucket {
    /// Isi dalam mili-permintaan.
    level: u64,
    /// Waktu isi terakhir dihitung (milidetik).
    updated_ms: u64,
}

impl Bucket {
    fn full(rate: Rate, now_ms: u64) -> Self {
        Self {
            level: capacity(rate),
            updated_ms: now_ms,
        }
    }

    /// Mengisi kembali sesuai waktu yang lewat. Waktu yang mundur tidak mengurangi isi.
    fn refill(&mut self, rate: Rate, now_ms: u64) {
        let elapsed = now_ms.saturating_sub(self.updated_ms);
        let added = elapsed.saturating_mul(u64::from(rate.per_second));
        self.level = self.level.saturating_add(added).min(capacity(rate));
        self.updated_ms = self.updated_ms.max(now_ms);
    }

    const fn ready(&self) -> bool {
        self.level >= COST
    }

    const fn take(&mut self) {
        self.level = self.level.saturating_sub(COST);
    }

    fn is_full(&self, rate: Rate) -> bool {
        self.level >= capacity(rate)
    }
}

/// Pembatas laju: ember per klien, ember limpahan, dan ember bersama.
pub struct RateLimiter {
    limits: Limits,
    keys: RandomState,
    global: Bucket,
    overflow: Bucket,
    clients: HashMap<u64, Bucket>,
}

impl RateLimiter {
    /// Pembatas baru dengan semua ember penuh pada waktu 0.
    #[must_use]
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            keys: RandomState::new(),
            global: Bucket::full(limits.global, 0),
            overflow: Bucket::full(limits.client, 0),
            clients: HashMap::new(),
        }
    }

    /// `true` jika satu permintaan dari `client` pada waktu `now_ms` boleh dilayani, lalu satu token diambil dari ember
    /// klien dan ember bersama. `false` tanpa mengambil token jika salah satu ember kosong.
    pub fn allow(&mut self, client: Option<IpAddr>, now_ms: u64) -> bool {
        let limits = self.limits;
        self.global.refill(limits.global, now_ms);
        let global_ready = self.global.ready();
        let key = client.map(|address| self.keys.hash_one(address));
        let bucket = self.client_bucket(key, now_ms);
        bucket.refill(limits.client, now_ms);
        if !bucket.ready() || !global_ready {
            return false;
        }
        bucket.take();
        self.global.take();
        true
    }

    /// Jumlah ember klien yang sedang disimpan (tanpa ember limpahan).
    #[must_use]
    pub fn tracked(&self) -> usize {
        self.clients.len()
    }

    fn client_bucket(&mut self, key: Option<u64>, now_ms: u64) -> &mut Bucket {
        let rate = self.limits.client;
        let Some(key) = key else {
            return &mut self.overflow;
        };
        if !self.clients.contains_key(&key) && self.clients.len() >= MAX_CLIENTS {
            self.clients.retain(|_, bucket| {
                bucket.refill(rate, now_ms);
                !bucket.is_full(rate)
            });
            if self.clients.len() >= MAX_CLIENTS {
                return &mut self.overflow;
            }
        }
        self.clients
            .entry(key)
            .or_insert_with(|| Bucket::full(rate, now_ms))
    }
}

#[cfg(test)]
mod tests;
