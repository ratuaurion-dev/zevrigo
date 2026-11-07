use proptest::prelude::*;
use zevrigo_primitives::params::{
    GENESIS_ALLOCATION, HALF_LIFE_DIVISOR, MAIN_SUPPLY, SLOW_START_BLOCKS, TAIL_REWARD,
};
use zevrigo_primitives::{Amount, BlockHeight};

use super::{ScheduleEntry, SupplySchedule, base_reward, block_subsidy, total_supply_after};

/// Baris pertama vektor uji TSK-M07: `base_reward(0)` dalam quanta.
const EXPECTED_BASE_REWARD_AT_ZERO: u128 = 43_959_096_939_479_752_879;

/// Tinggi blok pertama saat `subsidy(h) == TAIL_REWARD` (awal pasokan ekor).
const FIRST_TAIL_REWARD_HEIGHT: u64 = 7_034_557;

/// Vektor uji TSK-M07 untuk tinggi `h <= 10_081`: `(height, subsidy, mined_total_after)`.
const EARLY_SUPPLY_VECTORS: [(u64, u128, u128); 6] = [
    (1, 4_361_021_521_773_785, 4_361_021_521_773_785),
    (2, 8_722_043_043_167_199, 13_083_064_564_940_984),
    (3, 13_083_064_563_609_689, 26_166_129_128_550_673),
    (
        10_079,
        43_857_489_210_501_099_091,
        221_286_700_668_594_180_324_737,
    ),
    (
        10_080,
        43_861_821_304_218_669_173,
        221_330_562_489_898_398_993_910,
    ),
    (
        10_081,
        43_861_802_022_958_122_630,
        221_374_424_291_921_357_116_540,
    ),
];

/// Seluruh 13 baris ber-tinggi pada tabel vektor uji TSK-M07 (termasuk tahun 1 sampai tahun 30).
const FULL_SUPPLY_VECTORS: [(u64, u128, u128); 13] = [
    (1, 4_361_021_521_773_785, 4_361_021_521_773_785),
    (2, 8_722_043_043_167_199, 13_083_064_564_940_984),
    (3, 13_083_064_563_609_689, 26_166_129_128_550_673),
    (
        10_079,
        43_857_489_210_501_099_091,
        221_286_700_668_594_180_324_737,
    ),
    (
        10_080,
        43_861_821_304_218_669_173,
        221_330_562_489_898_398_993_910,
    ),
    (
        10_081,
        43_861_802_022_958_122_630,
        221_374_424_291_921_357_116_540,
    ),
    (
        262_800,
        39_249_975_237_177_189_617,
        10_712_547_081_484_604_794_231_902,
    ),
    (
        1_576_800,
        22_028_303_574_022_054_017,
        49_889_111_869_368_096_617_665_101,
    ),
    (
        2_628_000,
        13_876_961_569_101_545_001,
        68_432_118_867_183_472_267_455_686,
    ),
    (
        5_256_000,
        4_370_968_907_532_367_991,
        90_056_740_719_420_160_465_206_925,
    ),
    (
        6_570_000,
        2_453_123_331_311_921_530,
        94_419_534_467_875_057_113_290_180,
    ),
    (
        6_832_800,
        2_185_484_426_813_902_955,
        95_028_370_421_022_234_186_635_652,
    ),
    (
        7_884_000,
        2_000_000_000_000_000_000,
        97_149_205_117_656_017_545_141_240,
    ),
];

#[test]
fn base_reward_at_zero_and_saturated_supply() {
    assert_eq!(
        base_reward(Amount::ZERO),
        Amount::from_quanta(EXPECTED_BASE_REWARD_AT_ZERO)
    );
    assert_eq!(base_reward(MAIN_SUPPLY), TAIL_REWARD);
    assert_eq!(base_reward(Amount::from_quanta(u128::MAX)), TAIL_REWARD);
}

#[test]
fn genesis_and_total_supply_after_boundaries() {
    assert_eq!(
        block_subsidy(BlockHeight::GENESIS, Amount::ZERO),
        Ok(Amount::ZERO)
    );
    assert_eq!(total_supply_after(Amount::ZERO), Some(GENESIS_ALLOCATION));

    let mut schedule = SupplySchedule::default();
    assert_eq!(
        schedule.next(),
        Some(ScheduleEntry {
            height: BlockHeight::GENESIS,
            subsidy: Amount::ZERO,
            mined_total_after: Amount::ZERO,
        })
    );

    assert_eq!(total_supply_after(Amount::from_quanta(u128::MAX)), None);
}

#[test]
fn early_supply_vectors_up_to_10_081_match_exact_quanta() {
    let mut checkpoint_iter = EARLY_SUPPLY_VECTORS.into_iter().peekable();

    for entry in SupplySchedule::new().take(10_082) {
        if let Some(&(target_h, expected_subsidy, expected_mined_after)) = checkpoint_iter.peek()
            && entry.height.get() == target_h
        {
            assert_eq!(
                entry.subsidy,
                Amount::from_quanta(expected_subsidy),
                "subsidi tidak cocok pada tinggi {target_h}"
            );
            assert_eq!(
                entry.mined_total_after,
                Amount::from_quanta(expected_mined_after),
                "mined_total_after tidak cocok pada tinggi {target_h}"
            );
            checkpoint_iter.next();
        }
    }

    assert_eq!(checkpoint_iter.next(), None);
}

