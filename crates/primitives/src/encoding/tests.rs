use proptest::prelude::*;

use super::{
    DecodeError, MIN_BYTE_SIZE, MIN_INPUT_SIZE, MIN_KEY_SIZE, MIN_OUTPUT_SIZE, MIN_SIG_ENTRY_SIZE,
    MIN_TX_SIZE, MIN_WITNESS_SIZE, Reader, varint_u32_len, varint_u64_len, varint_u128_len,
    write_u8, write_u32_le, write_u64_le, write_varint_u32, write_varint_u64, write_varint_u128,
};

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

fn expected_varint_len_u128(value: u128) -> usize {
    if value == 0 {
        return 1;
    }
    let bits = 128_u32.saturating_sub(value.leading_zeros());
    let len_u32 = bits.saturating_add(6) / 7;
    usize::try_from(len_u32).expect("panjang varint muat usize")
}

#[test]
fn min_element_size_constants_match_spec_section_10() {
    assert_eq!(MIN_BYTE_SIZE, 1);
    assert_eq!(MIN_KEY_SIZE, 32);
    assert_eq!(MIN_INPUT_SIZE, 33);
    assert_eq!(MIN_OUTPUT_SIZE, 34);
    assert_eq!(MIN_WITNESS_SIZE, 65);
    assert_eq!(MIN_SIG_ENTRY_SIZE, 65);
    assert_eq!(MIN_TX_SIZE, 46);
}

#[test]
fn spec_section_2_1_all_thirteen_varint_vectors() {
    let vectors: [(u128, &str, usize); 13] = [
        (0, "00", 1),
        (1, "01", 1),
        (127, "7f", 1),
        (128, "8001", 2),
        (255, "ff01", 2),
        (300, "ac02", 2),
        (16_383, "ff7f", 2),
        (16_384, "808001", 3),
        (u128::from(u32::MAX), "ffffffff0f", 5),
        (1_000_000_000_000_000_000, "808090bbbad6adf00d", 9),
        (1_500_000_000_000_000_000, "8080d8d8d7c1c4e814", 9),
        (u128::from(u64::MAX), "ffffffffffffffffff01", 10),
        (u128::MAX, "ffffffffffffffffffffffffffffffffffff03", 19),
    ];

    for (value, expected_hex, expected_len) in vectors {
        let expected_bytes = decode_hex(expected_hex);
        assert_eq!(expected_bytes.len(), expected_len);

        // u128: semua 13 nilai berada dalam rentang
        let mut out_128 = Vec::new();
        write_varint_u128(&mut out_128, value);
        assert_eq!(encode_hex(&out_128), expected_hex);
        assert_eq!(out_128.len(), expected_len);
        assert_eq!(varint_u128_len(value), expected_len);

        let mut r128 = Reader::new(&expected_bytes);
        assert_eq!(r128.read_varint_u128(), Ok(value));
        assert_eq!(r128.finish(), Ok(()));

        // u64: cocok dalam rentang, VarintOverflow di luar rentang
        if let Ok(val_u64) = u64::try_from(value) {
            let mut out_64 = Vec::new();
            write_varint_u64(&mut out_64, val_u64);
            assert_eq!(encode_hex(&out_64), expected_hex);
            assert_eq!(varint_u64_len(val_u64), expected_len);

            let mut r64 = Reader::new(&expected_bytes);
            assert_eq!(r64.read_varint_u64(), Ok(val_u64));
            assert_eq!(r64.finish(), Ok(()));
        } else {
            let mut r64 = Reader::new(&expected_bytes);
            assert_eq!(r64.read_varint_u64(), Err(DecodeError::VarintOverflow));
        }

        // u32: cocok dalam rentang, VarintOverflow di luar rentang
        if let Ok(val_u32) = u32::try_from(value) {
            let mut out_32 = Vec::new();
            write_varint_u32(&mut out_32, val_u32);
            assert_eq!(encode_hex(&out_32), expected_hex);
            assert_eq!(varint_u32_len(val_u32), expected_len);

            let mut r32 = Reader::new(&expected_bytes);
            assert_eq!(r32.read_varint_u32(), Ok(val_u32));
            assert_eq!(r32.finish(), Ok(()));
        } else {
            let mut r32 = Reader::new(&expected_bytes);
            assert_eq!(r32.read_varint_u32(), Err(DecodeError::VarintOverflow));
        }
    }
}

