use proptest::prelude::*;

use super::{block_id, block_merkle_root, condition_hash, network_id, txid, wtxid};
use zevrigo_primitives::amount::Amount;
use zevrigo_primitives::block::{Block, Header};
use zevrigo_primitives::encoding::Decode;
use zevrigo_primitives::hash::Hash32;
use zevrigo_primitives::tx::{Condition, Input, Output, OutputKind, Transaction, Witness};

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

const V4_HEX: &str = concat!(
    "01000000",
    "0100000000000000",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "2222222222222222222222222222222222222222222222222222222222222222",
    "00d2496b00000000",
    "ffff001d",
    "2a00000000000000"
);

const V5_HEX: &str = concat!(
    "010203",
    "0101010101010101010101010101010101010101010101010101010101010101",
    "0202020202020202020202020202020202020202020202020202020202020202",
    "0303030303030303030303030303030303030303030303030303030303030303",
    "0000000000000000"
);

// H1: txid(V1) di bawah domain "zevrigo 2026-10-28 txid"
// printf '%s' "${V1:0:242}" | xxd -r -p | b3sum --derive-key "zevrigo 2026-10-28 txid" --no-names
const H1_TXID_V1: &str = "9107ff8ed0d2975dd2ee1365b3d0d4ccae8dc52706de61ede65ced3d7992e0bb";

// H2: wtxid(V1) di bawah domain "zevrigo 2026-10-28 wtxid"
// printf '%s' "$V1" | xxd -r -p | b3sum --derive-key "zevrigo 2026-10-28 wtxid" --no-names
const H2_WTXID_V1: &str = "cdd33ed9bf0b56611e3e05ff77c112c8834adf562d49dc9a6f8d647a141d0464";

// H3: txid(V2) di bawah domain "zevrigo 2026-10-28 txid"
// printf '%s' "$V2" | xxd -r -p | b3sum --derive-key "zevrigo 2026-10-28 txid" --no-names
const H3_TXID_V2: &str = "2d70454ac710074a7c01dcfb379fbaa4e949915d8bbd364b5c687e2422c8a04c";

// H4: wtxid(V2) di bawah domain "zevrigo 2026-10-28 wtxid"
// printf '%s' "$V2" | xxd -r -p | b3sum --derive-key "zevrigo 2026-10-28 wtxid" --no-names
const H4_WTXID_V2: &str = "1759737204b9829349fc19c7643949256ae7264be58a42c3c6cf86b5b8850984";

// H5 txid: txid(V3) di bawah domain "zevrigo 2026-10-28 txid"
// printf '%s' "$V3" | xxd -r -p | b3sum --derive-key "zevrigo 2026-10-28 txid" --no-names
const H5_TXID_V3: &str = "584c095f3eb41eb31b9b8103fec8f26a1a2389c485e9c4a400a162373d388bb2";

// H5 wtxid: wtxid(V3) di bawah domain "zevrigo 2026-10-28 wtxid"
// printf '%s' "$V3" | xxd -r -p | b3sum --derive-key "zevrigo 2026-10-28 wtxid" --no-names
const H5_WTXID_V3: &str = "e64573f7995913b02ba1538c2ecd6974b9b1b18907529bfd163222966184f35b";

// H6: block_id(V4) di bawah domain "zevrigo 2026-10-28 block id"
// printf '%s' "$V4" | xxd -r -p | b3sum --derive-key "zevrigo 2026-10-28 block id" --no-names
const H6_BLOCK_ID_V4: &str = "b5e337ac17736bcfa40aaaeab79c8621dede48b263ecf06ae1d5e21423f4108d";

// H7: condition_hash(V5) di bawah domain "zevrigo 2026-10-28 condition"
// printf '%s' "$V5" | xxd -r -p | b3sum --derive-key "zevrigo 2026-10-28 condition" --no-names
const H7_CONDITION_HASH_V5: &str =
    "5f7860bfdc3c469f0a2cc3860669adeaf5fc6fd88146f5e33f2a8a71469cc38e";

// H8: block_merkle_root(V8)
// daun 0 = H(merkle leaf, H4) -> 8a3186fdeb1741efa81a0e91eb52adc7817fa2783be2ef2f268595c32e251a06
// daun 1 = H(merkle leaf, H2) -> cfced85661d1cc9348d2e89555317d03954af1f9fdd236febabf695c654e7459
// root   = H(merkle node, daun0 || daun1)
const H8_BLOCK_MERKLE_ROOT_V8: &str =
    "6f6d1c10ac2546e3e749b2efa00a9df4d1bebdade68149b31e9814e59a0e96ce";

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

