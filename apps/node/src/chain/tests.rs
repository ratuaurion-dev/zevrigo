use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use homeostasis_consensus::{
    BlockError, ChainParams, DifficultyError, SupplySchedule, TxError, check_block_context_free,
    regtest_genesis,
};
use homeostasis_crypto::{SecretKey, block_id, block_merkle_root, sighash_v1, sign, txid};
use homeostasis_mempool::{DEFAULT_MAX_BYTES, MempoolError};
use homeostasis_primitives::params::GENESIS_ALLOCATION;
use homeostasis_primitives::{
    Amount, Block, BlockHeight, Decode, Encode, Hash32, Header, Input, Network, OutPoint, Output,
    OutputKind, Transaction, Witness,
};
use homeostasis_rpc::{
    GetBlockTemplateParams, Hex32, KindNumber, RpcBackend, RpcError, SendRawTransactionResult,
    SubmitBlockResult, SubmitStatus,
};
use homeostasis_storage::{ChainStore, MemoryStore, RedbStore};

use super::{Clock, Node, NodeError, block_rule, mempool_reason};

/// Cap waktu genesis regtest.
const T0: u64 = 1_793_145_600;
/// Alokasi genesis dalam quanta.
const G: u128 = GENESIS_ALLOCATION.as_quanta();
/// Subsidy tinggi 1 (Spesifikasi Format Bagian 5.5).
const SUBSIDY_1: u128 = 1_910_127_426_536_917;
/// Target genesis regtest dan `pow_limit`.
const LIMIT: u32 = 0x207f_ffff;

struct Fixture<S> {
    node: Node<S>,
    now: Arc<AtomicU64>,
}

fn params() -> ChainParams {
    ChainParams::regtest().unwrap()
}

fn clock(now: &Arc<AtomicU64>) -> Clock {
    let now = Arc::clone(now);
    Box::new(move || now.load(Ordering::SeqCst))
}

fn open<S: ChainStore>(store: S) -> Fixture<S> {
    let now = Arc::new(AtomicU64::new(T0));
    let node = Node::open(store, params(), DEFAULT_MAX_BYTES, clock(&now)).unwrap();
    Fixture { node, now }
}

fn secret(n: u8) -> SecretKey {
    let mut bytes = [0_u8; 32];
    if let Some(last) = bytes.last_mut() {
        *last = n;
    }
    SecretKey::from_bytes(&bytes).unwrap()
}

fn key(n: u8) -> [u8; 32] {
    *secret(n).public_key().as_bytes()
}

fn out_to(amount: u128, owner: u8) -> Output {
    Output::new(
        Amount::from_quanta(amount),
        OutputKind::SingleKey,
        key(owner),
    )
    .unwrap()
}

/// Chainwork 256-bit big-endian bernilai `n`.
fn work(n: u8) -> Hex32 {
    let mut bytes = [0_u8; 32];
    if let Some(last) = bytes.last_mut() {
        *last = n;
    }
    Hex32::new(bytes)
}

fn payout(owner: u8) -> GetBlockTemplateParams {
    GetBlockTemplateParams {
        payout_kind: KindNumber::new(OutputKind::SingleKey),
        payout_payload: Hex32::new(key(owner)),
    }
}

/// Header `h` dengan prev, Merkle root, cap waktu, dan target pilihan, nonce 0.
fn header_with(h: &Header, prev: Hash32, merkle: Hash32, timestamp: u64, target: u32) -> Header {
    Header::new(h.height(), prev, merkle, timestamp, target, 0)
}

/// Blok berisi `txs` dengan header seperti `h` dan Merkle root yang benar.
fn with_txs(h: &Header, txs: Vec<Transaction>) -> Block {
    let draft = Block::new(*h, txs.clone()).unwrap();
    let merkle = block_merkle_root(&draft).unwrap();
    let header = header_with(h, h.prev_block_id(), merkle, h.timestamp(), h.target());
    Block::new(header, txs).unwrap()
}