#[test]
fn rejection_vectors_r1_to_r5() {
    // R1: varint u64 `80` -> data terpotong
    let r1_bytes = decode_hex("80");
    assert_eq!(
        Reader::new(&r1_bytes).read_varint_u64(),
        Err(DecodeError::UnexpectedEof)
    );
    assert_eq!(
        Reader::new(&r1_bytes).read_varint_u32(),
        Err(DecodeError::UnexpectedEof)
    );
    assert_eq!(
        Reader::new(&r1_bytes).read_varint_u128(),
        Err(DecodeError::UnexpectedEof)
    );

    // R2: varint u64 `8000` -> varint tidak minimal
    let r2_bytes = decode_hex("8000");
    assert_eq!(
        Reader::new(&r2_bytes).read_varint_u64(),
        Err(DecodeError::NonMinimalVarint)
    );
    assert_eq!(
        Reader::new(&r2_bytes).read_varint_u32(),
        Err(DecodeError::NonMinimalVarint)
    );
    assert_eq!(
        Reader::new(&r2_bytes).read_varint_u128(),
        Err(DecodeError::NonMinimalVarint)
    );

    // R3: varint u32 `ffffffff1f` -> varint melampaui lebar
    let r3_bytes = decode_hex("ffffffff1f");
    assert_eq!(
        Reader::new(&r3_bytes).read_varint_u32(),
        Err(DecodeError::VarintOverflow)
    );

    // R4: varint u64 `ffffffffffffffffff02` -> varint melampaui lebar
    let r4_bytes = decode_hex("ffffffffffffffffff02");
    assert_eq!(
        Reader::new(&r4_bytes).read_varint_u64(),
        Err(DecodeError::VarintOverflow)
    );

    // R5: varint u128 `ffffffffffffffffffffffffffffffffffff04` -> varint melampaui lebar
    let r5_bytes = decode_hex("ffffffffffffffffffffffffffffffffffff04");
    assert_eq!(
        Reader::new(&r5_bytes).read_varint_u128(),
        Err(DecodeError::VarintOverflow)
    );
}

#[test]
fn reader_fixed_primitives_and_allocation_guard_boundaries() {
    let mut buf = Vec::new();
    write_u8(&mut buf, 0xaa);
    write_u32_le(&mut buf, 0x1d00_ffff);
    write_u64_le(&mut buf, 1_800_000_000);

    let mut r = Reader::new(&buf);
    assert_eq!(r.remaining(), 13);
    assert_eq!(r.read_u8(), Ok(0xaa));
    assert_eq!(r.read_u32_le(), Ok(0x1d00_ffff));
    assert_eq!(r.read_u64_le(), Ok(1_800_000_000));
    assert_eq!(r.remaining(), 0);
    assert_eq!(r.finish(), Ok(()));

    // Sisa 1 byte ditolak finish()
    let mut trailing = buf;
    trailing.push(0x00);
    let mut r_trail = Reader::new(&trailing);
    let _ = r_trail.read_slice(13).expect("13 byte tersedia");
    assert_eq!(
        r_trail.finish(),
        Err(DecodeError::TrailingBytes { remaining: 1 })
    );

    // Pelindung alokasi: 2 elemen @ 33 byte pada sisa 66 byte lolos; pada sisa 65 byte gagal
    let mut ok_guard = vec![0x02];
    ok_guard.extend_from_slice(&[0_u8; 66]);
    let mut r_ok = Reader::new(&ok_guard);
    assert_eq!(r_ok.read_vec_len(MIN_INPUT_SIZE), Ok(2));

    let mut fail_guard = vec![0x02];
    fail_guard.extend_from_slice(&[0_u8; 65]);
    let mut r_fail = Reader::new(&fail_guard);
    assert_eq!(
        r_fail.read_vec_len(MIN_INPUT_SIZE),
        Err(DecodeError::ImpossibleElementCount {
            count: 2,
            remaining: 65,
            min_elem_size: MIN_INPUT_SIZE,
        })
    );
}

