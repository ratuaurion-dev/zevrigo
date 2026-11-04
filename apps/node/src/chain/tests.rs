use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use homeostasis_consensus::{
    BlockError, ChainParams, DifficultyError, SupplySchedule, TxError, check_block_context_free,
    check_header_context_free, regtest_genesis,
};
use homeostasis_crypto::{SecretKey, block_id, block_merkle_root, sighash_v1, sign, txid};
use homeostasis_mempool::{DEFAULT_MAX_BYTES, MempoolError};
use homeostasis_primitives::params::GENESIS_ALLOCATION;
use homeostasis_primitives::{
    Amount, Block, BlockHeight, Decode, Encode, Hash32, Header, Input, Network, OutPoint, Output,
    OutputKind, Transaction, Witness,
};
use homeostasis_rpc::{
    GetBlockTemplateParams, Hex32, KindNumber, MempoolOutput, MempoolScan, MempoolSpend, Quanta,
    RpcBackend, RpcError, SendRawTransactionResult, SubmitBlockResult, SubmitStatus,
};
use homeostasis_storage::{ChainStore, MemoryStore, RedbStore};

use super::{
    Clock, HeadersOutcome, MAX_LOCATOR_IDS, Node, NodeError, NodeLimits, PRESYNC_COMMIT_INTERVAL,
    Presync, PresyncOutcome, block_rule, locator_heights, mempool_reason,
};

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
    assert_eq!(
        template.full_target.bytes(),
        &params().pow_limit().to_be_bytes()
    );
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

    // Induk tidak dikenal: orphan, tidak disimpan.
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

/// Jam jauh di depan untuk uji cabang: semua blok bercap waktu `T0 + 120 × h` lolos V8.
const FAR: u64 = T0 + 120 * 1_000;

/// Blok setinggi `height` di atas `prev`: coinbase tepat jadwal pasokan ditambah `fee` ke `owner`, transaksi `txs`,
/// cap waktu `T0 + 120 × height` (sesuai jadwal, sehingga target tetap `pow_limit`), dan PoW yang lolos. Pemilik yang
/// berbeda membuat cabang berbeda di tinggi yang sama.
fn block_on(prev: Hash32, height: u64, owner: u8, txs: Vec<Transaction>, fee: u128) -> Block {
    let index = usize::try_from(height).unwrap();
    let schedule: Vec<_> = SupplySchedule::new().take(index + 1).collect();
    let reward = schedule[index].subsidy.as_quanta() + fee;
    let coinbase = Transaction::coinbase(
        BlockHeight::new(height),
        Vec::new(),
        vec![out_to(reward, owner)],
    )
    .unwrap();
    let mut all = vec![coinbase];
    all.extend(txs);
    let header = Header::new(
        BlockHeight::new(height),
        prev,
        Hash32::ZERO,
        T0 + 120 * height,
        LIMIT,
        0,
    );
    solve(&with_txs(&header, all))
}

fn id_of(block: &Block) -> Hash32 {
    block_id(block.header())
}

fn submit<S: ChainStore + Send + 'static>(fx: &Fixture<S>, block: &Block) -> SubmitBlockResult {
    fx.node.submit_block(&block.to_bytes()).unwrap()
}

/// Rantai `count` blok di atas `prev` mulai tinggi `from`, semuanya ke `owner`; setiap blok wajib `accepted`.
fn extend<S: ChainStore + Send + 'static>(
    fx: &Fixture<S>,
    prev: Hash32,
    from: u64,
    count: u64,
    owner: u8,
) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut parent = prev;
    for height in from..from + count {
        let block = block_on(parent, height, owner, Vec::new(), 0);
        assert_eq!(
            submit(fx, &block).status,
            SubmitStatus::Accepted,
            "tinggi {height}"
        );
        parent = id_of(&block);
        blocks.push(block);
    }
    blocks
}

fn side_count<S: ChainStore>(fx: &Fixture<S>) -> usize {
    fx.node.lock().unwrap().side.len()
}

/// Apakah badan blok `block` tersimpan sebagai blok cabang di penyimpanan node.
fn side_body_stored<S: ChainStore>(fx: &Fixture<S>, block: &Block) -> bool {
    fx.node
        .lock()
        .unwrap()
        .store
        .side_block(&id_of(block))
        .unwrap()
        .is_some()
}

fn assert_active<S: ChainStore + Send + 'static>(fx: &Fixture<S>, blocks: &[Block]) {
    let info = fx.node.chain_info().unwrap();
    let height = u64::try_from(blocks.len()).unwrap();
    assert_eq!(info.height, height);
    assert_eq!(
        info.best_block_id,
        Hex32::from(id_of(blocks.last().unwrap()))
    );
    assert_eq!(
        info.chainwork,
        work(u8::try_from(2 * (blocks.len() + 1)).unwrap())
    );
    let schedule: Vec<_> = SupplySchedule::new().take(blocks.len() + 1).collect();
    assert_eq!(
        info.mined_total.amount(),
        schedule[blocks.len()].mined_total_after
    );
    for (h, block) in (1_u64..).zip(blocks) {
        assert_eq!(
            fx.node.block_hash(h),
            Ok(Hex32::from(id_of(block))),
            "tinggi {h}"
        );
    }
}

/// Chain A tiga blok ke kunci 2, lalu cabang B empat blok ke kunci 4 dari genesis. Mengembalikan (A, B).
fn reorg_scenario<S: ChainStore + Send + 'static>(fx: &Fixture<S>) -> (Vec<Block>, Vec<Block>) {
    fx.now.store(FAR, Ordering::SeqCst);
    let genesis = params().network_id();
    let a = extend(fx, genesis, 1, 3, 2);
    // B1 sampai B3: cabang, chainwork paling banyak sama dengan A, sehingga A tetap aktif.
    let mut b = extend(fx, genesis, 1, 3, 4);
    assert_active(fx, &a);
    assert_eq!(side_count(fx), 3);
    // B4: chainwork B lebih besar, reorg sedalam 3 blok.
    let b4 = block_on(id_of(&b[2]), 4, 4, Vec::new(), 0);
    assert_eq!(submit(fx, &b4).status, SubmitStatus::Accepted);
    b.push(b4);
    assert_active(fx, &b);
    (a, b)
}