/// Mencari nonce agar PoW lolos (V3). Di `pow_limit` regtest butuh sekitar dua percobaan.
fn solve(block: &Block) -> Block {
    let params = params();
    let h = block.header();
    let mut nonce = 0_u64;
    loop {
        let header = Header::new(
            h.height(),
            h.prev_block_id(),
            h.merkle_root(),
            h.timestamp(),
            h.target(),
            nonce,
        );
        let candidate = Block::new(header, block.txs().to_vec()).unwrap();
        if !matches!(
            check_block_context_free(&candidate, &params),
            Err(BlockError::HighHash)
        ) {
            return candidate;
        }
        nonce += 1;
    }
}

fn template_block<S: ChainStore + Send + 'static>(fx: &Fixture<S>, owner: u8) -> Block {
    let template = fx.node.block_template(&payout(owner)).unwrap();
    Block::from_bytes(template.hex.as_slice()).unwrap()
}

/// Memajukan jam ke jadwal blok berikutnya (120 detik per blok), lalu menambang blok dari template.
fn mine<S: ChainStore + Send + 'static>(fx: &Fixture<S>, owner: u8) -> SubmitBlockResult {
    let height = fx.node.chain_info().unwrap().height + 1;
    fx.now.store(T0 + 120 * height, Ordering::SeqCst);
    let block = solve(&template_block(fx, owner));
    fx.node.submit_block(&block.to_bytes()).unwrap()
}

fn rejected(rule: &str) -> SubmitBlockResult {
    SubmitBlockResult {
        status: SubmitStatus::Invalid,
        block_id: None,
        reason: Some(rule.to_owned()),
    }
}

/// Hasil `invalid` dengan `block_id`, untuk blok yang lolos decode.
fn rejected_block(block: &Block, rule: &str) -> SubmitBlockResult {
    SubmitBlockResult {
        block_id: Some(Hex32::from(block_id(block.header()))),
        ..rejected(rule)
    }
}

/// Transaksi yang membelanjakan output genesis ke kunci 3, ditandatangani kunci `signer`.
fn spend_genesis(signer: u8, amount: u128) -> Transaction {
    let genesis = regtest_genesis().unwrap();
    let coinbase = genesis.txs().first().unwrap();
    let spent = coinbase.body().outputs().first().unwrap().clone();
    let inputs = vec![Input::new(txid(coinbase), 0)];
    let outputs = vec![out_to(amount, 3)];
    let unsigned = Transaction::regular(
        inputs.clone(),
        outputs.clone(),
        vec![Witness::single_key([0; 64])],
    )
    .unwrap();
    let msg = sighash_v1(
        &params().network_id(),
        &unsigned,
        0,
        std::slice::from_ref(&spent),
    )
    .unwrap();
    let sig = sign(&secret(signer), msg.as_bytes(), &[0; 32]).unwrap();
    Transaction::regular(inputs, outputs, vec![Witness::single_key(*sig.as_bytes())]).unwrap()
}

#[test]
fn starts_from_regtest_genesis() {
    let fx = open(MemoryStore::default());
    let genesis_id = Hex32::from(params().network_id());
    let info = fx.node.chain_info().unwrap();
    assert_eq!(info.network.network(), Network::Regtest);
    assert_eq!(info.height, 0);
    assert_eq!(info.best_block_id, genesis_id);
    assert_eq!(info.chainwork, work(2));
    assert_eq!(info.next_target.get(), LIMIT);
    assert_eq!(info.median_time_past, T0);
    assert_eq!(info.mined_total.amount(), Amount::ZERO);
    assert_eq!(info.total_supply.amount(), GENESIS_ALLOCATION);
    assert_eq!(info.mempool_count, 0);

    assert_eq!(fx.node.block_hash(0), Ok(genesis_id));
    assert_eq!(fx.node.block_hash(1), Err(RpcError::NotFound));
    let block = fx.node.block(genesis_id).unwrap();
    assert_eq!(block.height, 0);
    assert_eq!(block.size, 186);
    assert_eq!(block.txids.len(), 1);
    assert_eq!(
        fx.node.block(Hex32::new([0xee; 32])),
        Err(RpcError::NotFound)
    );
    assert!(fx.node.mempool().unwrap().txids.is_empty());
    assert_eq!(fx.node.params().network(), Network::Regtest);
}

