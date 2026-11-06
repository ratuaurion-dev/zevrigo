use proptest::prelude::*;
use zevrigo_primitives::amount::Amount;
use zevrigo_primitives::hash::Hash32;
use zevrigo_primitives::params::Network;
use zevrigo_primitives::tx::OutputKind;

use super::{
    CompactHex, DecimalU64, Hex32, HexBytes, KindNumber, NetworkName, Quanta, ValueError,
    decode_hex, encode_hex,
};

const H11: &str = "1111111111111111111111111111111111111111111111111111111111111111";

#[test]
fn hex32_accepts_lowercase_and_rejects_bad_input() {
    let v = Hex32::try_from(H11).expect("huruf kecil");
    assert_eq!(v.bytes(), &[0x11_u8; 32]);
    assert_eq!(v.to_hash32(), Hash32::from_bytes([0x11; 32]));
    assert_eq!(String::from(v), H11);
    let quoted = format!("\"{H11}\"");
    assert_eq!(serde_json::to_string(&v).expect("json"), quoted);
    assert_eq!(serde_json::from_str::<Hex32>(&quoted).expect("json"), v);

    let upper = "AB".repeat(32);
    assert_eq!(
        Hex32::try_from(upper.as_str()),
        Err(ValueError::HexChar { index: 0 })
    );
    let short = H11.get(1..).expect("potong");
    assert_eq!(
        Hex32::try_from(short),
        Err(ValueError::HexLength {
            expected: 64,
            actual: 63
        })
    );
    let long = format!("{H11}1");
    assert_eq!(
        Hex32::try_from(long.as_str()),
        Err(ValueError::HexLength {
            expected: 64,
            actual: 65
        })
    );
    let bad_char = format!("{}g", "1".repeat(63));
    assert_eq!(
        Hex32::try_from(bad_char.as_str()),
        Err(ValueError::HexChar { index: 63 })
    );
    assert!(serde_json::from_str::<Hex32>("11").is_err());
}

#[test]
fn hex_bytes_rules() {
    assert_eq!(decode_hex(""), Ok(Vec::new()));
    assert_eq!(decode_hex("00ff7a"), Ok(vec![0x00, 0xff, 0x7a]));
    assert_eq!(encode_hex(&[0x00, 0xff, 0x7a]), "00ff7a");
    assert_eq!(
        decode_hex("abc"),
        Err(ValueError::HexOddLength { actual: 3 })
    );
    assert_eq!(decode_hex("00FF"), Err(ValueError::HexChar { index: 2 }));
    assert_eq!(decode_hex("0x00"), Err(ValueError::HexChar { index: 1 }));

    let v: HexBytes = serde_json::from_str("\"0100\"").expect("json");
    assert_eq!(v.as_slice(), &[0x01_u8, 0x00]);
    assert_eq!(serde_json::to_string(&v).expect("json"), "\"0100\"");
    assert_eq!(HexBytes::try_from("0100"), Ok(v.clone()));
    assert_eq!(v.into_vec(), vec![0x01_u8, 0x00]);
    assert!(serde_json::from_str::<HexBytes>("\"010\"").is_err());
    assert!(serde_json::from_str::<HexBytes>("[1,0]").is_err());
}

#[test]
fn quanta_string_canonical_rules() {
    let cases = [
        ("0", 0_u128),
        ("1", 1),
        ("1910127426536917", 1_910_127_426_536_917),
        ("340282366920938463463374607431768211455", u128::MAX),
    ];
    for (text, q) in cases {
        let v = Quanta::try_from(text).expect("kanonis");
        assert_eq!(v.amount(), Amount::from_quanta(q));
        assert_eq!(String::from(v), text);
    }
    for text in [
        "", "00", "01", "+1", "-1", " 1", "1 ", "1.5", "1e3", "1_000", "x",
    ] {
        assert_eq!(Quanta::try_from(text), Err(ValueError::Decimal), "{text}");
    }
    assert_eq!(
        Quanta::try_from("340282366920938463463374607431768211456"),
        Err(ValueError::DecimalOverflow)
    );

    let five = Quanta::new(Amount::from_quanta(5));
    assert_eq!(serde_json::to_string(&five).expect("json"), "\"5\"");
    assert_eq!(serde_json::from_str::<Quanta>("\"5\"").expect("json"), five);
    assert!(serde_json::from_str::<Quanta>("5").is_err());
    assert_eq!(Quanta::from(Amount::from_quanta(5)), five);
}

