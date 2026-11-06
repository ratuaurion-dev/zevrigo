use proptest::prelude::*;
use zevrigo_primitives::params::{
    GENESIS_ALLOCATION, HALF_LIFE_DIVISOR, MAIN_SUPPLY, SLOW_START_BLOCKS, TAIL_REWARD,
};
use zevrigo_primitives::{Amount, BlockHeight};

use super::{ScheduleEntry, SupplySchedule, base_reward, block_subsidy, total_supply_after};

/// Baris pertama vektor uji TSK-M07: `base_reward(0)` dalam quanta.
const EXPECTED_BASE_REWARD_AT_ZERO: u128 = 19_254_084_459_492_131_761;

/// Tinggi blok pertama saat `subsidy(h) == TAIL_REWARD` (awal pasokan ekor).
const FIRST_TAIL_REWARD_HEIGHT: u64 = 6_733_393;

/// Vektor uji TSK-M07 untuk tinggi `h <= 10_081`: `(height, subsidy, mined_total_after)`.
const EARLY_SUPPLY_VECTORS: [(u64, u128, u128); 6] = [
    (1, 1_910_127_426_536_917, 1_910_127_426_536_917),
    (2, 3_820_254_852_907_233, 5_730_382_279_444_150),
    (3, 5_730_382_278_861_043, 11_460_764_558_305_193),
    (
        10_079,
        19_209_580_274_199_481_401,
        96_923_574_892_844_250_978_088,
    ),
    (
        10_080,
        19_211_477_731_247_777_097,
        96_942_786_370_575_498_755_185,
    ),
    (
        10_081,
        19_211_469_286_055_657_712,
        96_961_997_839_861_554_412_897,
    ),
];

/// Seluruh 13 baris ber-tinggi pada tabel vektor uji TSK-M07 (termasuk tahun 1 sampai tahun 30).
const FULL_SUPPLY_VECTORS: [(u64, u128, u128); 13] = [
    (1, 1_910_127_426_536_917, 1_910_127_426_536_917),
    (2, 3_820_254_852_907_233, 5_730_382_279_444_150),
    (3, 5_730_382_278_861_043, 11_460_764_558_305_193),
    (
        10_079,
        19_209_580_274_199_481_401,
        96_923_574_892_844_250_978_088,
    ),
    (
        10_080,
        19_211_477_731_247_777_097,
        96_942_786_370_575_498_755_185,
    ),
    (
        10_081,
        19_211_469_286_055_657_712,
        96_961_997_839_861_554_412_897,
    ),
    (
        262_800,
        17_191_489_153_883_609_052,
        4_692_095_621_690_256_899_802_646,
    ),
    (
        1_576_800,
        9_648_396_965_421_659_659,
        21_851_430_998_783_226_318_217_064,
    ),
    (
        2_628_000,
        6_078_109_167_266_476_710,
        29_973_268_063_826_360_852_707_222,
    ),
    (
        5_256_000,
        1_914_484_381_499_177_180,
        39_444_852_435_106_030_283_184_651,
    ),
    (
        6_570_000,
        1_074_468_019_114_621_630,
        41_355_756_096_929_275_015_017_236,
    ),
    (
        6_832_800,
        1_000_000_000_000_000_000,
        41_624_566_977_945_720_492_858_347,
    ),
    (
        7_884_000,
        1_000_000_000_000_000_000,
        42_675_766_977_945_720_492_858_347,
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
