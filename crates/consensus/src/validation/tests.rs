use std::collections::BTreeMap;

use proptest::prelude::*;
use zevrigo_crypto::{
    SecretKey, block_id, condition_hash, merkle_root, sighash_v1, sign, txid, wtxid,
};
use zevrigo_pow::mine_regtest;
use zevrigo_primitives::params::GENESIS_ALLOCATION;
use zevrigo_primitives::{
    Amount, Block, BlockHeight, Condition, DecodeError, Encode, Hash32, Header, Input, Network,
    OutPoint, Output, OutputKind, SigEntry, Transaction, UtxoEntry, Witness,
};

use super::{
    BlockError, BlockSummary, ParentContext, TxError, check_block_context_free,
    check_block_transactions, check_future_timestamp, check_header_context, check_transaction,
    decode_block,
};
use crate::chain_params::{ChainParams, REGTEST_GENESIS_KEY, regtest_genesis};
use crate::difficulty::DifficultyError;
use crate::supply::block_subsidy;

/// Cap waktu genesis regtest.
const T0: u64 = 1_793_145_600;
/// Cap waktu blok 1 (tepat jadwal).
const T1: u64 = T0 + 120;
/// Jam lokal paling awal yang masih menerima blok 1 (V8).
const T1_EARLIEST_NOW: u64 = T1 - 3_600;
/// Satu detik sebelum `T1_EARLIEST_NOW`.
const T1_TOO_EARLY_NOW: u64 = T1_EARLIEST_NOW - 1;
/// Satu detik setelah genesis.
const T0_PLUS_ONE: u64 = T0 + 1;
/// `pow_limit` dan target genesis regtest.
const LIMIT: u32 = 0x207f_ffff;

/// Alokasi genesis `G` dalam quanta.
const G_Q: u128 = GENESIS_ALLOCATION.as_quanta();
/// Biaya contoh.
const FEE: u128 = 1_000;
/// Output setelah membelanjakan genesis dengan satu biaya.
const SPEND: u128 = G_Q - FEE;
/// Output setelah dua biaya.
const SPEND_TWICE: u128 = SPEND - FEE;
/// Output satu quanta lebih besar dari genesis.
const G_PLUS_ONE: u128 = G_Q + 1;

/// Subsidy tinggi 1 dengan `M = 0` (Spesifikasi Format Bagian 5.5, TSK-M07).
const SUBSIDY_1: u128 = 4_361_021_521_773_785;
/// Subsidy tinggi 100 dengan `M = 0`, dihitung Quick dengan referensi Python.
const SUBSIDY_100: u128 = 436_102_152_177_378_500;
const SUBSIDY_100_PLUS_FEE: u128 = SUBSIDY_100 + FEE;
const SUBSIDY_100_PLUS_TWO_FEES: u128 = SUBSIDY_100_PLUS_FEE + FEE;
const SUBSIDY_100_PLUS_FEE_PLUS_ONE: u128 = SUBSIDY_100_PLUS_FEE + 1;

/// Kunci waktu kondisi contoh dan satu blok sebelumnya.
const LOCK: u64 = 150;
const LOCK_MINUS_ONE: u64 = LOCK - 1;
/// Jumlah output kondisi contoh dan output pembelanjaannya.
const COND_AMOUNT: u128 = 5_000;
const COND_SPEND: u128 = 4_000;
const COND_FEE: u128 = COND_AMOUNT - COND_SPEND;

/// Vektor blok 1 regtest (dihitung Quick dengan `blake3_ref.py`).
const BLOCK1_COINBASE_TXID: &str =
    "cb49e918fb022e9845ca43fad8ee738cb66934d00138ceb31f947e4859fb86b6";
const BLOCK1_MERKLE_ROOT: &str = "4d755d02a67f9b24b8c5976b536519678389491721cc02ae75dfd14790eb367d";
const BLOCK1_ID: &str = "6baf7b66864361e35e8d4a852053f82efa926cf0a9c8ae91368d6c3124c4c572";