#[test]
fn compact_hex_and_decimal_u64_rules() {
    let t = CompactHex::try_from("207fffff").expect("compact");
    assert_eq!(t.get(), 0x207f_ffff);
    assert_eq!(String::from(CompactHex::new(0x0100_0000)), "01000000");
    assert_eq!(
        CompactHex::try_from("207FFFFF"),
        Err(ValueError::HexChar { index: 3 })
    );
    assert_eq!(
        CompactHex::try_from("7fffff"),
        Err(ValueError::HexLength {
            expected: 8,
            actual: 6
        })
    );
    assert!(serde_json::from_str::<CompactHex>("545259519").is_err());

    assert_eq!(DecimalU64::try_from("0").map(DecimalU64::get), Ok(0));
    assert_eq!(
        DecimalU64::try_from("18446744073709551615").map(DecimalU64::get),
        Ok(u64::MAX)
    );
    assert_eq!(
        DecimalU64::try_from("18446744073709551616"),
        Err(ValueError::DecimalOverflow)
    );
    assert_eq!(DecimalU64::try_from("007"), Err(ValueError::Decimal));
    assert_eq!(
        String::from(DecimalU64::new(u64::MAX)),
        "18446744073709551615"
    );
    assert!(serde_json::from_str::<DecimalU64>("0").is_err());
}

#[test]
fn network_and_output_kind_encoding() {
    let cases = [
        ("mainnet", Network::Mainnet),
        ("testnet", Network::Testnet),
        ("regtest", Network::Regtest),
    ];
    for (name, network) in cases {
        let v = NetworkName::try_from(name).expect("jaringan");
        assert_eq!(v.network(), network);
        assert_eq!(String::from(NetworkName::new(network)), name);
    }
    assert_eq!(
        NetworkName::try_from("Regtest"),
        Err(ValueError::UnknownNetwork)
    );
    assert_eq!(
        serde_json::to_string(&NetworkName::new(Network::Regtest)).expect("json"),
        "\"regtest\""
    );

    assert_eq!(
        serde_json::to_string(&KindNumber::new(OutputKind::ConditionHash)).expect("json"),
        "1"
    );
    assert_eq!(
        serde_json::from_str::<KindNumber>("0")
            .expect("json")
            .kind(),
        OutputKind::SingleKey
    );
    assert_eq!(
        KindNumber::try_from(2_u8),
        Err(ValueError::UnknownOutputKind { actual: 2 })
    );
    assert!(serde_json::from_str::<KindNumber>("\"0\"").is_err());
    assert!(serde_json::from_str::<KindNumber>("256").is_err());
}

proptest! {
    #[test]
    fn prop_quanta_round_trip(q in any::<u128>()) {
        let text = String::from(Quanta::new(Amount::from_quanta(q)));
        prop_assert_eq!(&text, &q.to_string());
        prop_assert_eq!(
            Quanta::try_from(text.as_str()).map(Quanta::amount),
            Ok(Amount::from_quanta(q))
        );
    }

    #[test]
    fn prop_hex32_round_trip(bytes in any::<[u8; 32]>()) {
        let text = String::from(Hex32::new(bytes));
        prop_assert_eq!(Hex32::try_from(text.as_str()), Ok(Hex32::new(bytes)));
        prop_assert_eq!(text, Hash32::from_bytes(bytes).to_string());
    }

    #[test]
    fn prop_hex_bytes_round_trip(bytes in proptest::collection::vec(any::<u8>(), 0..=64)) {
        prop_assert_eq!(decode_hex(&encode_hex(&bytes)), Ok(bytes));
    }
}
