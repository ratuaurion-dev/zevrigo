use proptest::prelude::*;

use super::{Block, GENESIS_EXTRA_LEN, HEADER_SIZE, Header};
use crate::amount::Amount;
use crate::encoding::{Decode, DecodeError, Encode, Reader};
use crate::hash::Hash32;
use crate::height::BlockHeight;
use crate::params::MAX_BLOCK_SIZE;
use crate::tx::{Input, Output, OutputKind, Transaction, Witness};

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

const V4_HEX: &str = concat!(
    "01000000",
    "0100000000000000",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "2222222222222222222222222222222222222222222222222222222222222222",
    "00d2496b00000000",
    "ffff001d",
    "2a00000000000000"
);

fn decode_hex(hex: &str) -> Vec<u8> {
    let (pairs, rem) = hex.as_bytes().as_chunks::<2>();
    assert!(rem.is_empty(), "panjang hex harus genap");
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
    Transaction::from_bytes(&decode_hex(V1_HEX)).expect("V1 valid")
}

fn build_v2() -> Transaction {
    Transaction::from_bytes(&decode_hex(V2_HEX)).expect("V2 valid")
}

fn build_v4() -> Header {
    Header::new(
        BlockHeight::new(1),
        Hash32::from_bytes([0x11; 32]),
        Hash32::from_bytes([0x22; 32]),
        1_800_000_000,
        0x1d00_ffff,
        42,
    )
}

fn build_v8() -> Block {
    Block::new(build_v4(), vec![build_v2(), build_v1()]).expect("V8 valid")
}

#[test]
fn vector_v4_header_encoding() {
    let header = build_v4();
    let array = header.to_array();
    assert_eq!(array.len(), HEADER_SIZE);
    assert_eq!(encode_hex(&array), V4_HEX);

    let from_arr = Header::from_array(&array).expect("from_array berhasil");
    assert_eq!(from_arr, header);

    let from_b = Header::from_bytes(&array).expect("from_bytes berhasil");
    assert_eq!(from_b, header);

    assert_eq!(header.to_bytes(), array.to_vec());
}

#[test]
fn vector_v8_block_encoding() {
    let block = build_v8();
    let encoded = block.to_bytes();
    assert_eq!(encoded.len(), 336);

    let mut expected = decode_hex(V4_HEX);
    expected.push(0x02); // tx_count = 2
    expected.extend_from_slice(&decode_hex(V2_HEX));
    expected.extend_from_slice(&decode_hex(V1_HEX));
    assert_eq!(encoded, expected);

    let decoded = Block::from_bytes(&encoded).expect("decode V8 berhasil");
    assert_eq!(decoded, block);
}

