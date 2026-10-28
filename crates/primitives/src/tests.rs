use crate::{Amount, BlockHeight, Hash32, MODULE_ID, QUANTA_PER_HSIS, params};

#[test]
fn module_id_matches() {
    assert_eq!(MODULE_ID, "MOD-01");
}

#[test]
fn crate_root_reexports_are_accessible() {
    assert_eq!(Amount::from_quanta(QUANTA_PER_HSIS), params::TAIL_REWARD);
    assert_eq!(BlockHeight::GENESIS.get(), 0);
    assert_eq!(Hash32::ZERO.as_bytes().len(), 32);
}
