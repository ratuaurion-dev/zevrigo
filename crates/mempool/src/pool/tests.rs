use std::collections::BTreeMap;

use zevrigo_consensus::TxError;
use zevrigo_crypto::{SecretKey, SighashError, sighash_v1, sign, txid};
use zevrigo_primitives::{
    Amount, BlockHeight, Hash32, Input, OutPoint, Output, OutputKind, Transaction, UtxoEntry,
    Witness,
};

use super::{Mempool, MempoolEntry, MempoolError, rule_name};

/// `network_id` contoh untuk sighash.
const NET: Hash32 = Hash32::from_bytes([0x5a; 32]);
/// Tinggi blok berikutnya; set UTXO contoh berada di tinggi 1.
const NEXT: BlockHeight = BlockHeight::new(2);

fn h(byte: u8) -> Hash32 {
    Hash32::from_bytes([byte; 32])
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

/// Output jenis 0 milik kunci rahasia `owner`.
fn out_to(amount: u128, owner: u8) -> Output {
    Output::new(
        Amount::from_quanta(amount),
        OutputKind::SingleKey,
        key(owner),
    )
    .unwrap()
}

/// UTXO biasa milik kunci 1, 10_000 quanta.
fn u1() -> (OutPoint, Output) {
    (OutPoint::new(h(0x01), 0), out_to(10_000, 1))
}

/// UTXO biasa milik kunci 2, 20_000 quanta.
fn u2() -> (OutPoint, Output) {
    (OutPoint::new(h(0x02), 0), out_to(20_000, 2))
}

/// UTXO coinbase tinggi 1 milik kunci 1, 30_000 quanta (matang di tinggi 101).
fn cb() -> (OutPoint, Output) {
    (OutPoint::new(h(0x03), 0), out_to(30_000, 1))
}

fn chain() -> BTreeMap<OutPoint, UtxoEntry> {
    let mut map = BTreeMap::new();
    for (outpoint, output, is_coinbase) in [
        (u1().0, u1().1, false),
        (u2().0, u2().1, false),
        (cb().0, cb().1, true),
    ] {
        map.insert(
            outpoint,
            UtxoEntry::new(output, BlockHeight::new(1), is_coinbase),
        );
    }
    map
}

/// Transaksi jenis 0 satu input yang ditandatangani kunci `signer` atas output `spent`.
fn spend(prev: OutPoint, spent: &Output, signer: u8, outputs: Vec<Output>) -> Transaction {
    let inputs = vec![Input::new(prev.txid(), prev.index())];
    let unsigned = Transaction::regular(
        inputs.clone(),
        outputs.clone(),
        vec![Witness::single_key([0; 64])],
    )
    .unwrap();
    let msg = sighash_v1(&NET, &unsigned, 0, std::slice::from_ref(spent)).unwrap();
    let sig = sign(&secret(signer), msg.as_bytes(), &[0; 32]).unwrap();
    Transaction::regular(inputs, outputs, vec![Witness::single_key(*sig.as_bytes())]).unwrap()
}

fn size(tx: &Transaction) -> usize {
    tx.encoded_size().unwrap()
}

fn pool() -> Mempool {
    Mempool::new(1_000_000)
}

fn chosen(pool: &Mempool, max_bytes: usize) -> Vec<Hash32> {
    pool.select_for_block(max_bytes)
        .into_iter()
        .map(MempoolEntry::txid)
        .collect()
}

#[test]
fn accepts_valid_transaction_and_reports_fee() {
    let (op, out) = u1();
    let tx = spend(op, &out, 1, vec![out_to(9_000, 3)]);
    let id = txid(&tx);
    let mut pool = pool();
    assert_eq!(
        pool.add(tx.clone(), id, NEXT, &NET, &chain()),
        Ok(Amount::from_quanta(1_000))
    );
    assert_eq!(pool.len(), 1);
    assert!(!pool.is_empty());
    assert!(pool.contains(&id));
    assert_eq!(pool.total_bytes(), size(&tx));
    assert_eq!(pool.spender(&op), Some(id));
    assert_eq!(pool.txids(), vec![id]);
    let entry = pool.get(&id).unwrap();
    assert_eq!(entry.tx(), &tx);
    assert_eq!(entry.txid(), id);
    assert_eq!(entry.fee(), Amount::from_quanta(1_000));
    assert_eq!(entry.size(), size(&tx));
    assert_eq!(entry.sequence(), 0);
}

#[test]
fn rejects_duplicate_and_conflicting_spend() {
    let (op, out) = u1();
    let tx = spend(op, &out, 1, vec![out_to(9_000, 3)]);
    let id = txid(&tx);
    let mut pool = pool();
    pool.add(tx.clone(), id, NEXT, &NET, &chain()).unwrap();
    let snapshot = pool.clone();

    let duplicate = pool.add(tx, id, NEXT, &NET, &chain()).unwrap_err();
    assert_eq!(duplicate, MempoolError::Duplicate { txid: id });
    assert_eq!(duplicate.rule(), None);

    let rival = spend(op, &out, 1, vec![out_to(8_000, 4)]);
    let rival_id = txid(&rival);
    let conflict = pool.add(rival, rival_id, NEXT, &NET, &chain()).unwrap_err();
    assert_eq!(
        conflict,
        MempoolError::Conflict {
            outpoint: op,
            existing: id
        }
    );
    assert_eq!(conflict.rule(), Some("T1"));
    assert_eq!(pool, snapshot);
}

#[test]
fn rejects_consensus_invalid_with_rule_names() {
    let mut pool = pool();
    let snapshot = pool.clone();
    let (op, out) = u1();

    let missing_op = OutPoint::new(h(0x99), 0);
    let missing = spend(missing_op, &out, 1, vec![out_to(9_000, 3)]);
    let missing_id = txid(&missing);
    let err = pool
        .add(missing, missing_id, NEXT, &NET, &chain())
        .unwrap_err();
    assert_eq!(
        err,
        MempoolError::Invalid(TxError::MissingUtxo {
            outpoint: missing_op
        })
    );
    assert_eq!(err.rule(), Some("T1"));

    let (cb_op, cb_out) = cb();
    let immature = spend(cb_op, &cb_out, 1, vec![out_to(29_000, 3)]);
    let immature_id = txid(&immature);
    let err = pool
        .add(immature.clone(), immature_id, NEXT, &NET, &chain())
        .unwrap_err();
    assert_eq!(
        err,
        MempoolError::Invalid(TxError::ImmatureCoinbase { outpoint: cb_op })
    );
    assert_eq!(err.rule(), Some("T2"));
    let mut later = Mempool::default();
    assert_eq!(
        later.add(immature, immature_id, BlockHeight::new(101), &NET, &chain()),
        Ok(Amount::from_quanta(1_000))
    );

    let greedy = spend(op, &out, 1, vec![out_to(10_001, 3)]);
    let greedy_id = txid(&greedy);
    let err = pool
        .add(greedy, greedy_id, NEXT, &NET, &chain())
        .unwrap_err();
    assert_eq!(err, MempoolError::Invalid(TxError::OutputsExceedInputs));
    assert_eq!(err.rule(), Some("T5"));

    let forged = spend(op, &out, 2, vec![out_to(9_000, 3)]);
    let forged_id = txid(&forged);
    let err = pool
        .add(forged, forged_id, NEXT, &NET, &chain())
        .unwrap_err();
    assert_eq!(
        err,
        MempoolError::Invalid(TxError::InvalidSignature { input_index: 0 })
    );
    assert_eq!(err.rule(), Some("T6"));

    let coinbase =
        Transaction::coinbase(BlockHeight::new(2), Vec::new(), vec![out_to(5, 3)]).unwrap();
    let coinbase_id = txid(&coinbase);
    let err = pool
        .add(coinbase, coinbase_id, NEXT, &NET, &chain())
        .unwrap_err();
    assert_eq!(err, MempoolError::Invalid(TxError::NotRegular));
    assert_eq!(err.rule(), Some("struktur"));

    assert_eq!(pool, snapshot);
}

#[test]
fn chained_spend_and_block_selection() {
    let (op1, out1) = u1();
    let (op2, out2) = u2();
    let a = spend(op1, &out1, 1, vec![out_to(4_000, 3), out_to(5_000, 4)]);
    let a_id = txid(&a);
    let b = spend(
        OutPoint::new(a_id, 0),
        &out_to(4_000, 3),
        3,
        vec![out_to(3_000, 5)],
    );
    let b_id = txid(&b);
    let c = spend(op2, &out2, 2, vec![out_to(19_000, 6)]);
    let c_id = txid(&c);

    let mut pool = pool();
    let fee = Ok(Amount::from_quanta(1_000));
    assert_eq!(pool.add(a.clone(), a_id, NEXT, &NET, &chain()), fee);
    assert_eq!(pool.add(b.clone(), b_id, NEXT, &NET, &chain()), fee);
    assert_eq!(pool.add(c.clone(), c_id, NEXT, &NET, &chain()), fee);
    assert_eq!(pool.spender(&OutPoint::new(a_id, 0)), Some(b_id));
    assert_eq!(pool.spender(&OutPoint::new(a_id, 1)), None);

    let arrival: Vec<Hash32> = pool.in_arrival_order().map(MempoolEntry::txid).collect();
    assert_eq!(arrival, vec![a_id, b_id, c_id]);
    let mut sorted = vec![a_id, b_id, c_id];
    sorted.sort();
    assert_eq!(pool.txids(), sorted);
    assert_eq!(pool.total_bytes(), size(&a) + size(&b) + size(&c));

    assert_eq!(chosen(&pool, usize::MAX), vec![a_id, b_id, c_id]);
    let small = size(&b).max(size(&c));
    assert!(size(&a) > small);
    assert_eq!(chosen(&pool, small), vec![c_id]);
    assert_eq!(chosen(&pool, size(&a)), vec![a_id]);
    assert_eq!(chosen(&pool, 0), Vec::<Hash32>::new());
}

#[test]
fn full_and_too_large_are_rejected() {
    let (op, out) = u1();
    let tx = spend(op, &out, 1, vec![out_to(9_000, 3)]);
    let id = txid(&tx);
    let need = size(&tx);

    let mut tight = Mempool::new(need - 1);
    let err = tight.add(tx.clone(), id, NEXT, &NET, &chain()).unwrap_err();
    assert_eq!(
        err,
        MempoolError::Full {
            size: need,
            used: 0,
            limit: need - 1
        }
    );
    assert_eq!(err.rule(), None);
    assert!(tight.is_empty());

    let mut exact = Mempool::new(need);
    assert!(exact.add(tx, id, NEXT, &NET, &chain()).is_ok());
    assert_eq!(exact.total_bytes(), need);

    let one = out_to(1, 3);
    let huge = Transaction::regular(
        vec![Input::new(h(0x01), 0)],
        vec![one; 3_000],
        vec![Witness::single_key([0; 64])],
    )
    .unwrap();
    let huge_id = txid(&huge);
    let huge_size = size(&huge);
    assert!(huge_size > 100_000);
    let mut pool = pool();
    let err = pool.add(huge, huge_id, NEXT, &NET, &chain()).unwrap_err();
    assert_eq!(err, MempoolError::TooLarge { size: huge_size });
    assert_eq!(err.rule(), None);
}

#[test]
fn revalidate_after_block_and_reorg() {
    let (op1, out1) = u1();
    let (op2, out2) = u2();
    let a = spend(op1, &out1, 1, vec![out_to(9_000, 3)]);
    let a_id = txid(&a);
    let b = spend(
        OutPoint::new(a_id, 0),
        &out_to(9_000, 3),
        3,
        vec![out_to(8_000, 4)],
    );
    let b_id = txid(&b);
    let c = spend(op2, &out2, 2, vec![out_to(19_000, 5)]);
    let c_id = txid(&c);
    let mut pool = pool();
    pool.add(a, a_id, NEXT, &NET, &chain()).unwrap();
    pool.add(b.clone(), b_id, NEXT, &NET, &chain()).unwrap();
    pool.add(c, c_id, NEXT, &NET, &chain()).unwrap();

    // Ujung chain tidak berubah (misalnya reorg kembali ke ujung yang sama): tidak ada yang terbuang.
    assert_eq!(pool.revalidate(NEXT, &NET, &chain()), Vec::<Hash32>::new());
    let arrival: Vec<Hash32> = pool.in_arrival_order().map(MempoolEntry::txid).collect();
    assert_eq!(arrival, vec![a_id, b_id, c_id]);

    // Blok tinggi 2 memuat A dan D, transaksi lain yang membelanjakan u2 (bentrok dengan C).
    let d_id = txid(&spend(op2, &out2, 2, vec![out_to(18_000, 6)]));
    let mut after = chain();
    after.remove(&op1);
    after.remove(&op2);
    after.insert(
        OutPoint::new(a_id, 0),
        UtxoEntry::new(out_to(9_000, 3), BlockHeight::new(2), false),
    );
    after.insert(
        OutPoint::new(d_id, 0),
        UtxoEntry::new(out_to(18_000, 6), BlockHeight::new(2), false),
    );
    assert_eq!(
        pool.revalidate(BlockHeight::new(3), &NET, &after),
        vec![a_id, c_id]
    );
    assert_eq!(pool.txids(), vec![b_id]);
    assert_eq!(pool.spender(&OutPoint::new(a_id, 0)), Some(b_id));
    assert_eq!(pool.spender(&op2), None);
    assert_eq!(pool.total_bytes(), size(&b));
    assert_eq!(
        pool.get(&b_id).map(MempoolEntry::fee),
        Some(Amount::from_quanta(1_000))
    );
}

#[test]
fn rule_name_covers_all_tx_errors() {
    let op = OutPoint::new(h(0x01), 0);
    let cases = [
        (TxError::NotRegular, "struktur"),
        (TxError::WitnessCountMismatch, "struktur"),
        (TxError::IndexOverflow, "struktur"),
        (TxError::DuplicateInput { outpoint: op }, "T1"),
        (TxError::MissingUtxo { outpoint: op }, "T1"),
        (TxError::ImmatureCoinbase { outpoint: op }, "T2"),
        (TxError::WitnessKindMismatch { input_index: 0 }, "T3"),
        (TxError::ConditionHashMismatch { input_index: 0 }, "T4"),
        (TxError::ConditionLocked { input_index: 0 }, "T4"),
        (TxError::AmountOverflow, "T5"),
        (TxError::OutputsExceedInputs, "T5"),
        (TxError::Sighash(SighashError::Coinbase), "T6"),
        (TxError::InvalidSignature { input_index: 0 }, "T6"),
    ];
    for (error, name) in cases {
        assert_eq!(rule_name(&error), name);
        assert_eq!(MempoolError::Invalid(error).rule(), Some(name));
    }
}
