use proptest::prelude::*;
use redb::TableDefinition;
use zevrigo_primitives::amount::Amount;
use zevrigo_primitives::block::{Block, Header};
use zevrigo_primitives::hash::Hash32;
use zevrigo_primitives::height::BlockHeight;
use zevrigo_primitives::tx::{Input, Output, OutputKind, Transaction, Witness};
use zevrigo_primitives::utxo::{OutPoint, UtxoEntry};

use super::RedbStore;
use crate::memory::MemoryStore;
use crate::store::{ChainStore, StoreError};

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

/// Blok 1 skenario test 7 dan 8: coinbase txid h(0x02); A txid h(0x03) menghabiskan (h(0x01),0);
/// B txid h(0x04) menghabiskan (h(0x03),0). block_id h(0xb1), prev h(0xb0).
fn scripted_block_1() -> (Block, Hash32, Vec<Hash32>) {
    let cb = make_coinbase(1, vec![make_output(30, 0x02)]);
    let tx_a = make_regular(vec![Input::new(h(0x01), 0)], vec![make_output(8, 0x03)]);
    let tx_b = make_regular(vec![Input::new(h(0x03), 0)], vec![make_output(7, 0x04)]);
    let block = Block::new(make_header(1, h(0xb0)), vec![cb, tx_a, tx_b]).expect("blok 1");
    (block, h(0xb1), vec![h(0x02), h(0x03), h(0x04)])
}

/// Membandingkan isi kedua store: tip, block_id_at 0..=max_height, utxo setiap outpoint, block dan
/// undo_data untuk blk_hash(0) sampai blk_hash(25), serta utxos_by_payloads untuk payload 0..=4 dan 0x99.
fn assert_same_state(mem: &MemoryStore, rdb: &RedbStore, outpoints: &[OutPoint], max_height: u64) {
    assert_eq!(mem.tip(), rdb.tip());
    let payloads: Vec<[u8; 32]> = (0_u8..=4).chain([0x99]).map(|p| [p; 32]).collect();
    assert_eq!(
        mem.utxos_by_payloads(&payloads),
        rdb.utxos_by_payloads(&payloads)
    );
    for height in 0..=max_height {
        assert_eq!(
            mem.block_id_at(BlockHeight::new(height)),
            rdb.block_id_at(BlockHeight::new(height))
        );
    }
    for op in outpoints {
        assert_eq!(mem.utxo(op), rdb.utxo(op));
    }
    for b in 0_u8..=25 {
        let id = blk_hash(b);
        assert_eq!(mem.block(&id), rdb.block(&id));
        assert_eq!(mem.undo_data(&id), rdb.undo_data(&id));
    }
}

/// Blok sah tinggi b seperti test 9 kartu TSK-M10. Mengubah `available` dan mengembalikan
/// (blok, txids, outpoint yang dibuat).
fn build_valid_block(
    b: u8,
    prev: Hash32,
    coinbase_outs: u8,
    reg_count: u8,
    seed: u64,
    available: &mut Vec<OutPoint>,
) -> (Block, Vec<Hash32>, Vec<OutPoint>) {
    let cb_txid = tx_hash(b, 0);
    let mut cb_outputs = Vec::new();
    let mut created = Vec::new();
    for j in 0..coinbase_outs {
        cb_outputs.push(make_output(10, j));
        let op = OutPoint::new(cb_txid, u32::from(j));
        available.push(op);
        created.push(op);
    }
    let mut txs = vec![make_coinbase(u64::from(b), cb_outputs)];
    let mut txids = vec![cb_txid];
    for t in 1..=reg_count {
        let len = u64::try_from(available.len()).expect("len muat u64");
        let Some(pick) = seed.wrapping_add(u64::from(t)).checked_rem(len) else {
            continue;
        };
        let spent_op = available.remove(usize::try_from(pick).expect("indeks muat usize"));
        let txid = tx_hash(b, t);
        txs.push(make_regular(
            vec![Input::new(spent_op.txid(), spent_op.index())],
            vec![make_output(10, t)],
        ));
        txids.push(txid);
        let op = OutPoint::new(txid, 0);
        available.push(op);
        created.push(op);
    }
    let block = Block::new(make_header(u64::from(b), prev), txs).expect("blok sah");
    (block, txids, created)
}