proptest! {
    #[test]
    fn proptest_varint_u32_roundtrip_len_and_non_minimal(val in any::<u32>(), pad_zeros in 1_usize..=5) {
        let mut encoded = Vec::new();
        write_varint_u32(&mut encoded, val);

        let expected_len = expected_varint_len_u128(u128::from(val));
        prop_assert_eq!(encoded.len(), expected_len);
        prop_assert_eq!(varint_u32_len(val), expected_len);

        let mut reader = Reader::new(&encoded);
        prop_assert_eq!(reader.read_varint_u32(), Ok(val));
        prop_assert_eq!(reader.finish(), Ok(()));

        // Setiap prefiks lebih pendek ditolak dengan UnexpectedEof
        for prefix_len in 0..encoded.len() {
            if let Some((prefix, _)) = encoded.split_at_checked(prefix_len) {
                prop_assert_eq!(
                    Reader::new(prefix).read_varint_u32(),
                    Err(DecodeError::UnexpectedEof)
                );
            }
        }

        // Encoding tidak minimal (menambahkan grup nol di ujung) wajib ditolak
        let mut non_minimal = encoded;
        if let Some(last) = non_minimal.last_mut() {
            *last |= 0x80;
        }
        non_minimal.resize(
            non_minimal
                .len()
                .saturating_add(pad_zeros.saturating_sub(1)),
            0x80,
        );
        non_minimal.push(0x00);

        let res = Reader::new(&non_minimal).read_varint_u32();
        if non_minimal.len() <= 5 {
            prop_assert_eq!(res, Err(DecodeError::NonMinimalVarint));
        } else {
            prop_assert_eq!(res, Err(DecodeError::VarintOverflow));
        }
    }

    #[test]
    fn proptest_varint_u64_roundtrip_len_and_non_minimal(val in any::<u64>(), pad_zeros in 1_usize..=5) {
        let mut encoded = Vec::new();
        write_varint_u64(&mut encoded, val);

        let expected_len = expected_varint_len_u128(u128::from(val));
        prop_assert_eq!(encoded.len(), expected_len);
        prop_assert_eq!(varint_u64_len(val), expected_len);

        let mut reader = Reader::new(&encoded);
        prop_assert_eq!(reader.read_varint_u64(), Ok(val));
        prop_assert_eq!(reader.finish(), Ok(()));

        let mut non_minimal = encoded;
        if let Some(last) = non_minimal.last_mut() {
            *last |= 0x80;
        }
        non_minimal.resize(
            non_minimal
                .len()
                .saturating_add(pad_zeros.saturating_sub(1)),
            0x80,
        );
        non_minimal.push(0x00);

        let res = Reader::new(&non_minimal).read_varint_u64();
        if non_minimal.len() <= 10 {
            prop_assert_eq!(res, Err(DecodeError::NonMinimalVarint));
        } else {
            prop_assert_eq!(res, Err(DecodeError::VarintOverflow));
        }
    }

    #[test]
    fn proptest_varint_u128_roundtrip_len_and_non_minimal(val in any::<u128>(), pad_zeros in 1_usize..=5) {
        let mut encoded = Vec::new();
        write_varint_u128(&mut encoded, val);

        let expected_len = expected_varint_len_u128(val);
        prop_assert_eq!(encoded.len(), expected_len);
        prop_assert_eq!(varint_u128_len(val), expected_len);

        let mut reader = Reader::new(&encoded);
        prop_assert_eq!(reader.read_varint_u128(), Ok(val));
        prop_assert_eq!(reader.finish(), Ok(()));

        let mut non_minimal = encoded;
        if let Some(last) = non_minimal.last_mut() {
            *last |= 0x80;
        }
        non_minimal.resize(
            non_minimal
                .len()
                .saturating_add(pad_zeros.saturating_sub(1)),
            0x80,
        );
        non_minimal.push(0x00);

        let res = Reader::new(&non_minimal).read_varint_u128();
        if non_minimal.len() <= 19 {
            prop_assert_eq!(res, Err(DecodeError::NonMinimalVarint));
        } else {
            prop_assert_eq!(res, Err(DecodeError::VarintOverflow));
        }
    }

    #[test]
    fn proptest_varint_arbitrary_bytes_canonical_and_no_panic(bytes in proptest::collection::vec(any::<u8>(), 0..=64)) {
        let mut r32 = Reader::new(&bytes);
        if let Ok(v32) = r32.read_varint_u32()
            && r32.finish().is_ok()
        {
            let mut reencoded = Vec::new();
            write_varint_u32(&mut reencoded, v32);
            prop_assert_eq!(reencoded, bytes.clone());
        }

        let mut r64 = Reader::new(&bytes);
        if let Ok(v64) = r64.read_varint_u64()
            && r64.finish().is_ok()
        {
            let mut reencoded = Vec::new();
            write_varint_u64(&mut reencoded, v64);
            prop_assert_eq!(reencoded, bytes.clone());
        }

        let mut r128 = Reader::new(&bytes);
        if let Ok(v128) = r128.read_varint_u128()
            && r128.finish().is_ok()
        {
            let mut reencoded = Vec::new();
            write_varint_u128(&mut reencoded, v128);
            prop_assert_eq!(reencoded, bytes);
        }
    }
}