#[test]
fn heavier_branch_reorgs_and_back() {
    let fx = open(MemoryStore::default());
    let (mut a, b) = reorg_scenario(&fx);
    assert_eq!(side_count(&fx), 3);
    assert!(
        fx.node
            .scan_utxos(&[Hex32::new(key(2))])
            .unwrap()
            .is_empty()
    );
    assert_eq!(fx.node.scan_utxos(&[Hex32::new(key(4))]).unwrap().len(), 4);
    // Blok cabang dikenal (duplicate), tetapi bukan isi chain aktif untuk get_block.
    assert_eq!(submit(&fx, &a[0]).status, SubmitStatus::Duplicate);
    assert_eq!(submit(&fx, &b[0]).status, SubmitStatus::Duplicate);
    assert_eq!(
        fx.node.block(Hex32::from(id_of(&a[0]))),
        Err(RpcError::NotFound)
    );

    // A4 menyamai chainwork B: B tetap aktif (yang lebih dulu diterima). A5 membuat A lebih berat: reorg kembali.
    let a4 = block_on(id_of(&a[2]), 4, 2, Vec::new(), 0);
    assert_eq!(submit(&fx, &a4).status, SubmitStatus::Accepted);
    assert_active(&fx, &b);
    let a5 = block_on(id_of(&a4), 5, 2, Vec::new(), 0);
    assert_eq!(submit(&fx, &a5).status, SubmitStatus::Accepted);
    a.push(a4);
    a.push(a5);
    assert_active(&fx, &a);
    assert_eq!(side_count(&fx), 4);
    assert!(
        fx.node
            .scan_utxos(&[Hex32::new(key(4))])
            .unwrap()
            .is_empty()
    );
    assert_eq!(fx.node.scan_utxos(&[Hex32::new(key(2))]).unwrap().len(), 5);
    assert_eq!(fx.node.scan_utxos(&[Hex32::new(key(1))]).unwrap().len(), 1);
}

#[test]
fn reorg_returns_transactions_to_mempool() {
    let fx = open(MemoryStore::default());
    fx.now.store(FAR, Ordering::SeqCst);
    let genesis = params().network_id();
    let main = extend(&fx, genesis, 1, 99, 2);
    let a99 = id_of(main.last().unwrap());
    let tx = spend_genesis(1, G - 1_000);
    let a100 = block_on(a99, 100, 2, vec![tx.clone()], 1_000);
    assert_eq!(submit(&fx, &a100).status, SubmitStatus::Accepted);
    assert!(fx.node.mempool().unwrap().txids.is_empty());
    assert_eq!(fx.node.scan_utxos(&[Hex32::new(key(3))]).unwrap().len(), 1);

    // Cabang B dari blok 99 tanpa transaksi itu: B100 menyamai, B101 melebihi chainwork A.
    let b100 = block_on(a99, 100, 4, Vec::new(), 0);
    assert_eq!(submit(&fx, &b100).status, SubmitStatus::Accepted);
    assert_eq!(
        fx.node.chain_info().unwrap().best_block_id,
        Hex32::from(id_of(&a100))
    );
    let b101 = block_on(id_of(&b100), 101, 4, Vec::new(), 0);
    assert_eq!(submit(&fx, &b101).status, SubmitStatus::Accepted);
    let info = fx.node.chain_info().unwrap();
    assert_eq!(info.height, 101);
    assert_eq!(info.best_block_id, Hex32::from(id_of(&b101)));

    // Transaksi dari blok yang dilepas kembali ke mempool, dan alokasi genesis belum dibelanjakan di chain B.
    assert_eq!(
        fx.node.mempool().unwrap().txids,
        vec![Hex32::from(txid(&tx))]
    );
    assert!(
        fx.node
            .scan_utxos(&[Hex32::new(key(3))])
            .unwrap()
            .is_empty()
    );
    assert_eq!(fx.node.scan_utxos(&[Hex32::new(key(1))]).unwrap().len(), 1);
    let template = fx.node.block_template(&payout(4)).unwrap();
    assert_eq!(template.height, 102);
    assert_eq!(template.fees.amount(), Amount::from_quanta(1_000));
    let block = Block::from_bytes(template.hex.as_slice()).unwrap();
    assert_eq!(block.txs().len(), 2);
    assert_eq!(submit(&fx, &solve(&block)).status, SubmitStatus::Accepted);
    assert!(fx.node.mempool().unwrap().txids.is_empty());
    assert_eq!(fx.node.scan_utxos(&[Hex32::new(key(3))]).unwrap().len(), 1);
}

#[test]
fn invalid_branch_is_marked_and_active_chain_restored() {
    let fx = open(MemoryStore::default());
    fx.now.store(FAR, Ordering::SeqCst);
    let genesis = params().network_id();
    let a = extend(&fx, genesis, 1, 2, 2);
    let b1 = block_on(genesis, 1, 4, Vec::new(), 0);
    assert_eq!(submit(&fx, &b1).status, SubmitStatus::Accepted);
    // B2 mengklaim satu quanta terlalu banyak (V10). Headernya sah, jadi diterima sebagai blok cabang dulu.
    let b2 = block_on(id_of(&b1), 2, 4, Vec::new(), 1);
    assert_eq!(submit(&fx, &b2).status, SubmitStatus::Accepted);
    // B3 membuat cabang B lebih berat; reorg gagal di B2, chain A dipulihkan.
    let b3 = block_on(id_of(&b2), 3, 4, Vec::new(), 0);
    assert_eq!(submit(&fx, &b3), rejected_block(&b3, "V5"));
    assert_eq!(super::INVALID_ANCESTOR_RULE, "V5");
    assert_active(&fx, &a);
    assert_eq!(fx.node.scan_utxos(&[Hex32::new(key(2))]).unwrap().len(), 2);
    assert!(
        fx.node
            .scan_utxos(&[Hex32::new(key(4))])
            .unwrap()
            .is_empty()
    );
    assert_eq!(submit(&fx, &b2), rejected_block(&b2, "V10"));
    assert_eq!(submit(&fx, &b3), rejected_block(&b3, "V5"));
    let b4 = block_on(id_of(&b3), 4, 4, Vec::new(), 0);
    assert_eq!(submit(&fx, &b4), rejected_block(&b4, "V5"));
    // B1 sendiri sah dan tetap blok cabang.
    assert_eq!(submit(&fx, &b1).status, SubmitStatus::Duplicate);
    assert_eq!(side_count(&fx), 1);

    // Cabang sah lain di atas B1 menang setelah lebih berat.
    let c2 = block_on(id_of(&b1), 2, 5, Vec::new(), 0);
    assert_eq!(submit(&fx, &c2).status, SubmitStatus::Accepted);
    assert_active(&fx, &a);
    let c3 = block_on(id_of(&c2), 3, 5, Vec::new(), 0);
    assert_eq!(submit(&fx, &c3).status, SubmitStatus::Accepted);
    assert_active(&fx, &[b1, c2, c3]);
    assert_eq!(side_count(&fx), 2);
}

