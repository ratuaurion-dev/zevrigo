use proptest::prelude::*;
use zevrigo_primitives::amount::Amount;
use zevrigo_primitives::block::{Block, Header};
use zevrigo_primitives::hash::Hash32;
use zevrigo_primitives::height::BlockHeight;
use zevrigo_primitives::tx::{Input, Output, OutputKind, Transaction, Witness};
use zevrigo_primitives::utxo::{OutPoint, UtxoEntry};

use super::MemoryStore;
use crate::store::{BlockUndo, ChainStore, StoreError};

fn h(byte: u8) -> Hash32 {
    Hash32::from_bytes([byte; 32])
}

fn tx_hash(block: u8, tx: u8) -> Hash32 {
    let mut bytes = [0_u8; 32];
    for (dst, src) in bytes.iter_mut().zip([0x7a, block, tx]) {
        *dst = src;
    }
    Hash32::from_bytes(bytes)
}

fn blk_hash(block: u8) -> Hash32 {
    let mut bytes = [0_u8; 32];
    for (dst, src) in bytes.iter_mut().zip([0x6b, block]) {
        *dst = src;
    }
    Hash32::from_bytes(bytes)
}

fn make_output(q: u128, p: u8) -> Output {
    Output::new(Amount::from_quanta(q), OutputKind::SingleKey, [p; 32]).expect("output")
}

fn make_header(n: u64, prev: Hash32) -> Header {
    Header::new(
        BlockHeight::new(n),
        prev,
        Hash32::ZERO,
        1_800_000_000,
        0x1d00_ffff,
        0,
    )
}

fn make_coinbase(n: u64, outputs: Vec<Output>) -> Transaction {
    let extra = if n == 0 { vec![0_u8; 32] } else { Vec::new() };
    Transaction::coinbase(BlockHeight::new(n), extra, outputs).expect("coinbase")
}

fn make_regular(inputs: Vec<Input>, outputs: Vec<Output>) -> Transaction {
    let witnesses = vec![Witness::single_key([0_u8; 64]); inputs.len()];
    Transaction::regular(inputs, outputs, witnesses).expect("regular")
}

fn make_genesis() -> (Block, Hash32, Vec<Hash32>, UtxoEntry, UtxoEntry) {
    let out0 = make_output(10, 0x01);
    let out1 = make_output(20, 0x02);
    let cb = make_coinbase(0, vec![out0.clone(), out1.clone()]);
    let header = make_header(0, Hash32::ZERO);
    let block = Block::new(header, vec![cb]).expect("genesis block");
    let block_id = h(0xb0);
    let txids = vec![h(0x01)];
    let e0 = UtxoEntry::new(out0, BlockHeight::new(0), true);
    let e1 = UtxoEntry::new(out1, BlockHeight::new(0), true);
    (block, block_id, txids, e0, e1)
}

#[test]
fn empty_store_has_no_tip_no_utxo_and_cannot_undo() {
    let mut store = MemoryStore::default();
    assert_eq!(store.tip(), Ok(None));
    let op = OutPoint::new(h(0x01), 0);
    assert_eq!(store.utxo(&op), Ok(None));
    assert_eq!(store.undo_tip(), Err(StoreError::EmptyChain));
}

#[test]
fn apply_genesis_creates_coinbase_utxos_at_height_zero() {
    let mut store = MemoryStore::default();
    let (g, block_id, txids, e0, e1) = make_genesis();

    assert!(store.apply_block(&g, block_id, &txids).is_ok());

    assert_eq!(store.tip(), Ok(Some((BlockHeight::new(0), h(0xb0)))));
    assert_eq!(store.block_id_at(BlockHeight::new(0)), Ok(Some(h(0xb0))));

    let op0 = OutPoint::new(h(0x01), 0);
    let op1 = OutPoint::new(h(0x01), 1);
    let op2 = OutPoint::new(h(0x01), 2);
    assert_eq!(store.utxo(&op0), Ok(Some(e0)));
    assert_eq!(store.utxo(&op1), Ok(Some(e1)));
    assert_eq!(store.utxo(&op2), Ok(None));

    assert_eq!(store.block(&h(0xb0)), Ok(Some(g)));
    assert_eq!(
        store.undo_data(&h(0xb0)),
        Ok(Some(BlockUndo::new(Vec::new(), vec![op0, op1])))
    );
}

