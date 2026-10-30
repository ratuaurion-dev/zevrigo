use homeostasis_primitives::{Amount, BlockHeight, Network};

use crate::{
    MODULE_ID, REGTEST_POW_LIMIT_COMPACT, SupplySchedule, U256, base_reward, block_subsidy,
    block_work, decode_compact, encode_compact, median_time_past, pow_limit, regtest_genesis,
    total_supply_after,
};

#[test]
fn module_id_matches() {
    assert_eq!(MODULE_ID, "MOD-04");
}

#[test]
fn crate_root_reexports_are_accessible() {
    assert_eq!(
        block_subsidy(BlockHeight::GENESIS, Amount::ZERO),
        Ok(Amount::ZERO)
    );
    assert!(base_reward(Amount::ZERO) > Amount::ZERO);
    assert!(total_supply_after(Amount::ZERO).is_some());
    assert!(SupplySchedule::new().next().is_some());
}

#[test]
fn difficulty_reexports_are_accessible() {
    let limit = pow_limit(Network::Regtest).unwrap();
    assert_eq!(decode_compact(REGTEST_POW_LIMIT_COMPACT), Ok(limit));
    assert_eq!(encode_compact(&limit), Ok(REGTEST_POW_LIMIT_COMPACT));
    assert_eq!(block_work(&limit), Some(U256::from_u128(2)));
    assert_eq!(median_time_past(&[1]), Some(1));
    assert!(regtest_genesis().is_ok());
}
