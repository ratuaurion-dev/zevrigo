use proptest::prelude::*;

use super::{Amount, AmountParseError, QUANTA_PER_ZVRG};

#[test]
fn quanta_per_zvrg_is_ten_to_the_eighteenth() {
    assert_eq!(10_u128.checked_pow(18), Some(QUANTA_PER_ZVRG));
    assert_eq!(Amount::ZERO.as_quanta(), 0);
    assert_eq!(Amount::default(), Amount::ZERO);
}

#[test]
fn format_known_vectors() {
    assert_eq!(Amount::ZERO.to_zvrg_string(), "0");
    assert_eq!(Amount::from_quanta(QUANTA_PER_ZVRG).to_zvrg_string(), "1");
    assert_eq!(
        Amount::from_quanta(1_500_000_000_000_000_000).to_zvrg_string(),
        "1.5"
    );
    assert_eq!(
        Amount::from_quanta(1).to_zvrg_string(),
        "0.000000000000000001"
    );
    assert_eq!(
        Amount::from_quanta(12_300_000_000_000_000_000).to_zvrg_string(),
        "12.3"
    );

    assert_eq!(Amount::ZERO.to_string(), "0 ZVRG");
    assert_eq!(
        Amount::from_quanta(1_500_000_000_000_000_000).to_string(),
        "1.5 ZVRG"
    );
}

#[test]
fn parse_accepted_vectors() {
    let cases: [(&str, u128, &str); 6] = [
        ("0", 0, "0"),
        ("1", QUANTA_PER_ZVRG, "1"),
        ("1.5", 1_500_000_000_000_000_000, "1.5"),
        ("0.000000000000000001", 1, "0.000000000000000001"),
        ("12.30", 12_300_000_000_000_000_000, "12.3"),
        ("0.000000000000000000", 0, "0"),
    ];

    for (input, expected_quanta, canonical_str) in cases {
        let parsed = Amount::from_zvrg_str(input);
        assert_eq!(
            parsed,
            Ok(Amount::from_quanta(expected_quanta)),
            "input: {input}"
        );
        if let Ok(amount) = parsed {
            assert_eq!(amount.to_zvrg_string(), canonical_str, "input: {input}");
        }
    }
}

#[test]
fn parse_rejected_vectors() {
    let invalid_format_cases = [
        "", ".5", "5.", "01", "00", "00.5", "1e3", " 1", "1 ", "1,5", "-1", "+1", "1_000", "1.2.3",
        "1..5", "abc", "1.5a",
    ];

    for input in invalid_format_cases {
        assert_eq!(
            Amount::from_zvrg_str(input),
            Err(AmountParseError::InvalidFormat),
            "seharusnya ditolak sebagai InvalidFormat: {input:?}"
        );
    }

    assert_eq!(
        Amount::from_zvrg_str("1.0000000000000000001"),
        Err(AmountParseError::TooManyDecimals)
    );
    assert_eq!(
        Amount::from_zvrg_str("0.0000000000000000000"),
        Err(AmountParseError::TooManyDecimals)
    );

    assert_eq!(
        Amount::from_zvrg_str("340282366920938463463.374607431768211456"),
        Err(AmountParseError::Overflow)
    );
    assert_eq!(
        Amount::from_zvrg_str("340282366920938463464"),
        Err(AmountParseError::Overflow)
    );
    assert_eq!(
        Amount::from_zvrg_str("999999999999999999999999999999999999999999"),
        Err(AmountParseError::Overflow)
    );
}

#[test]
fn u128_max_round_trip() {
    let max_amount = Amount::from_quanta(u128::MAX);
    let text = max_amount.to_zvrg_string();
    assert_eq!(text, "340282366920938463463.374607431768211455");
    assert_eq!(Amount::from_zvrg_str(&text), Ok(max_amount));
}

#[test]
fn checked_and_saturating_operations() {
    let a = Amount::from_quanta(100);
    let b = Amount::from_quanta(40);

    assert_eq!(a.checked_add(b), Some(Amount::from_quanta(140)));
    assert_eq!(a.checked_sub(b), Some(Amount::from_quanta(60)));
    assert_eq!(b.checked_sub(a), None);
    assert_eq!(b.saturating_sub(a), Amount::ZERO);
    assert_eq!(a.saturating_sub(b), Amount::from_quanta(60));

    let max = Amount::from_quanta(u128::MAX);
    assert_eq!(max.checked_add(Amount::from_quanta(1)), None);
    assert_eq!(max.checked_add(Amount::ZERO), Some(max));

    assert_eq!(Amount::checked_sum([]), Some(Amount::ZERO));
    assert_eq!(
        Amount::checked_sum([
            Amount::from_quanta(10),
            Amount::from_quanta(20),
            Amount::from_quanta(30),
        ]),
        Some(Amount::from_quanta(60))
    );
    assert_eq!(Amount::checked_sum([max, Amount::from_quanta(1)]), None);
}

proptest! {
    #[test]
    fn prop_zvrg_string_round_trip(quanta in any::<u128>()) {
        let amount = Amount::from_quanta(quanta);
        let text = amount.to_zvrg_string();
        let reparsed = Amount::from_zvrg_str(&text);
        prop_assert_eq!(reparsed, Ok(amount));
    }

    #[test]
    fn prop_checked_add_matches_u128_overflow(a in any::<u128>(), b in any::<u128>()) {
        let lhs = Amount::from_quanta(a);
        let rhs = Amount::from_quanta(b);
        let expected = a.checked_add(b).map(Amount::from_quanta);
        prop_assert_eq!(lhs.checked_add(rhs), expected);

        // Uji juga pasangan komplementer dekat u128::MAX agar cabang None teruji rutin.
        let near_max = u128::MAX.saturating_sub(a);
        let near_lhs = Amount::from_quanta(near_max);
        let near_expected = near_max.checked_add(b).map(Amount::from_quanta);
        prop_assert_eq!(near_lhs.checked_add(rhs), near_expected);
    }

    #[test]
    fn prop_checked_and_saturating_sub_consistency(a in any::<u128>(), b in any::<u128>()) {
        let lhs = Amount::from_quanta(a);
        let rhs = Amount::from_quanta(b);
        prop_assert_eq!(lhs.checked_sub(rhs), a.checked_sub(b).map(Amount::from_quanta));
        prop_assert_eq!(lhs.saturating_sub(rhs), Amount::from_quanta(a.saturating_sub(b)));
    }
}
