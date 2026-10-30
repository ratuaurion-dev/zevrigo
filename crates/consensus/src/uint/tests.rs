use proptest::prelude::*;

use super::U256;

/// 32 byte big-endian dengan satu byte `value` di posisi `index` (0 = paling signifikan).
fn be_with_byte(index: usize, value: u8) -> [u8; 32] {
    let mut out = [0_u8; 32];
    if let Some(byte) = out.get_mut(index) {
        *byte = value;
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

#[test]
fn u256_constants_bits_and_be_bytes() {
    assert_eq!(U256::ZERO.to_be_bytes(), [0_u8; 32]);
    assert_eq!(U256::MAX.to_be_bytes(), [0xff_u8; 32]);
    assert_eq!(U256::ONE, U256::from_u128(1));
    assert_eq!(U256::from_u128(0x0102).to_be_bytes(), be_from_u128(0x0102));
    let top = U256::from_be_bytes(be_with_byte(0, 0x80));
    assert_eq!(top.to_be_bytes(), be_with_byte(0, 0x80));
    assert_eq!(top.bits(), 256);
    assert_eq!(U256::ZERO.bits(), 0);
    assert_eq!(U256::ONE.bits(), 1);
    assert_eq!(U256::from_u128(u128::MAX).bits(), 128);
    assert!(U256::ZERO.is_zero());
    assert!(!U256::ONE.is_zero());
}

#[test]
fn u256_ordering_is_numeric() {
    let above_u128 = U256::from_be_bytes(be_with_byte(15, 0x01));
    assert!(above_u128 > U256::from_u128(u128::MAX));
    assert!(U256::MAX > above_u128);
    assert!(U256::ZERO < U256::ONE);
    let high_limb = U256::from_be_bytes(be_with_byte(0, 0x01));
    assert!(high_limb > U256::from_be_bytes(be_with_byte(8, 0xff)));
}

#[test]
fn u256_add_and_sub_detect_overflow() {
    let two_pow_128 = U256::from_be_bytes(be_with_byte(15, 0x01));
    assert_eq!(U256::MAX.checked_add(U256::ONE), None);
    assert_eq!(U256::MAX.checked_add(U256::ZERO), Some(U256::MAX));
    assert_eq!(
        U256::from_u128(u128::MAX).checked_add(U256::ONE),
        Some(two_pow_128)
    );
    assert_eq!(U256::ZERO.checked_sub(U256::ONE), None);
    assert_eq!(
        two_pow_128.checked_sub(U256::ONE),
        Some(U256::from_u128(u128::MAX))
    );
}

#[test]
fn u256_division_vectors() {
    let top = U256::from_be_bytes(be_with_byte(0, 0x80));
    assert_eq!(U256::ONE.checked_div(U256::ZERO), None);
    assert_eq!(U256::MAX.checked_div(U256::ONE), Some(U256::MAX));
    assert_eq!(U256::MAX.checked_div(U256::MAX), Some(U256::ONE));
    assert_eq!(U256::ZERO.checked_div(U256::MAX), Some(U256::ZERO));
    assert_eq!(U256::MAX.checked_div(top), Some(U256::ONE));
    assert_eq!(
        U256::MAX.checked_div(U256::from_u128(2)),
        top.checked_sub(U256::ONE)
    );
}

#[test]
fn u256_mul_shift_helpers() {
    let top = U256::from_be_bytes(be_with_byte(0, 0x80));
    assert_eq!(U256::MAX.mul_u64_shr(2, 1), Some(U256::MAX));
    assert_eq!(U256::MAX.mul_u64_shr(2, 0), None);
    assert_eq!(U256::MAX.mul_u64_shr(1, 300), Some(U256::ZERO));
    assert_eq!(U256::MAX.mul_u64_shr(1, u32::MAX), Some(U256::ZERO));
    assert_eq!(U256::ONE.checked_mul_u64_shl(1, 255), Some(top));
    assert_eq!(U256::ONE.checked_mul_u64_shl(2, 255), None);
    assert_eq!(U256::ONE.checked_mul_u64_shl(1, 256), None);
    assert_eq!(U256::ZERO.checked_mul_u64_shl(5, 1_000), Some(U256::ZERO));
}

proptest! {
    #[test]
    fn prop_u256_matches_u128(a in any::<u128>(), b in any::<u128>()) {
        let (ua, ub) = (U256::from_u128(a), U256::from_u128(b));
        prop_assert_eq!(ua.cmp(&ub), a.cmp(&b));
        prop_assert_eq!(ua.checked_sub(ub), a.checked_sub(b).map(U256::from_u128));
        prop_assert_eq!(ua.checked_div(ub), a.checked_div(b).map(U256::from_u128));
        prop_assert_eq!(ua.bits(), 128_u32.saturating_sub(a.leading_zeros()));
        let sum = ua.checked_add(ub);
        prop_assert!(sum.is_some());
        if let Some(exact) = a.checked_add(b) {
            prop_assert_eq!(sum, Some(U256::from_u128(exact)));
        }
    }

    #[test]
    fn prop_be_bytes_round_trip(bytes in any::<[u8; 32]>()) {
        prop_assert_eq!(U256::from_be_bytes(bytes).to_be_bytes(), bytes);
    }

    #[test]
    fn prop_mul_shift_matches_u128(a in any::<u64>(), m in any::<u64>(), n in 0_u32..64) {
        // a × m < 2^128, jadi perkalian u128 ini tidak pernah wrap.
        let product = u128::from(a).wrapping_mul(u128::from(m));
        let base = U256::from_u128(u128::from(a));
        prop_assert_eq!(base.mul_u64_shr(m, n), Some(U256::from_u128(product.wrapping_shr(n))));
        if product.leading_zeros() >= n {
            prop_assert_eq!(
                base.checked_mul_u64_shl(m, n),
                Some(U256::from_u128(product.wrapping_shl(n)))
            );
        }
    }

    #[test]
    fn prop_division_identity(a in any::<[u8; 32]>(), b in 1_u64..=u64::MAX) {
        let a = U256::from_be_bytes(a);
        let divisor = U256::from_u128(u128::from(b));
        let quotient = a.checked_div(divisor).unwrap();
        let product = quotient.checked_mul_u64_shl(b, 0).unwrap();
        prop_assert!(product <= a);
        let remainder = a.checked_sub(product).unwrap();
        prop_assert!(remainder < divisor);
    }
}