#[test]
fn rejection_vectors_r18_to_r23() {
    let v4_bytes = decode_hex(V4_HEX);

    // R18: V4 dengan versi `02000000` -> versi tidak dikenal
    let mut r18 = v4_bytes.clone();
    if let Some(first) = r18.first_mut() {
        *first = 0x02;
    }
    assert_eq!(
        Header::from_bytes(&r18),
        Err(DecodeError::UnsupportedVersion { actual: 2 })
    );

    // R19: Blok dengan transaksi [V1, V2] dan [V2, V2] -> posisi coinbase salah
    let v1 = build_v1();
    let v2 = build_v2();
    let header = build_v4();

    // Kasus 1: [V1, V2] (transaksi pertama bukan coinbase)
    assert_eq!(
        Block::new(header, vec![v1.clone(), v2.clone()]),
        Err(DecodeError::InvalidCoinbasePosition)
    );
    let mut r19_case1 = v4_bytes.clone();
    r19_case1.push(0x02);
    r19_case1.extend_from_slice(&v1.to_bytes());
    r19_case1.extend_from_slice(&v2.to_bytes());
    assert_eq!(
        Block::from_bytes(&r19_case1),
        Err(DecodeError::InvalidCoinbasePosition)
    );

    // Kasus 2: [V2, V2] (transaksi kedua adalah coinbase)
    assert_eq!(
        Block::new(header, vec![v2.clone(), v2.clone()]),
        Err(DecodeError::InvalidCoinbasePosition)
    );
    let mut r19_case2 = v4_bytes.clone();
    r19_case2.push(0x02);
    r19_case2.extend_from_slice(&v2.to_bytes());
    r19_case2.extend_from_slice(&v2.to_bytes());
    assert_eq!(
        Block::from_bytes(&r19_case2),
        Err(DecodeError::InvalidCoinbasePosition)
    );

    // R20: V8 dengan height header 2 -> tinggi coinbase tidak sama
    let header_h2 = Header::new(
        BlockHeight::new(2),
        Hash32::from_bytes([0x11; 32]),
        Hash32::from_bytes([0x22; 32]),
        1_800_000_000,
        0x1d00_ffff,
        42,
    );
    assert_eq!(
        Block::new(header_h2, vec![v2.clone(), v1.clone()]),
        Err(DecodeError::CoinbaseHeightMismatch {
            header: BlockHeight::new(2),
            coinbase: BlockHeight::new(1),
        })
    );
    let mut r20 = header_h2.to_bytes();
    r20.push(0x02);
    r20.extend_from_slice(&v2.to_bytes());
    r20.extend_from_slice(&v1.to_bytes());
    assert_eq!(
        Block::from_bytes(&r20),
        Err(DecodeError::CoinbaseHeightMismatch {
            header: BlockHeight::new(2),
            coinbase: BlockHeight::new(1),
        })
    );

    // R21: Blok genesis (height 0) dengan coinbase tinggi 0 dan extra kosong -> extra genesis tidak tepat 32
    let genesis_header = Header::new(
        BlockHeight::GENESIS,
        Hash32::ZERO,
        Hash32::ZERO,
        1_800_000_000,
        0x1d00_ffff,
        0,
    );
    let empty_extra_cb = Transaction::coinbase(
        BlockHeight::GENESIS,
        Vec::new(),
        vec![Output::new(Amount::from_quanta(1), OutputKind::SingleKey, [0x55; 32]).expect("out")],
    )
    .expect("cb valid");
    assert_eq!(
        Block::new(genesis_header, vec![empty_extra_cb.clone()]),
        Err(DecodeError::InvalidGenesisExtraLength { actual: 0 })
    );
    let mut r21 = genesis_header.to_bytes();
    r21.push(0x01);
    r21.extend_from_slice(&empty_extra_cb.to_bytes());
    assert_eq!(
        Block::from_bytes(&r21),
        Err(DecodeError::InvalidGenesisExtraLength { actual: 0 })
    );

    // R22: Blok tanpa transaksi (V4 + 00) -> blok kosong
    let mut r22 = v4_bytes;
    r22.push(0x00);
    assert_eq!(Block::from_bytes(&r22), Err(DecodeError::EmptyBlock));
    assert_eq!(
        Block::new(build_v4(), Vec::new()),
        Err(DecodeError::EmptyBlock)
    );

    // R23: Input 100 001 byte (isi apa saja) ke Block::from_bytes -> blok terlalu besar, sebelum membaca
    let r23_zeros = vec![0_u8; MAX_BLOCK_SIZE.saturating_add(1)];
    assert_eq!(
        Block::from_bytes(&r23_zeros),
        Err(DecodeError::BlockTooLarge {
            size: MAX_BLOCK_SIZE.saturating_add(1)
        })
    );
}

#[test]
fn koreksi_1_block_decode_from_with_100k_extra_bytes_succeeds() {
    // Sebuah Reader berisi encoding V8 diikuti 100 000 byte tambahan ->
    // Block::decode_from berhasil dan reader masih menyisakan tepat 100 000 byte.
    let v8_bytes = build_v8().to_bytes();
    let mut stream = v8_bytes;
    let extra_p2p_bytes = vec![0x7a; 100_000];
    stream.extend_from_slice(&extra_p2p_bytes);

    let mut reader = Reader::new(&stream);
    let decoded_block =
        Block::decode_from(&mut reader).expect("decode_from V8 dalam stream harus sukses");
    assert_eq!(decoded_block, build_v8());
    assert_eq!(reader.remaining(), 100_000);
    assert_eq!(reader.read_slice(100_000), Ok(extra_p2p_bytes.as_slice()));
    assert_eq!(reader.finish(), Ok(()));
}

