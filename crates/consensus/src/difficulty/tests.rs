use homeostasis_pow::Target;
use homeostasis_primitives::BlockHeight;
use proptest::prelude::*;

use super::{
    ASERT_HALF_LIFE_SECS, DifficultyError, asert_target, block_work, chain_work, decode_compact,
    encode_compact, median_time_past, timestamp_after_median, timestamp_within_future_limit,
};
use crate::uint::U256;

/// Cap waktu acuan vektor ASERT (= cap waktu genesis regtest).
const T0: u64 = 1_793_145_600;

/// `pow_limit` regtest.
const POW_LIMIT: u32 = 0x207f_ffff;

/// Batas cap waktu masa depan dari `now = T0`.
const FUTURE_LIMIT: u64 = T0 + 3_600;

/// Satu detik setelah batas masa depan.
const FUTURE_LIMIT_PLUS_ONE: u64 = FUTURE_LIMIT + 1;

/// Vektor A1 sampai A11: `(anchor, parent_height, time_delta, target_wajib)` (Spesifikasi Validasi Bagian 4).
const ASERT_VECTORS: [(u32, u64, u64, u32); 11] = [
    (0x1e0f_ffff, 0, 0, 0x1e0f_ffff),
    (0x1e0f_ffff, 1_000, 120_000, 0x1e0f_ffff),
    (0x1e0f_ffff, 720, 172_800, 0x1e1f_fffe),
    (0x1e0f_ffff, 720, 0, 0x1e07_ffff),
    (0x1e0f_ffff, 1, 43_320, 0x1e16_a01e),
    (0x1e0f_ffff, 100, 11_999, 0x1e0f_fff7),
    (0x207f_ffff, 10, 260_400, 0x207f_ffff),
    (0x1e0f_ffff, 1_000_000, 0, 0x0101_0000),
    (0x1e0f_ffff, 12_345, 1_482_177, 0x1e10_19ae),
    (0x1e0f_ffff, 12_345, 1_480_623, 0x1e0f_e697),
    (0x1d7f_ffff, 262_800, 31_539_600, 0x1d83_c27e),
];

/// Vektor K1 sampai K6: `(T, compact, decode(compact))` untuk target yang muat di u128.
const COMPACT_VECTORS: [(u128, u32, u128); 6] = [
    (0x01, 0x0101_0000, 0x01),
    (0xff, 0x01ff_0000, 0xff),
    (0x0100, 0x0201_0000, 0x0100),
    (0x1234, 0x0212_3400, 0x1234),
    (0x0012_3456, 0x0312_3456, 0x0012_3456),
    (0x1234_5678, 0x0412_3456, 0x1234_5600),
];

/// 32 byte big-endian dengan `prefix` di depan dan nol di belakang.
fn be_prefix(prefix: &[u8]) -> [u8; 32] {
    let mut out = [0_u8; 32];
    for (dst, src) in out.iter_mut().zip(prefix) {
        *dst = *src;
    }
    out
}

fn be_from_u128(x: u128) -> [u8; 32] {
    let mut out = [0_u8; 32];
    for (dst, src) in out.iter_mut().skip(16).zip(x.to_be_bytes()) {
        *dst = src;
    }
    out
}

fn target_from_u128(x: u128) -> Target {
    Target::from_be_bytes(be_from_u128(x))
}

fn target(compact: u32) -> Target {
    decode_compact(compact).unwrap()
}

#[test]
fn compact_vectors_k1_to_k9() {
    for (value, compact, back) in COMPACT_VECTORS {
        assert_eq!(encode_compact(&target_from_u128(value)), Ok(compact));
        assert_eq!(decode_compact(compact), Ok(target_from_u128(back)));
    }
    let two_pow_255 = Target::from_be_bytes(be_prefix(&[0x80]));
    assert_eq!(encode_compact(&two_pow_255), Ok(0x2080_0000));
    assert_eq!(decode_compact(0x2080_0000), Ok(two_pow_255));
    assert_eq!(encode_compact(&Target::MAX), Ok(0x20ff_ffff));
    assert_eq!(
        decode_compact(0x20ff_ffff),
        Ok(Target::from_be_bytes(be_prefix(&[0xff, 0xff, 0xff])))
    );
    let regtest_limit = Target::from_be_bytes(be_prefix(&[0x7f, 0xff, 0xff]));
    assert_eq!(decode_compact(POW_LIMIT), Ok(regtest_limit));
    assert_eq!(encode_compact(&regtest_limit), Ok(POW_LIMIT));
}

#[test]
fn compact_rejects_invalid_encodings() {
    for compact in [
        0x0000_0000,
        0x00ff_ffff,
        0x2101_0000,
        0xff80_0000,
        0x0300_ffff,
        0x0300_0001,
        0x0101_0001,
        0x0201_0001,
    ] {
        assert_eq!(
            decode_compact(compact),
            Err(DifficultyError::InvalidCompact { compact })
        );
    }
    assert_eq!(
        encode_compact(&Target::ZERO),
        Err(DifficultyError::ZeroTarget)
    );
}

#[test]
fn asert_vectors_a1_to_a11() {
    assert_eq!(ASERT_HALF_LIFE_SECS, 86_400);
    let limit = target(POW_LIMIT);
    for (anchor, parent, delta, expected) in ASERT_VECTORS {
        let parent_time = T0.checked_add(delta).unwrap();
        let next = asert_target(
            &target(anchor),
            T0,
            BlockHeight::new(parent),
            parent_time,
            &limit,
        )
        .unwrap();
        assert_eq!(
            encode_compact(&next),
            Ok(expected),
            "anchor {anchor:#010x}, parent {parent}, delta {delta}"
        );
    }
}