#[test]
fn apply_rejects_txid_count_mismatch_without_changes() {
    let mut store = MemoryStore::default();
    let (g, g_id, g_txids, _, _) = make_genesis();
    store.apply_block(&g, g_id, &g_txids).expect("genesis");
    let snapshot = store.clone();

    let cb1 = make_coinbase(1, vec![make_output(30, 0x03)]);
    let b1 = Block::new(make_header(1, h(0xb0)), vec![cb1]).expect("block 1");

    assert_eq!(
        store.apply_block(&b1, h(0xb1), &[]),
        Err(StoreError::TxidCountMismatch { txs: 1, txids: 0 })
    );
    assert_eq!(store, snapshot);

    assert_eq!(
        store.apply_block(&b1, h(0xb1), &[h(0x02), h(0x03)]),
        Err(StoreError::TxidCountMismatch { txs: 1, txids: 2 })
    );
    assert_eq!(store, snapshot);
}

#[test]
fn apply_rejects_unconnected_block_without_changes() {
    let mut store = MemoryStore::default();
    let snapshot_empty = store.clone();

    // (a) Store kosong, blok tinggi 1
    let cb_h1 = make_coinbase(1, vec![make_output(10, 0x01)]);
    let b_h1 = Block::new(make_header(1, Hash32::ZERO), vec![cb_h1]).expect("b_h1");
    assert_eq!(
        store.apply_block(&b_h1, h(0xaa), &[h(0x01)]),
        Err(StoreError::UnexpectedHeight {
            expected: 0,
            actual: 1
        })
    );
    assert_eq!(store, snapshot_empty);

    // Terapkan genesis
    let (g, g_id, g_txids, _, _) = make_genesis();
    store.apply_block(&g, g_id, &g_txids).expect("genesis");
    let snapshot_g = store.clone();

    // (b) Setelah G, blok tinggi 2
    let cb_h2 = make_coinbase(2, vec![make_output(10, 0x01)]);
    let b_h2 = Block::new(make_header(2, h(0xb0)), vec![cb_h2]).expect("b_h2");
    assert_eq!(
        store.apply_block(&b_h2, h(0xbb), &[h(0x01)]),
        Err(StoreError::UnexpectedHeight {
            expected: 1,
            actual: 2
        })
    );
    assert_eq!(store, snapshot_g);

    // (c) Setelah G, blok tinggi 1 dengan prev h(0xee)
    let cb_prev = make_coinbase(1, vec![make_output(10, 0x01)]);
    let b_prev = Block::new(make_header(1, h(0xee)), vec![cb_prev]).expect("b_prev");
    assert_eq!(
        store.apply_block(&b_prev, h(0xcc), &[h(0x01)]),
        Err(StoreError::PrevMismatch {
            expected: h(0xb0),
            actual: h(0xee)
        })
    );
    assert_eq!(store, snapshot_g);
}

#[test]
fn apply_spend_then_undo_restores_previous_state_exactly() {
    let mut store = MemoryStore::default();
    let (g, g_id, g_txids, e0, e1) = make_genesis();
    store.apply_block(&g, g_id, &g_txids).expect("genesis");
    let s0 = store.clone();

    let cb = make_coinbase(1, vec![make_output(30, 0x03)]);
    let reg = make_regular(
        vec![Input::new(h(0x01), 0)],
        vec![make_output(4, 0x04), make_output(5, 0x04)],
    );
    let b1 = Block::new(make_header(1, h(0xb0)), vec![cb, reg]).expect("b1");
    let txids = vec![h(0x02), h(0x03)];

    assert!(store.apply_block(&b1, h(0xb1), &txids).is_ok());

    let op_g0 = OutPoint::new(h(0x01), 0);
    let op_g1 = OutPoint::new(h(0x01), 1);
    let op_b1_cb = OutPoint::new(h(0x02), 0);
    let op_b1_reg0 = OutPoint::new(h(0x03), 0);
    let op_b1_reg1 = OutPoint::new(h(0x03), 1);

    assert_eq!(store.utxo(&op_g0), Ok(None));
    assert_eq!(store.utxo(&op_g1), Ok(Some(e1)));

    let utxo_cb = store.utxo(&op_b1_cb).expect("lookup").expect("found");
    assert_eq!(utxo_cb.height(), BlockHeight::new(1));
    assert!(utxo_cb.is_coinbase());

    let utxo_reg0 = store.utxo(&op_b1_reg0).expect("lookup").expect("found");
    assert_eq!(utxo_reg0.height(), BlockHeight::new(1));
    assert!(!utxo_reg0.is_coinbase());

    let utxo_reg1 = store.utxo(&op_b1_reg1).expect("lookup").expect("found");
    assert_eq!(utxo_reg1.height(), BlockHeight::new(1));
    assert!(!utxo_reg1.is_coinbase());

    let expected_undo = BlockUndo::new(vec![(op_g0, e0)], vec![op_b1_cb, op_b1_reg0, op_b1_reg1]);
    assert_eq!(store.undo_data(&h(0xb1)), Ok(Some(expected_undo)));

    assert_eq!(store.undo_tip(), Ok((h(0xb1), b1)));
    assert_eq!(store, s0);

    assert_eq!(store.undo_tip(), Ok((h(0xb0), g)));
    assert_eq!(store, MemoryStore::default());

    assert_eq!(store.undo_tip(), Err(StoreError::EmptyChain));
}

