use proptest::prelude::*;
use zevrigo_primitives::amount::Amount;
use zevrigo_primitives::hash::Hash32;
use zevrigo_primitives::height::BlockHeight;
use zevrigo_primitives::tx::{Output, OutputKind};
use zevrigo_primitives::utxo::{OutPoint, UtxoEntry};

use super::{
    decode_block_undo, decode_outpoint_key, decode_utxo_entry, encode_block_undo,
    encode_utxo_entry, outpoint_key,
};
use crate::store::{BlockUndo, StoreError};

fn h(byte: u8) -> Hash32 {
    Hash32::from_bytes([byte; 32])
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn from_hex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks(2)
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).expect("utf8"), 16).expect("hex"))
        .collect()
}

/// K1 dari kartu TSK-M11 (dihitung Quick).
const K1: &str = "111111111111111111111111111111111111111111111111111111111111111101020304";
/// U1 dari kartu TSK-M11, 43 byte.
const U1: &str =
    "0500000000000000010a000101010101010101010101010101010101010101010101010101010101010101";
/// B1 dari kartu TSK-M11, 123 byte.
const B1: &str = "010000001111111111111111111111111111111111111111111111111111111111111111000000010500000000000000010a00010101010101010101010101010101010101010101010101010101010101010101000000222222222222222222222222222222222222222222222222222222222222222200000000";

fn entry_e() -> UtxoEntry {
    let out =
        Output::new(Amount::from_quanta(10), OutputKind::SingleKey, [0x01; 32]).expect("output");
    UtxoEntry::new(out, BlockHeight::new(5), true)
}

fn undo_b1() -> BlockUndo {
    BlockUndo::new(
        vec![(OutPoint::new(h(0x11), 1), entry_e())],
        vec![OutPoint::new(h(0x22), 0)],
    )
}

#[test]
fn vectors_outpoint_key_utxo_entry_and_block_undo() {
    assert_eq!(
        to_hex(&outpoint_key(&OutPoint::new(h(0x11), 0x0102_0304))),
        K1
    );
    assert_eq!(to_hex(&encode_utxo_entry(&entry_e())), U1);
    assert_eq!(
        encode_block_undo(&undo_b1()).map(|bytes| to_hex(&bytes)),
        Ok(B1.to_owned())
    );
    assert_eq!(decode_utxo_entry(&from_hex(U1)), Ok(entry_e()));
    assert_eq!(decode_block_undo(&from_hex(B1)), Ok(undo_b1()));
}

#[test]
fn decode_outpoint_key_inverts_outpoint_key() {
    let op = OutPoint::new(h(0x11), 0x0102_0304);
    assert_eq!(decode_outpoint_key(&outpoint_key(&op)), Ok(op));
    let max = OutPoint::new(h(0xff), u32::MAX);
    assert_eq!(decode_outpoint_key(&outpoint_key(&max)), Ok(max));
}

proptest! {
    #[test]
    fn prop_outpoint_key_order_matches_outpoint_ord(
        ta in any::<[u8; 32]>(),
        ia in any::<u32>(),
        tb in any::<[u8; 32]>(),
        ib in any::<u32>(),
    ) {
        let a = OutPoint::new(Hash32::from_bytes(ta), ia);
        let b = OutPoint::new(Hash32::from_bytes(tb), ib);
        prop_assert_eq!(outpoint_key(&a).cmp(&outpoint_key(&b)), a.cmp(&b));
    }

    #[test]
    fn prop_utxo_entry_round_trip(
        q in 1_u128..=u128::MAX,
        cond in any::<bool>(),
        p in any::<[u8; 32]>(),
        height in any::<u64>(),
        cb in any::<bool>(),
    ) {
        let kind = if cond { OutputKind::ConditionHash } else { OutputKind::SingleKey };
        let out = Output::new(Amount::from_quanta(q), kind, p).expect("output");
        let e = UtxoEntry::new(out, BlockHeight::new(height), cb);
        prop_assert_eq!(decode_utxo_entry(&encode_utxo_entry(&e)), Ok(e));
    }

    #[test]
    fn prop_block_undo_round_trip(
        spent in proptest::collection::vec(
            (any::<[u8; 32]>(), any::<u32>(), 1_u128..=1_000_000, any::<u64>(), any::<bool>()),
            0..=8,
        ),
        created in proptest::collection::vec((any::<[u8; 32]>(), any::<u32>()), 0..=8),
    ) {
        let mut spent_list = Vec::new();
        for (txid, index, q, height, cb) in spent {
            let out = Output::new(Amount::from_quanta(q), OutputKind::SingleKey, [0x01; 32]).expect("output");
            let entry = UtxoEntry::new(out, BlockHeight::new(height), cb);
            spent_list.push((OutPoint::new(Hash32::from_bytes(txid), index), entry));
        }
        let created_list: Vec<OutPoint> = created
            .into_iter()
            .map(|(txid, index)| OutPoint::new(Hash32::from_bytes(txid), index))
            .collect();
        let undo = BlockUndo::new(spent_list, created_list);
        let encoded = encode_block_undo(&undo).expect("encode");
        prop_assert_eq!(decode_block_undo(&encoded), Ok(undo));
    }
}

#[test]
fn decode_rejects_corrupt_bytes() {
    // C1: kosong
    assert_eq!(decode_utxo_entry(&[]), Err(StoreError::CorruptData));

    // C2: byte coinbase = 02
    let c2 = from_hex(
        "0500000000000000020a000101010101010101010101010101010101010101010101010101010101010101",
    );
    assert_eq!(decode_utxo_entry(&c2), Err(StoreError::CorruptData));

    // C3: U1 ditambah 00
    let mut c3 = from_hex(U1);
    c3.push(0x00);
    assert_eq!(decode_utxo_entry(&c3), Err(StoreError::CorruptData));

    // C4: U1 tanpa byte terakhir (42 byte)
    let mut c4 = from_hex(U1);
    assert_eq!(c4.pop(), Some(0x01));
    assert_eq!(c4.len(), 42);
    assert_eq!(decode_utxo_entry(&c4), Err(StoreError::CorruptData));

    // C5: B1 dengan 4 byte pertama 02000000 (jumlah spent 2, isi hanya 1)
    let mut c5 = from_hex(B1);
    for (dst, src) in c5.iter_mut().zip([0x02_u8, 0x00, 0x00, 0x00]) {
        *dst = src;
    }
    assert_eq!(c5.first(), Some(&0x02_u8));
    assert_eq!(decode_block_undo(&c5), Err(StoreError::CorruptData));
}