#[test]
fn mines_blocks_from_template() {
    let fx = open(MemoryStore::default());
    let genesis_id = Hex32::from(params().network_id());
    fx.now.store(T0 + 120, Ordering::SeqCst);
    let template = fx.node.block_template(&payout(2)).unwrap();
    assert_eq!(template.height, 1);
    assert_eq!(template.prev_block_id, genesis_id);
    assert_eq!(template.target.get(), LIMIT);
    assert_eq!(template.min_timestamp, T0 + 1);
    assert_eq!(template.subsidy.amount(), Amount::from_quanta(SUBSIDY_1));
    assert_eq!(template.fees.amount(), Amount::ZERO);
    let block = Block::from_bytes(template.hex.as_slice()).unwrap();
    assert_eq!(block.header().timestamp(), T0 + 120);
    assert_eq!(block.header().nonce(), 0);
    assert_eq!(block.txs().len(), 1);

    let mut ids = Vec::new();
    for _ in 0..3 {
        let result = mine(&fx, 2);
        assert_eq!(result.status, SubmitStatus::Accepted);
        assert_eq!(result.reason, None);
        ids.push(result.block_id.unwrap());
    }
    let schedule: Vec<_> = SupplySchedule::new().take(4).collect();
    let info = fx.node.chain_info().unwrap();
    assert_eq!(info.height, 3);
    assert_eq!(Some(&info.best_block_id), ids.last());
    assert_eq!(info.chainwork, work(8));
    assert_eq!(info.median_time_past, T0 + 240);
    assert_eq!(info.mined_total.amount(), schedule[3].mined_total_after);
    assert_eq!(
        info.total_supply.amount(),
        GENESIS_ALLOCATION
            .checked_add(schedule[3].mined_total_after)
            .unwrap()
    );
    for (height, id) in (1_u64..).zip(&ids) {
        assert_eq!(fx.node.block_hash(height), Ok(*id));
    }

    let mut mined: Vec<(u64, u64, Amount, bool, u64)> = fx
        .node
        .scan_utxos(&[Hex32::new(key(2))])
        .unwrap()
        .iter()
        .map(|u| {
            (
                u.height,
                u.confirmations,
                u.amount.amount(),
                u.is_coinbase,
                u.spendable_at,
            )
        })
        .collect();
    mined.sort();
    assert_eq!(
        mined,
        vec![
            (1, 3, schedule[1].subsidy, true, 101),
            (2, 2, schedule[2].subsidy, true, 102),
            (3, 1, schedule[3].subsidy, true, 103),
        ]
    );
    let genesis_utxo = fx.node.scan_utxos(&[Hex32::new(key(1))]).unwrap();
    assert_eq!(genesis_utxo.len(), 1);
    assert_eq!(genesis_utxo[0].amount.amount(), GENESIS_ALLOCATION);
    assert_eq!(genesis_utxo[0].height, 0);
    assert_eq!(genesis_utxo[0].confirmations, 4);
    assert_eq!(genesis_utxo[0].spendable_at, 100);
}