#[test]
fn apply_rejects_missing_utxo_and_double_spend_without_changes() {
    let mut store = MemoryStore::default();
    let (g, g_id, g_txids, _, _) = make_genesis();
    store.apply_block(&g, g_id, &g_txids).expect("genesis");
    let snapshot = store.clone();

    // (a) Menghabiskan outpoint yang tidak ada: (h(0x99), 0)
    let cb_a = make_coinbase(1, vec![make_output(10, 0x01)]);
    let reg_a = make_regular(vec![Input::new(h(0x99), 0)], vec![make_output(5, 0x02)]);
    let b_a = Block::new(make_header(1, h(0xb0)), vec![cb_a, reg_a]).expect("b_a");
    assert_eq!(
        store.apply_block(&b_a, h(0xb1), &[h(0x02), h(0x03)]),
        Err(StoreError::MissingUtxo {
            outpoint: OutPoint::new(h(0x99), 0)
        })
    );
    assert_eq!(store, snapshot);

    // (b) Double spend di blok yang sama: tx 1 dan tx 2 menghabiskan (h(0x01), 0)
    let cb_b = make_coinbase(1, vec![make_output(10, 0x01)]);
    let reg_b1 = make_regular(vec![Input::new(h(0x01), 0)], vec![make_output(5, 0x03)]);
    let reg_b2 = make_regular(vec![Input::new(h(0x01), 0)], vec![make_output(5, 0x04)]);
    let b_b = Block::new(make_header(1, h(0xb0)), vec![cb_b, reg_b1, reg_b2]).expect("b_b");
    assert_eq!(
        store.apply_block(&b_b, h(0xb1), &[h(0x02), h(0x03), h(0x04)]),
        Err(StoreError::MissingUtxo {
            outpoint: OutPoint::new(h(0x01), 0)
        })
    );
    assert_eq!(store, snapshot);
}

#[test]
fn apply_rejects_duplicate_outpoint_without_changes() {
    let mut store = MemoryStore::default();
    let (g, g_id, g_txids, _, _) = make_genesis();
    store.apply_block(&g, g_id, &g_txids).expect("genesis");
    let snapshot = store.clone();

    // (a) Coinbase blok 1 memakai txid h(0x01) yang sudah ada di UTXO genesis
    let cb_a = make_coinbase(1, vec![make_output(10, 0x01)]);
    let b_a = Block::new(make_header(1, h(0xb0)), vec![cb_a]).expect("b_a");
    assert_eq!(
        store.apply_block(&b_a, h(0xb1), &[h(0x01)]),
        Err(StoreError::DuplicateOutpoint {
            outpoint: OutPoint::new(h(0x01), 0)
        })
    );
    assert_eq!(store, snapshot);

    // (b) Dua transaksi di blok yang sama memakai txid yang sama: h(0x02)
    let cb_b = make_coinbase(1, vec![make_output(10, 0x01)]);
    let reg_b = make_regular(vec![Input::new(h(0x01), 0)], vec![make_output(5, 0x02)]);
    let b_b = Block::new(make_header(1, h(0xb0)), vec![cb_b, reg_b]).expect("b_b");
    assert_eq!(
        store.apply_block(&b_b, h(0xb1), &[h(0x02), h(0x02)]),
        Err(StoreError::DuplicateOutpoint {
            outpoint: OutPoint::new(h(0x02), 0)
        })
    );
    assert_eq!(store, snapshot);
}