#[test]
#[ignore = "simulasi 30 tahun (7_884_000 blok) dijalankan lewat make test-slow"]
fn slow_thirty_year_vectors_and_tail_onset_height_match_exact_quanta() {
    assert_eq!(
        base_reward(Amount::ZERO),
        Amount::from_quanta(EXPECTED_BASE_REWARD_AT_ZERO)
    );

    let mut checkpoint_iter = FULL_SUPPLY_VECTORS.into_iter().peekable();
    let mut first_tail_height: Option<u64> = None;
    let mut prev_mined = Amount::ZERO;

    for entry in SupplySchedule::new().take(7_884_001) {
        let h = entry.height.get();

        if h >= 1 && first_tail_height.is_none() && entry.subsidy == TAIL_REWARD {
            first_tail_height = Some(h);
        }

        if first_tail_height.is_some() {
            assert_eq!(
                entry.subsidy, TAIL_REWARD,
                "setelah awal pasokan ekor, subsidi wajib tepat TAIL_REWARD pada tinggi {h}"
            );
            assert_eq!(
                prev_mined.checked_add(TAIL_REWARD),
                Some(entry.mined_total_after),
                "setelah awal pasokan ekor, mined_total_after wajib bertambah tepat TAIL_REWARD pada tinggi {h}"
            );
        }

        if let Some(&(target_h, expected_subsidy, expected_mined_after)) = checkpoint_iter.peek()
            && h == target_h
        {
            assert_eq!(
                entry.subsidy,
                Amount::from_quanta(expected_subsidy),
                "subsidi tidak cocok pada tinggi {target_h}"
            );
            assert_eq!(
                entry.mined_total_after,
                Amount::from_quanta(expected_mined_after),
                "mined_total_after tidak cocok pada tinggi {target_h}"
            );
            checkpoint_iter.next();
        }

        prev_mined = entry.mined_total_after;
    }

    assert_eq!(checkpoint_iter.next(), None);
    assert_eq!(first_tail_height, Some(FIRST_TAIL_REWARD_HEIGHT));
}

proptest! {
    #[test]
    fn prop_base_reward_within_bounds(m in any::<u128>()) {
        let reward = base_reward(Amount::from_quanta(m));
        prop_assert!(reward >= TAIL_REWARD);
        prop_assert!(reward <= base_reward(Amount::ZERO));
    }

    #[test]
    fn prop_base_reward_monotonic_across_main_supply_and_full_u128(
        a_active in 0_u128..=MAIN_SUPPLY.as_quanta(),
        b_active in 0_u128..=MAIN_SUPPLY.as_quanta(),
        a_any in any::<u128>(),
        b_any in any::<u128>(),
    ) {
        let (lo_active, hi_active) = if a_active <= b_active {
            (a_active, b_active)
        } else {
            (b_active, a_active)
        };
        prop_assert!(
            base_reward(Amount::from_quanta(lo_active))
                >= base_reward(Amount::from_quanta(hi_active))
        );

        let (lo_any, hi_any) = if a_any <= b_any {
            (a_any, b_any)
        } else {
            (b_any, a_any)
        };
        prop_assert!(
            base_reward(Amount::from_quanta(lo_any)) >= base_reward(Amount::from_quanta(hi_any))
        );
    }

    #[test]
    fn prop_slow_start_and_post_slow_start_relations(
        m in 0_u128..=MAIN_SUPPLY.as_quanta(),
        slow_h in 1_u64..SLOW_START_BLOCKS,
        post_h in SLOW_START_BLOCKS..=1_000_000_000_u64,
    ) {
        let mined = Amount::from_quanta(m);
        let base = base_reward(mined);

        let slow_sub = block_subsidy(BlockHeight::new(slow_h), mined);
        prop_assert!(slow_sub.is_ok());
        if let Ok(sub) = slow_sub {
            prop_assert!(sub <= base);
        }

        let post_sub = block_subsidy(BlockHeight::new(post_h), mined);
        prop_assert_eq!(post_sub, Ok(base));
    }

    #[test]
    fn prop_block_subsidy_never_errors_or_panics_in_domain(
        m_domain in 0_u128..=MAIN_SUPPLY.as_quanta().saturating_add(1_000_000_000_000_000_000_000_000_000_000_u128),
        h_domain in 0_u64..=1_000_000_000_u64,
        m_any in any::<u128>(),
        h_any in any::<u64>(),
    ) {
        prop_assert!(block_subsidy(BlockHeight::new(h_domain), Amount::from_quanta(m_domain)).is_ok());
        prop_assert!(block_subsidy(BlockHeight::new(h_any), Amount::from_quanta(m_any)).is_ok());
    }

    #[test]
    fn prop_tail_emission_adds_exact_tail_reward_per_block(
        start_h in FIRST_TAIL_REWARD_HEIGHT..=1_000_000_000_u64,
        extra_m in 0_u128..=1_000_000_000_000_000_000_000_000_000_u128,
        steps in 1_usize..=64_usize,
    ) {
        // Ambang M saat (S - M) / D <= T pasti terpenuhi ketika M >= S - T * D.
        let tail_m_floor = MAIN_SUPPLY
            .as_quanta()
            .saturating_sub(TAIL_REWARD.as_quanta().saturating_mul(HALF_LIFE_DIVISOR));
        let start_mined = Amount::from_quanta(tail_m_floor.saturating_add(extra_m));

        let mut prev_mined = start_mined;
        let schedule = SupplySchedule::from_state(BlockHeight::new(start_h), start_mined);
        let mut count: usize = 0;

        for entry in schedule.take(steps) {
            prop_assert_eq!(entry.subsidy, TAIL_REWARD);
            prop_assert_eq!(prev_mined.checked_add(TAIL_REWARD), Some(entry.mined_total_after));
            prev_mined = entry.mined_total_after;
            count = count.saturating_add(1);
        }

        prop_assert_eq!(count, steps);
    }
}
