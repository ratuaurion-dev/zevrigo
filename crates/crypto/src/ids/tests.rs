use proptest::prelude::*;

use super::{block_id, block_merkle_root, condition_hash, network_id, txid, wtxid};
use homeostasis_primitives::amount::Amount;
use homeostasis_primitives::block::{Block, Header};
use homeostasis_primitives::encoding::Decode;
use homeostasis_primitives::hash::Hash32;
use homeostasis_primitives::tx::{Condition, Input, Output, OutputKind, Transaction, Witness};

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

// H1: txid(V1) di bawah domain "homeostasis 2026-10-28 txid"
// printf '%s' "${V1:0:242}" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 txid" --no-names
const H1_TXID_V1: &str = "5b403b217e76db7c9a6ac42da2addef7072e1037345091542114a09fed8316f0";

// H2: wtxid(V1) di bawah domain "homeostasis 2026-10-28 wtxid"
// printf '%s' "$V1" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 wtxid" --no-names
const H2_WTXID_V1: &str = "dfe2644d59ef5116f14d0dc1917005d8bbe53eee415524d72a7e742510cf1661";

// H3: txid(V2) di bawah domain "homeostasis 2026-10-28 txid"
// printf '%s' "$V2" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 txid" --no-names
const H3_TXID_V2: &str = "9adb1818e0a08c686e44d00f4f51a3d5655b776b782697b7b95a5d6089e6b289";

// H4: wtxid(V2) di bawah domain "homeostasis 2026-10-28 wtxid"
// printf '%s' "$V2" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 wtxid" --no-names
const H4_WTXID_V2: &str = "e8b8e3058b89963b68eceecc2e6585fc5442b8e3ecd04bdeabd69a44b69a115d";

// H5 txid: txid(V3) di bawah domain "homeostasis 2026-10-28 txid"
// printf '%s' "$V3" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 txid" --no-names
const H5_TXID_V3: &str = "935d24f13c14464a2e708bfa6f81141e4371bb5a5d872813e7ce9b0f0d33be76";

// H5 wtxid: wtxid(V3) di bawah domain "homeostasis 2026-10-28 wtxid"
// printf '%s' "$V3" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 wtxid" --no-names
const H5_WTXID_V3: &str = "fc4c66a00fcad51a02fc2650e18abb48616d7044d20f3a8c4459e57fde5ff67a";

// H6: block_id(V4) di bawah domain "homeostasis 2026-10-28 block id"
// printf '%s' "$V4" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 block id" --no-names
const H6_BLOCK_ID_V4: &str = "1de077cc1a2c8ad7c055df207ce40cca9643f961d0df21c1c67c7f121d107cfa";

// H7: condition_hash(V5) di bawah domain "homeostasis 2026-10-28 condition"
// printf '%s' "$V5" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 condition" --no-names
const H7_CONDITION_HASH_V5: &str =
    "dbb0fcdcbf4666937f85300d06ca7e2574a31761e71ed7807e468991dbcd8a35";

// H8: block_merkle_root(V8)
// daun 0 = H(merkle leaf, H4) -> 7288f1fddefb43470453bfc25e327f46f4df4c0e7c02469c5ca9750208d6d3f5
// daun 1 = H(merkle leaf, H2) -> 3e15cb5f9e12d3f7bf3d1f7d509ab92aab74ccd744f8c34f38bbb0dab5d63e41
// root   = H(merkle node, daun0 || daun1)
const H8_BLOCK_MERKLE_ROOT_V8: &str =
    "de41614cd36307ab95dfef64c5c898af62f789e6f76bad1388e332ebbe973d59";

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
}
