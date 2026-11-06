use std::time::Duration;

use libp2p::identity::Keypair;
use libp2p::multiaddr::Protocol;
use libp2p::{Multiaddr, PeerId};
use zevrigo_primitives::{Block, BlockHeight, Hash32};

use super::{NetEvent, Network, NetworkConfig, Offense, ip_of, peer_of, strip_p2p};
use crate::message::{Ack, Blocks, GetBlocks, GetHeaders, Headers, Status};
use crate::test_support::{id, sample_block, sample_tx};

const NET_A: Hash32 = id(0xa1);
const NET_B: Hash32 = id(0xb2);

fn status(network_id: Hash32, height: u64) -> Status {
    Status::new(network_id, BlockHeight::new(height), id(5), [0; 32])
}

fn node(network_id: Hash32, connect: Vec<Multiaddr>, height: u64) -> Network {
    let mut config = NetworkConfig::new(network_id);
    config.listen = vec!["/ip4/127.0.0.1/tcp/0".parse().unwrap()];
    config.connect = connect;
    config.redial_interval = Duration::from_millis(200);
    Network::new(
        Keypair::generate_ed25519(),
        config,
        status(network_id, height),
    )
    .unwrap()
}

/// Menunggu event pertama yang cocok (paling lama 20 detik); event lain dibuang.
async fn next_matching<F>(net: &mut Network, mut wanted: F) -> NetEvent
where
    F: FnMut(&NetEvent) -> bool,
{
    let wait = async {
        loop {
            let event = net.next_event().await;
            if wanted(&event) {
                return event;
            }
        }
    };
    tokio::time::timeout(Duration::from_secs(20), wait)
        .await
        .unwrap()
}

/// Alamat dengar lengkap dengan `/p2p/<PeerId>`.
async fn dial_address(net: &mut Network) -> Multiaddr {
    let event = next_matching(net, |event| matches!(event, NetEvent::Listening { .. })).await;
    let NetEvent::Listening { address } = event else {
        unreachable!()
    };
    address.with(Protocol::P2p(net.local_peer_id()))
}

/// Rekan uji: menjawab semua permintaan. Jika `short_blocks`, jawaban `Blocks` kurang satu entri.
async fn serve(mut net: Network, short_blocks: bool) {
    loop {
        match net.next_event().await {
            NetEvent::GetHeaders { reply, .. } => {
                let headers = Headers {
                    headers: vec![*sample_block(1).header()],
                };
                net.reply_headers(reply, headers);
            }
            NetEvent::GetBlocks { request, reply, .. } => {
                let mut entries: Vec<Option<Block>> = request.ids.iter().map(|_| None).collect();
                if let Some(first) = entries.first_mut() {
                    *first = Some(sample_block(1));
                }
                if short_blocks {
                    entries.pop();
                }
                net.reply_blocks(reply, Blocks { entries });
            }
            NetEvent::Block { reply, .. } => {
                net.reply_ack(reply, Ack::Accepted);
            }
            NetEvent::Tx { reply, .. } => {
                net.reply_ack(reply, Ack::Rejected);
            }
            _ => {}
        }
    }
}

async fn wait_status(net: &mut Network, from: PeerId) -> Status {
    let event = next_matching(
        net,
        |event| matches!(event, NetEvent::PeerStatus { peer, .. } if *peer == from),
    )
    .await;
    let NetEvent::PeerStatus { status, .. } = event else {
        unreachable!()
    };
    status
}

#[test]
fn address_helpers_find_ip_and_peer() {
    let peer = Keypair::generate_ed25519().public().to_peer_id();
    let plain: Multiaddr = "/ip4/10.1.2.3/tcp/28800".parse().unwrap();
    let full = plain.clone().with(Protocol::P2p(peer));
    assert_eq!(ip_of(&full), Some("10.1.2.3".parse().unwrap()));
    assert_eq!(peer_of(&full), Some(peer));
    assert_eq!(peer_of(&plain), None);
    assert_eq!(strip_p2p(&full), plain);
    let v6: Multiaddr = "/ip6/::1/tcp/1".parse().unwrap();
    assert_eq!(ip_of(&v6), Some("::1".parse().unwrap()));
}