#[test]
fn memory_and_redb_agree_after_reorg_and_reopen() {
    let mem = open(MemoryStore::default());
    let rdb = open(RedbStore::in_memory().unwrap());
    let (_, b) = reorg_scenario(&mem);
    let (_, b_rdb) = reorg_scenario(&rdb);
    assert_eq!(b, b_rdb);
    let info = mem.node.chain_info().unwrap();
    assert_eq!(info, rdb.node.chain_info().unwrap());
    let payloads = [Hex32::new(key(1)), Hex32::new(key(2)), Hex32::new(key(4))];
    assert_eq!(
        mem.node.scan_utxos(&payloads).unwrap(),
        rdb.node.scan_utxos(&payloads).unwrap()
    );

    // Dibuka ulang: chain aktif (cabang B) utuh; indeks blok cabang di memori, badan blok cabang lama dibuang (TSK-M35).
    let reopened = open(rdb.node.into_store().unwrap());
    assert_eq!(reopened.node.chain_info().unwrap(), info);
    assert_active(&reopened, &b);
    assert_eq!(side_count(&reopened), 0);
}

#[test]
fn locator_heights_follow_spacing() {
    assert_eq!(locator_heights(0), vec![0]);
    assert_eq!(locator_heights(5), vec![5, 4, 3, 2, 1, 0]);
    assert_eq!(locator_heights(9), (0..=9).rev().collect::<Vec<u64>>());
    assert_eq!(
        locator_heights(30),
        vec![30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 19, 15, 7, 0]
    );
    for tip in [1_u64 << 53, u64::MAX] {
        let heights = locator_heights(tip);
        assert!(heights.len() <= MAX_LOCATOR_IDS, "{tip}");
        assert_eq!(heights.first(), Some(&tip));
        assert_eq!(heights.last(), Some(&0));
        assert!(heights.windows(2).all(|pair| pair[0] > pair[1]));
    }
}

#[test]
fn headers_then_blocks_sync_two_nodes() {
    let a = open(MemoryStore::default());
    let b = open(MemoryStore::default());
    a.now.store(FAR, Ordering::SeqCst);
    b.now.store(FAR, Ordering::SeqCst);
    let genesis = params().network_id();
    let blocks = extend(&a, genesis, 1, 20, 2);
    let ids: Vec<Hash32> = blocks.iter().map(id_of).collect();

    // Node baru: locator hanya genesis. Node A menjawab header sesudahnya, dengan batas `max` dan `stop`.
    let locator = b.node.locator().unwrap();
    assert_eq!(locator, vec![genesis]);
    let headers = a
        .node
        .headers_after(&locator, &Hash32::ZERO, 2_000)
        .unwrap();
    assert_eq!(headers.iter().map(block_id).collect::<Vec<_>>(), ids);
    assert_eq!(
        a.node
            .headers_after(&locator, &ids[4], 2_000)
            .unwrap()
            .len(),
        5
    );
    assert_eq!(
        a.node
            .headers_after(&locator, &Hash32::ZERO, 3)
            .unwrap()
            .len(),
        3
    );
    assert!(
        a.node
            .headers_after(&[ids[19], genesis], &Hash32::ZERO, 2_000)
            .unwrap()
            .is_empty()
    );
    assert!(
        a.node
            .headers_after(&[Hash32::from_bytes([0x55; 32])], &Hash32::ZERO, 2_000)
            .unwrap()
            .is_empty()
    );
    let a_locator = a.node.locator().unwrap();
    assert_eq!(a_locator.len(), locator_heights(20).len());
    assert_eq!(a_locator.first(), Some(&ids[19]));
    assert_eq!(a_locator.get(10), Some(&ids[8]));
    assert_eq!(a_locator.last(), Some(&genesis));

    assert_eq!(
        b.node.accept_headers(&headers).unwrap(),
        HeadersOutcome::Accepted {
            added: 20,
            last: Some(ids[19])
        }
    );
    // Header yang sudah dikenal tidak ditambah lagi; badan blok belum ada, chain aktif tetap genesis.
    assert_eq!(
        b.node.accept_headers(&headers[..5]).unwrap(),
        HeadersOutcome::Accepted {
            added: 0,
            last: Some(ids[4])
        }
    );
    assert_eq!(
        b.node.accept_headers(&[]).unwrap(),
        HeadersOutcome::Accepted {
            added: 0,
            last: None
        }
    );
    assert_eq!(b.node.chain_info().unwrap().height, 0);

    let first = b.node.missing_blocks(16).unwrap();
    assert_eq!(first, ids[..16].to_vec());
    let bodies = a.node.blocks_by_id(&first).unwrap();
    assert_eq!(bodies.len(), 16);
    for (body, expected) in bodies.iter().zip(&blocks) {
        assert_eq!(body.as_ref(), Some(expected));
        let receipt = b.node.receive_block(&expected.to_bytes()).unwrap();
        assert_eq!(receipt.result.status, SubmitStatus::Accepted);
        assert!(receipt.new_tip);
    }
    let rest = b.node.missing_blocks(16).unwrap();
    assert_eq!(rest, ids[16..].to_vec());
    for body in a.node.blocks_by_id(&rest).unwrap().into_iter().flatten() {
        assert!(b.node.receive_block(&body.to_bytes()).unwrap().new_tip);
    }
    assert!(b.node.missing_blocks(16).unwrap().is_empty());
    assert_active(&b, &blocks);
    assert_eq!(b.node.tip_summary().unwrap(), a.node.tip_summary().unwrap());

    let again = b.node.receive_block(&blocks[3].to_bytes()).unwrap();
    assert_eq!(again.result.status, SubmitStatus::Duplicate);
    assert!(!again.new_tip);
    assert_eq!(
        a.node
            .blocks_by_id(&[Hash32::from_bytes([0x55; 32]), ids[0]])
            .unwrap(),
        vec![None, Some(blocks[0].clone())]
    );
}

