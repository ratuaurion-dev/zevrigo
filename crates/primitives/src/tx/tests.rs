use std::collections::BTreeSet;

use proptest::prelude::*;

use super::{
    CoinbaseTxBody, Condition, Input, MAX_COINBASE_EXTRA_LEN, Output, OutputKind, PUBKEY_XONLY_LEN,
    RegularTxBody, SIGNATURE_LEN, SigEntry, Transaction, TxBody, Witness,
};
use crate::amount::{Amount, QUANTA_PER_ZVRG};
use crate::encoding::{Decode, DecodeError, Encode, MIN_KEY_SIZE, MIN_TX_SIZE};
use crate::hash::Hash32;
use crate::height::BlockHeight;

const V1_HEX: &str = concat!(
    "010001111111111111111111111111111111111111111111111111111111111111111100",
    "028080d8d8d7c1c4e81400",
    "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9",
    "8080e4ceceb58bbc0300",
    "2222222222222222222222222222222222222222222222222222222222222222",
    "00",
    "3333333333333333333333333333333333333333333333333333333333333333",
    "3333333333333333333333333333333333333333333333333333333333333333"
);

const V2_HEX: &str = concat!(
    "010101000000000000000001d58bf09583a8b20300",
    "4444444444444444444444444444444444444444444444444444444444444444"
);

const V3_HEX: &str = concat!(
    "0101000000000000000020",
    "abababababababababababababababababababababababababababababababab",
    "01808080cccfadc9ccc680fe1c00",
    "5555555555555555555555555555555555555555555555555555555555555555"
);

const V5_HEX: &str = concat!(
    "010203",
    "0101010101010101010101010101010101010101010101010101010101010101",
    "0202020202020202020202020202020202020202020202020202020202020202",
    "0303030303030303030303030303030303030303030303030303030303030303",
    "0000000000000000"
);

const V7_BODY_HEX: &str = concat!(
    "010001999999999999999999999999999999999999999999999999999999999999999905",
    "01808090bbbad6adf00d01",
    "7777777777777777777777777777777777777777777777777777777777777777"
);

const R11_HEX: &str = concat!(
    "01000001808090bbbad6adf00d00",
    "2222222222222222222222222222222222222222222222222222222222222222"
);

