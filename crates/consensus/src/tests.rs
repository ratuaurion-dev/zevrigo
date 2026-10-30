use std::collections::BTreeMap;

use homeostasis_primitives::{Amount, BlockHeight, Network, OutPoint, UtxoEntry};

use crate::{
    BlockError, ChainParams, MODULE_ID, REGTEST_POW_LIMIT_COMPACT, SupplySchedule, TxError, U256,
    UtxoLookup, base_reward, block_subsidy, block_work, check_block_transactions,
    check_transaction, decode_compact, encode_compact, median_time_past, pow_limit,
    regtest_genesis, total_supply_after,
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

#[test]
fn validation_reexports_are_accessible() {
    let params = ChainParams::regtest().unwrap();
    let genesis = regtest_genesis().unwrap();
    let utxos: BTreeMap<OutPoint, UtxoEntry> = BTreeMap::new();
    assert_eq!(utxos.lookup(&OutPoint::new(params.network_id(), 0)), None);
    assert_eq!(
        check_block_transactions(&genesis, &params, Amount::ZERO, &utxos),
        Err(BlockError::Genesis)
    );
    let coinbase = genesis.txs().first().unwrap();
    assert_eq!(
        check_transaction(coinbase, BlockHeight::new(1), &params.network_id(), &utxos),
        Err(TxError::NotRegular)
    );
}