#[test]
fn bad_headers_are_rejected_or_deferred() {
    let a = open(MemoryStore::default());
    let b = open(MemoryStore::default());
    a.now.store(FAR, Ordering::SeqCst);
    b.now.store(FAR, Ordering::SeqCst);
    let genesis = params().network_id();
    let blocks = extend(&a, genesis, 1, 3, 2);
    let headers: Vec<Header> = blocks.iter().map(|block| *block.header()).collect();

    // Induk tidak dikenal, dan header yang tidak menyambung ke header sebelumnya (h1 tetap disimpan).
    assert_eq!(
        b.node.accept_headers(&headers[1..]).unwrap(),
        HeadersOutcome::Disconnected
    );
    assert_eq!(
        b.node.accept_headers(&[headers[0], headers[2]]).unwrap(),
        HeadersOutcome::Disconnected
    );

    // V3: hash di atas target.
    let params = params();
    let h = headers[0];
    let high = (0_u64..)
        .map(|nonce| {
            Header::new(
                h.height(),
                h.prev_block_id(),
                h.merkle_root(),
                h.timestamp(),
                h.target(),
                nonce,
            )
        })
        .find(|header| {
            matches!(
                check_header_context_free(header, &params),
                Err(BlockError::HighHash)
            )
        })
        .unwrap();
    assert_eq!(
        b.node.accept_headers(&[high]).unwrap(),
        HeadersOutcome::Invalid { rule: "V3" }
    );
    // V5: tinggi 2 langsung di atas genesis.
    let tall = *block_on(genesis, 2, 4, Vec::new(), 0).header();
    assert_eq!(
        b.node.accept_headers(&[tall]).unwrap(),
        HeadersOutcome::Invalid { rule: "V5" }
    );

    // V8: jam lokal tertinggal lebih dari 1 jam dari h2. Pemrosesan berhenti tanpa menyimpan h2.
    b.now.store(T0 + 240 - 3_601, Ordering::SeqCst);
    assert_eq!(
        b.node.accept_headers(&headers).unwrap(),
        HeadersOutcome::Deferred { added: 0 }
    );
    b.now.store(FAR, Ordering::SeqCst);
    assert_eq!(
        b.node.accept_headers(&headers).unwrap(),
        HeadersOutcome::Accepted {
            added: 2,
            last: Some(id_of(&blocks[2]))
        }
    );
    assert_eq!(
        b.node.missing_blocks(16).unwrap(),
        blocks.iter().map(id_of).collect::<Vec<_>>()
    );
    for block in &blocks {
        assert!(b.node.receive_block(&block.to_bytes()).unwrap().new_tip);
    }
    assert!(b.node.missing_blocks(16).unwrap().is_empty());
    assert_active(&b, &blocks);
}

/// Node `MemoryStore` dengan batas memori `limits`, jam di [`FAR`].
fn open_limited(limits: NodeLimits) -> Fixture<MemoryStore> {
    let now = Arc::new(AtomicU64::new(FAR));
    let node = Node::open_with_limits(
        MemoryStore::default(),
        params(),
        DEFAULT_MAX_BYTES,
        clock(&now),
        limits,
    )
    .unwrap();
    Fixture { node, now }
}

fn set(ids: impl IntoIterator<Item = Hash32>) -> BTreeSet<Hash32> {
    ids.into_iter().collect()
}

fn side_set(fx: &Fixture<MemoryStore>) -> BTreeSet<Hash32> {
    set(fx.node.lock().unwrap().side.keys().copied())
}

fn header_set(fx: &Fixture<MemoryStore>) -> BTreeSet<Hash32> {
    set(fx.node.lock().unwrap().headers.keys().copied())
}

fn invalid_set(fx: &Fixture<MemoryStore>) -> BTreeSet<Hash32> {
    set(fx.node.lock().unwrap().invalid.keys().copied())
}

#[test]
fn default_limits() {
    assert_eq!(NodeLimits::default(), NodeLimits::DEFAULT);
    assert_eq!(
        NodeLimits::DEFAULT,
        NodeLimits {
            max_side_blocks: 10_000,
            max_side_bytes: 67_108_864,
            max_headers: 100_000,
            max_invalid: 10_000,
        }
    );
    let fx = open(MemoryStore::default());
    assert_eq!(fx.node.lock().unwrap().limits, NodeLimits::DEFAULT);
}