#[tokio::test]
async fn peers_exchange_status_headers_blocks_and_acks() {
    let mut a = node(NET_A, Vec::new(), 7);
    let a_id = a.local_peer_id();
    let a_addr = dial_address(&mut a).await;
    tokio::spawn(serve(a, false));

    let mut b = node(NET_A, vec![a_addr], 0);
    assert_eq!(wait_status(&mut b, a_id).await, status(NET_A, 7));
    assert_eq!(b.connected_peers(), vec![a_id]);

    let sent = b.request_headers(
        &a_id,
        GetHeaders {
            locator: vec![id(1)],
            stop: Hash32::default(),
        },
    );
    let event = next_matching(&mut b, |event| matches!(event, NetEvent::Headers { .. })).await;
    let NetEvent::Headers {
        peer,
        request,
        headers,
    } = event
    else {
        unreachable!()
    };
    assert_eq!((peer, request), (a_id, sent));
    assert_eq!(headers.headers, vec![*sample_block(1).header()]);

    let sent = b.request_blocks(
        &a_id,
        GetBlocks {
            ids: vec![id(1), id(2)],
        },
    );
    let event = next_matching(&mut b, |event| matches!(event, NetEvent::Blocks { .. })).await;
    let NetEvent::Blocks {
        request, blocks, ..
    } = event
    else {
        unreachable!()
    };
    assert_eq!(request, sent);
    assert_eq!(blocks.entries, vec![Some(sample_block(1)), None]);

    b.send_block(&a_id, sample_block(2));
    let event = next_matching(&mut b, |event| matches!(event, NetEvent::BlockAck { .. })).await;
    assert!(matches!(
        event,
        NetEvent::BlockAck {
            ack: Ack::Accepted,
            ..
        }
    ));

    b.send_tx(&a_id, sample_tx());
    let event = next_matching(&mut b, |event| matches!(event, NetEvent::TxAck { .. })).await;
    assert!(matches!(
        event,
        NetEvent::TxAck {
            ack: Ack::Rejected,
            ..
        }
    ));
    assert!(!b.is_banned(&a_id));
}

#[tokio::test]
async fn other_network_is_disconnected_without_ban() {
    let mut a = node(NET_A, Vec::new(), 1);
    let a_id = a.local_peer_id();
    let a_addr = dial_address(&mut a).await;
    tokio::spawn(serve(a, false));

    let mut b = node(NET_B, vec![a_addr], 0);
    let event = next_matching(&mut b, |event| {
        matches!(
            event,
            NetEvent::PeerDisconnected { .. }
                | NetEvent::PeerStatus { .. }
                | NetEvent::PeerPunished { .. }
        )
    })
    .await;
    assert!(matches!(event, NetEvent::PeerDisconnected { peer } if peer == a_id));
    assert!(!b.is_banned(&a_id));
}

#[tokio::test]
async fn wrong_blocks_entry_count_is_punished() {
    let mut a = node(NET_A, Vec::new(), 3);
    let a_id = a.local_peer_id();
    let a_addr = dial_address(&mut a).await;
    tokio::spawn(serve(a, true));

    let mut b = node(NET_A, vec![a_addr], 0);
    wait_status(&mut b, a_id).await;
    b.request_blocks(
        &a_id,
        GetBlocks {
            ids: vec![id(1), id(2)],
        },
    );
    let event = next_matching(&mut b, |event| {
        matches!(event, NetEvent::PeerPunished { .. })
    })
    .await;
    let NetEvent::PeerPunished { peer, offense } = event else {
        unreachable!()
    };
    assert_eq!(peer, a_id);
    assert_eq!(
        offense,
        Offense::WrongEntryCount {
            expected: 2,
            actual: 1
        }
    );
    assert!(b.is_banned(&a_id));
    let event = next_matching(&mut b, |event| {
        matches!(event, NetEvent::PeerDisconnected { .. })
    })
    .await;
    assert!(matches!(event, NetEvent::PeerDisconnected { peer } if peer == a_id));
    assert!(b.connected_peers().is_empty());
}

#[tokio::test]
async fn dialer_reconnects_after_disconnect() {
    let mut a = node(NET_A, Vec::new(), 1);
    let a_id = a.local_peer_id();
    let a_addr = dial_address(&mut a).await;
    tokio::spawn(serve(a, false));

    let mut b = node(NET_A, vec![a_addr], 0);
    wait_status(&mut b, a_id).await;
    b.disconnect(&a_id);
    let event = next_matching(&mut b, |event| {
        matches!(event, NetEvent::PeerDisconnected { .. })
    })
    .await;
    assert!(matches!(event, NetEvent::PeerDisconnected { peer } if peer == a_id));
    let event = next_matching(&mut b, |event| {
        matches!(event, NetEvent::PeerConnected { .. })
    })
    .await;
    assert!(matches!(event, NetEvent::PeerConnected { peer, .. } if peer == a_id));
    assert_eq!(wait_status(&mut b, a_id).await, status(NET_A, 1));
}