#[test]
fn output_spent_in_same_block_is_not_in_undo_data() {
    let mut store = MemoryStore::default();
    let (g, g_id, g_txids, e0, _) = make_genesis();
    store.apply_block(&g, g_id, &g_txids).expect("genesis");
    let s0 = store.clone();

    let cb = make_coinbase(1, vec![make_output(30, 0x02)]);
    let tx_a = make_regular(vec![Input::new(h(0x01), 0)], vec![make_output(8, 0x03)]);
    let tx_b = make_regular(vec![Input::new(h(0x03), 0)], vec![make_output(7, 0x04)]);
    let b1 = Block::new(make_header(1, h(0xb0)), vec![cb, tx_a, tx_b]).expect("b1");
    let txids = vec![h(0x02), h(0x03), h(0x04)];

    assert!(store.apply_block(&b1, h(0xb1), &txids).is_ok());

    let op_a = OutPoint::new(h(0x03), 0);
    let op_b = OutPoint::new(h(0x04), 0);
    let op_cb = OutPoint::new(h(0x02), 0);
    let op_g0 = OutPoint::new(h(0x01), 0);

    assert_eq!(store.utxo(&op_a), Ok(None));
    assert!(store.utxo(&op_b).unwrap().is_some());

    let expected_undo = BlockUndo::new(vec![(op_g0, e0)], vec![op_cb, op_b]);
    assert_eq!(store.undo_data(&h(0xb1)), Ok(Some(expected_undo)));

    assert_eq!(store.undo_tip(), Ok((h(0xb1), b1)));
    assert_eq!(store, s0);
}

#[test]
fn utxos_by_payloads_filters_sorts_and_follows_undo() {
    let mut store = MemoryStore::default();
    let (g, g_id, g_txids, e0, e1) = make_genesis();
    store.apply_block(&g, g_id, &g_txids).expect("genesis");

    let cb = make_coinbase(1, vec![make_output(30, 0x02)]);
    let reg = make_regular(
        vec![Input::new(h(0x01), 0)],
        vec![make_output(4, 0x02), make_output(5, 0x04)],
    );
    let b1 = Block::new(make_header(1, h(0xb0)), vec![cb, reg]).expect("b1");
    store
        .apply_block(&b1, h(0xb1), &[h(0x02), h(0x03)])
        .expect("b1");

    let g1 = (OutPoint::new(h(0x01), 1), e1.clone());
    let cb1 = (
        OutPoint::new(h(0x02), 0),
        UtxoEntry::new(make_output(30, 0x02), BlockHeight::new(1), true),
    );
    let reg0 = (
        OutPoint::new(h(0x03), 0),
        UtxoEntry::new(make_output(4, 0x02), BlockHeight::new(1), false),
    );
    let reg1 = (
        OutPoint::new(h(0x03), 1),
        UtxoEntry::new(make_output(5, 0x04), BlockHeight::new(1), false),
    );

    assert_eq!(
        store.utxos_by_payloads(&[[0x02_u8; 32]]),
        Ok(vec![g1.clone(), cb1.clone(), reg0.clone()])
    );
    assert_eq!(
        store.utxos_by_payloads(&[[0x04_u8; 32], [0x02; 32], [0x04; 32]]),
        Ok(vec![g1, cb1, reg0, reg1])
    );
    assert_eq!(store.utxos_by_payloads(&[[0x01_u8; 32]]), Ok(Vec::new()));
    assert_eq!(store.utxos_by_payloads(&[]), Ok(Vec::new()));

    assert_eq!(store.undo_tip(), Ok((h(0xb1), b1)));
    assert_eq!(
        store.utxos_by_payloads(&[[0x01_u8; 32], [0x02; 32], [0x04; 32]]),
        Ok(vec![
            (OutPoint::new(h(0x01), 0), e0),
            (OutPoint::new(h(0x01), 1), e1)
        ])
    );
}

