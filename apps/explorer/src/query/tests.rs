use homeostasis_consensus::{SupplySchedule, block_subsidy, regtest_network_id};
use homeostasis_crypto::txid;
use homeostasis_primitives::params::GENESIS_ALLOCATION;
use homeostasis_primitives::{Amount, BlockHeight, Hash32};

use super::SpentBy;
use crate::index::Indexer;
use crate::test_support::{Chain, LIMIT, T0, genesis_output, out, payload, spend};

const FEE: u128 = 1_000;

/// Chain genesis + 3 blok ke pemilik 2; blok 3 memuat transaksi yang membelanjakan alokasi genesis ke pemilik 3
/// (alokasi − 1_500) dan pemilik 2 (500), dengan biaya [`FEE`].
fn indexed() -> (Chain, Indexer, Hash32) {
    let (genesis_txid, allocation, _) = genesis_output();
    let tx = spend(
        &[(genesis_txid, 0)],
        vec![out(allocation - 1_500, 3), out(500, 2)],
    );
    let tx_id = txid(&tx);
    let mut chain = Chain::new();
    chain.push(2, Vec::new(), 0);
    chain.push(2, Vec::new(), 0);
    chain.push(2, vec![tx], FEE);
    let mut index = Indexer::open_in_memory(regtest_network_id().unwrap()).unwrap();
    index.sync_step(&chain.source()).unwrap();
    (chain, index, tx_id)
}