#[test]
fn side_blocks_are_trimmed_leaves_with_least_work_first() {
    let fx = open_limited(NodeLimits {
        max_side_blocks: 3,
        ..NodeLimits::DEFAULT
    });
    let genesis = params().network_id();
    let a = extend(&fx, genesis, 1, 3, 2);
    // Lima cabang satu blok dari genesis (chainwork sama): yang lebih dulu diterima tetap, yang lebih akhir dibuang.
    let forks: Vec<Block> = (5_u8..10)
        .map(|owner| block_on(genesis, 1, owner, Vec::new(), 0))
        .collect();
    for fork in &forks {
        assert_eq!(submit(&fx, fork).status, SubmitStatus::Accepted);
        assert!(side_count(&fx) <= 3);
    }
    let first_three = set(forks[..3].iter().map(id_of));
    assert_eq!(side_set(&fx), first_three);
    // Blok yang dibuang dilupakan: dikirim lagi, diterima lagi lalu dibuang lagi; blok yang tersimpan tetap duplicate.
    assert_eq!(submit(&fx, &forks[3]).status, SubmitStatus::Accepted);
    assert_eq!(submit(&fx, &forks[0]).status, SubmitStatus::Duplicate);
    assert_eq!(side_set(&fx), first_three);

    // Cabang di atas fork pertama: fork pertama bukan daun, jadi daun paling ringan dan paling akhir yang dibuang.
    let f2 = block_on(id_of(&forks[0]), 2, 5, Vec::new(), 0);
    assert_eq!(submit(&fx, &f2).status, SubmitStatus::Accepted);
    assert_eq!(
        side_set(&fx),
        set([id_of(&forks[0]), id_of(&forks[1]), id_of(&f2)])
    );
    let f3 = block_on(id_of(&f2), 3, 5, Vec::new(), 0);
    assert_eq!(submit(&fx, &f3).status, SubmitStatus::Accepted);
    assert_eq!(
        side_set(&fx),
        set([id_of(&forks[0]), id_of(&f2), id_of(&f3)])
    );
    assert_active(&fx, &a);

    // Cabang menjadi lebih berat: reorg tetap berjalan, chain lama menjadi cabang dalam batas.
    let f4 = block_on(id_of(&f3), 4, 5, Vec::new(), 0);
    assert_eq!(submit(&fx, &f4).status, SubmitStatus::Accepted);
    assert_active(&fx, &[forks[0].clone(), f2, f3, f4]);
    assert_eq!(side_set(&fx), set(a.iter().map(id_of)));
}

#[test]
fn side_blocks_are_trimmed_by_size() {
    let genesis = params().network_id();
    let forks: Vec<Block> = (5_u8..8)
        .map(|owner| block_on(genesis, 1, owner, Vec::new(), 0))
        .collect();
    let size = forks[0].to_bytes().len();
    assert!(forks.iter().all(|fork| fork.to_bytes().len() == size));
    let fx = open_limited(NodeLimits {
        max_side_bytes: 2 * size + size / 2,
        ..NodeLimits::DEFAULT
    });
    extend(&fx, genesis, 1, 2, 2);
    for fork in &forks {
        assert_eq!(submit(&fx, fork).status, SubmitStatus::Accepted);
    }
    assert_eq!(side_set(&fx), set([id_of(&forks[0]), id_of(&forks[1])]));
    let bytes: usize = fx
        .node
        .lock()
        .unwrap()
        .side
        .values()
        .map(|entry| entry.size)
        .sum();
    assert_eq!(bytes, 2 * size);
}

#[test]
fn heavier_branch_being_synced_is_never_trimmed() {
    let a = open(MemoryStore::default());
    a.now.store(FAR, Ordering::SeqCst);
    let genesis = params().network_id();
    let long = extend(&a, genesis, 1, 5, 4);
    let b = open_limited(NodeLimits {
        max_side_blocks: 2,
        ..NodeLimits::DEFAULT
    });
    let short = extend(&b, genesis, 1, 3, 2);
    let headers: Vec<Header> = long.iter().map(|block| *block.header()).collect();
    assert_eq!(
        b.node.accept_headers(&headers).unwrap(),
        HeadersOutcome::Accepted {
            added: 5,
            last: Some(id_of(&long[4]))
        }
    );
    // Cabang yang lebih berat diunduh blok demi blok: tiga blok cabang melebihi batas 2 tetapi tidak dibuang; cabang
    // sampah yang datang kemudian yang dibuang.
    for block in &long[..3] {
        let receipt = b.node.receive_block(&block.to_bytes()).unwrap();
        assert_eq!(receipt.result.status, SubmitStatus::Accepted);
        assert!(!receipt.new_tip);
    }
    let junk = block_on(genesis, 1, 9, Vec::new(), 0);
    assert_eq!(submit(&b, &junk).status, SubmitStatus::Accepted);
    assert_eq!(side_set(&b), set(long[..3].iter().map(id_of)));

    // Blok keempat membuat cabang lebih berat: reorg; chain lama dipangkas ke batas dari daunnya.
    for block in &long[3..] {
        assert!(b.node.receive_block(&block.to_bytes()).unwrap().new_tip);
    }
    assert_active(&b, &long);
    assert_eq!(side_set(&b), set(short[..2].iter().map(id_of)));
    assert!(b.node.missing_blocks(16).unwrap().is_empty());
}

#[test]
fn header_index_is_bounded_and_keeps_heavier_headers() {
    let a = open(MemoryStore::default());
    a.now.store(FAR, Ordering::SeqCst);
    let genesis = params().network_id();
    let blocks = extend(&a, genesis, 1, 5, 2);
    let headers: Vec<Header> = blocks.iter().map(|block| *block.header()).collect();
    let ids: Vec<Hash32> = blocks.iter().map(id_of).collect();
    let junk = *block_on(genesis, 1, 9, Vec::new(), 0).header();

    // Indeks satu header: header sampah sama berat dengan header pertama chain, jadi tidak dibuang.
    let one = open_limited(NodeLimits {
        max_headers: 1,
        ..NodeLimits::DEFAULT
    });
    assert_eq!(
        one.node.accept_headers(&[junk]).unwrap(),
        HeadersOutcome::Accepted {
            added: 1,
            last: Some(block_id(&junk))
        }
    );
    assert_eq!(
        one.node.accept_headers(&headers).unwrap(),
        HeadersOutcome::Full { added: 0 }
    );
    assert_eq!(header_set(&one), set([block_id(&junk)]));

    // Indeks tiga header: header sampah (chainwork 4) dibuang untuk header ketiga chain (chainwork 8); induk header
    // baru tidak pernah dibuang, jadi header keempat menunggu.
    let b = open_limited(NodeLimits {
        max_headers: 3,
        ..NodeLimits::DEFAULT
    });
    assert_eq!(
        b.node.accept_headers(&[junk]).unwrap(),
        HeadersOutcome::Accepted {
            added: 1,
            last: Some(block_id(&junk))
        }
    );
    assert_eq!(
        b.node.accept_headers(&headers).unwrap(),
        HeadersOutcome::Full { added: 3 }
    );
    assert_eq!(header_set(&b), set(ids[..3].iter().copied()));

    // Badan blok mengosongkan indeks; sinkronisasi header berlanjut sampai chain lengkap.
    assert_eq!(b.node.missing_blocks(16).unwrap(), ids[..3].to_vec());
    for block in &blocks[..3] {
        assert!(b.node.receive_block(&block.to_bytes()).unwrap().new_tip);
    }
    assert!(header_set(&b).is_empty());
    assert_eq!(
        b.node.accept_headers(&headers).unwrap(),
        HeadersOutcome::Accepted {
            added: 2,
            last: Some(ids[4])
        }
    );
    assert_eq!(b.node.missing_blocks(16).unwrap(), ids[3..].to_vec());
    for block in &blocks[3..] {
        assert!(b.node.receive_block(&block.to_bytes()).unwrap().new_tip);
    }
    assert_active(&b, &blocks);
}