#[test]
fn block_new_and_decode_from_rejects_oversized_block() {
    // Buat blok yang valid secara struktur tetapi total byte > 100 000 byte
    let header = Header::new(
        BlockHeight::new(1),
        Hash32::ZERO,
        Hash32::ZERO,
        1_800_000_000,
        0x1d00_ffff,
        0,
    );
    let cb = build_v2(); // 53 byte
    // Setiap transaksi regular berukuran minimal 186 byte (seperti V1)
    // 600 transaksi * 186 byte = 111 600 byte > 100 000 byte
    let v1 = build_v1();
    let mut txs = Vec::with_capacity(601);
    txs.push(cb);
    for _ in 0..600 {
        txs.push(v1.clone());
    }

    // Block::new harus menolak dengan BlockTooLarge
    let new_res = Block::new(header, txs.clone());
    assert!(matches!(new_res, Err(DecodeError::BlockTooLarge { .. })));

    // Block::decode_from pada stream harus menolak dengan BlockTooLarge
    let mut large_encoded = header.to_bytes();
    crate::encoding::write_vec(&mut large_encoded, &txs);
    let mut r = Reader::new(&large_encoded);
    let decode_res = Block::decode_from(&mut r);
    assert!(matches!(decode_res, Err(DecodeError::BlockTooLarge { .. })));
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

pub(crate) fn arb_header() -> impl Strategy<Value = Header> {
    (
        any::<u64>(),
        any::<[u8; 32]>(),
        any::<[u8; 32]>(),
        any::<u64>(),
        any::<u32>(),
        any::<u64>(),
    )
        .prop_map(|(h, prev, merkle, ts, target, nonce)| {
            Header::new(
                BlockHeight::new(h),
                Hash32::from_bytes(prev),
                Hash32::from_bytes(merkle),
                ts,
                target,
                nonce,
            )
        })
}

pub(crate) fn arb_valid_block() -> impl Strategy<Value = Block> {
    (
        any::<u64>(),
        any::<[u8; 32]>(),
        any::<[u8; 32]>(),
        any::<u64>(),
        any::<u32>(),
        any::<u64>(),
        proptest::collection::vec(any::<u8>(), 0..=32),
        proptest::collection::vec((1_u128..=10_000_000, any::<[u8; 32]>()), 1..=2),
        proptest::collection::vec(
            (
                any::<[u8; 32]>(),
                any::<u32>(),
                1_u128..=10_000_000,
                any::<[u8; 32]>(),
                any::<[u8; 64]>(),
            ),
            0..=2,
        ),
    )
        .prop_map(
            |(h, prev, merkle, ts, target, nonce, extra_raw, cb_outs, reg_specs)| {
                let height = BlockHeight::new(h);
                let prev_id = if height == BlockHeight::GENESIS {
                    Hash32::ZERO
                } else {
                    Hash32::from_bytes(prev)
                };
                let extra = if height == BlockHeight::GENESIS {
                    vec![0x42; GENESIS_EXTRA_LEN]
                } else {
                    extra_raw
                };
                let header = Header::new(
                    height,
                    prev_id,
                    Hash32::from_bytes(merkle),
                    ts,
                    target,
                    nonce,
                );
                let cb_outputs = cb_outs
                    .into_iter()
                    .map(|(q, p)| {
                        Output::new(Amount::from_quanta(q), OutputKind::SingleKey, p).expect("out")
                    })
                    .collect();
                let coinbase_tx =
                    Transaction::coinbase(height, extra, cb_outputs).expect("cb valid");

                let mut txs = Vec::new();
                txs.push(coinbase_tx);
                for (p_txid, p_idx, q, p, sig) in reg_specs {
                    let input = Input::new(Hash32::from_bytes(p_txid), p_idx);
                    let out =
                        Output::new(Amount::from_quanta(q), OutputKind::SingleKey, p).expect("out");
                    let wit = Witness::single_key(sig);
                    let reg_tx =
                        Transaction::regular(vec![input], vec![out], vec![wit]).expect("reg valid");
                    txs.push(reg_tx);
                }
                Block::new(header, txs).expect("blok harus valid")
            },
        )
}

proptest! {
    #[test]
    fn proptest_header_roundtrip_and_prefixes(header in arb_header()) {
        let bytes = header.to_bytes();
        prop_assert_eq!(bytes.len(), HEADER_SIZE);
        prop_assert_eq!(Header::from_bytes(&bytes), Ok(header));
        assert_all_strict_prefixes_rejected::<Header>(&bytes);
    }

    #[test]
    fn proptest_block_roundtrip_and_prefixes(block in arb_valid_block()) {
        let bytes = block.to_bytes();
        prop_assert!(bytes.len() <= MAX_BLOCK_SIZE);
        prop_assert_eq!(Block::from_bytes(&bytes), Ok(block));
        assert_all_strict_prefixes_rejected::<Block>(&bytes);
    }

    #[test]
    fn proptest_block_and_header_arbitrary_bytes_up_to_2048_canonical_and_no_panic(
        bytes in proptest::collection::vec(any::<u8>(), 0..=2048)
    ) {
        if let Ok(hdr) = Header::from_bytes(&bytes) {
            prop_assert_eq!(hdr.to_bytes(), bytes.clone());
        }
        if let Ok(blk) = Block::from_bytes(&bytes) {
            prop_assert_eq!(blk.to_bytes(), bytes);
        }
    }
}