#[test]
fn queries_follow_indexed_chain() {
    let (chain, index, tx_id) = indexed();
    let (genesis_txid, allocation, genesis_payload) = genesis_output();
    let schedule: Vec<_> = SupplySchedule::new().take(4).collect();

    let tip = index.block_at(3).unwrap().unwrap();
    assert_eq!(tip.block.block_id, chain.id(3));
    assert_eq!(tip.block.prev_block_id, chain.id(2));
    assert_eq!(tip.block.timestamp, T0 + 360);
    assert_eq!(tip.block.target, LIMIT);
    assert_eq!(tip.block.tx_count, 2);
    assert_eq!(tip.block.subsidy, schedule[3].subsidy);
    assert_eq!(tip.block.fees, Amount::from_quanta(FEE));
    assert_eq!(tip.txids.len(), 2);
    assert_eq!(tip.txids[1], tx_id);
    assert_eq!(tip.next_block_id, None);
    assert_eq!(
        index.block_at(2).unwrap().unwrap().next_block_id,
        Some(chain.id(3))
    );
    assert_eq!(
        index
            .block_by_id(&chain.id(1))
            .unwrap()
            .unwrap()
            .block
            .height,
        1
    );
    assert_eq!(
        index.block_by_id(&Hash32::from_bytes([7; 32])).unwrap(),
        None
    );
    assert_eq!(index.block_at(4).unwrap(), None);
    let genesis = index.block_at(0).unwrap().unwrap();
    assert_eq!(genesis.block.subsidy, Amount::ZERO);
    assert_eq!(genesis.txids, vec![genesis_txid]);

    let heights =
        |rows: Vec<super::BlockRow>| rows.iter().map(|row| row.height).collect::<Vec<_>>();
    assert_eq!(heights(index.blocks_before(None, 2).unwrap()), vec![3, 2]);
    assert_eq!(
        heights(index.blocks_before(Some(2), 20).unwrap()),
        vec![1, 0]
    );
    assert!(index.blocks_before(Some(0), 20).unwrap().is_empty());

    let detail = index.tx(&tx_id).unwrap().unwrap();
    assert_eq!(detail.tx.height, 3);
    assert_eq!(detail.tx.block_id, chain.id(3));
    assert_eq!(detail.tx.position, 1);
    assert!(!detail.tx.is_coinbase);
    assert_eq!(detail.tx.fee, Amount::from_quanta(FEE));
    assert_eq!(detail.inputs.len(), 1);
    assert_eq!(detail.inputs[0].txid, genesis_txid);
    assert_eq!(detail.inputs[0].amount, Amount::from_quanta(allocation));
    assert_eq!(detail.inputs[0].payload, genesis_payload);
    assert_eq!(
        detail.inputs[0].spent_by,
        Some(SpentBy {
            txid: tx_id,
            input: 0,
            height: 3
        })
    );
    assert_eq!(detail.outputs.len(), 2);
    assert_eq!(
        detail.outputs[0].amount,
        Amount::from_quanta(allocation - 1_500)
    );
    assert_eq!(detail.outputs[0].payload, payload(3));
    assert_eq!(detail.outputs[0].kind, 0);
    assert_eq!(detail.outputs[1].index, 1);
    assert_eq!(detail.outputs[1].spent_by, None);
    let coinbase = index.tx(&tip.txids[0]).unwrap().unwrap();
    assert!(coinbase.tx.is_coinbase);
    assert!(coinbase.inputs.is_empty());
    assert_eq!(coinbase.tx.fee, Amount::ZERO);
    assert_eq!(index.tx(&Hash32::from_bytes([7; 32])).unwrap(), None);

    let receiver = index.address(0, &payload(3), 0, 25).unwrap();
    assert_eq!(receiver.balance, Amount::from_quanta(allocation - 1_500));
    assert_eq!(receiver.received, receiver.balance);
    assert_eq!(receiver.sent, Amount::ZERO);
    assert_eq!((receiver.utxo_count, receiver.tx_count), (1, 1));
    assert_eq!(receiver.txs[0].txid, tx_id);
    let creator = index.address(0, &genesis_payload, 0, 25).unwrap();
    assert_eq!(creator.balance, Amount::ZERO);
    assert_eq!(creator.received, Amount::from_quanta(allocation));
    assert_eq!(creator.sent, Amount::from_quanta(allocation));
    assert_eq!((creator.utxo_count, creator.tx_count), (0, 2));
    assert_eq!(creator.txs[0].txid, tx_id);
    assert_eq!(creator.txs[0].sent, Amount::from_quanta(allocation));
    assert_eq!(creator.txs[1].txid, genesis_txid);
    assert_eq!(creator.txs[1].received, Amount::from_quanta(allocation));
    let nobody = index.address(1, &payload(3), 0, 25).unwrap();
    assert_eq!((nobody.tx_count, nobody.balance), (0, Amount::ZERO));

    let supply = index.supply().unwrap().unwrap();
    assert_eq!(supply.height, 3);
    assert_eq!(supply.mined_total, schedule[3].mined_total_after);
    assert_eq!(supply.scheduled_mined_total, schedule[3].mined_total_after);
    assert!(supply.matches_schedule);
    assert_eq!(supply.genesis_allocation, GENESIS_ALLOCATION);
    assert_eq!(
        supply.total_supply,
        GENESIS_ALLOCATION.checked_add(supply.mined_total).unwrap()
    );
    assert_eq!(supply.fees_total, Amount::from_quanta(FEE));
    assert_eq!(
        supply.next_subsidy,
        block_subsidy(BlockHeight::new(4), supply.mined_total).unwrap()
    );
    assert_eq!(supply.tip_target, LIMIT);
    assert_eq!(supply.average_block_seconds, Some(120));

    let allocation_info = index.allocation().unwrap().unwrap();
    assert_eq!(allocation_info.genesis_block_id, chain.id(0));
    assert_eq!(allocation_info.allocated, GENESIS_ALLOCATION);
    assert_eq!(allocation_info.unspent, Amount::ZERO);
    assert_eq!(allocation_info.outputs.len(), 1);
    assert_eq!(allocation_info.outputs[0].txid, genesis_txid);
    assert_eq!(
        allocation_info.outputs[0].spent_by.map(|by| by.txid),
        Some(tx_id)
    );
}

#[test]
fn address_history_pages_newest_first() {
    let (chain, index, tx_id) = indexed();
    let owner = index.address(0, &payload(2), 0, 2).unwrap();
    // Tiga coinbase dan satu output 500 di blok 3: empat output, tiga transaksi coinbase + satu transaksi biasa.
    assert_eq!((owner.utxo_count, owner.tx_count), (4, 4));
    assert_eq!(owner.sent, Amount::ZERO);
    assert_eq!(owner.balance, owner.received);
    let first: Vec<(u64, Hash32)> = owner.txs.iter().map(|tx| (tx.height, tx.txid)).collect();
    let block_3 = index.block_at(3).unwrap().unwrap();
    assert_eq!(first, vec![(3, tx_id), (3, block_3.txids[0])]);
    assert_eq!(owner.txs[0].received, Amount::from_quanta(500));
    let second = index.address(0, &payload(2), 1, 2).unwrap();
    let heights: Vec<u64> = second.txs.iter().map(|tx| tx.height).collect();
    assert_eq!(heights, vec![2, 1]);
    assert!(index.address(0, &payload(2), 2, 2).unwrap().txs.is_empty());
    assert_eq!(chain.height(), 3);
}