#[test]
fn invalid_block_records_are_bounded() {
    let fx = open_limited(NodeLimits {
        max_invalid: 2,
        ..NodeLimits::DEFAULT
    });
    let genesis = params().network_id();
    // Tinggi 2 langsung di atas genesis: V5, dicatat sebagai tidak sah.
    let bad: Vec<Block> = (5_u8..8)
        .map(|owner| block_on(genesis, 2, owner, Vec::new(), 0))
        .collect();
    for block in &bad {
        assert_eq!(submit(&fx, block), rejected_block(block, "V5"));
    }
    assert_eq!(invalid_set(&fx), set([id_of(&bad[1]), id_of(&bad[2])]));
    // Catatan tertua terlupa: blok itu diperiksa ulang dan tetap ditolak; kini catatan berikutnya yang terlupa.
    assert_eq!(submit(&fx, &bad[0]), rejected_block(&bad[0], "V5"));
    assert_eq!(invalid_set(&fx), set([id_of(&bad[2]), id_of(&bad[0])]));
}

#[test]
fn side_block_bodies_live_in_store_and_are_cleared_on_reopen() {
    let fx = open(RedbStore::in_memory().unwrap());
    let (a, b) = reorg_scenario(&fx);
    // Chain A yang dilepas menjadi blok cabang: badannya di penyimpanan; blok chain aktif B tidak.
    for block in &a {
        assert!(side_body_stored(&fx, block));
    }
    for block in &b {
        assert!(!side_body_stored(&fx, block));
    }
    // GetBlocks membaca badan blok cabang dari penyimpanan.
    assert_eq!(
        fx.node.blocks_by_id(&[id_of(&a[0]), id_of(&b[0])]).unwrap(),
        vec![Some(a[0].clone()), Some(b[0].clone())]
    );

    // Reorg kembali ke A: badan blok A dibaca dari penyimpanan lalu dihapus dari sana; B pindah ke penyimpanan.
    let a4 = block_on(id_of(&a[2]), 4, 2, Vec::new(), 0);
    assert_eq!(submit(&fx, &a4).status, SubmitStatus::Accepted);
    assert!(side_body_stored(&fx, &a4));
    let a5 = block_on(id_of(&a4), 5, 2, Vec::new(), 0);
    assert_eq!(submit(&fx, &a5).status, SubmitStatus::Accepted);
    let mut chain_a = a.clone();
    chain_a.push(a4);
    chain_a.push(a5);
    assert_active(&fx, &chain_a);
    for block in &chain_a {
        assert!(!side_body_stored(&fx, block));
    }
    for block in &b {
        assert!(side_body_stored(&fx, block));
    }

    // Dibuka ulang: chain aktif utuh, indeks dan badan blok cabang dibuang.
    let reopened = open(fx.node.into_store().unwrap());
    assert_active(&reopened, &chain_a);
    assert_eq!(side_count(&reopened), 0);
    for block in &b {
        assert!(!side_body_stored(&reopened, block));
    }
}

#[test]
fn trimmed_and_invalid_side_blocks_leave_store() {
    let fx = open_limited(NodeLimits {
        max_side_blocks: 2,
        ..NodeLimits::DEFAULT
    });
    let genesis = params().network_id();
    let a = extend(&fx, genesis, 1, 2, 2);
    // Tiga cabang satu blok dari genesis: yang paling akhir dibuang dari memori dan dari penyimpanan.
    let forks: Vec<Block> = (5_u8..8)
        .map(|owner| block_on(genesis, 1, owner, Vec::new(), 0))
        .collect();
    for fork in &forks {
        assert_eq!(submit(&fx, fork).status, SubmitStatus::Accepted);
    }
    assert_eq!(side_set(&fx), set([id_of(&forks[0]), id_of(&forks[1])]));
    assert!(side_body_stored(&fx, &forks[0]));
    assert!(side_body_stored(&fx, &forks[1]));
    assert!(!side_body_stored(&fx, &forks[2]));

    // Di atas fork pertama: blok tinggi 2 mengklaim satu quanta terlalu banyak (V10). Headernya sah, jadi disimpan sebagai
    // blok cabang; daun paling ringan (fork kedua) dibuang.
    let bad = block_on(id_of(&forks[0]), 2, 5, Vec::new(), 1);
    assert_eq!(submit(&fx, &bad).status, SubmitStatus::Accepted);
    assert_eq!(side_set(&fx), set([id_of(&forks[0]), id_of(&bad)]));
    assert!(!side_body_stored(&fx, &forks[1]));

    // Anaknya membuat cabang lebih berat; reorg gagal di blok tinggi 2. Kedua blok dibuang dari memori dan dari
    // penyimpanan, chain A dipulihkan dan blok A tidak tertinggal di penyimpanan.
    let child = block_on(id_of(&bad), 3, 5, Vec::new(), 0);
    assert_eq!(submit(&fx, &child), rejected_block(&child, "V5"));
    assert_active(&fx, &a);
    assert_eq!(side_set(&fx), set([id_of(&forks[0])]));
    assert!(side_body_stored(&fx, &forks[0]));
    assert!(!side_body_stored(&fx, &bad));
    assert!(!side_body_stored(&fx, &child));
    for block in &a {
        assert!(!side_body_stored(&fx, block));
    }
}