fn decode_hex(hex: &str) -> Vec<u8> {
    let (pairs, rem) = hex.as_bytes().as_chunks::<2>();
    assert!(rem.is_empty(), "panjang string hex harus genap");
    let mut out = Vec::with_capacity(pairs.len());
    for &[hi, lo] in pairs {
        let hi_val = char::from(hi).to_digit(16).expect("hex nibble valid");
        let lo_val = char::from(lo).to_digit(16).expect("hex nibble valid");
        let byte = u8::try_from(hi_val.wrapping_shl(4) | lo_val).expect("byte muat u8");
        out.push(byte);
    }
    out
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::new();
    for &b in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

fn build_v1() -> Transaction {
    let key_bip340 = <[u8; 32]>::try_from(decode_hex(
        "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9",
    ))
    .expect("32 byte");

    let input = Input::new(Hash32::from_bytes([0x11; 32]), 0);
    let out1 = Output::new(
        Amount::from_quanta(1_500_000_000_000_000_000),
        OutputKind::SingleKey,
        key_bip340,
    )
    .expect("output 1 valid");
    let out2 = Output::new(
        Amount::from_quanta(250_000_000_000_000_000),
        OutputKind::SingleKey,
        [0x22; 32],
    )
    .expect("output 2 valid");
    let witness = Witness::single_key([0x33; 64]);

    Transaction::regular(vec![input], vec![out1, out2], vec![witness]).expect("V1 valid")
}

fn build_v5() -> Condition {
    Condition::new(
        2,
        vec![[0x01; 32], [0x02; 32], [0x03; 32]],
        BlockHeight::GENESIS,
    )
    .expect("V5 valid")
}

fn build_v6() -> Witness {
    Witness::condition(
        build_v5(),
        vec![SigEntry::new(0, [0x44; 64]), SigEntry::new(2, [0x66; 64])],
    )
    .expect("V6 valid")
}

#[test]
fn regular_tx_body_constructor_validations() {
    assert_eq!(
        RegularTxBody::new(Vec::new(), Vec::new()),
        Err(DecodeError::EmptyInputs)
    );
    let valid_input = Input::new(Hash32::ZERO, 0);
    assert_eq!(
        RegularTxBody::new(vec![valid_input], Vec::new()),
        Err(DecodeError::EmptyOutputs)
    );
}

#[test]
fn minimum_coinbase_transaction_size_is_46_bytes() {
    let min_out = Output::new(Amount::from_quanta(1), OutputKind::SingleKey, [0_u8; 32])
        .expect("output 1 quanta valid");
    let min_cb = Transaction::coinbase(BlockHeight::new(1), Vec::new(), vec![min_out])
        .expect("coinbase terkecil valid");
    assert_eq!(min_cb.to_bytes().len(), MIN_TX_SIZE);
    assert_eq!(min_cb.encoded_size(), Some(MIN_TX_SIZE));
}

#[test]
fn vector_v1_and_v1b_regular_transaction() {
    let tx = build_v1();
    let encoded = tx.to_bytes();
    assert_eq!(encoded.len(), 186);
    assert_eq!(tx.encoded_size(), Some(186));
    assert_eq!(encode_hex(&encoded), V1_HEX);

    let decoded = Transaction::from_bytes(&encoded).expect("decode V1 berhasil");
    assert_eq!(decoded, tx);

    // V1b: 121 byte pertama V1 (encoding badan saja)
    let body_bytes = tx.body_to_bytes();
    assert_eq!(body_bytes.len(), 121);
    assert_eq!(tx.body().encoded_size(), Some(121));
    let (v1_first_121, _) = encoded.split_at_checked(121).expect("V1 >= 121 byte");
    assert_eq!(body_bytes.as_slice(), v1_first_121);
    assert_eq!(TxBody::from_bytes(&body_bytes), Ok(tx.body().clone()));
}

#[test]
fn vector_v2_coinbase_height_1() {
    let out = Output::new(
        Amount::from_quanta(1_910_127_426_536_917),
        OutputKind::SingleKey,
        [0x44; 32],
    )
    .expect("output V2 valid");
    let tx = Transaction::coinbase(BlockHeight::new(1), Vec::new(), vec![out]).expect("V2 valid");

    let encoded = tx.to_bytes();
    assert_eq!(encoded.len(), 53);
    assert_eq!(tx.encoded_size(), Some(53));
    assert_eq!(encode_hex(&encoded), V2_HEX);
    assert_eq!(tx.body_to_bytes(), encoded);
    assert_eq!(Transaction::from_bytes(&encoded), Ok(tx));
}

#[test]
fn vector_v3_genesis_coinbase() {
    let genesis_amount = 4_380_000_u128
        .checked_mul(QUANTA_PER_ZVRG)
        .expect("tidak overflow");
    let out = Output::new(
        Amount::from_quanta(genesis_amount),
        OutputKind::SingleKey,
        [0x55; 32],
    )
    .expect("output V3 valid");
    let tx =
        Transaction::coinbase(BlockHeight::GENESIS, vec![0xab; 32], vec![out]).expect("V3 valid");

    let encoded = tx.to_bytes();
    assert_eq!(encoded.len(), 89);
    assert_eq!(tx.encoded_size(), Some(89));
    assert_eq!(encode_hex(&encoded), V3_HEX);
    assert_eq!(Transaction::from_bytes(&encoded), Ok(tx));
}

#[test]
fn vector_v5_v6_v7_condition_witness_and_transaction() {
    // V5: Condition 2-dari-3
    let cond = build_v5();
    let v5_bytes = cond.to_bytes();
    assert_eq!(v5_bytes.len(), 107);
    assert_eq!(cond.encoded_size(), 107);
    assert_eq!(encode_hex(&v5_bytes), V5_HEX);
    assert_eq!(Condition::from_bytes(&v5_bytes), Ok(cond));

    // V6: Witness jenis 1 (01 + V5 + 02 + 00 + 44*64 + 02 + 66*64)
    let witness = build_v6();
    let v6_bytes = witness.to_bytes();
    assert_eq!(v6_bytes.len(), 239);
    assert_eq!(witness.encoded_size(), 239);

    let mut expected_v6 = vec![0x01];
    expected_v6.extend_from_slice(&v5_bytes);
    expected_v6.push(0x02);
    expected_v6.push(0x00);
    expected_v6.extend_from_slice(&[0x44; 64]);
    expected_v6.push(0x02);
    expected_v6.extend_from_slice(&[0x66; 64]);
    assert_eq!(v6_bytes, expected_v6);
    assert_eq!(Witness::from_bytes(&v6_bytes), Ok(witness.clone()));

    // V7: Transaksi biasa yang membelanjakan output jenis 1 memakai witness V6
    let input = Input::new(Hash32::from_bytes([0x99; 32]), 5);
    let output = Output::new(
        Amount::from_quanta(QUANTA_PER_ZVRG),
        OutputKind::ConditionHash,
        [0x77; 32],
    )
    .expect("output V7 valid");
    let tx_v7 = Transaction::regular(vec![input], vec![output], vec![witness]).expect("V7 valid");

    let v7_bytes = tx_v7.to_bytes();
    assert_eq!(v7_bytes.len(), 318);
    assert_eq!(tx_v7.encoded_size(), Some(318));

    let mut expected_v7 = decode_hex(V7_BODY_HEX);
    expected_v7.extend_from_slice(&v6_bytes);
    assert_eq!(v7_bytes, expected_v7);
    assert_eq!(Transaction::from_bytes(&v7_bytes), Ok(tx_v7));
}

#[test]
fn rejection_vectors_r6_to_r14_and_r17() {
    let v1_bytes = decode_hex(V1_HEX);
    let v2_bytes = decode_hex(V2_HEX);

    // R6: V1 + 00 -> sisa byte
    let mut r6 = v1_bytes.clone();
    r6.push(0x00);
    assert_eq!(
        Transaction::from_bytes(&r6),
        Err(DecodeError::TrailingBytes { remaining: 1 })
    );

    // R7: V1 tanpa byte terakhir -> data terpotong
    let (r7, _) = v1_bytes
        .split_at_checked(v1_bytes.len().saturating_sub(1))
        .expect("V1 tidak kosong");
    assert_eq!(Transaction::from_bytes(r7), Err(DecodeError::UnexpectedEof));

    // R8: V1 dengan byte pertama 02 -> versi tidak dikenal
    let mut r8 = v1_bytes.clone();
    if let Some(first) = r8.first_mut() {
        *first = 0x02;
    }
    assert_eq!(
        Transaction::from_bytes(&r8),
        Err(DecodeError::UnsupportedVersion { actual: 2 })
    );

    // R9: V1 dengan byte kedua 02 -> tx_kind tidak dikenal
    let mut r9 = v1_bytes;
    if let Some(second) = r9.get_mut(1) {
        *second = 0x02;
    }
    assert_eq!(
        Transaction::from_bytes(&r9),
        Err(DecodeError::UnknownTxKind { actual: 2 })
    );

    // R10: 0100ffffffff0f (klaim 2^32 - 1 input) -> jumlah elemen mustahil, tanpa alokasi
    let r10 = decode_hex("0100ffffffff0f");
    assert_eq!(
        Transaction::from_bytes(&r10),
        Err(DecodeError::ImpossibleElementCount {
            count: u64::from(u32::MAX),
            remaining: 0,
            min_elem_size: 33,
        })
    );

    // R11: 0 input -> input kosong
    let r11 = decode_hex(R11_HEX);
    assert_eq!(Transaction::from_bytes(&r11), Err(DecodeError::EmptyInputs));

    // R12: V2 dengan amount `00` -> amount nol
    // Pada V2: offset 0..12 = `0101` + height(8) + extra_len(00) + out_count(01);
    // offset 12..20 = amount (8 byte), offset 20..53 = kind(00) + payload(44*32).
    let (v2_prefix, _) = v2_bytes.split_at_checked(12).expect("prefix V2");
    let (_, v2_suffix) = v2_bytes.split_at_checked(20).expect("suffix V2");
    let mut r12 = v2_prefix.to_vec();
    r12.push(0x00);
    r12.extend_from_slice(v2_suffix);
    assert_eq!(
        Transaction::from_bytes(&r12),
        Err(DecodeError::ZeroOutputAmount)
    );
    assert_eq!(
        Output::new(Amount::ZERO, OutputKind::SingleKey, [0x44; 32]),
        Err(DecodeError::ZeroOutputAmount)
    );

    // R13: V2 dengan kind output `02` -> jenis output tidak dikenal
    let mut r13 = v2_bytes;
    if let Some(kind_byte) = r13.get_mut(20) {
        *kind_byte = 0x02;
    }
    assert_eq!(
        Transaction::from_bytes(&r13),
        Err(DecodeError::UnknownOutputKind { actual: 2 })
    );

    // R14: Coinbase dengan extra 33 byte -> extra terlalu panjang
    let valid_out = Output::new(Amount::from_quanta(1), OutputKind::SingleKey, [0x44; 32])
        .expect("output valid");
    assert_eq!(
        CoinbaseTxBody::new(
            BlockHeight::new(1),
            vec![0xaa; MAX_COINBASE_EXTRA_LEN.saturating_add(1)],
            vec![valid_out.clone()],
        ),
        Err(DecodeError::CoinbaseExtraTooLong { len: 33 })
    );
    let mut r14 = decode_hex("01010100000000000000");
    r14.push(33); // extra_len = 33
    r14.extend_from_slice(&[0xaa; 33]);
    r14.push(1); // 1 output
    valid_out.encode_to(&mut r14);
    assert_eq!(
        Transaction::from_bytes(&r14),
        Err(DecodeError::CoinbaseExtraTooLong { len: 33 })
    );

    // R17: V7 dengan byte jenis witness `02` -> jenis witness tidak dikenal
    let mut r17 = decode_hex(V7_BODY_HEX);
    let v6_bytes = build_v6().to_bytes();
    let witness_offset = r17.len();
    r17.extend_from_slice(&v6_bytes);
    if let Some(w_kind) = r17.get_mut(witness_offset) {
        *w_kind = 0x02;
    }
    assert_eq!(
        Transaction::from_bytes(&r17),
        Err(DecodeError::UnknownWitnessKind { actual: 2 })
    );
}

#[test]
fn rejection_vector_r15_invalid_conditions_and_n17_with_full_keys() {
    let v5_bytes = decode_hex(V5_HEX);

    // 1. Kunci 01 dan 02 ditukar
    let mut swapped = Vec::new();
    swapped.extend_from_slice(&[0x01, 0x02, 0x03]);
    swapped.extend_from_slice(&[0x02; 32]);
    swapped.extend_from_slice(&[0x01; 32]);
    swapped.extend_from_slice(&[0x03; 32]);
    swapped.extend_from_slice(&0_u64.to_le_bytes());
    assert_eq!(
        Condition::from_bytes(&swapped),
        Err(DecodeError::InvalidCondition)
    );
    assert_eq!(
        Condition::new(
            2,
            vec![[0x02; 32], [0x01; 32], [0x03; 32]],
            BlockHeight::GENESIS
        ),
        Err(DecodeError::InvalidCondition)
    );

    // 2. m = 0
    let mut m_zero = v5_bytes.clone();
    if let Some(m_byte) = m_zero.get_mut(1) {
        *m_byte = 0;
    }
    assert_eq!(
        Condition::from_bytes(&m_zero),
        Err(DecodeError::InvalidCondition)
    );

    // 3. m = 4 (lebih besar dari n = 3)
    let mut m_four = v5_bytes;
    if let Some(m_byte) = m_four.get_mut(1) {
        *m_byte = 4;
    }
    assert_eq!(
        Condition::from_bytes(&m_four),
        Err(DecodeError::InvalidCondition)
    );

    // 4. n = 17 dengan 17 kunci nyata (17 * 32 byte urut naik) + lock_height (8 byte)
    //    Lolos pelindung alokasi (552 / 32 = 17 >= 17), lalu ditolak sebagai InvalidCondition.
    let mut seventeen_keys = Vec::with_capacity(17);
    let mut n17_bytes = vec![0x01, 0x02, 17];
    for idx in 1_u8..=17_u8 {
        let key = [idx; 32];
        n17_bytes.extend_from_slice(&key);
        seventeen_keys.push(key);
    }
    n17_bytes.extend_from_slice(&0_u64.to_le_bytes());
    assert_eq!(
        Condition::from_bytes(&n17_bytes),
        Err(DecodeError::InvalidCondition)
    );
    assert_eq!(
        Condition::new(2, seventeen_keys, BlockHeight::GENESIS),
        Err(DecodeError::InvalidCondition)
    );

    // 5. Kunci kembar
    let mut dup_bytes = vec![0x01, 0x02, 0x03];
    dup_bytes.extend_from_slice(&[0x01; 32]);
    dup_bytes.extend_from_slice(&[0x01; 32]);
    dup_bytes.extend_from_slice(&[0x03; 32]);
    dup_bytes.extend_from_slice(&0_u64.to_le_bytes());
    assert_eq!(
        Condition::from_bytes(&dup_bytes),
        Err(DecodeError::InvalidCondition)
    );
}

#[test]
fn condition_claim_n17_without_sufficient_bytes_triggers_allocation_guard() {
    // Klaim n = 17 tetapi hanya menyediakan byte dari V5 (3 kunci + lock_height = 104 byte < 544 byte).
    // Pelindung alokasi wajib berjalan sebelum pemeriksaan batas n <= 16.
    let mut claimed_n17_short = decode_hex(V5_HEX);
    if let Some(n_byte) = claimed_n17_short.get_mut(2) {
        *n_byte = 17;
    }
    assert_eq!(
        Condition::from_bytes(&claimed_n17_short),
        Err(DecodeError::ImpossibleElementCount {
            count: 17,
            remaining: 104,
            min_elem_size: MIN_KEY_SIZE,
        })
    );
}

#[test]
fn rejection_vector_r16_invalid_witness_signatures() {
    let v5_bytes = decode_hex(V5_HEX);

    // 1. Urutan indeks 2 lalu 0
    let mut wrong_order = vec![0x01];
    wrong_order.extend_from_slice(&v5_bytes);
    wrong_order.push(0x02);
    wrong_order.push(0x02);
    wrong_order.extend_from_slice(&[0x66; 64]);
    wrong_order.push(0x00);
    wrong_order.extend_from_slice(&[0x44; 64]);
    assert_eq!(
        Witness::from_bytes(&wrong_order),
        Err(DecodeError::InvalidWitnessSignatures)
    );

    // 2. Hanya 1 SigEntry untuk kondisi m = 2
    let mut one_sig = vec![0x01];
    one_sig.extend_from_slice(&v5_bytes);
    one_sig.push(0x01);
    one_sig.push(0x00);
    one_sig.extend_from_slice(&[0x44; 64]);
    assert_eq!(
        Witness::from_bytes(&one_sig),
        Err(DecodeError::InvalidWitnessSignatures)
    );

    // 3. Indeks 3 (melampaui n = 3, indeks sah hanya 0..=2)
    let mut out_of_bounds_idx = vec![0x01];
    out_of_bounds_idx.extend_from_slice(&v5_bytes);
    out_of_bounds_idx.push(0x02);
    out_of_bounds_idx.push(0x00);
    out_of_bounds_idx.extend_from_slice(&[0x44; 64]);
    out_of_bounds_idx.push(0x03);
    out_of_bounds_idx.extend_from_slice(&[0x66; 64]);
    assert_eq!(
        Witness::from_bytes(&out_of_bounds_idx),
        Err(DecodeError::InvalidWitnessSignatures)
    );
}

// --- Strategi generator proptest ---

pub(crate) fn arb_output() -> impl Strategy<Value = Output> {
    (
        1_u128..=u128::MAX,
        prop_oneof![Just(OutputKind::SingleKey), Just(OutputKind::ConditionHash)],
        any::<[u8; PUBKEY_XONLY_LEN]>(),
    )
        .prop_map(|(quanta, kind, payload)| {
            Output::new(Amount::from_quanta(quanta), kind, payload).expect("quanta >= 1 valid")
        })
}

pub(crate) fn arb_condition() -> impl Strategy<Value = Condition> {
    (
        proptest::collection::btree_set(any::<[u8; PUBKEY_XONLY_LEN]>(), 1..=16),
        any::<u64>(),
        any::<prop::sample::Index>(),
    )
        .prop_map(|(key_set, lock_h, m_idx)| {
            let keys: Vec<[u8; PUBKEY_XONLY_LEN]> = key_set.into_iter().collect();
            let m_zero_based = m_idx.index(keys.len());
            let m = u8::try_from(m_zero_based.saturating_add(1)).expect("1..=16 muat u8");
            Condition::new(m, keys, BlockHeight::new(lock_h)).expect("kondisi terurut valid")
        })
}

pub(crate) fn arb_input() -> impl Strategy<Value = Input> {
    (any::<[u8; 32]>(), any::<u32>())
        .prop_map(|(hash, idx)| Input::new(Hash32::from_bytes(hash), idx))
}

pub(crate) fn arb_witness() -> impl Strategy<Value = Witness> {
    let single = any::<[u8; SIGNATURE_LEN]>().prop_map(Witness::single_key);
    let cond_wit = (
        arb_condition(),
        proptest::collection::vec(
            (any::<prop::sample::Index>(), any::<[u8; SIGNATURE_LEN]>()),
            16,
        ),
    )
        .prop_map(|(cond, raw_pool)| {
            let m_usize = usize::from(cond.m());
            let n_usize = usize::from(cond.n());
            // Pilih tepat `m` indeks unik dari `0..n` secara deterministik
            let mut chosen = BTreeSet::new();
            for (idx_selector, _) in &raw_pool {
                if chosen.len() == m_usize {
                    break;
                }
                let candidate = idx_selector.index(n_usize);
                chosen.insert(candidate);
            }
            for fallback in 0..n_usize {
                if chosen.len() == m_usize {
                    break;
                }
                chosen.insert(fallback);
            }
            let sigs: Vec<SigEntry> = chosen
                .into_iter()
                .zip(raw_pool)
                .map(|(k_idx, (_, sig_bytes))| {
                    let key_index = u8::try_from(k_idx).expect("k_idx < 16");
                    SigEntry::new(key_index, sig_bytes)
                })
                .collect();
            Witness::condition(cond, sigs).expect("witness kondisi valid")
        });
    prop_oneof![single, cond_wit]
}

pub(crate) fn arb_regular_transaction() -> impl Strategy<Value = Transaction> {
    (
        proptest::collection::vec((arb_input(), arb_witness()), 1..=4),
        proptest::collection::vec(arb_output(), 1..=4),
    )
        .prop_map(|(in_wit, outputs)| {
            let (inputs, witnesses): (Vec<Input>, Vec<Witness>) = in_wit.into_iter().unzip();
            Transaction::regular(inputs, outputs, witnesses).expect("tx biasa valid")
        })
}

pub(crate) fn arb_coinbase_transaction() -> impl Strategy<Value = Transaction> {
    (
        any::<u64>(),
        proptest::collection::vec(any::<u8>(), 0..=MAX_COINBASE_EXTRA_LEN),
        proptest::collection::vec(arb_output(), 1..=4),
    )
        .prop_map(|(h, extra, outputs)| {
            Transaction::coinbase(BlockHeight::new(h), extra, outputs).expect("coinbase valid")
        })
}

pub(crate) fn arb_transaction() -> impl Strategy<Value = Transaction> {
    prop_oneof![arb_regular_transaction(), arb_coinbase_transaction()]
}

fn assert_all_strict_prefixes_rejected<T: Decode>(bytes: &[u8]) {
    for prefix_len in 0..bytes.len() {
        if let Some((prefix, _)) = bytes.split_at_checked(prefix_len) {
            assert!(
                T::from_bytes(prefix).is_err(),
                "prefiks sepanjang {prefix_len} dari {} seharusnya ditolak",
                bytes.len()
            );
        }
    }
}

proptest! {
    #[test]
    fn proptest_output_roundtrip_and_prefixes(out in arb_output()) {
        let bytes = out.to_bytes();
        prop_assert_eq!(bytes.len(), out.encoded_size());
        prop_assert_eq!(Output::from_bytes(&bytes), Ok(out));
        assert_all_strict_prefixes_rejected::<Output>(&bytes);
    }

    #[test]
    fn proptest_condition_roundtrip_and_prefixes(cond in arb_condition()) {
        let bytes = cond.to_bytes();
        prop_assert_eq!(bytes.len(), cond.encoded_size());
        prop_assert_eq!(Condition::from_bytes(&bytes), Ok(cond));
        assert_all_strict_prefixes_rejected::<Condition>(&bytes);
    }

    #[test]
    fn proptest_input_roundtrip_and_prefixes(inp in arb_input()) {
        let bytes = inp.to_bytes();
        prop_assert_eq!(bytes.len(), inp.encoded_size());
        prop_assert_eq!(Input::from_bytes(&bytes), Ok(inp));
        assert_all_strict_prefixes_rejected::<Input>(&bytes);
    }

    #[test]
    fn proptest_witness_roundtrip_and_prefixes(wit in arb_witness()) {
        let bytes = wit.to_bytes();
        prop_assert_eq!(bytes.len(), wit.encoded_size());
        prop_assert_eq!(Witness::from_bytes(&bytes), Ok(wit));
        assert_all_strict_prefixes_rejected::<Witness>(&bytes);
    }

    #[test]
    fn proptest_transaction_and_body_roundtrip_and_prefixes(tx in arb_transaction()) {
        let full_bytes = tx.to_bytes();
        prop_assert_eq!(Some(full_bytes.len()), tx.encoded_size());
        prop_assert_eq!(Transaction::from_bytes(&full_bytes), Ok(tx.clone()));
        assert_all_strict_prefixes_rejected::<Transaction>(&full_bytes);

        let body_bytes = tx.body_to_bytes();
        prop_assert_eq!(Some(body_bytes.len()), tx.body().encoded_size());
        prop_assert_eq!(TxBody::from_bytes(&body_bytes), Ok(tx.body().clone()));
        assert_all_strict_prefixes_rejected::<TxBody>(&body_bytes);
    }

    #[test]
    fn proptest_tx_types_arbitrary_bytes_up_to_2048_canonical_and_no_panic(
        bytes in proptest::collection::vec(any::<u8>(), 0..=2048)
    ) {
        if let Ok(val) = Output::from_bytes(&bytes) {
            prop_assert_eq!(val.to_bytes(), bytes.clone());
        }
        if let Ok(val) = Condition::from_bytes(&bytes) {
            prop_assert_eq!(val.to_bytes(), bytes.clone());
        }
        if let Ok(val) = Input::from_bytes(&bytes) {
            prop_assert_eq!(val.to_bytes(), bytes.clone());
        }
        if let Ok(val) = SigEntry::from_bytes(&bytes) {
            prop_assert_eq!(val.to_bytes(), bytes.clone());
        }
        if let Ok(val) = Witness::from_bytes(&bytes) {
            prop_assert_eq!(val.to_bytes(), bytes.clone());
        }
        if let Ok(val) = TxBody::from_bytes(&bytes) {
            prop_assert_eq!(val.to_bytes(), bytes.clone());
        }
        if let Ok(val) = Transaction::from_bytes(&bytes) {
            prop_assert_eq!(val.to_bytes(), bytes);
        }
    }
}
