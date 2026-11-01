//! Daftar blokir rekan (Spesifikasi P2P Bagian 7): 24 jam per `PeerId` dan per alamat IP, hanya di memori.
//!
//! Alamat loopback (`127.0.0.0/8`, `::1`) tidak diblokir per IP, supaya beberapa node di satu mesin (uji TSK-F04) tidak
//! saling memblokir; blokir per `PeerId` tetap berlaku.

use std::collections::BTreeMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

use libp2p::PeerId;

/// Lama blokir (24 jam).
pub const BAN_DURATION: Duration = Duration::from_secs(86_400);

/// Rekan dan alamat IP yang diblokir beserta waktu berakhirnya.
#[derive(Debug, Default)]
pub struct BanList {
    peers: BTreeMap<PeerId, Instant>,
    ips: BTreeMap<IpAddr, Instant>,
}

impl BanList {
    /// Daftar kosong.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Memblokir `peer` (dan `ip` jika bukan loopback) selama [`BAN_DURATION`] sejak `now`.
    pub fn ban(&mut self, peer: PeerId, ip: Option<IpAddr>, now: Instant) {
        let Some(until) = now.checked_add(BAN_DURATION) else {
            return;
        };
        self.peers.insert(peer, until);
        if let Some(ip) = ip.filter(|ip| !ip.is_loopback()) {
            self.ips.insert(ip, until);
        }
    }

    /// `true` jika `peer` masih diblokir pada `now`.
    #[must_use]
    pub fn is_peer_banned(&self, peer: &PeerId, now: Instant) -> bool {
        self.peers.get(peer).is_some_and(|until| now < *until)
    }

    /// `true` jika `ip` masih diblokir pada `now`.
    #[must_use]
    pub fn is_ip_banned(&self, ip: &IpAddr, now: Instant) -> bool {
        self.ips.get(ip).is_some_and(|until| now < *until)
    }

    /// Menghapus blokir yang sudah berakhir pada `now`; mengembalikan rekan yang blokirnya berakhir.
    pub fn expire(&mut self, now: Instant) -> Vec<PeerId> {
        let expired: Vec<PeerId> = self
            .peers
            .iter()
            .filter(|(_, until)| now >= **until)
            .map(|(peer, _)| *peer)
            .collect();
        for peer in &expired {
            self.peers.remove(peer);
        }
        self.ips.retain(|_, until| now < *until);
        expired
    }
}

#[cfg(test)]
mod tests;