#[test]
fn submit_block_statuses_and_rules() {
    let fx = open(MemoryStore::default());
    assert_eq!(fx.node.submit_block(&[0x00, 0x01]).unwrap(), rejected("V1"));

    fx.now.store(T0 + 120, Ordering::SeqCst);
    let good = solve(&template_block(&fx, 2));
    let h = *good.header();
    let txs = good.txs().to_vec();
    let submit = |block: &Block| fx.node.submit_block(&block.to_bytes()).unwrap();

    // V8: jam lokal tertinggal lebih dari 1 jam, blok ditunda.
    fx.now.store(T0 + 120 - 3_601, Ordering::SeqCst);
    let deferred = submit(&good);
    assert_eq!(deferred.status, SubmitStatus::Deferred);
    assert_eq!(deferred.block_id, Some(Hex32::from(block_id(&h))));
    fx.now.store(T0 + 120, Ordering::SeqCst);

    // V2: target di atas pow_limit.
    let above = solve(
        &Block::new(
            header_with(
                &h,
                h.prev_block_id(),
                h.merkle_root(),
                h.timestamp(),
                0x20ff_ffff,
            ),
            txs.clone(),
        )
        .unwrap(),
    );
    assert_eq!(submit(&above), rejected_block(&above, "V2"));

    // V3: hash PoW di atas target.
    let params = params();
    let high = (0_u64..)
        .map(|nonce| {
            let header = Header::new(
                h.height(),
                h.prev_block_id(),
                h.merkle_root(),
                h.timestamp(),
                h.target(),
                nonce,
            );
            Block::new(header, txs.clone()).unwrap()
        })
        .find(|block| {
            matches!(
                check_block_context_free(block, &params),
                Err(BlockError::HighHash)
            )
        })
        .unwrap();
    assert_eq!(submit(&high), rejected_block(&high, "V3"));

    // V4: Merkle root salah.
    let merkle = solve(
        &Block::new(
            header_with(
                &h,
                h.prev_block_id(),
                Hash32::from_bytes([0x44; 32]),
                h.timestamp(),
                LIMIT,
            ),
            txs.clone(),
        )
        .unwrap(),
    );
    assert_eq!(submit(&merkle), rejected_block(&merkle, "V4"));

    // Induk bukan ujung chain: orphan, tidak disimpan.
    let orphan = solve(
        &Block::new(
            header_with(
                &h,
                Hash32::from_bytes([0x77; 32]),
                h.merkle_root(),
                h.timestamp(),
                LIMIT,
            ),
            txs.clone(),
        )
        .unwrap(),
    );
    let result = submit(&orphan);
    assert_eq!(result.status, SubmitStatus::Orphan);
    assert_eq!(result.reason, None);

    // V5: tinggi 2 di atas genesis.
    let coinbase_2 =
        Transaction::coinbase(BlockHeight::new(2), Vec::new(), vec![out_to(SUBSIDY_1, 2)]).unwrap();
    let tall_header = Header::new(
        BlockHeight::new(2),
        h.prev_block_id(),
        Hash32::ZERO,
        h.timestamp(),
        LIMIT,
        0,
    );
    let tall = solve(&with_txs(&tall_header, vec![coinbase_2]));
    assert_eq!(submit(&tall), rejected_block(&tall, "V5"));

    // V6: target sah (0xffff00 × 2^224, sekitar 2^248) tetapi bukan target ASERT; sekitar 256 percobaan nonce.
    let hard = solve(
        &Block::new(
            header_with(
                &h,
                h.prev_block_id(),
                h.merkle_root(),
                h.timestamp(),
                0x1fff_ff00,
            ),
            txs.clone(),
        )
        .unwrap(),
    );
    assert_eq!(submit(&hard), rejected_block(&hard, "V6"));

    // V7: cap waktu tidak lebih besar dari median (cap waktu genesis).
    let early = solve(
        &Block::new(
            header_with(&h, h.prev_block_id(), h.merkle_root(), T0, LIMIT),
            txs.clone(),
        )
        .unwrap(),
    );
    assert_eq!(submit(&early), rejected_block(&early, "V7"));

    // V10: coinbase mengklaim satu quanta lebih banyak.
    let greedy_cb = Transaction::coinbase(
        BlockHeight::new(1),
        Vec::new(),
        vec![out_to(SUBSIDY_1 + 1, 2)],
    )
    .unwrap();
    let greedy = solve(&with_txs(&h, vec![greedy_cb]));
    assert_eq!(submit(&greedy), rejected_block(&greedy, "V10"));

    assert_eq!(fx.node.chain_info().unwrap().height, 0);
    let accepted = submit(&good);
    assert_eq!(accepted.status, SubmitStatus::Accepted);
    assert_eq!(accepted.block_id, deferred.block_id);
    assert_eq!(submit(&good).status, SubmitStatus::Duplicate);
    assert_eq!(fx.node.chain_info().unwrap().height, 1);
}

