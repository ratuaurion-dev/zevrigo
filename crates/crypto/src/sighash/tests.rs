use homeostasis_primitives::amount::Amount;
use homeostasis_primitives::encoding::Decode;
use homeostasis_primitives::hash::Hash32;
use homeostasis_primitives::tx::{Output, OutputKind, Transaction};

use super::{SighashError, sighash_v1};
use crate::ids::wtxid;

// V1 dan V2: vektor kartu TSK-M03, sama persis dengan crates/crypto/src/ids/tests.rs.
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

/// H6 = block_id(V4), dipakai sebagai network_id contoh.
const H6: &str = "1de077cc1a2c8ad7c055df207ce40cca9643f961d0df21c1c67c7f121d107cfa";
/// H3 = txid(V2), dipakai sebagai network_id lain.
const H3: &str = "9adb1818e0a08c686e44d00f4f51a3d5655b776b782697b7b95a5d6089e6b289";

// Output yang dibelanjakan O(q): amount q quanta, kind 0, payload 0x77 x 32.
// encode(O(2_000_000_000_000_000_000)) = 8080a0f6f4acdbe01b00 || 77 x 32 (42 byte)
// S1 = sighash_v1(H6, V1, 0, [O(2 HSIS)]); preimage 110 byte = H6 || H1 || 00000000 || encode(O):
// printf '%s' "1de077cc1a2c8ad7c055df207ce40cca9643f961d0df21c1c67c7f121d107cfa5b403b217e76db7c9a6ac42da2addef7072e1037345091542114a09fed8316f0000000008080a0f6f4acdbe01b007777777777777777777777777777777777777777777777777777777777777777" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 sighash v1" --no-names
const S1: &str = "f2ee67583afb2b9b35b88f175ae593b4e1800390b5c6b69a6ce5a2f9c11922ad";
// S2 = S1 dengan network_id H3 (preimage S1 dengan 32 byte pertama diganti H3).
const S2: &str = "c3635edb315f2813f1361a360321711813580bd92c075d02ec77a256bcce0e00";
// S3 = S1 dengan amount 2_000_000_000_000_000_001 quanta (encode 8180a0f6f4acdbe01b00 || 77 x 32).
const S3: &str = "d9b77ec8b60e72c105623bfcba49c7cf4275a6d8cf81ba5780f183e8c26f6dbf";

const TWO_HSIS: u128 = 2_000_000_000_000_000_000;

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

fn h(hex: &str) -> Hash32 {
    Hash32::from_hex(hex).expect("hex 32 byte")
}

fn v1() -> Transaction {
    Transaction::from_bytes(&decode_hex(V1_HEX)).expect("V1 valid")
}

fn spent(q: u128) -> Output {
    Output::new(Amount::from_quanta(q), OutputKind::SingleKey, [0x77; 32]).expect("output")
}

#[test]
fn vectors_sighash_v1_s1_to_s3() {
    let tx = v1();
    assert_eq!(sighash_v1(&h(H6), &tx, 0, &[spent(TWO_HSIS)]), Ok(h(S1)));
    assert_eq!(sighash_v1(&h(H3), &tx, 0, &[spent(TWO_HSIS)]), Ok(h(S2)));
    let more = TWO_HSIS.checked_add(1).expect("tidak overflow");
    assert_eq!(sighash_v1(&h(H6), &tx, 0, &[spent(more)]), Ok(h(S3)));
}

#[test]
fn sighash_rejects_coinbase() {
    let coinbase = Transaction::from_bytes(&decode_hex(V2_HEX)).expect("V2 valid");
    assert_eq!(
        sighash_v1(&h(H6), &coinbase, 0, &[]),
        Err(SighashError::Coinbase)
    );
}

#[test]
fn sighash_rejects_input_index_out_of_range() {
    let tx = v1();
    assert_eq!(
        sighash_v1(&h(H6), &tx, 1, &[spent(TWO_HSIS)]),
        Err(SighashError::InputIndexOutOfRange {
            index: 1,
            inputs: 1
        })
    );
    assert_eq!(
        sighash_v1(&h(H6), &tx, u32::MAX, &[spent(TWO_HSIS)]),
        Err(SighashError::InputIndexOutOfRange {
            index: u32::MAX,
            inputs: 1
        })
    );
}

#[test]
fn sighash_rejects_spent_count_mismatch() {
    let tx = v1();
    assert_eq!(
        sighash_v1(&h(H6), &tx, 0, &[]),
        Err(SighashError::SpentCountMismatch {
            inputs: 1,
            spent: 0
        })
    );
    assert_eq!(
        sighash_v1(&h(H6), &tx, 0, &[spent(TWO_HSIS), spent(TWO_HSIS)]),
        Err(SighashError::SpentCountMismatch {
            inputs: 1,
            spent: 2
        })
    );
}

#[test]
fn sighash_does_not_depend_on_witness() {
    let mut bytes = decode_hex(V1_HEX);
    let last = bytes.last_mut().expect("V1 tidak kosong");
    assert_eq!(*last, 0x33);
    *last = 0x32;
    let mutated = Transaction::from_bytes(&bytes).expect("V1 dengan tanda tangan lain");
    let original = v1();
    assert_ne!(wtxid(&mutated), wtxid(&original));
    assert_eq!(
        sighash_v1(&h(H6), &mutated, 0, &[spent(TWO_HSIS)]),
        sighash_v1(&h(H6), &original, 0, &[spent(TWO_HSIS)])
    );
}
