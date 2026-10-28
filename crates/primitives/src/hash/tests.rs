use proptest::prelude::*;

use super::{Hash32, HashParseError};

#[test]
fn zero_hash_display_debug_and_round_trip() {
    let expected = "0000000000000000000000000000000000000000000000000000000000000000";
    assert_eq!(Hash32::ZERO, Hash32::default());
    assert_eq!(Hash32::ZERO.as_bytes(), &[0_u8; 32]);
    assert_eq!(Hash32::ZERO.to_string(), expected);
    assert_eq!(format!("{:?}", Hash32::ZERO), expected);
    assert_eq!(Hash32::from_hex(expected), Ok(Hash32::ZERO));
}

#[test]
fn hex_case_insensitive_parse_and_lowercase_format() {
    let upper = "000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F";
    let lower = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    let mixed = "000102030405060708090a0B0c0D0e0F101112131415161718191A1b1C1d1E1f";

    let expected_bytes: [u8; 32] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f,
    ];
    let expected = Hash32::from_bytes(expected_bytes);

    assert_eq!(Hash32::from_hex(upper), Ok(expected));
    assert_eq!(Hash32::from_hex(lower), Ok(expected));
    assert_eq!(Hash32::from_hex(mixed), Ok(expected));
    assert_eq!(expected.to_string(), lower);
    assert_eq!(format!("{expected:?}"), lower);
}

#[test]
fn hex_rejects_invalid_length_and_characters() {
    assert_eq!(
        Hash32::from_hex(""),
        Err(HashParseError::InvalidLength { actual: 0 })
    );
    assert_eq!(
        Hash32::from_hex("000000000000000000000000000000000000000000000000000000000000000"),
        Err(HashParseError::InvalidLength { actual: 63 })
    );
    assert_eq!(
        Hash32::from_hex("00000000000000000000000000000000000000000000000000000000000000000"),
        Err(HashParseError::InvalidLength { actual: 65 })
    );
    assert_eq!(
        Hash32::from_hex("0x0000000000000000000000000000000000000000000000000000000000000000"),
        Err(HashParseError::InvalidLength { actual: 66 })
    );

    // 32 karakter UTF-8 dua-byte (64 byte, tetapi hanya 32 karakter).
    let utf8_32_chars = "é".repeat(32);
    assert_eq!(utf8_32_chars.len(), 64);
    assert_eq!(
        Hash32::from_hex(&utf8_32_chars),
        Err(HashParseError::InvalidLength { actual: 32 })
    );

    assert_eq!(
        Hash32::from_hex("g000000000000000000000000000000000000000000000000000000000000000"),
        Err(HashParseError::InvalidHexChar { index: 0 })
    );
    assert_eq!(
        Hash32::from_hex("000000000000000000000000000000000000000000000000000000000000000z"),
        Err(HashParseError::InvalidHexChar { index: 63 })
    );
    assert_eq!(
        Hash32::from_hex("0000000000 00000000000000000000000000000000000000000000000000000"),
        Err(HashParseError::InvalidHexChar { index: 10 })
    );
}

proptest! {
    #[test]
    fn prop_hash32_hex_round_trip(bytes in any::<[u8; 32]>()) {
        let hash = Hash32::from_bytes(bytes);
        let hex_lower = hash.to_string();
        prop_assert_eq!(hex_lower.len(), 64);
        prop_assert!(hex_lower.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
        prop_assert_eq!(format!("{hash:?}"), hex_lower.as_str());
        prop_assert_eq!(Hash32::from_hex(&hex_lower), Ok(hash));
        prop_assert_eq!(Hash32::from_hex(&hex_lower.to_ascii_uppercase()), Ok(hash));
    }
}