#[test]
fn transaction_flow_through_mempool_and_block() {
    let fx = open(MemoryStore::default());
    let tx = spend_genesis(1, G - 1_000);
    let id = Hex32::from(txid(&tx));
    let send = |tx: &Transaction| fx.node.send_raw_transaction(&tx.to_bytes());
    let reason = |rule: &str| -> Result<SendRawTransactionResult, RpcError> {
        Err(RpcError::InvalidTransaction {
            reason: rule.to_owned(),
        })
    };

    assert_eq!(
        fx.node.send_raw_transaction(&[0x01]),
        Err(RpcError::InvalidEncoding)
    );
    // Genesis baru boleh dibelanjakan di blok 100.
    assert_eq!(send(&tx), reason("T2"));
    for _ in 0..99 {
        assert_eq!(mine(&fx, 2).status, SubmitStatus::Accepted);
    }
    assert_eq!(send(&spend_genesis(2, G - 1_000)), reason("T6"));
    let coinbase =
        Transaction::coinbase(BlockHeight::new(100), Vec::new(), vec![out_to(5, 3)]).unwrap();
    assert_eq!(send(&coinbase), reason("struktur"));

    assert_eq!(send(&tx).map(|r| r.txid), Ok(id));
    assert_eq!(send(&tx).map(|r| r.txid), Ok(id));
    assert_eq!(send(&spend_genesis(1, G - 2_000)), reason("T1"));
    assert_eq!(fx.node.mempool().unwrap().txids, vec![id]);
    assert_eq!(fx.node.chain_info().unwrap().mempool_count, 1);

    fx.now.store(T0 + 120 * 100, Ordering::SeqCst);
    let template = fx.node.block_template(&payout(2)).unwrap();
    assert_eq!(template.height, 100);
    assert_eq!(template.fees.amount(), Amount::from_quanta(1_000));
    let block = Block::from_bytes(template.hex.as_slice()).unwrap();
    assert_eq!(block.txs().len(), 2);
    let result = fx.node.submit_block(&solve(&block).to_bytes()).unwrap();
    assert_eq!(result.status, SubmitStatus::Accepted);

    let info = fx.node.chain_info().unwrap();
    assert_eq!(info.height, 100);
    assert_eq!(info.mempool_count, 0);
    let schedule: Vec<_> = SupplySchedule::new().take(101).collect();
    assert_eq!(info.mined_total.amount(), schedule[100].mined_total_after);
    assert!(fx.node.mempool().unwrap().txids.is_empty());
    assert!(
        fx.node
            .scan_utxos(&[Hex32::new(key(1))])
            .unwrap()
            .is_empty()
    );
    let received = fx.node.scan_utxos(&[Hex32::new(key(3))]).unwrap();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].txid, id);
    assert_eq!(received[0].amount.amount(), Amount::from_quanta(G - 1_000));
    assert!(!received[0].is_coinbase);
    assert_eq!(received[0].height, 100);
    assert_eq!(received[0].confirmations, 1);
    assert_eq!(received[0].spendable_at, 101);
}

