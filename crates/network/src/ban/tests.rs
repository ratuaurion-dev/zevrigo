use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::{Duration, Instant};

use libp2p::identity::Keypair;

use super::{BAN_DURATION, BanList};

#[test]
fn bans_last_24_hours() {
    let peer = Keypair::generate_ed25519().public().to_peer_id();
    let other = Keypair::generate_ed25519().public().to_peer_id();
    let ip = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7));
    let now = Instant::now();
    let mut bans = BanList::new();
    bans.ban(peer, Some(ip), now);
    assert!(bans.is_peer_banned(&peer, now));
    assert!(!bans.is_peer_banned(&other, now));
    assert!(bans.is_ip_banned(&ip, now));

    let almost = now
        .checked_add(BAN_DURATION - Duration::from_secs(1))
        .unwrap();
    assert!(bans.expire(almost).is_empty());
    assert!(bans.is_peer_banned(&peer, almost));

    let later = now.checked_add(BAN_DURATION).unwrap();
    assert!(!bans.is_peer_banned(&peer, later));
    assert!(!bans.is_ip_banned(&ip, later));
    assert_eq!(bans.expire(later), vec![peer]);
    assert!(bans.expire(later).is_empty());
}

#[test]
fn loopback_addresses_are_not_banned_by_ip() {
    let peer = Keypair::generate_ed25519().public().to_peer_id();
    let now = Instant::now();
    let mut bans = BanList::new();
    let v4 = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let v6 = IpAddr::V6(Ipv6Addr::LOCALHOST);
    bans.ban(peer, Some(v4), now);
    bans.ban(peer, Some(v6), now);
    assert!(bans.is_peer_banned(&peer, now));
    assert!(!bans.is_ip_banned(&v4, now));
    assert!(!bans.is_ip_banned(&v6, now));
}