/// Header dari setiap blok `blocks`.
fn headers_of(blocks: &[Block]) -> Vec<Header> {
    blocks.iter().map(|block| *block.header()).collect()
}

/// Rantai `count` blok di atas `prev` mulai tinggi `from`, semuanya ke `owner`, tanpa dikirim ke node.
fn chain_on(prev: Hash32, from: u64, count: u64, owner: u8) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut parent = prev;
    for height in from..from + count {
        let block = block_on(parent, height, owner, Vec::new(), 0);
        parent = id_of(&block);
        blocks.push(block);
    }
    blocks
}

#[test]
fn presync_syncs_branch_longer_than_header_index() {
    let b = open_limited(NodeLimits {
        max_headers: 3,
        ..NodeLimits::DEFAULT
    });
    let genesis = params().network_id();
    extend(&b, genesis, 1, 5, 2);
    // Cabang dari genesis baru lebih berat daripada chain aktif (chainwork 12) di tinggi 6 (chainwork 14).
    let long = chain_on(genesis, 1, 8, 4);
    let headers = headers_of(&long);
    let ids: Vec<Hash32> = long.iter().map(id_of).collect();
    // Tanpa presync: indeks header penuh sebelum cabang terbukti lebih berat, jadi tidak ada badan blok yang diminta.
    assert_eq!(
        b.node.accept_headers(&headers).unwrap(),
        HeadersOutcome::Full { added: 3 }
    );
    assert!(b.node.missing_blocks(16).unwrap().is_empty());

    // Tahap hitung: header diperiksa tanpa disimpan sampai chainwork melebihi chain aktif.
    let mut presync = Presync::default();
    assert_eq!(
        b.node.presync_headers(&mut presync, &headers[..4]).unwrap(),
        PresyncOutcome::Continue { after: ids[3] }
    );
    assert_eq!(
        b.node.presync_headers(&mut presync, &headers[4..]).unwrap(),
        PresyncOutcome::Redownload { fork: genesis }
    );
    assert_eq!(header_set(&b), set(ids[..3].iter().copied()));

    // Unduh ulang dari titik cabang: header disimpan sebagai header terbukti, melebihi batas indeks header.
    assert_eq!(
        b.node.presync_headers(&mut presync, &headers[..2]).unwrap(),
        PresyncOutcome::Continue { after: ids[1] }
    );
    assert_eq!(
        b.node.presync_headers(&mut presync, &headers[2..]).unwrap(),
        PresyncOutcome::Done { added: 3 }
    );
    assert_eq!(header_set(&b), set(ids[..6].iter().copied()));

    // Badan blok diunduh dan node reorg ke cabang itu; sisanya lewat sinkronisasi header biasa.
    assert_eq!(b.node.missing_blocks(16).unwrap(), ids[..6].to_vec());
    for block in &long[..6] {
        let receipt = b.node.receive_block(&block.to_bytes()).unwrap();
        assert_eq!(receipt.result.status, SubmitStatus::Accepted);
    }
    assert_active(&b, &long[..6]);
    assert!(header_set(&b).is_empty());
    assert_eq!(
        b.node.accept_headers(&headers).unwrap(),
        HeadersOutcome::Accepted {
            added: 2,
            last: Some(ids[7])
        }
    );
    for block in &long[6..] {
        assert!(b.node.receive_block(&block.to_bytes()).unwrap().new_tip);
    }
    assert_active(&b, &long);
}

#[test]
fn presync_stops_on_bad_or_changed_headers() {
    let b = open_limited(NodeLimits {
        max_headers: 3,
        ..NodeLimits::DEFAULT
    });
    let genesis = params().network_id();
    extend(&b, genesis, 1, 5, 2);
    let long = chain_on(genesis, 1, 8, 4);
    let headers = headers_of(&long);
    let fresh = |headers: &[Header]| {
        let mut presync = Presync::default();
        b.node.presync_headers(&mut presync, headers).unwrap()
    };

    // Tidak menyambung ke blok yang dikenal atau ke header sebelumnya, dan header tidak sah: rekan disanksi.
    assert_eq!(fresh(&headers[1..]), PresyncOutcome::Disconnected);
    assert_eq!(
        fresh(&[headers[0], headers[2]]),
        PresyncOutcome::Disconnected
    );
    let tall = *block_on(genesis, 2, 9, Vec::new(), 0).header();
    assert_eq!(fresh(&[tall]), PresyncOutcome::Invalid { rule: "V5" });
    // Pesan kosong dan V8: berhenti tanpa sanksi.
    assert_eq!(fresh(&[]), PresyncOutcome::NotHeavier);
    b.now.store(T0 + 240 - 3_601, Ordering::SeqCst);
    assert_eq!(fresh(&headers), PresyncOutcome::Deferred);
    b.now.store(FAR, Ordering::SeqCst);

    // Unduh ulang dengan cabang lain dari genesis: ID di ujung yang diingat berbeda.
    let other = headers_of(&chain_on(genesis, 1, 8, 5));
    let mut presync = Presync::default();
    assert_eq!(
        b.node.presync_headers(&mut presync, &headers).unwrap(),
        PresyncOutcome::Redownload { fork: genesis }
    );
    assert_eq!(
        b.node.presync_headers(&mut presync, &other).unwrap(),
        PresyncOutcome::Mismatch
    );
    // Unduh ulang yang melompati header, atau berhenti sebelum ujungnya.
    let mut presync = Presync::default();
    assert_eq!(
        b.node.presync_headers(&mut presync, &headers).unwrap(),
        PresyncOutcome::Redownload { fork: genesis }
    );
    assert_eq!(
        b.node
            .presync_headers(&mut presync, &[headers[0], headers[2]])
            .unwrap(),
        PresyncOutcome::Mismatch
    );
    let mut presync = Presync::default();
    assert_eq!(
        b.node.presync_headers(&mut presync, &headers).unwrap(),
        PresyncOutcome::Redownload { fork: genesis }
    );
    assert_eq!(
        b.node.presync_headers(&mut presync, &headers[..3]).unwrap(),
        PresyncOutcome::Continue {
            after: id_of(&long[2])
        }
    );
    assert_eq!(
        b.node.presync_headers(&mut presync, &[]).unwrap(),
        PresyncOutcome::Mismatch
    );
    // Tidak ada header yang tersimpan dari unduh ulang yang gagal sebelum komitmen.
    assert!(header_set(&b).is_empty());
}

