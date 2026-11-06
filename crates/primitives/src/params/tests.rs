use crate::amount::{Amount, QUANTA_PER_ZVRG};

use super::Network;
use super::{
    ASERT_HALF_LIFE_BLOCKS, BLOCK_TIME_SECS, BLOCKS_PER_YEAR, COINBASE_MATURITY,
    GENESIS_ALLOCATION, HALF_LIFE_DIVISOR, MAIN_SUPPLY, MAX_BLOCK_SIZE, MAX_FUTURE_BLOCK_TIME_SECS,
    MEDIAN_TIME_SPAN, RANDOMX_KEY_INTERVAL, RANDOMX_KEY_LAG, SLOW_START_BLOCKS, TAIL_REWARD,
};

#[test]
fn protocol_parameters_match_source_of_truth() {
    assert_eq!(BLOCK_TIME_SECS, 120);
    assert_eq!(BLOCKS_PER_YEAR, 262_800);
    assert_eq!(
        MAIN_SUPPLY,
        Amount::from_quanta(43_800_000_000_000_000_000_000_000)
    );
    assert_eq!(MAIN_SUPPLY.to_zvrg_string(), "43800000");
    assert_eq!(
        GENESIS_ALLOCATION,
        Amount::from_quanta(4_380_000_000_000_000_000_000_000)
    );
    assert_eq!(GENESIS_ALLOCATION.to_zvrg_string(), "4380000");
    assert_eq!(HALF_LIFE_DIVISOR, 2_274_842);
    assert_eq!(TAIL_REWARD, Amount::from_quanta(QUANTA_PER_ZVRG));
    assert_eq!(TAIL_REWARD.to_zvrg_string(), "1");
    assert_eq!(SLOW_START_BLOCKS, 10_080);
    assert_eq!(COINBASE_MATURITY, 100);
    assert_eq!(MAX_BLOCK_SIZE, 100_000);
    assert_eq!(RANDOMX_KEY_INTERVAL, 2_048);
    assert_eq!(RANDOMX_KEY_LAG, 64);
    assert_eq!(ASERT_HALF_LIFE_BLOCKS, 720);
    assert_eq!(MEDIAN_TIME_SPAN, 11);
    assert_eq!(MAX_FUTURE_BLOCK_TIME_SECS, 3_600);
}

#[test]
fn supply_and_tail_reward_invariants_hold() {
    let annual_tail = MAIN_SUPPLY
        .as_quanta()
        .checked_mul(6)
        .and_then(|v| v.checked_div(1_000));
    let per_block_tail = annual_tail.and_then(|v| v.checked_div(u128::from(BLOCKS_PER_YEAR)));

    assert_eq!(per_block_tail, Some(TAIL_REWARD.as_quanta()));
    assert_eq!(
        MAIN_SUPPLY.as_quanta().checked_div(10),
        Some(GENESIS_ALLOCATION.as_quanta())
    );
    assert_eq!(TAIL_REWARD.as_quanta(), QUANTA_PER_ZVRG);
}

#[test]
fn network_variants_and_hrp_match_spec() {
    assert_eq!(
        Network::ALL,
        [Network::Mainnet, Network::Testnet, Network::Regtest]
    );
    assert_eq!(Network::Mainnet.hrp(), "zvrg");
    assert_eq!(Network::Testnet.hrp(), "tzvrg");
    assert_eq!(Network::Regtest.hrp(), "rzvrg");
}
