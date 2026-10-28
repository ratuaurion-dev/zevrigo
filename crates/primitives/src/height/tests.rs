use proptest::prelude::*;

use super::BlockHeight;

#[test]
fn genesis_and_basic_operations() {
    assert_eq!(BlockHeight::GENESIS.get(), 0);
    assert_eq!(BlockHeight::default(), BlockHeight::GENESIS);
    assert_eq!(BlockHeight::GENESIS.to_string(), "0");

    let h1 = BlockHeight::GENESIS.next();
    assert_eq!(h1, Some(BlockHeight::new(1)));

    let h10 = BlockHeight::new(10);
    assert_eq!(h10.checked_add(5), Some(BlockHeight::new(15)));
    assert_eq!(h10.checked_sub(4), Some(BlockHeight::new(6)));
    assert_eq!(h10.checked_sub(11), None);
    assert!(BlockHeight::GENESIS < h10);
}

#[test]
fn u64_boundary_operations() {
    assert_eq!(BlockHeight::GENESIS.checked_sub(1), None);

    let max = BlockHeight::new(u64::MAX);
    assert_eq!(max.next(), None);
    assert_eq!(max.checked_add(1), None);
    assert_eq!(max.checked_add(0), Some(max));
}

proptest! {
    #[test]
    fn prop_height_checked_ops_match_u64(h in any::<u64>(), delta in any::<u64>()) {
        let height = BlockHeight::new(h);
        prop_assert_eq!(height.checked_add(delta), h.checked_add(delta).map(BlockHeight::new));
        prop_assert_eq!(height.checked_sub(delta), h.checked_sub(delta).map(BlockHeight::new));
        prop_assert_eq!(height.next(), h.checked_add(1).map(BlockHeight::new));
    }
}