/// Blok cacat untuk op 1 test 10: 0 = txids kosong; 1 = header dan coinbase bertinggi b+1;
/// 2 = prev h(0xee); selain itu (3) = tambahan transaksi tx_hash(b,1) yang menghabiskan (h(0x99),0).
fn build_faulty_block(
    fault: u64,
    b: u8,
    prev: Hash32,
    coinbase_outs: u8,
    valid: Block,
    txids: Vec<Hash32>,
) -> (Block, Vec<Hash32>) {
    match fault {
        0 => (valid, Vec::new()),
        1 => {
            let bad_h = u64::from(b).checked_add(1).expect("b + 1");
            let mut cb_outputs = Vec::new();
            for j in 0..coinbase_outs {
                cb_outputs.push(make_output(10, j));
            }
            let mut txs = vec![make_coinbase(bad_h, cb_outputs)];
            txs.extend(valid.txs().iter().skip(1).cloned());
            (
                Block::new(make_header(bad_h, prev), txs).expect("blok tinggi b+1"),
                txids,
            )
        }
        2 => (
            Block::new(make_header(u64::from(b), h(0xee)), valid.txs().to_vec())
                .expect("blok prev salah"),
            txids,
        ),
        _ => {
            let mut txs = valid.txs().to_vec();
            txs.push(make_regular(
                vec![Input::new(h(0x99), 0)],
                vec![make_output(5, 0x99)],
            ));
            let mut bad_txids = txids;
            bad_txids.push(tx_hash(b, 1));
            (
                Block::new(make_header(u64::from(b), prev), txs).expect("blok input hilang"),
                bad_txids,
            )
        }
    }
}

#[test]
fn in_memory_store_starts_empty() {
    let mut store = RedbStore::in_memory().expect("in_memory");
    assert_eq!(store.tip(), Ok(None));
    assert_eq!(store.utxo(&OutPoint::new(h(0x01), 0)), Ok(None));
    assert_eq!(store.undo_tip(), Err(StoreError::EmptyChain));
}

#[test]
fn scripted_chain_matches_memory_store_step_by_step() {
    let mut mem = MemoryStore::default();
    let mut rdb = RedbStore::in_memory().expect("in_memory");
    let outpoints = [
        OutPoint::new(h(0x01), 0),
        OutPoint::new(h(0x01), 1),
        OutPoint::new(h(0x02), 0),
        OutPoint::new(h(0x03), 0),
        OutPoint::new(h(0x04), 0),
    ];
    let (g, g_id, g_txids, _, _) = make_genesis();
    let (b1, b1_id, b1_txids) = scripted_block_1();

    assert_eq!(mem.apply_block(&g, g_id, &g_txids), Ok(()));
    assert_eq!(rdb.apply_block(&g, g_id, &g_txids), Ok(()));
    assert_same_state(&mem, &rdb, &outpoints, 1);
    assert_eq!(mem.block(&g_id), rdb.block(&g_id));
    assert_eq!(mem.undo_data(&g_id), rdb.undo_data(&g_id));

    assert_eq!(mem.apply_block(&b1, b1_id, &b1_txids), Ok(()));
    assert_eq!(rdb.apply_block(&b1, b1_id, &b1_txids), Ok(()));
    assert_same_state(&mem, &rdb, &outpoints, 1);
    assert_eq!(mem.block(&b1_id), rdb.block(&b1_id));
    assert_eq!(mem.undo_data(&b1_id), rdb.undo_data(&b1_id));

    assert_eq!(mem.undo_tip(), Ok((b1_id, b1.clone())));
    assert_eq!(rdb.undo_tip(), Ok((b1_id, b1)));
    assert_same_state(&mem, &rdb, &outpoints, 1);
    assert_eq!(mem.block(&b1_id), rdb.block(&b1_id));
    assert_eq!(mem.undo_data(&b1_id), rdb.undo_data(&b1_id));

    assert_eq!(mem.undo_tip(), Ok((g_id, g.clone())));
    assert_eq!(rdb.undo_tip(), Ok((g_id, g)));
    assert_same_state(&mem, &rdb, &outpoints, 1);
    assert_eq!(mem.block(&g_id), rdb.block(&g_id));
    assert_eq!(mem.undo_data(&g_id), rdb.undo_data(&g_id));

    assert_eq!(mem.undo_tip(), Err(StoreError::EmptyChain));
    assert_eq!(rdb.undo_tip(), Err(StoreError::EmptyChain));
    assert_same_state(&mem, &rdb, &outpoints, 1);
}