fn params() -> ChainParams {
    ChainParams::regtest().unwrap()
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

fn out_to(amount: u128, payload: [u8; 32]) -> Output {
    Output::new(Amount::from_quanta(amount), OutputKind::SingleKey, payload).unwrap()
}

fn coinbase_to(height: u64, amount: u128, payload: [u8; 32]) -> Transaction {
    Transaction::coinbase(
        BlockHeight::new(height),
        Vec::new(),
        vec![out_to(amount, payload)],
    )
    .unwrap()
}

/// Blok untuk tahap konteks UTXO saja (header tidak diperiksa di tahap ini).
fn block_with(height: u64, coinbase: Transaction, txs: Vec<Transaction>) -> Block {
    let header = Header::new(
        BlockHeight::new(height),
        Hash32::ZERO,
        Hash32::ZERO,
        0,
        LIMIT,
        0,
    );
    let mut all = vec![coinbase];
    all.extend(txs);
    Block::new(header, all).unwrap()
}

fn block_at(height: u64, coinbase_amount: u128, txs: Vec<Transaction>) -> Block {
    block_with(height, coinbase_to(height, coinbase_amount, key(9)), txs)
}

fn genesis_output() -> Output {
    out_to(G_Q, REGTEST_GENESIS_KEY)
}

fn genesis_outpoint() -> OutPoint {
    let genesis = regtest_genesis().unwrap();
    OutPoint::new(txid(genesis.txs().first().unwrap()), 0)
}

fn genesis_input() -> Input {
    let outpoint = genesis_outpoint();
    Input::new(outpoint.txid(), outpoint.index())
}

fn no_utxos() -> BTreeMap<OutPoint, UtxoEntry> {
    BTreeMap::new()
}

fn genesis_utxos() -> BTreeMap<OutPoint, UtxoEntry> {
    let mut map = BTreeMap::new();
    map.insert(
        genesis_outpoint(),
        UtxoEntry::new(genesis_output(), BlockHeight::GENESIS, true),
    );
    map
}

fn unsigned_tx(inputs: Vec<Input>, outputs: Vec<Output>) -> Transaction {
    let witnesses = vec![Witness::single_key([0; 64]); inputs.len()];
    Transaction::regular(inputs, outputs, witnesses).unwrap()
}

/// Transaksi jenis 0 yang input ke-i ditandatangani `keys[i]` atas sighash v1 `network_id`.
fn signed_tx(
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    spent: &[Output],
    keys: &[&SecretKey],
    network_id: &Hash32,
) -> Transaction {
    let unsigned = unsigned_tx(inputs.clone(), outputs.clone());
    let witnesses = keys
        .iter()
        .enumerate()
        .map(|(index, sk)| {
            let index = u32::try_from(index).unwrap();
            let msg = sighash_v1(network_id, &unsigned, index, spent).unwrap();
            Witness::single_key(*sign(sk, msg.as_bytes(), &[0; 32]).unwrap().as_bytes())
        })
        .collect();
    Transaction::regular(inputs, outputs, witnesses).unwrap()
}

fn spend_genesis(amount: u128, payload: [u8; 32], network_id: &Hash32) -> Transaction {
    signed_tx(
        vec![genesis_input()],
        vec![out_to(amount, payload)],
        &[genesis_output()],
        &[&secret(1)],
        network_id,
    )
}

fn block1() -> Block {
    let params = params();
    let coinbase = coinbase_to(1, SUBSIDY_1, REGTEST_GENESIS_KEY);
    let root = merkle_root(&[wtxid(&coinbase)]).unwrap();
    let header = Header::new(BlockHeight::new(1), params.network_id(), root, T1, LIMIT, 0);
    let mined = mine_regtest(&header, &params.pow_limit(), 64).unwrap();
    Block::new(mined, vec![coinbase]).unwrap()
}

fn header_with(target: u32, merkle: Hash32) -> Header {
    Header::new(
        BlockHeight::new(1),
        params().network_id(),
        merkle,
        T1,
        target,
        0,
    )
}

#[test]
fn block1_passes_all_stages() {
    let params = params();
    let block = block1();
    let header = block.header();
    assert_eq!(header.nonce(), 3);
    assert_eq!(header.merkle_root().to_string(), BLOCK1_MERKLE_ROOT);
    assert_eq!(
        txid(block.txs().first().unwrap()).to_string(),
        BLOCK1_COINBASE_TXID
    );
    assert_eq!(block_id(header).to_string(), BLOCK1_ID);

    assert_eq!(decode_block(&block.to_bytes()), Ok(block.clone()));
    assert_eq!(check_block_context_free(&block, &params), Ok(()));
    let parent = ParentContext {
        block_id: params.network_id(),
        height: BlockHeight::GENESIS,
        timestamps: &[T0],
    };
    assert_eq!(check_header_context(header, &parent, &params), Ok(()));
    assert_eq!(check_future_timestamp(header, T1), Ok(()));
    assert_eq!(
        check_block_transactions(&block, &params, Amount::ZERO, &no_utxos()),
        Ok(BlockSummary {
            subsidy: Amount::from_quanta(SUBSIDY_1),
            fees: Amount::ZERO,
        })
    );
}

#[test]
fn regtest_pow_rejected_on_testnet_and_mainnet() {
    let regtest = params();
    let block = block1();
    for network in [Network::Testnet, Network::Mainnet] {
        let other = ChainParams::new(
            network,
            regtest.pow_limit(),
            regtest.genesis_target(),
            regtest.genesis_timestamp(),
            regtest.network_id(),
        );
        assert_eq!(
            check_block_context_free(&block, &other),
            Err(BlockError::PowUnavailable { network })
        );
    }
}

#[test]
fn context_free_rejections_v2_to_v4() {
    let params = params();
    let coinbase = coinbase_to(1, SUBSIDY_1, REGTEST_GENESIS_KEY);
    let root = merkle_root(&[wtxid(&coinbase)]).unwrap();

    let invalid = Block::new(header_with(0x2101_0000, root), vec![coinbase.clone()]).unwrap();
    assert_eq!(
        check_block_context_free(&invalid, &params),
        Err(BlockError::Difficulty(DifficultyError::InvalidCompact {
            compact: 0x2101_0000
        }))
    );

    let above = Block::new(header_with(0x20ff_ffff, root), vec![coinbase.clone()]).unwrap();
    assert_eq!(
        check_block_context_free(&above, &params),
        Err(BlockError::TargetAboveLimit)
    );

    let hard = Block::new(header_with(0x1e0f_ffff, root), vec![coinbase.clone()]).unwrap();
    assert_eq!(
        check_block_context_free(&hard, &params),
        Err(BlockError::HighHash)
    );

    let wrong_root =
        mine_regtest(&header_with(LIMIT, Hash32::ZERO), &params.pow_limit(), 64).unwrap();
    assert_eq!(wrong_root.nonce(), 0);
    let wrong = Block::new(wrong_root, vec![coinbase]).unwrap();
    assert_eq!(
        check_block_context_free(&wrong, &params),
        Err(BlockError::MerkleRootMismatch)
    );
}

#[test]
fn decode_block_rejects_bad_bytes_v1() {
    assert!(matches!(decode_block(&[]), Err(BlockError::Decode(_))));
    let mut bytes = block1().to_bytes();
    bytes.push(0);
    assert!(matches!(
        decode_block(&bytes),
        Err(BlockError::Decode(DecodeError::TrailingBytes { .. }))
    ));
}

#[test]
fn header_context_rejections_v5_and_v7() {
    let params = params();
    let block = block1();
    let header = block.header();

    let wrong_parent = ParentContext {
        block_id: Hash32::ZERO,
        height: BlockHeight::GENESIS,
        timestamps: &[T0],
    };
    assert_eq!(
        check_header_context(header, &wrong_parent, &params),
        Err(BlockError::PrevBlockMismatch)
    );

    let wrong_height = ParentContext {
        block_id: params.network_id(),
        height: BlockHeight::new(1),
        timestamps: &[T0],
    };
    assert_eq!(
        check_header_context(header, &wrong_height, &params),
        Err(BlockError::HeightMismatch {
            parent: BlockHeight::new(1),
            actual: BlockHeight::new(1),
        })
    );

    let no_times = ParentContext {
        block_id: params.network_id(),
        height: BlockHeight::GENESIS,
        timestamps: &[],
    };
    assert_eq!(
        check_header_context(header, &no_times, &params),
        Err(BlockError::MissingParentTimestamps)
    );

    let same_time = ParentContext {
        block_id: params.network_id(),
        height: BlockHeight::GENESIS,
        timestamps: &[T1],
    };
    assert_eq!(
        check_header_context(header, &same_time, &params),
        Err(BlockError::TimestampNotAfterMedian {
            timestamp: T1,
            median: T1,
        })
    );
}

#[test]
fn header_context_enforces_asert_target_v6() {
    let params = params();
    // Induk di tinggi 720 dengan cap waktu genesis: chain satu hari lebih cepat, target menjadi setengah.
    let parent = ParentContext {
        block_id: Hash32::from_bytes([0x11; 32]),
        height: BlockHeight::new(720),
        timestamps: &[T0],
    };
    let at_limit = Header::new(
        BlockHeight::new(721),
        parent.block_id,
        Hash32::ZERO,
        T0_PLUS_ONE,
        LIMIT,
        0,
    );
    assert_eq!(
        check_header_context(&at_limit, &parent, &params),
        Err(BlockError::TargetMismatch {
            expected: 0x203f_ffff,
            actual: LIMIT,
        })
    );
    let halved = Header::new(
        BlockHeight::new(721),
        parent.block_id,
        Hash32::ZERO,
        T0_PLUS_ONE,
        0x203f_ffff,
        0,
    );
    assert_eq!(check_header_context(&halved, &parent, &params), Ok(()));
}

#[test]
fn future_timestamp_rule_v8() {
    let block = block1();
    let header = block.header();
    assert_eq!(check_future_timestamp(header, T1_EARLIEST_NOW), Ok(()));
    assert_eq!(
        check_future_timestamp(header, T1_TOO_EARLY_NOW),
        Err(BlockError::TimestampTooFarInFuture {
            timestamp: T1,
            now: T1_TOO_EARLY_NOW,
        })
    );
}

#[test]
fn spend_genesis_at_maturity() {
    let params = params();
    let network_id = params.network_id();
    assert_eq!(
        block_subsidy(BlockHeight::new(100), Amount::ZERO),
        Ok(Amount::from_quanta(SUBSIDY_100))
    );
    let tx = spend_genesis(SPEND, key(2), &network_id);
    assert_eq!(
        check_transaction(&tx, BlockHeight::new(100), &network_id, &genesis_utxos()),
        Ok(Amount::from_quanta(FEE))
    );
    let block = block_at(100, SUBSIDY_100_PLUS_FEE, vec![tx]);
    assert_eq!(
        check_block_transactions(&block, &params, Amount::ZERO, &genesis_utxos()),
        Ok(BlockSummary {
            subsidy: Amount::from_quanta(SUBSIDY_100),
            fees: Amount::from_quanta(FEE),
        })
    );
}

#[test]
fn immature_coinbase_rejected_t2() {
    let params = params();
    let network_id = params.network_id();
    let tx = spend_genesis(SPEND, key(2), &network_id);
    let block = block_at(99, SUBSIDY_100_PLUS_FEE, vec![tx]);
    assert_eq!(
        check_block_transactions(&block, &params, Amount::ZERO, &genesis_utxos()),
        Err(BlockError::Transaction {
            index: 1,
            error: TxError::ImmatureCoinbase {
                outpoint: genesis_outpoint()
            },
        })
    );

    // Output coinbase blok yang sama juga belum matang.
    let coinbase = coinbase_to(100, SUBSIDY_100, key(1));
    let outpoint = OutPoint::new(txid(&coinbase), 0);
    let spend = unsigned_tx(
        vec![Input::new(outpoint.txid(), 0)],
        vec![out_to(1, key(2))],
    );
    let block = block_with(100, coinbase, vec![spend]);
    assert_eq!(
        check_block_transactions(&block, &params, Amount::ZERO, &no_utxos()),
        Err(BlockError::Transaction {
            index: 1,
            error: TxError::ImmatureCoinbase { outpoint },
        })
    );
}

#[test]
fn missing_and_duplicate_inputs_t1() {
    let network_id = params().network_id();
    let height = BlockHeight::new(100);
    let tx = spend_genesis(SPEND, key(2), &network_id);
    assert_eq!(
        check_transaction(&tx, height, &network_id, &no_utxos()),
        Err(TxError::MissingUtxo {
            outpoint: genesis_outpoint()
        })
    );
    let twice = unsigned_tx(
        vec![genesis_input(), genesis_input()],
        vec![out_to(SPEND, key(2))],
    );
    assert_eq!(
        check_transaction(&twice, height, &network_id, &genesis_utxos()),
        Err(TxError::DuplicateInput {
            outpoint: genesis_outpoint()
        })
    );
}

#[test]
fn double_spend_across_transactions_t1() {
    let params = params();
    let network_id = params.network_id();
    let first = spend_genesis(SPEND, key(2), &network_id);
    let second = spend_genesis(SPEND, key(3), &network_id);
    let block = block_at(100, SUBSIDY_100_PLUS_TWO_FEES, vec![first, second]);
    assert_eq!(
        check_block_transactions(&block, &params, Amount::ZERO, &genesis_utxos()),
        Err(BlockError::Transaction {
            index: 2,
            error: TxError::MissingUtxo {
                outpoint: genesis_outpoint()
            },
        })
    );
}

#[test]
fn chained_spend_in_block_respects_order() {
    let params = params();
    let network_id = params.network_id();
    let first = spend_genesis(SPEND, key(2), &network_id);
    let first_out = OutPoint::new(txid(&first), 0);
    let second = signed_tx(
        vec![Input::new(first_out.txid(), 0)],
        vec![out_to(SPEND_TWICE, key(3))],
        &[out_to(SPEND, key(2))],
        &[&secret(2)],
        &network_id,
    );

    let ordered = block_at(
        100,
        SUBSIDY_100_PLUS_TWO_FEES,
        vec![first.clone(), second.clone()],
    );
    assert_eq!(
        check_block_transactions(&ordered, &params, Amount::ZERO, &genesis_utxos()),
        Ok(BlockSummary {
            subsidy: Amount::from_quanta(SUBSIDY_100),
            fees: Amount::from_quanta(FEE)
                .checked_add(Amount::from_quanta(FEE))
                .unwrap(),
        })
    );

    let reversed = block_at(100, SUBSIDY_100_PLUS_TWO_FEES, vec![second, first]);
    assert_eq!(
        check_block_transactions(&reversed, &params, Amount::ZERO, &genesis_utxos()),
        Err(BlockError::Transaction {
            index: 1,
            error: TxError::MissingUtxo {
                outpoint: first_out
            },
        })
    );
}

#[test]
fn witness_kind_mismatch_t3() {
    let network_id = params().network_id();
    let condition = Condition::new(1, vec![key(1)], BlockHeight::GENESIS).unwrap();
    let witness = Witness::condition(condition, vec![SigEntry::new(0, [0; 64])]).unwrap();
    let tx = Transaction::regular(
        vec![genesis_input()],
        vec![out_to(SPEND, key(2))],
        vec![witness],
    )
    .unwrap();
    assert_eq!(
        check_transaction(&tx, BlockHeight::new(100), &network_id, &genesis_utxos()),
        Err(TxError::WitnessKindMismatch { input_index: 0 })
    );
}

#[test]
fn outputs_exceed_inputs_t5() {
    let network_id = params().network_id();
    let tx = unsigned_tx(vec![genesis_input()], vec![out_to(G_PLUS_ONE, key(2))]);
    assert_eq!(
        check_transaction(&tx, BlockHeight::new(100), &network_id, &genesis_utxos()),
        Err(TxError::OutputsExceedInputs)
    );
    let exact = spend_genesis(G_Q, key(2), &network_id);
    assert_eq!(
        check_transaction(&exact, BlockHeight::new(100), &network_id, &genesis_utxos()),
        Ok(Amount::ZERO)
    );
}

#[test]
fn invalid_signatures_t6() {
    let network_id = params().network_id();
    let height = BlockHeight::new(100);
    let other_network = spend_genesis(SPEND, key(2), &Hash32::ZERO);
    assert_eq!(
        check_transaction(&other_network, height, &network_id, &genesis_utxos()),
        Err(TxError::InvalidSignature { input_index: 0 })
    );
    let wrong_key = signed_tx(
        vec![genesis_input()],
        vec![out_to(SPEND, key(2))],
        &[genesis_output()],
        &[&secret(2)],
        &network_id,
    );
    assert_eq!(
        check_transaction(&wrong_key, height, &network_id, &genesis_utxos()),
        Err(TxError::InvalidSignature { input_index: 0 })
    );
    let unsigned = unsigned_tx(vec![genesis_input()], vec![out_to(SPEND, key(2))]);
    assert_eq!(
        check_transaction(&unsigned, height, &network_id, &genesis_utxos()),
        Err(TxError::InvalidSignature { input_index: 0 })
    );
}

/// Kondisi 2-dari-3 dengan kunci waktu `LOCK`, beserta kunci rahasia yang urutannya sama dengan `keys`.
fn condition_setup() -> (Condition, Vec<SecretKey>) {
    let mut pairs: Vec<([u8; 32], SecretKey)> = (1..=3).map(|n| (key(n), secret(n))).collect();
    pairs.sort_by_key(|pair| pair.0);
    let keys = pairs.iter().map(|pair| pair.0).collect();
    let condition = Condition::new(2, keys, BlockHeight::new(LOCK)).unwrap();
    (condition, pairs.into_iter().map(|pair| pair.1).collect())
}

fn condition_outpoint() -> OutPoint {
    OutPoint::new(Hash32::from_bytes([0xab; 32]), 0)
}

fn condition_utxos(condition: &Condition) -> BTreeMap<OutPoint, UtxoEntry> {
    let output = Output::new(
        Amount::from_quanta(COND_AMOUNT),
        OutputKind::ConditionHash,
        *condition_hash(condition).as_bytes(),
    )
    .unwrap();
    let mut map = BTreeMap::new();
    map.insert(
        condition_outpoint(),
        UtxoEntry::new(output, BlockHeight::new(10), false),
    );
    map
}

/// Membelanjakan output kondisi dengan `revealed` dan tanda tangan `signers` (indeks kunci, kunci rahasia).
fn condition_spend(
    spent: &Condition,
    revealed: &Condition,
    signers: &[(u8, &SecretKey)],
    network_id: &Hash32,
) -> Transaction {
    let inputs = vec![Input::new(condition_outpoint().txid(), 0)];
    let outputs = vec![out_to(COND_SPEND, key(4))];
    let spent_output = condition_utxos(spent)
        .into_values()
        .next()
        .unwrap()
        .output()
        .clone();
    let dummy = signers
        .iter()
        .map(|(index, _)| SigEntry::new(*index, [0; 64]))
        .collect();
    let unsigned = Transaction::regular(
        inputs.clone(),
        outputs.clone(),
        vec![Witness::condition(revealed.clone(), dummy).unwrap()],
    )
    .unwrap();
    let msg = sighash_v1(network_id, &unsigned, 0, &[spent_output]).unwrap();
    let sigs = signers
        .iter()
        .map(|(index, sk)| {
            SigEntry::new(
                *index,
                *sign(sk, msg.as_bytes(), &[0; 32]).unwrap().as_bytes(),
            )
        })
        .collect();
    Transaction::regular(
        inputs,
        outputs,
        vec![Witness::condition(revealed.clone(), sigs).unwrap()],
    )
    .unwrap()
}

#[test]
fn condition_spend_rules_t4_and_t6() {
    let network_id = params().network_id();
    let (condition, secrets) = condition_setup();
    let utxos = condition_utxos(&condition);
    let first = secrets.first().unwrap();
    let second = secrets.get(1).unwrap();
    let third = secrets.get(2).unwrap();

    let valid = condition_spend(
        &condition,
        &condition,
        &[(0, first), (2, third)],
        &network_id,
    );
    assert_eq!(
        check_transaction(&valid, BlockHeight::new(LOCK), &network_id, &utxos),
        Ok(Amount::from_quanta(COND_FEE))
    );
    assert_eq!(
        check_transaction(
            &valid,
            BlockHeight::new(LOCK_MINUS_ONE),
            &network_id,
            &utxos
        ),
        Err(TxError::ConditionLocked { input_index: 0 })
    );

    let other = Condition::new(2, condition.keys().to_vec(), BlockHeight::GENESIS).unwrap();
    let wrong_condition =
        condition_spend(&condition, &other, &[(0, first), (1, second)], &network_id);
    assert_eq!(
        check_transaction(
            &wrong_condition,
            BlockHeight::new(LOCK),
            &network_id,
            &utxos
        ),
        Err(TxError::ConditionHashMismatch { input_index: 0 })
    );

    let swapped = condition_spend(
        &condition,
        &condition,
        &[(0, first), (2, second)],
        &network_id,
    );
    assert_eq!(
        check_transaction(&swapped, BlockHeight::new(LOCK), &network_id, &utxos),
        Err(TxError::InvalidSignature { input_index: 0 })
    );
}

#[test]
fn coinbase_must_claim_exact_amount_dec17() {
    let params = params();
    let network_id = params.network_id();
    let tx = spend_genesis(SPEND, key(2), &network_id);
    for claimed in [SUBSIDY_100, SUBSIDY_100_PLUS_FEE_PLUS_ONE] {
        let block = block_at(100, claimed, vec![tx.clone()]);
        assert_eq!(
            check_block_transactions(&block, &params, Amount::ZERO, &genesis_utxos()),
            Err(BlockError::CoinbaseAmountMismatch {
                expected: Amount::from_quanta(SUBSIDY_100_PLUS_FEE),
                actual: Amount::from_quanta(claimed),
            })
        );
    }
}

#[test]
fn genesis_and_coinbase_are_not_checked_as_regular() {
    let params = params();
    let genesis = regtest_genesis().unwrap();
    assert_eq!(
        check_block_transactions(&genesis, &params, Amount::ZERO, &no_utxos()),
        Err(BlockError::Genesis)
    );
    assert_eq!(
        check_transaction(
            genesis.txs().first().unwrap(),
            BlockHeight::new(1),
            &params.network_id(),
            &no_utxos(),
        ),
        Err(TxError::NotRegular)
    );
}

proptest! {
    #[test]
    fn prop_coinbase_must_claim_exact_amount(delta in 1_u128..=1_000_000_u128, above in any::<bool>()) {
        let params = params();
        let claimed = if above {
            SUBSIDY_1.checked_add(delta)
        } else {
            SUBSIDY_1.checked_sub(delta)
        }
        .unwrap();
        let block = block_at(1, claimed, Vec::new());
        prop_assert_eq!(
            check_block_transactions(&block, &params, Amount::ZERO, &no_utxos()),
            Err(BlockError::CoinbaseAmountMismatch {
                expected: Amount::from_quanta(SUBSIDY_1),
                actual: Amount::from_quanta(claimed),
            })
        );
    }
}