proptest! {
    #[test]
    fn prop_apply_chain_then_undo_all_restores_each_snapshot(
        plan in proptest::collection::vec((1_u8..=3, 0_u8..=2, any::<u64>()), 1..=8),
    ) {
        let mut store = MemoryStore::default();
        let mut available: Vec<OutPoint> = Vec::new();
        let mut snapshots: Vec<MemoryStore> = Vec::new();

        for (b_usize, (coinbase_outs, reg_count, seed)) in plan.iter().enumerate() {
            let b = u8::try_from(b_usize).expect("b fits u8");
            let prev = if b == 0 {
                Hash32::ZERO
            } else {
                let prev_b = b.checked_sub(1).expect("prev_b");
                blk_hash(prev_b)
            };

            let mut cb_outputs = Vec::new();
            for j in 0..*coinbase_outs {
                cb_outputs.push(make_output(10, j));
            }
            let cb_tx = make_coinbase(u64::from(b), cb_outputs);
            let cb_txid = tx_hash(b, 0);

            for j in 0..*coinbase_outs {
                let j_u32 = u32::from(j);
                available.push(OutPoint::new(cb_txid, j_u32));
            }

            let mut txs = vec![cb_tx];
            let mut txids = vec![cb_txid];

            for t in 1..=*reg_count {
                if available.is_empty() {
                    continue;
                }
                let t_u64 = u64::from(t);
                let avail_len_u64 = u64::try_from(available.len()).expect("len fits u64");
                let pick_idx_u64 = seed
                    .wrapping_add(t_u64)
                    .checked_rem(avail_len_u64)
                    .expect("rem");
                let pick_idx = usize::try_from(pick_idx_u64).expect("idx fits usize");
                let spent_op = available.remove(pick_idx);

                let reg_txid = tx_hash(b, t);
                let reg_output = make_output(10, t);
                let reg_tx = make_regular(
                    vec![Input::new(spent_op.txid(), spent_op.index())],
                    vec![reg_output],
                );
                txs.push(reg_tx);
                txids.push(reg_txid);
                available.push(OutPoint::new(reg_txid, 0));
            }

            let header = make_header(u64::from(b), prev);
            let block = Block::new(header, txs).expect("valid block");
            let block_id = blk_hash(b);

            snapshots.push(store.clone());
            prop_assert!(store.apply_block(&block, block_id, &txids).is_ok());
        }

        for expected_snapshot in snapshots.into_iter().rev() {
            let undo_res = store.undo_tip();
            prop_assert!(undo_res.is_ok());
            prop_assert_eq!(&store, &expected_snapshot);
        }

        prop_assert_eq!(&store, &MemoryStore::default());
    }

    #[test]
    fn prop_failed_apply_leaves_state_unchanged(
        n in 1_u8..=4,
        fault in 0_u8..=4,
    ) {
        let mut store = MemoryStore::default();

        for b in 0..n {
            let prev = if b == 0 {
                Hash32::ZERO
            } else {
                let prev_b = b.checked_sub(1).expect("prev_b");
                blk_hash(prev_b)
            };
            let cb = make_coinbase(u64::from(b), vec![make_output(10, 0)]);
            let header = make_header(u64::from(b), prev);
            let block = Block::new(header, vec![cb]).expect("block");
            store
                .apply_block(&block, blk_hash(b), &[tx_hash(b, 0)])
                .expect("apply valid block");
        }

        let snapshot = store.clone();
        let prev_tip = blk_hash(n.checked_sub(1).expect("prev_tip"));

        let (faulty_block, faulty_txids) = match fault {
            0 => {
                let cb = make_coinbase(u64::from(n), vec![make_output(10, 0)]);
                let header = make_header(u64::from(n), prev_tip);
                let block = Block::new(header, vec![cb]).expect("block");
                (block, Vec::new())
            }
            1 => {
                let wrong_h = u64::from(n).checked_add(1).expect("wrong_h");
                let cb = make_coinbase(wrong_h, vec![make_output(10, 0)]);
                let header = make_header(wrong_h, prev_tip);
                let block = Block::new(header, vec![cb]).expect("block");
                (block, vec![tx_hash(n, 0)])
            }
            2 => {
                let cb = make_coinbase(u64::from(n), vec![make_output(10, 0)]);
                let header = make_header(u64::from(n), h(0xee));
                let block = Block::new(header, vec![cb]).expect("block");
                (block, vec![tx_hash(n, 0)])
            }
            3 => {
                let cb = make_coinbase(u64::from(n), vec![make_output(10, 0)]);
                let reg = make_regular(
                    vec![Input::new(h(0x99), 0)],
                    vec![make_output(10, 1)],
                );
                let header = make_header(u64::from(n), prev_tip);
                let block = Block::new(header, vec![cb, reg]).expect("block");
                (block, vec![tx_hash(n, 0), tx_hash(n, 1)])
            }
            _ => {
                let cb = make_coinbase(u64::from(n), vec![make_output(10, 0)]);
                let header = make_header(u64::from(n), prev_tip);
                let block = Block::new(header, vec![cb]).expect("block");
                (block, vec![tx_hash(0, 0)])
            }
        };

        let res = store.apply_block(&faulty_block, blk_hash(n), &faulty_txids);
        prop_assert!(res.is_err());
        prop_assert_eq!(&store, &snapshot);
    }
}
