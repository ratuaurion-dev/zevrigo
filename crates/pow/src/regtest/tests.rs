use proptest::prelude::*;
use zevrigo_primitives::block::Header;
use zevrigo_primitives::hash::Hash32;
use zevrigo_primitives::height::BlockHeight;

use super::{PowError, mine_regtest, regtest_pow_hash, verify_regtest_pow};
use crate::target::Target;

// Vektor V4 dari kartu TSK-M03:
// printf '%s' "$V4" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 regtest pow" --no-names
const P1_HASH_HEX: &str = "0364aacccf064eac20c82cf0bbb69da7bdfe2c85208267a85707aeaeb6475c17";
const P3_TARGET_HEX: &str = "0364aacccf064eac20c82cf0bbb69da7bdfe2c85208267a85707aeaeb6475c16";

// Nonce 3 di bawah T16:
// printf '%s' "$PFX$(printf '%02x%02x000000000000' 3 0)" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 regtest pow" --no-names
const P5_HASH_HEX: &str = "0d9248920ecaf57c4531d401a45d35ce21170b6e483ad891d8aeb7753139446c";

// Nonce 367 di bawah T256:
// printf '%s' "$PFX$(printf '%02x%02x000000000000' $((367 % 256)) $((367 / 256)))" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 regtest pow" --no-names
const P6_HASH_HEX: &str = "004d9338d441ce96b3cc233909cd1f80d1277612f53bea921482550e37d3d8d4";

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

fn target_t16() -> Target {
    let mut bytes = [0xff; 32];
    if let Some(first) = bytes.first_mut() {
        *first = 0x0f;
    }
    Target::from_be_bytes(bytes)
}

fn target_t256() -> Target {
    let mut bytes = [0xff; 32];
    if let Some(first) = bytes.first_mut() {
        *first = 0x00;
    }
    Target::from_be_bytes(bytes)
}

#[test]
fn vector_p1_regtest_pow_hash_of_v4() {
    let v4 = build_v4();
    let expected = Hash32::from_hex(P1_HASH_HEX).expect("P1 hex");
    assert_eq!(regtest_pow_hash(&v4), expected);
}

#[test]
fn vectors_p2_to_p4_verify_boundaries() {
    let v4 = build_v4();
    let p1_hash = Hash32::from_hex(P1_HASH_HEX).expect("P1 hex");
    let target_p1 = Target::from_be_bytes(*p1_hash.as_bytes());

    // P2: target == P1 -> Ok(())
    assert_eq!(verify_regtest_pow(&v4, &target_p1), Ok(()));

    // P3: target == P1 - 1 -> Err(PowError::AboveTarget)
    let p3_target_hash = Hash32::from_hex(P3_TARGET_HEX).expect("P3 target hex");
    let target_p3 = Target::from_be_bytes(*p3_target_hash.as_bytes());
    assert_eq!(
        verify_regtest_pow(&v4, &target_p3),
        Err(PowError::AboveTarget {
            pow_hash: p1_hash,
            target: target_p3,
        })
    );

    // P4: Target::MAX -> Ok(()), Target::ZERO -> Err
    assert_eq!(verify_regtest_pow(&v4, &Target::MAX), Ok(()));
    assert_eq!(
        verify_regtest_pow(&v4, &Target::ZERO),
        Err(PowError::AboveTarget {
            pow_hash: p1_hash,
            target: Target::ZERO,
        })
    );
}

#[test]
fn vectors_p5_to_p7_mine_regtest_first_nonce_and_attempt_limit() {
    let v4 = build_v4();
    let v4_nonce_0 = Header::new(
        v4.height(),
        v4.prev_block_id(),
        v4.merkle_root(),
        v4.timestamp(),
        v4.target(),
        0,
    );

    // P5: mine_regtest(V4 dengan nonce 0, T16, 1_000) -> Some, nonce 3
    let p5_res = mine_regtest(&v4_nonce_0, &target_t16(), 1_000).expect("P5 should succeed");
    assert_eq!(p5_res.nonce(), 3);
    assert_eq!(p5_res.height(), v4.height());
    assert_eq!(p5_res.prev_block_id(), v4.prev_block_id());
    assert_eq!(p5_res.merkle_root(), v4.merkle_root());
    assert_eq!(p5_res.timestamp(), v4.timestamp());
    assert_eq!(p5_res.target(), v4.target());
    let p5_hash = Hash32::from_hex(P5_HASH_HEX).expect("P5 hash hex");
    assert_eq!(regtest_pow_hash(&p5_res), p5_hash);

    // P6: mine_regtest(V4, T256, 1_000) -> Some, nonce 367
    let p6_res = mine_regtest(&v4, &target_t256(), 1_000).expect("P6 should succeed");
    assert_eq!(p6_res.nonce(), 367);
    assert_eq!(p6_res.height(), v4.height());
    assert_eq!(p6_res.prev_block_id(), v4.prev_block_id());
    assert_eq!(p6_res.merkle_root(), v4.merkle_root());
    assert_eq!(p6_res.timestamp(), v4.timestamp());
    assert_eq!(p6_res.target(), v4.target());
    let p6_hash = Hash32::from_hex(P6_HASH_HEX).expect("P6 hash hex");
    assert_eq!(regtest_pow_hash(&p6_res), p6_hash);

    // P7: mine_regtest(V4, T256, 325) -> None; mine_regtest(V4, T256, 326) -> Some(nonce 367)
    assert_eq!(mine_regtest(&v4, &target_t256(), 325), None);
    let p7_res =
        mine_regtest(&v4, &target_t256(), 326).expect("P7 with 326 attempts should succeed");
    assert_eq!(p7_res.nonce(), 367);
}