#[test]
fn presync_checks_commitment_every_interval() {
    assert_eq!(PRESYNC_COMMIT_INTERVAL, 1_000);
    let b = open_limited(NodeLimits {
        max_headers: 3,
        ..NodeLimits::DEFAULT
    });
    let genesis = params().network_id();
    // Chain aktif 1_001 blok (chainwork 2_004); cabang dari genesis baru lebih berat di tinggi 1_002.
    extend(&b, genesis, 1, 1_001, 2);
    let long = chain_on(genesis, 1, 1_003, 4);
    let headers = headers_of(&long);
    let ids: Vec<Hash32> = long.iter().map(id_of).collect();
    // Cabang yang sama sampai tinggi 1_001, lalu berbeda.
    let mut forked = long[..1_001].to_vec();
    forked.extend(chain_on(ids[1_000], 1_002, 2, 5));
    let forked = headers_of(&forked);

    let mut presync = Presync::default();
    assert_eq!(
        b.node.presync_headers(&mut presync, &headers).unwrap(),
        PresyncOutcome::Redownload { fork: genesis }
    );
    // Komitmen tinggi 1_000 cocok: 1_000 header disimpan sebagai header terbukti; ujungnya berbeda, presync berhenti.
    assert_eq!(
        b.node.presync_headers(&mut presync, &forked).unwrap(),
        PresyncOutcome::Mismatch
    );
    assert_eq!(header_set(&b), set(ids[..1_000].iter().copied()));
    // Header terbukti itu tidak lebih berat daripada chain aktif, jadi kembali menjadi header biasa.
    assert!(b.node.missing_blocks(16).unwrap().is_empty());
    let proven = |fx: &Fixture<MemoryStore>| {
        fx.node
            .lock()
            .unwrap()
            .headers
            .values()
            .filter(|entry| entry.proven)
            .count()
    };
    assert_eq!(proven(&b), 0);

    // Presync ulang dengan cabang yang benar: sisanya disimpan, badan blok diminta dari yang terendah.
    let mut presync = Presync::default();
    assert_eq!(
        b.node.presync_headers(&mut presync, &headers).unwrap(),
        PresyncOutcome::Redownload { fork: genesis }
    );
    assert_eq!(
        b.node.presync_headers(&mut presync, &headers).unwrap(),
        PresyncOutcome::Done { added: 2 }
    );
    assert_eq!(header_set(&b).len(), 1_002);
    assert_eq!(b.node.missing_blocks(16).unwrap(), ids[..16].to_vec());
    assert_eq!(proven(&b), 1_002);
}

#[test]
fn scan_mempool_reports_pending_spends_and_outputs() {
    let fx = open(MemoryStore::default());
    fx.now.store(FAR, Ordering::SeqCst);
    let blocks = extend(&fx, params().network_id(), 1, 99, 2);
    let owner = Hex32::new(key(1));
    let receiver = Hex32::new(key(3));
    let stranger = Hex32::new(key(9));
    assert_eq!(
        fx.node.scan_mempool(&[owner, receiver]).unwrap(),
        MempoolScan::default()
    );

    let tx = spend_genesis(1, G - 1_000);
    let id = Hex32::from(txid(&tx));
    assert_eq!(
        fx.node.send_raw_transaction(&tx.to_bytes()).unwrap().txid,
        id
    );
    let genesis = regtest_genesis().unwrap();
    let genesis_txid = Hex32::from(txid(genesis.txs().first().unwrap()));
    let expected = MempoolScan {
        spent: vec![MempoolSpend {
            txid: genesis_txid,
            index: 0,
            spent_by: id,
        }],
        outputs: vec![MempoolOutput {
            txid: id,
            index: 0,
            amount: Quanta::new(Amount::from_quanta(G - 1_000)),
            kind: KindNumber::new(OutputKind::SingleKey),
            payload: receiver,
        }],
    };
    assert_eq!(
        fx.node.scan_mempool(&[receiver, owner, receiver]).unwrap(),
        expected
    );
    // Hanya payload yang diminta: pemilik alokasi melihat UTXO-nya dibelanjakan, penerima melihat output tertunda.
    let only_owner = fx.node.scan_mempool(&[owner]).unwrap();
    assert_eq!(only_owner.spent, expected.spent);
    assert!(only_owner.outputs.is_empty());
    let only_receiver = fx.node.scan_mempool(&[receiver]).unwrap();
    assert!(only_receiver.spent.is_empty());
    assert_eq!(only_receiver.outputs, expected.outputs);
    assert_eq!(
        fx.node.scan_mempool(&[stranger]).unwrap(),
        MempoolScan::default()
    );

    // Setelah transaksi masuk blok, tidak ada lagi yang tertunda.
    let tip = id_of(blocks.last().unwrap());
    let block = block_on(tip, 100, 2, vec![tx], 1_000);
    assert_eq!(submit(&fx, &block).status, SubmitStatus::Accepted);
    assert_eq!(
        fx.node.scan_mempool(&[owner, receiver]).unwrap(),
        MempoolScan::default()
    );
    assert_eq!(fx.node.scan_utxos(&[receiver]).unwrap().len(), 1);
}