#[test]
fn reopen_rebuilds_state_and_rejects_wrong_genesis() {
    let fx = open(MemoryStore::default());
    for _ in 0..3 {
        assert_eq!(mine(&fx, 2).status, SubmitStatus::Accepted);
    }
    let before = fx.node.chain_info().unwrap();
    let store = fx.node.into_store().unwrap();
    let again = open(store);
    assert_eq!(again.node.chain_info().unwrap(), before);

    let mut other = MemoryStore::default();
    let coinbase =
        Transaction::coinbase(BlockHeight::GENESIS, vec![0; 32], vec![out_to(5, 9)]).unwrap();
    let fake = Block::new(
        Header::new(
            BlockHeight::GENESIS,
            Hash32::ZERO,
            Hash32::ZERO,
            T0,
            LIMIT,
            0,
        ),
        vec![coinbase.clone()],
    )
    .unwrap();
    let fake_id = Hash32::from_bytes([0xee; 32]);
    other
        .apply_block(&fake, fake_id, &[txid(&coinbase)])
        .unwrap();
    let now = Arc::new(AtomicU64::new(T0));
    let result = Node::open(other, params(), DEFAULT_MAX_BYTES, clock(&now));
    assert!(matches!(
        result.err(),
        Some(NodeError::WrongGenesis { found: Some(found), .. }) if found == fake_id
    ));
}

#[test]
fn memory_and_redb_nodes_agree() {
    let mem = open(MemoryStore::default());
    let rdb = open(RedbStore::in_memory().unwrap());
    for _ in 0..3 {
        let a = mine(&mem, 2);
        let b = mine(&rdb, 2);
        assert_eq!(a.status, SubmitStatus::Accepted);
        assert_eq!(a, b);
    }
    let info = mem.node.chain_info().unwrap();
    assert_eq!(info, rdb.node.chain_info().unwrap());
    let payloads = [Hex32::new(key(1)), Hex32::new(key(2))];
    assert_eq!(
        mem.node.scan_utxos(&payloads).unwrap(),
        rdb.node.scan_utxos(&payloads).unwrap()
    );
    assert_eq!(
        mem.node.block(info.best_block_id).unwrap(),
        rdb.node.block(info.best_block_id).unwrap()
    );
}

#[test]
fn rule_names_for_blocks_and_mempool() {
    let outpoint = OutPoint::new(Hash32::ZERO, 0);
    assert_eq!(block_rule(&BlockError::HighHash, "V2"), "V3");
    let difficulty = BlockError::Difficulty(DifficultyError::ZeroTarget);
    assert_eq!(block_rule(&difficulty, "V2"), "V2");
    assert_eq!(block_rule(&difficulty, "V6"), "V6");
    let immature = BlockError::Transaction {
        index: 1,
        error: TxError::ImmatureCoinbase { outpoint },
    };
    assert_eq!(block_rule(&immature, "V9"), "T2");
    let structural = BlockError::Transaction {
        index: 1,
        error: TxError::NotRegular,
    };
    assert_eq!(block_rule(&structural, "V9"), "V9");
    assert_eq!(block_rule(&BlockError::Genesis, "V9"), "V9");
    let mismatch = BlockError::CoinbaseAmountMismatch {
        expected: Amount::ZERO,
        actual: Amount::from_quanta(1),
    };
    assert_eq!(block_rule(&mismatch, "V9"), "V10");

    let full = MempoolError::Full {
        size: 1,
        used: 0,
        limit: 0,
    };
    assert_eq!(mempool_reason(&full), "mempool_penuh");
    assert_eq!(
        mempool_reason(&MempoolError::TooLarge { size: 1 }),
        "terlalu_besar"
    );
    assert_eq!(
        mempool_reason(&MempoolError::Duplicate { txid: Hash32::ZERO }),
        "duplikat"
    );
    let conflict = MempoolError::Conflict {
        outpoint,
        existing: Hash32::ZERO,
    };
    assert_eq!(mempool_reason(&conflict), "T1");
    assert_eq!(
        mempool_reason(&MempoolError::Invalid(TxError::OutputsExceedInputs)),
        "T5"
    );
}
