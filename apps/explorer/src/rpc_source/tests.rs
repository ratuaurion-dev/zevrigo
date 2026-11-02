use std::net::SocketAddr;
use std::sync::Arc;

use homeostasis_consensus::regtest_network_id;
use homeostasis_primitives::{Encode, Hash32};
use homeostasis_rpc::{RpcClient, RpcServer};
use tokio::runtime::Runtime;

use super::RpcSource;
use crate::index::Indexer;
use crate::source::ChainSource;
use crate::test_support::{Chain, FakeNode};

#[test]
fn rpc_source_feeds_indexer_and_follows_reorg() {
    let rt = Runtime::new().unwrap();
    let mut chain = Chain::new();
    chain.push(2, Vec::new(), 0);
    chain.push(2, Vec::new(), 0);
    let node = Arc::new(FakeNode::new(chain.clone()));
    let server = rt
        .block_on(RpcServer::start(
            SocketAddr::from(([127, 0, 0, 1], 0)),
            Arc::clone(&node),
        ))
        .unwrap();
    let url = format!("http://{}", server.local_addr());
    let client = rt.block_on(async { RpcClient::new(&url) }).unwrap();
    let source = RpcSource::new(client, rt.handle().clone());

    assert_eq!(source.tip().unwrap(), (2, chain.id(2)));
    let view = source.node_view().unwrap();
    assert_eq!(view.best_block_id, chain.id(2));
    assert_eq!(view.chainwork[31], 3);
    assert_eq!(source.block_id_at(1).unwrap(), Some(chain.id(1)));
    assert_eq!(source.block_id_at(5).unwrap(), None);
    assert_eq!(source.block(&Hash32::from_bytes([7; 32])).unwrap(), None);
    let block = source.block(&chain.id(1)).unwrap().unwrap();
    assert_eq!(block.bytes, chain.blocks[1].to_bytes());
    assert_eq!(block.txids.len(), 1);
    let pending = Hash32::from_bytes([5; 32]);
    node.set_mempool(vec![pending]);
    assert_eq!(source.mempool().unwrap(), vec![pending]);

    let mut index = Indexer::open_in_memory(regtest_network_id().unwrap()).unwrap();
    assert_eq!(index.sync_step(&source).unwrap().added, 3);
    assert_eq!(index.mempool().unwrap(), vec![pending]);

    // Node berpindah ke cabang yang lebih panjang dari genesis: indexer mundur 2 blok lalu maju 3 blok.
    let mut branch = chain.fork_at(0);
    for _ in 0..3 {
        branch.push(4, Vec::new(), 0);
    }
    node.set_chain(branch.clone());
    let report = index.sync_step(&source).unwrap();
    assert_eq!((report.rolled_back, report.added), (2, 3));
    assert_eq!(index.tip().unwrap(), Some((3, branch.id(3))));

    // Node mati: galat sumber, bukan galat pemeriksaan.
    rt.block_on(server.stop());
    assert!(source.tip().is_err());
    assert!(index.sync_step(&source).is_err());
    assert_eq!(index.halted(), None);
}