#[test]
fn vectors_p8_p9_p11_nonce_overflow_and_zero_attempts() {
    let v4 = build_v4();
    let v4_max_nonce = Header::new(
        v4.height(),
        v4.prev_block_id(),
        v4.merkle_root(),
        v4.timestamp(),
        v4.target(),
        u64::MAX,
    );

    // P8: mine_regtest(V4 dengan nonce u64::MAX, Target::ZERO, 10) -> None tanpa panic
    assert_eq!(mine_regtest(&v4_max_nonce, &Target::ZERO, 10), None);

    // P9: mine_regtest(V4 dengan nonce u64::MAX, Target::MAX, 1) -> Some, header sama persis
    let p9_res = mine_regtest(&v4_max_nonce, &Target::MAX, 1);
    assert_eq!(p9_res, Some(v4_max_nonce));

    // P11: mine_regtest(V4, Target::MAX, 0) -> None
    assert_eq!(mine_regtest(&v4, &Target::MAX, 0), None);
}

#[test]
fn vector_p10_regtest_pow_hash_differs_from_block_id() {
    let v4 = build_v4();
    let h_pow = regtest_pow_hash(&v4);
    let h_block = zevrigo_crypto::block_id(&v4);
    assert_ne!(h_pow, h_block);
    assert_eq!(
        h_block,
        Hash32::from_hex("1de077cc1a2c8ad7c055df207ce40cca9643f961d0df21c1c67c7f121d107cfa")
            .expect("H6 hex")
    );
}

proptest! {
    #[test]
    fn prop_verify_is_monotonic_in_target(
        height in any::<u64>(),
        prev in any::<[u8; 32]>(),
        merkle in any::<[u8; 32]>(),
        timestamp in any::<u64>(),
        compact in any::<u32>(),
        nonce in any::<u64>(),
        x in any::<[u8; 32]>(),
        y in any::<[u8; 32]>(),
    ) {
        let h = Header::new(
            BlockHeight::new(height),
            Hash32::from_bytes(prev),
            Hash32::from_bytes(merkle),
            timestamp,
            compact,
            nonce,
        );
        let (lo, hi) = if x <= y { (x, y) } else { (y, x) };
        if verify_regtest_pow(&h, &Target::from_be_bytes(lo)).is_ok() {
            prop_assert!(verify_regtest_pow(&h, &Target::from_be_bytes(hi)).is_ok());
        }
    }

    #[test]
    fn prop_mine_regtest_result_is_valid_and_only_nonce_changes(
        height in any::<u64>(),
        prev in any::<[u8; 32]>(),
        merkle in any::<[u8; 32]>(),
        timestamp in any::<u64>(),
        compact in any::<u32>(),
        nonce in any::<u64>(),
        t in any::<[u8; 32]>(),
        max_attempts in 0_u64..=64,
    ) {
        let h = Header::new(
            BlockHeight::new(height),
            Hash32::from_bytes(prev),
            Hash32::from_bytes(merkle),
            timestamp,
            compact,
            nonce,
        );
        if let Some(h2) = mine_regtest(&h, &Target::from_be_bytes(t), max_attempts) {
            prop_assert!(verify_regtest_pow(&h2, &Target::from_be_bytes(t)).is_ok());
            prop_assert!(h2.nonce().checked_sub(h.nonce()).is_some_and(|d| d < max_attempts));
            prop_assert_eq!(h2.height(), h.height());
            prop_assert_eq!(h2.prev_block_id(), h.prev_block_id());
            prop_assert_eq!(h2.merkle_root(), h.merkle_root());
            prop_assert_eq!(h2.timestamp(), h.timestamp());
            prop_assert_eq!(h2.target(), h.target());
        }
    }
}