#[test]
fn utxos_by_payloads_matches_memory_store() {
    let mut mem = MemoryStore::default();
    let mut rdb = RedbStore::in_memory().expect("in_memory");
    let (g, g_id, g_txids, e0, e1) = make_genesis();
    let (b1, b1_id, b1_txids) = scripted_block_1();
    assert_eq!(mem.apply_block(&g, g_id, &g_txids), Ok(()));
    assert_eq!(rdb.apply_block(&g, g_id, &g_txids), Ok(()));
    assert_eq!(mem.apply_block(&b1, b1_id, &b1_txids), Ok(()));
    assert_eq!(rdb.apply_block(&b1, b1_id, &b1_txids), Ok(()));

    let all = [[0x01_u8; 32], [0x02; 32], [0x03; 32], [0x04; 32]];
    let expected = vec![
        (OutPoint::new(h(0x01), 1), e1.clone()),
        (
            OutPoint::new(h(0x02), 0),
            UtxoEntry::new(make_output(30, 0x02), BlockHeight::new(1), true),
        ),
        (
            OutPoint::new(h(0x04), 0),
            UtxoEntry::new(make_output(7, 0x04), BlockHeight::new(1), false),
        ),
    ];
    assert_eq!(rdb.utxos_by_payloads(&all), Ok(expected.clone()));
    assert_eq!(mem.utxos_by_payloads(&all), Ok(expected));
    assert_eq!(rdb.utxos_by_payloads(&[[0x99_u8; 32]]), Ok(Vec::new()));
    assert_eq!(rdb.utxos_by_payloads(&[]), Ok(Vec::new()));

    assert_eq!(rdb.undo_tip(), Ok((b1_id, b1)));
    assert_eq!(
        rdb.utxos_by_payloads(&all),
        Ok(vec![
            (OutPoint::new(h(0x01), 0), e0),
            (OutPoint::new(h(0x01), 1), e1)
        ])
    );
}

