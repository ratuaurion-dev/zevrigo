use homeostasis_primitives::{Amount, BlockHeight};

use crate::{MODULE_ID, SupplySchedule, base_reward, block_subsidy, total_supply_after};

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