fn build_v1() -> Transaction {
    Transaction::from_bytes(&decode_hex(V1_HEX)).expect("V1 valid")
}

fn build_v2() -> Transaction {
    Transaction::from_bytes(&decode_hex(V2_HEX)).expect("V2 valid")
}

fn build_v3() -> Transaction {
    Transaction::from_bytes(&decode_hex(V3_HEX)).expect("V3 valid")
}

fn build_v4() -> Header {
    Header::from_bytes(&decode_hex(V4_HEX)).expect("V4 valid")
}

fn build_v5() -> Condition {
    Condition::from_bytes(&decode_hex(V5_HEX)).expect("V5 valid")
}

fn build_v8() -> Block {
    Block::new(build_v4(), vec![build_v2(), build_v1()]).expect("V8 valid")
}

#[test]
fn vectors_h1_to_h9_match_b3sum_values() {
    let v1 = build_v1();
    let v2 = build_v2();
    let v3 = build_v3();
    let v4 = build_v4();
    let v5 = build_v5();
    let v8 = build_v8();

    // H1: txid(V1)
    let h1 = txid(&v1);
    assert_eq!(h1.to_string(), H1_TXID_V1);

    // H2: wtxid(V1)
    let h2 = wtxid(&v1);
    assert_eq!(h2.to_string(), H2_WTXID_V1);

    // H3: txid(V2)
    let h3 = txid(&v2);
    assert_eq!(h3.to_string(), H3_TXID_V2);

    // H4: wtxid(V2)
    let h4 = wtxid(&v2);
    assert_eq!(h4.to_string(), H4_WTXID_V2);

    // H5: txid(V3) dan wtxid(V3)
    let h5_txid = txid(&v3);
    assert_eq!(h5_txid.to_string(), H5_TXID_V3);
    let h5_wtxid = wtxid(&v3);
    assert_eq!(h5_wtxid.to_string(), H5_WTXID_V3);

    // H6: block_id(V4)
    let h6 = block_id(&v4);
    assert_eq!(h6.to_string(), H6_BLOCK_ID_V4);

    // H7: condition_hash(V5)
    let h7 = condition_hash(&v5);
    assert_eq!(h7.to_string(), H7_CONDITION_HASH_V5);

    // H8: block_merkle_root(V8)
    let h8 = block_merkle_root(&v8).expect("V8 memuat transaksi");
    assert_eq!(h8.to_string(), H8_BLOCK_MERKLE_ROOT_V8);

    // H9: block_id(header V8) sama dengan H6
    let h9 = block_id(v8.header());
    assert_eq!(h9.to_string(), H6_BLOCK_ID_V4);

    // network_id untuk genesis header sama dengan block_id
    assert_eq!(network_id(&v4), h6);
}

#[test]
fn txid_and_wtxid_domain_separation_on_identical_coinbase_bytes() {
    let v2 = build_v2();
    let v2_txid = txid(&v2);
    let v2_wtxid = wtxid(&v2);
    assert_ne!(
        v2_txid, v2_wtxid,
        "txid dan wtxid wajib berbeda meski byte preimagentya identik karena pemisahan domain"
    );
}

#[test]
fn witness_mutation_changes_wtxid_but_preserves_txid() {
    let mut v1_bytes = decode_hex(V1_HEX);
    let orig_v1 = Transaction::from_bytes(&v1_bytes).expect("V1 valid");
    let orig_txid = txid(&orig_v1);
    let orig_wtxid = wtxid(&orig_v1);

    // Ubah satu byte di tanda tangan witness (byte terakhir)
    if let Some(last_sig_byte) = v1_bytes.last_mut() {
        *last_sig_byte ^= 0x01;
    }
    let mutated_v1 = Transaction::from_bytes(&v1_bytes).expect("mutasi witness tetap valid");
    let mutated_txid = txid(&mutated_v1);
    let mutated_wtxid = wtxid(&mutated_v1);

    assert_eq!(
        mutated_txid, orig_txid,
        "txid tidak boleh berubah saat tanda tangan witness dimutasi"
    );
    assert_ne!(
        mutated_wtxid, orig_wtxid,
        "wtxid wajib berubah saat tanda tangan witness dimutasi"
    );
}

