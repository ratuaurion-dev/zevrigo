use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::{Limits, MAX_CLIENTS, Rate, RateLimiter};

fn ip(n: u32) -> Option<IpAddr> {
    Some(IpAddr::V4(Ipv4Addr::from(0x0a00_0000_u32 + n)))
}

fn limits(client: (u32, u32), global: (u32, u32)) -> Limits {
    Limits {
        client: Rate {
            per_second: client.0,
            burst: client.1,
        },
        global: Rate {
            per_second: global.0,
            burst: global.1,
        },
    }
}

#[test]
fn bucket_refills_with_integer_math() {
    let mut limiter = RateLimiter::new(limits((2, 3), (1_000, 1_000)));
    let a = ip(1);
    assert!(limiter.allow(a, 0));
    assert!(limiter.allow(a, 0));
    assert!(limiter.allow(a, 0));
    assert!(!limiter.allow(a, 0));
    // 2 per detik: 499 ms = 998 mili-permintaan (belum cukup), 500 ms = tepat satu permintaan.
    assert!(!limiter.allow(a, 499));
    assert!(limiter.allow(a, 500));
    assert!(!limiter.allow(a, 500));
    // Waktu mundur tidak mengisi ember.
    assert!(!limiter.allow(a, 100));
    // Ember tidak pernah melebihi kapasitas 3.
    for _ in 0..3 {
        assert!(limiter.allow(a, 60_000));
    }
    assert!(!limiter.allow(a, 60_000));
    assert_eq!(Limits::from_rate(5), Some(limits((5, 20), (100, 200))));
    assert_eq!(Limits::from_rate(0), None);
    let huge = Limits::from_rate(u32::MAX).unwrap();
    assert_eq!(huge.global.burst, u32::MAX);
}

#[test]
fn clients_are_separate_and_global_is_shared() {
    let mut limiter = RateLimiter::new(limits((1, 2), (1, 3)));
    let (a, b, c) = (ip(1), ip(2), Some(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    assert!(limiter.allow(a, 0));
    assert!(limiter.allow(a, 0));
    assert!(!limiter.allow(a, 0));
    assert!(limiter.allow(b, 0));
    // Ember bersama kosong: klien c ditolak tanpa kehilangan tokennya sendiri.
    assert!(!limiter.allow(c, 0));
    // 1 detik kemudian ember bersama berisi 1: c (masih penuh) dilayani, a sesudahnya ditolak oleh ember bersama.
    assert!(limiter.allow(c, 1_000));
    assert!(!limiter.allow(a, 1_000));
    assert_eq!(limiter.tracked(), 3);
}

#[test]
fn bounded_clients_without_storing_addresses() {
    let mut limiter = RateLimiter::new(limits((1, 1), (1_000_000, 10_000)));
    let count = u32::try_from(MAX_CLIENTS).unwrap();
    for n in 0..count {
        assert!(limiter.allow(ip(n), 0), "klien {n}");
    }
    assert_eq!(limiter.tracked(), MAX_CLIENTS);
    // Penuh dan tidak ada klien yang diam: klien baru berbagi ember limpahan.
    assert!(limiter.allow(ip(count), 0));
    assert!(!limiter.allow(ip(count + 1), 0));
    assert!(!limiter.allow(None, 0));
    assert_eq!(limiter.tracked(), MAX_CLIENTS);
    // Sesudah 1 detik semua ember penuh kembali: dibuang, lalu klien baru mendapat ember sendiri.
    assert!(limiter.allow(ip(count + 2), 1_000));
    assert_eq!(limiter.tracked(), 1);
    assert!(limiter.allow(None, 1_000));
    assert!(!limiter.allow(None, 1_000));
}
