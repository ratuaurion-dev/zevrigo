use homeostasis_primitives::Hash32;
use proptest::prelude::*;

use super::Target;

fn be_from_u128(x: u128) -> [u8; 32] {
    let mut out = [0_u8; 32];
    for (dst, src) in out.iter_mut().skip(16).zip(x.to_be_bytes()) {
        *dst = src;
    }
    out
}

#[test]
fn zero_and_max_constants_and_is_met_by_extremes() {
    assert_eq!(Target::ZERO.to_be_bytes(), [0x00; 32]);
    assert_eq!(Target::MAX.to_be_bytes(), [0xff; 32]);

    let max_hash = Hash32::from_bytes([0xff; 32]);
    assert!(Target::MAX.is_met_by(&max_hash));
    assert!(Target::ZERO.is_met_by(&Hash32::ZERO));

    let mut non_zero_bytes = [0x00; 32];
    if let Some(last) = non_zero_bytes.last_mut() {
        *last = 0x01;
    }
    let non_zero_hash = Hash32::from_bytes(non_zero_bytes);
    assert!(!Target::ZERO.is_met_by(&non_zero_hash));
}

proptest! {
    #[test]
    fn prop_be_bytes_round_trip(bytes in any::<[u8; 32]>()) {
        prop_assert_eq!(Target::from_be_bytes(bytes).to_be_bytes(), bytes);
    }

    #[test]
    fn prop_ord_matches_numeric_big_endian(a in any::<u128>(), b in any::<u128>()) {
        prop_assert_eq!(Target::from_be_bytes(be_from_u128(a)).cmp(&Target::from_be_bytes(be_from_u128(b))), a.cmp(&b));
        prop_assert_eq!(Target::from_be_bytes(be_from_u128(b)).is_met_by(&Hash32::from_bytes(be_from_u128(a))), a <= b);
    }
}