#[test]
fn body_mutation_changes_both_txid_and_wtxid() {
    let orig_v1 = build_v1();
    let orig_txid = txid(&orig_v1);
    let orig_wtxid = wtxid(&orig_v1);

    // Ubah prev_index dari 0 menjadi 1 pada input
    let mutated_input = Input::new(Hash32::from_bytes([0x11; 32]), 1);
    let key_bip340 = <[u8; 32]>::try_from(decode_hex(
        "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9",
    ))
    .expect("32 byte");
    let out1 = Output::new(
        Amount::from_quanta(1_500_000_000_000_000_000),
        OutputKind::SingleKey,
        key_bip340,
    )
    .expect("out1");
    let out2 = Output::new(
        Amount::from_quanta(250_000_000_000_000_000),
        OutputKind::SingleKey,
        [0x22; 32],
    )
    .expect("out2");
    let mutated_v1 = Transaction::regular(
        vec![mutated_input],
        vec![out1, out2],
        vec![Witness::single_key([0x33; 64])],
    )
    .expect("mutated v1");

    assert_ne!(txid(&mutated_v1), orig_txid);
    assert_ne!(wtxid(&mutated_v1), orig_wtxid);
}

#[test]
fn network_id_equals_block_id_of_same_header() {
    let header_v4 = build_v4();
    assert_eq!(
        network_id(&header_v4),
        block_id(&header_v4),
        "network_id wajib sama persis dengan block_id dari header yang sama"
    );
}

#[test]
fn merkle_root_changes_when_transactions_are_reordered() {
    let v1 = build_v1();
    let v2 = build_v2();

    let root_order_1 = crate::merkle::merkle_root(&[wtxid(&v2), wtxid(&v1)]);
    let root_order_2 = crate::merkle::merkle_root(&[wtxid(&v1), wtxid(&v2)]);

    assert_ne!(
        root_order_1, root_order_2,
        "urutan transaksi di dalam Merkle root mempengaruhi hasil akhir"
    );
}

proptest! {
    #[test]
    fn proptest_txid_is_independent_of_witness(
        prev_hash in any::<[u8; 32]>(),
        prev_idx in any::<u32>(),
        quanta in 1_u128..=10_000_000_000,
        out_payload in any::<[u8; 32]>(),
        sig1 in any::<[u8; 64]>(),
        sig2 in any::<[u8; 64]>(),
    ) {
        let input = Input::new(Hash32::from_bytes(prev_hash), prev_idx);
        let output = Output::new(Amount::from_quanta(quanta), OutputKind::SingleKey, out_payload).expect("output valid");

        let tx_with_sig1 = Transaction::regular(
            vec![input.clone()],
            vec![output.clone()],
            vec![Witness::single_key(sig1)],
        ).expect("tx1 valid");

        let tx_with_sig2 = Transaction::regular(
            vec![input],
            vec![output],
            vec![Witness::single_key(sig2)],
        ).expect("tx2 valid");

        prop_assert_eq!(txid(&tx_with_sig1), txid(&tx_with_sig2));
        if sig1 != sig2 {
            prop_assert_ne!(wtxid(&tx_with_sig1), wtxid(&tx_with_sig2));
        }
    }

    #[test]
    fn proptest_block_merkle_root_is_some_and_deterministic(
        height in 1_u64..1_000_000,
        quanta_cb in 1_u128..=10_000_000_000,
        miner_payload in any::<[u8; 32]>(),
        quanta_tx in 1_u128..=10_000_000_000,
        prev_hash in any::<[u8; 32]>(),
        prev_idx in any::<u32>(),
        tx_payload in any::<[u8; 32]>(),
        sig in any::<[u8; 64]>(),
    ) {
        let cb_out = Output::new(
            Amount::from_quanta(quanta_cb),
            OutputKind::SingleKey,
            miner_payload,
        ).expect("cb out");
        let cb = Transaction::coinbase(
            zevrigo_primitives::height::BlockHeight::new(height),
            vec![],
            vec![cb_out],
        ).expect("coinbase");

        let tx_in = Input::new(Hash32::from_bytes(prev_hash), prev_idx);
        let tx_out = Output::new(
            Amount::from_quanta(quanta_tx),
            OutputKind::SingleKey,
            tx_payload,
        ).expect("tx out");
        let reg_tx = Transaction::regular(
            vec![tx_in],
            vec![tx_out],
            vec![Witness::single_key(sig)],
        ).expect("reg tx");

        let header = Header::new(
            zevrigo_primitives::height::BlockHeight::new(height),
            Hash32::ZERO,
            Hash32::ZERO,
            1_800_000_000,
            0x1d00_ffff,
            0,
        );

        let block1 = Block::new(header, vec![cb.clone(), reg_tx.clone()]).expect("block1");
        let block2 = Block::new(header, vec![cb.clone(), reg_tx.clone()]).expect("block2");

        let root1 = block_merkle_root(&block1);
        let root2 = block_merkle_root(&block2);

        prop_assert!(root1.is_some());
        prop_assert_eq!(root1, root2);
        prop_assert_eq!(root1, crate::merkle::merkle_root(&[wtxid(&cb), wtxid(&reg_tx)]));
    }
}
