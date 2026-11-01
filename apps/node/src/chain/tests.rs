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
    GetBlockTemplateParams, Hex32, KindNumber, RpcBackend, RpcError, SendRawTransactionResult,
    SubmitBlockResult, SubmitStatus,
};
use homeostasis_storage::{ChainStore, MemoryStore, RedbStore};

use super::{
    Clock, HeadersOutcome, MAX_LOCATOR_IDS, Node, NodeError, block_rule, locator_heights,
    mempool_reason,
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

    // Dibuka ulang: chain aktif (cabang B) utuh; blok cabang tidak disimpan di penyimpanan (v0.1).
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