#[test]
fn block_work_vectors_w1_to_w5() {
    assert_eq!(block_work(&target(POW_LIMIT)), Some(U256::from_u128(2)));
    assert_eq!(
        block_work(&target(0x1e0f_ffff)),
        Some(U256::from_u128(1_048_577))
    );
    assert_eq!(
        block_work(&target(0x0101_0000)),
        Some(U256::from_be_bytes(be_prefix(&[0x80])))
    );
    assert_eq!(block_work(&target(0x20ff_ffff)), Some(U256::ONE));
    assert_eq!(
        block_work(&target(0x1d7f_ffff)),
        Some(U256::from_u128(33_554_436))
    );
    assert_eq!(block_work(&Target::MAX), Some(U256::ONE));
    assert_eq!(block_work(&Target::ZERO), None);
    // T = 2^128 - 1 memberi kerja tepat 2^128.
    assert_eq!(
        block_work(&target_from_u128(u128::MAX)),
        U256::from_u128(u128::MAX).checked_add(U256::ONE)
    );
}

#[test]
fn chain_work_sums_and_detects_overflow() {
    let easy = target(POW_LIMIT);
    assert_eq!(chain_work(&[easy; 6]), Some(U256::from_u128(12)));
    assert_eq!(chain_work(&[]), Some(U256::ZERO));
    let hardest = target(0x0101_0000);
    assert_eq!(chain_work(&[hardest, hardest]), None);
    assert_eq!(chain_work(&[easy, Target::ZERO]), None);
}

#[test]
fn median_time_past_vectors_m1_to_m4() {
    assert_eq!(median_time_past(&[100]), Some(100));
    assert_eq!(median_time_past(&[100, 90]), Some(100));
    assert_eq!(
        median_time_past(&[5, 1, 9, 3, 7, 11, 2, 8, 6, 10, 4]),
        Some(6)
    );
    assert_eq!(
        median_time_past(&[0, 5, 1, 9, 3, 7, 11, 2, 8, 6, 10, 4]),
        Some(6)
    );
    assert_eq!(median_time_past(&[]), None);
}

#[test]
fn timestamp_rules_v7_and_v8() {
    let prev = [5, 1, 9, 3, 7, 11, 2, 8, 6, 10, 4];
    assert!(!timestamp_after_median(6, &prev));
    assert!(timestamp_after_median(7, &prev));
    assert!(timestamp_after_median(0, &[]));
    assert!(timestamp_within_future_limit(FUTURE_LIMIT, T0));
    assert!(!timestamp_within_future_limit(FUTURE_LIMIT_PLUS_ONE, T0));
    assert!(timestamp_within_future_limit(u64::MAX, u64::MAX));
}

proptest! {
    #[test]
    fn prop_compact_encoding_is_canonical(compact in any::<u32>()) {
        if let Ok(value) = decode_compact(compact) {
            prop_assert_eq!(encode_compact(&value), Ok(compact));
        }
    }

    #[test]
    fn prop_encode_then_decode_rounds_down(bytes in any::<[u8; 32]>()) {
        let value = Target::from_be_bytes(bytes);
        prop_assume!(value != Target::ZERO);
        let back = decode_compact(encode_compact(&value).unwrap()).unwrap();
        prop_assert!(back <= value);
        let exact = U256::from_be_bytes(bytes);
        let error = exact.checked_sub(U256::from_be_bytes(back.to_be_bytes())).unwrap();
        prop_assert!(error <= exact.mul_u64_shr(1, 16).unwrap());
    }

    #[test]
    fn prop_asert_on_schedule_keeps_anchor(bytes in any::<[u8; 32]>(), parent in 0_u64..10_000_000) {
        let limit = target(POW_LIMIT);
        let raw = Target::from_be_bytes(bytes);
        prop_assume!(raw != Target::ZERO && raw <= limit);
        let anchor = decode_compact(encode_compact(&raw).unwrap()).unwrap();
        let parent_time = T0.checked_add(parent.checked_mul(120).unwrap()).unwrap();
        prop_assert_eq!(
            asert_target(&anchor, T0, BlockHeight::new(parent), parent_time, &limit),
            Ok(anchor)
        );
    }

    #[test]
    fn prop_asert_is_monotonic_in_parent_time(
        parent in 0_u64..1_000_000,
        d1 in 0_u64..20_000_000,
        d2 in 0_u64..20_000_000,
    ) {
        let limit = target(POW_LIMIT);
        let anchor = target(0x1e0f_ffff);
        let (lo, hi) = if d1 <= d2 { (d1, d2) } else { (d2, d1) };
        let height = BlockHeight::new(parent);
        let earlier = asert_target(&anchor, T0, height, T0.checked_add(lo).unwrap(), &limit).unwrap();
        let later = asert_target(&anchor, T0, height, T0.checked_add(hi).unwrap(), &limit).unwrap();
        prop_assert!(earlier <= later);
    }

    #[test]
    fn prop_block_work_is_non_increasing(a in any::<[u8; 32]>(), b in any::<[u8; 32]>()) {
        let (ta, tb) = (Target::from_be_bytes(a), Target::from_be_bytes(b));
        prop_assume!(ta != Target::ZERO && tb != Target::ZERO);
        let (lo, hi) = if ta <= tb { (ta, tb) } else { (tb, ta) };
        prop_assert!(block_work(&lo).unwrap() >= block_work(&hi).unwrap());
    }
}