#[test]
fn reopen_file_preserves_state() {
    let path = std::env::temp_dir().join("zevrigo-tsk-m11-reopen.redb");
    let _ = std::fs::remove_file(&path);
    let (g, g_id, g_txids, _, _) = make_genesis();
    let (b1, b1_id, b1_txids) = scripted_block_1();
    let mut mem = MemoryStore::default();
    assert_eq!(mem.apply_block(&g, g_id, &g_txids), Ok(()));
    assert_eq!(mem.apply_block(&b1, b1_id, &b1_txids), Ok(()));
    let expected_undo = mem.undo_data(&b1_id);
    assert!(matches!(expected_undo, Ok(Some(_))));

    {
        let mut rdb = RedbStore::create(&path).expect("create");
        assert_eq!(rdb.apply_block(&g, g_id, &g_txids), Ok(()));
        assert_eq!(rdb.apply_block(&b1, b1_id, &b1_txids), Ok(()));
    }
    {
        let mut rdb = RedbStore::open(&path).expect("open");
        assert_eq!(rdb.tip(), Ok(Some((BlockHeight::new(1), h(0xb1)))));
        assert!(matches!(rdb.utxo(&OutPoint::new(h(0x04), 0)), Ok(Some(_))));
        assert_eq!(rdb.utxo(&OutPoint::new(h(0x01), 0)), Ok(None));
        assert_eq!(rdb.undo_data(&h(0xb1)), expected_undo);
        assert_eq!(rdb.undo_tip(), Ok((h(0xb1), b1)));
    }
    {
        let rdb = RedbStore::open(&path).expect("open lagi");
        assert_eq!(rdb.tip(), Ok(Some((BlockHeight::new(0), h(0xb0)))));
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn open_rejects_unknown_schema_version() {
    let path = std::env::temp_dir().join("zevrigo-tsk-m11-schema.redb");
    let _ = std::fs::remove_file(&path);
    {
        let _store = RedbStore::create(&path).expect("create");
    }
    {
        let db = redb::Database::open(&path).expect("redb open");
        let write_txn = db.begin_write().expect("begin_write");
        {
            let mut meta = write_txn
                .open_table(TableDefinition::<&str, u64>::new("meta"))
                .expect("open meta");
            meta.insert("schema", 2_u64).expect("insert schema");
        }
        write_txn.commit().expect("commit");
    }
    assert_eq!(
        RedbStore::open(&path).err(),
        Some(StoreError::UnsupportedSchema { found: 2 })
    );
    let _ = std::fs::remove_file(&path);
}

/// Pemeriksaan badan blok cabang yang sama untuk `MemoryStore` dan `RedbStore`; `store` berisi genesis `g`.
fn check_side_blocks<S: ChainStore>(
    store: &mut S,
    g: &Block,
    g_id: Hash32,
    b1: &Block,
    b1_id: Hash32,
) {
    let tip = store.tip();
    assert_eq!(store.side_block(&b1_id), Ok(None));
    assert_eq!(store.remove_side_block(&b1_id), Ok(()));
    assert_eq!(store.put_side_block(b1_id, b1), Ok(()));
    assert_eq!(store.side_block(&b1_id), Ok(Some(b1.clone())));
    // Chain aktif tidak tersentuh: ujung, blok, dan undo data tetap.
    assert_eq!(store.tip(), tip);
    assert_eq!(store.block(&b1_id), Ok(None));
    assert_eq!(store.undo_data(&b1_id), Ok(None));
    // ID yang sama ditimpa; menghapus dua kali bukan galat.
    assert_eq!(store.put_side_block(b1_id, g), Ok(()));
    assert_eq!(store.side_block(&b1_id), Ok(Some(g.clone())));
    assert_eq!(store.remove_side_block(&b1_id), Ok(()));
    assert_eq!(store.side_block(&b1_id), Ok(None));
    assert_eq!(store.remove_side_block(&b1_id), Ok(()));
    // Mengosongkan semua badan blok cabang, chain aktif tetap.
    assert_eq!(store.put_side_block(b1_id, b1), Ok(()));
    assert_eq!(store.put_side_block(h(0xc2), g), Ok(()));
    assert_eq!(store.clear_side_blocks(), Ok(()));
    assert_eq!(store.side_block(&b1_id), Ok(None));
    assert_eq!(store.side_block(&h(0xc2)), Ok(None));
    assert_eq!(store.tip(), tip);
    assert_eq!(store.block(&g_id), Ok(Some(g.clone())));
}

#[test]
fn side_blocks_are_kept_apart_from_active_chain() {
    let (g, g_id, g_txids, _, _) = make_genesis();
    let (b1, b1_id, _) = scripted_block_1();
    let mut mem = MemoryStore::default();
    let mut rdb = RedbStore::in_memory().expect("in_memory");
    assert_eq!(mem.apply_block(&g, g_id, &g_txids), Ok(()));
    assert_eq!(rdb.apply_block(&g, g_id, &g_txids), Ok(()));
    check_side_blocks(&mut mem, &g, g_id, &b1, b1_id);
    check_side_blocks(&mut rdb, &g, g_id, &b1, b1_id);
}

#[test]
fn side_blocks_persist_until_cleared_and_old_files_open() {
    let path = std::env::temp_dir().join("zevrigo-tsk-m35-side.redb");
    let _ = std::fs::remove_file(&path);
    let (b1, b1_id, _) = scripted_block_1();
    {
        let mut rdb = RedbStore::create(&path).expect("create");
        assert_eq!(rdb.put_side_block(b1_id, &b1), Ok(()));
    }
    {
        let mut rdb = RedbStore::open(&path).expect("open");
        assert_eq!(rdb.side_block(&b1_id), Ok(Some(b1.clone())));
        assert_eq!(rdb.clear_side_blocks(), Ok(()));
    }
    {
        let rdb = RedbStore::open(&path).expect("open lagi");
        assert_eq!(rdb.side_block(&b1_id), Ok(None));
    }
    let _ = std::fs::remove_file(&path);

    // File dari versi sebelum TSK-M35 (skema 1 tanpa tabel side_blocks) tetap terbuka.
    {
        let db = redb::Database::create(&path).expect("redb create");
        let write_txn = db.begin_write().expect("begin_write");
        {
            let mut meta = write_txn
                .open_table(TableDefinition::<&str, u64>::new("meta"))
                .expect("open meta");
            meta.insert("schema", 1_u64).expect("insert schema");
        }
        write_txn.commit().expect("commit");
    }
    {
        let mut rdb = RedbStore::open(&path).expect("open file lama");
        assert_eq!(rdb.tip(), Ok(None));
        assert_eq!(rdb.side_block(&b1_id), Ok(None));
        assert_eq!(rdb.put_side_block(b1_id, &b1), Ok(()));
        assert_eq!(rdb.side_block(&b1_id), Ok(Some(b1)));
    }
    let _ = std::fs::remove_file(&path);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn prop_differential_against_memory_store(
        ops in proptest::collection::vec((0_u8..=2, 1_u8..=3, 0_u8..=2, any::<u64>()), 1..=24),
    ) {
        let mut mem = MemoryStore::default();
        let mut rdb = RedbStore::in_memory().expect("in_memory");
        let mut available: Vec<OutPoint> = Vec::new();
        let mut history: Vec<Vec<OutPoint>> = Vec::new();
        let mut seen: Vec<OutPoint> = vec![OutPoint::new(h(0x99), 0)];

        for (op, coinbase_outs, reg_count, seed) in ops {
            let n = history.len();
            let b = u8::try_from(n).expect("tinggi muat u8");
            let prev = match b.checked_sub(1) {
                None => Hash32::ZERO,
                Some(prev_b) => blk_hash(prev_b),
            };
            let block_id = blk_hash(b);
            match op {
                0 => {
                    history.push(available.clone());
                    let (block, txids, created) =
                        build_valid_block(b, prev, coinbase_outs, reg_count, seed, &mut available);
                    seen.extend(created);
                    let res_mem = mem.apply_block(&block, block_id, &txids);
                    let res_rdb = rdb.apply_block(&block, block_id, &txids);
                    prop_assert_eq!(&res_mem, &res_rdb);
                    prop_assert_eq!(res_mem, Ok(()));
                }
                1 => {
                    let mut fault = seed.checked_rem(4).expect("fault");
                    if fault == 2 && n == 0 {
                        fault = 1;
                    }
                    let mut scratch = available.clone();
                    let (valid, txids, _) =
                        build_valid_block(b, prev, coinbase_outs, reg_count, seed, &mut scratch);
                    let (bad, bad_txids) = build_faulty_block(fault, b, prev, coinbase_outs, valid, txids);
                    let res_mem = mem.apply_block(&bad, block_id, &bad_txids);
                    let res_rdb = rdb.apply_block(&bad, block_id, &bad_txids);
                    prop_assert!(res_mem.is_err());
                    prop_assert_eq!(res_mem, res_rdb);
                }
                _ => {
                    let res_mem = mem.undo_tip();
                    let res_rdb = rdb.undo_tip();
                    prop_assert_eq!(&res_mem, &res_rdb);
                    if res_mem.is_ok() {
                        available = history.pop().expect("history sesuai jumlah blok");
                    }
                }
            }
            assert_same_state(&mem, &rdb, &seen, 25);
        }

        for _ in 0..=24 {
            let res_mem = mem.undo_tip();
            let res_rdb = rdb.undo_tip();
            prop_assert_eq!(&res_mem, &res_rdb);
            assert_same_state(&mem, &rdb, &seen, 25);
            if res_mem == Err(StoreError::EmptyChain) {
                break;
            }
        }
        prop_assert_eq!(mem.tip(), Ok(None));
        prop_assert_eq!(rdb.tip(), Ok(None));
    }
}
