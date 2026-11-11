use libp2p::Multiaddr;

use super::{DnsTarget, Family, dns_target, is_websocket};

const PEER: &str = "12D3KooWQK1wnefoLrcVHbbnf5tLzbopUd3K3bFAoJpA7YJgL5pV";

fn addr(text: &str) -> Multiaddr {
    text.parse().unwrap()
}

fn target(host: &str, port: u16, family: Family) -> Option<DnsTarget> {
    Some(DnsTarget {
        host: host.to_owned(),
        port,
        family,
    })
}

#[test]
fn dns_target_reads_name_port_and_family() {
    assert_eq!(
        dns_target(&addr("/dns4/seed.example/tcp/443")),
        target("seed.example", 443, Family::V4)
    );
    assert_eq!(
        dns_target(&addr(&format!("/dns6/seed.example/tcp/1/p2p/{PEER}"))),
        target("seed.example", 1, Family::V6)
    );
    assert_eq!(
        dns_target(&addr("/dns/localhost/tcp/28800")),
        target("localhost", 28_800, Family::Any)
    );
    for text in [
        "/ip4/127.0.0.1/tcp/28800",
        "/dns4/seed.example/udp/443",
        "/dns4/seed.example",
        "/dns4/seed.example/tcp/443/ws",
        "/dns4/seed.example/tcp/443/tls/sni/seed.example/ws",
    ] {
        assert_eq!(dns_target(&addr(text)), None, "{text}");
    }
    assert!(Family::Any.accepts("::1".parse().unwrap()));
    assert!(Family::V4.accepts("127.0.0.1".parse().unwrap()));
    assert!(!Family::V4.accepts("::1".parse().unwrap()));
    assert!(!Family::V6.accepts("127.0.0.1".parse().unwrap()));
}

#[test]
fn websocket_addresses_are_recognized() {
    assert!(is_websocket(&addr("/ip4/127.0.0.1/tcp/28803/ws")));
    assert!(is_websocket(&addr(&format!(
        "/dns4/seed.example/tcp/443/tls/sni/seed.example/ws/p2p/{PEER}"
    ))));
    assert!(is_websocket(&addr("/dns4/seed.example/tcp/443/wss")));
    assert!(!is_websocket(&addr("/ip4/127.0.0.1/tcp/28800")));
    assert!(!is_websocket(&addr(&format!(
        "/dns4/seed.example/tcp/28800/p2p/{PEER}"
    ))));
}
