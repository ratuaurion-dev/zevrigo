use zevrigo_primitives::block::Header;
use zevrigo_primitives::hash::Hash32;
use zevrigo_primitives::height::BlockHeight;

use crate::{MODULE_ID, PowError, Target, mine_regtest, regtest_pow_hash, verify_regtest_pow};

#[test]
fn module_id_matches() {
    assert_eq!(MODULE_ID, "MOD-03");
}

#[test]
fn crate_root_reexports_are_accessible() {
    let header = Header::new(
        BlockHeight::new(1),
        Hash32::ZERO,
        Hash32::ZERO,
        1_800_000_000,
        0x1d00_ffff,
        0,
    );
    let target = Target::MAX;
    let pow_hash = regtest_pow_hash(&header);
    assert_ne!(pow_hash, Hash32::ZERO);

    assert_eq!(verify_regtest_pow(&header, &target), Ok(()));
    assert!(mine_regtest(&header, &target, 1).is_some());

    let err = PowError::AboveTarget {
        pow_hash,
        target: Target::ZERO,
    };
    assert_eq!(format!("{err}"), "hash PoW regtest di atas target");
}
