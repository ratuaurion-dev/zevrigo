// Contoh alamat Spesifikasi Format v0.1 Bagian 8 (payload: kunci publik vektor BIP340 #0) dan vektor sah bech32m
// resmi BIP350 (https://github.com/bitcoin/bips/blob/master/bip-0350.mediawiki). Alamat salah (jenis 2, bit sisa
// tidak nol, data kurang satu kelompok) dibuat Quick dengan implementasi referensi Python (alat/wallet_ref.py)
// sehingga checksum-nya sah dan hanya aturan yang diuji yang dilanggar.

use proptest::prelude::*;
use zevrigo_primitives::{Network, OutputKind};

use super::{AddressError, CHARSET, PAYLOAD_LEN, decode_address, encode_address, verify_checksum};

const PAYLOAD_HEX: &str = "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9";
const MAINNET_0: &str = "hsis1qlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmusllxnc3";
const TESTNET_0: &str = "thsis1qlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmusq7jsl6";
const REGTEST_0: &str = "rhsis1qlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmuspj8z0e";
const MAINNET_1: &str = "hsis1plycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmusq5kk90";
const KIND_2: &str = "hsis1zlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmusgf0ety";
const BAD_PADDING: &str = "hsis1qlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmu3zfjx9r";
const ONE_GROUP_SHORT: &str = "hsis1qlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmuyt9w0r";

fn decode_hex(hex: &str) -> Vec<u8> {
    let (pairs, rem) = hex.as_bytes().as_chunks::<2>();
    assert!(rem.is_empty(), "panjang string hex harus genap");
    let mut out = Vec::with_capacity(pairs.len());
    for &[hi, lo] in pairs {
        let hi_val = char::from(hi).to_digit(16).expect("hex nibble valid");
        let lo_val = char::from(lo).to_digit(16).expect("hex nibble valid");
        let byte = u8::try_from(hi_val.wrapping_shl(4) | lo_val).expect("byte muat u8");
        out.push(byte);
    }
    out
}

fn payload() -> [u8; PAYLOAD_LEN] {
    decode_hex(PAYLOAD_HEX).try_into().expect("32 byte")
}

fn to_values(data_part: &str) -> Vec<u8> {
    data_part
        .bytes()
        .map(|symbol| {
            let index = CHARSET
                .iter()
                .position(|&candidate| candidate == symbol)
                .expect("karakter bech32");
            u8::try_from(index).expect("indeks < 32")
        })
        .collect()
}

#[test]
fn format_spec_examples_round_trip() {
    let examples = [
        (Network::Mainnet, OutputKind::SingleKey, MAINNET_0, 64_usize),
        (Network::Testnet, OutputKind::SingleKey, TESTNET_0, 65),
        (Network::Regtest, OutputKind::SingleKey, REGTEST_0, 65),
        (Network::Mainnet, OutputKind::ConditionHash, MAINNET_1, 64),
    ];
    for (network, kind, address, len) in examples {
        assert_eq!(encode_address(network, kind, &payload()), address);
        assert_eq!(address.len(), len);
        assert_eq!(decode_address(network, address), Ok((kind, payload())));
    }
}

#[test]
fn bip350_checksum_constant_vectors() {
    assert!(verify_checksum("a", &to_values("lqfn3a")));
    assert!(verify_checksum(
        "abcdef",
        &to_values("l7aum6echk45nj3s0wdvt2fg8x9yrzpqzd3ryx")
    ));
    // Checksum bech32 lama (BIP173, konstanta 1) tidak lolos sebagai bech32m.
    assert!(!verify_checksum("a", &to_values("2uel5l")));
}

#[test]
fn decode_accepts_uppercase_and_rejects_mixed_case() {
    let upper = MAINNET_0.to_ascii_uppercase();
    assert_eq!(
        decode_address(Network::Mainnet, &upper),
        Ok((OutputKind::SingleKey, payload()))
    );
    let mixed = MAINNET_0.replacen("hsis", "ZVRG", 1);
    assert_eq!(
        decode_address(Network::Mainnet, &mixed),
        Err(AddressError::MixedCase)
    );
}

#[test]
fn decode_rejects_wrong_network_separator_length_and_characters() {
    assert_eq!(
        decode_address(Network::Mainnet, REGTEST_0),
        Err(AddressError::WrongNetwork)
    );
    assert_eq!(
        decode_address(Network::Regtest, TESTNET_0),
        Err(AddressError::WrongNetwork)
    );
    assert_eq!(
        decode_address(Network::Mainnet, "hsisqlycg5qvjtrp"),
        Err(AddressError::MissingSeparator)
    );
    assert_eq!(
        decode_address(Network::Mainnet, ""),
        Err(AddressError::MissingSeparator)
    );
    let truncated = MAINNET_0.get(..63).expect("panjang 64");
    assert_eq!(
        decode_address(Network::Mainnet, truncated),
        Err(AddressError::InvalidLength)
    );
    assert_eq!(
        decode_address(Network::Mainnet, ONE_GROUP_SHORT),
        Err(AddressError::InvalidLength)
    );
    // "b", "i", dan "o" tidak ada di himpunan karakter bech32.
    for symbol in ["b", "i", "o"] {
        let bad = MAINNET_0.replacen("lycg", &format!("{symbol}ycg"), 1);
        assert_eq!(
            decode_address(Network::Mainnet, &bad),
            Err(AddressError::InvalidCharacter)
        );
    }
}

#[test]
fn decode_rejects_bad_checksum_kind_and_padding() {
    let flipped = MAINNET_0.replacen("llxnc3", "llxnc4", 1);
    assert_eq!(
        decode_address(Network::Mainnet, &flipped),
        Err(AddressError::InvalidChecksum)
    );
    let swapped = MAINNET_0.replacen("lycg", "ylcg", 1);
    assert_eq!(
        decode_address(Network::Mainnet, &swapped),
        Err(AddressError::InvalidChecksum)
    );
    assert_eq!(
        decode_address(Network::Mainnet, KIND_2),
        Err(AddressError::UnknownKind { kind: 2 })
    );
    assert_eq!(
        decode_address(Network::Mainnet, BAD_PADDING),
        Err(AddressError::InvalidPadding)
    );
}

proptest! {
    #[test]
    fn prop_round_trip_all_networks_and_kinds(
        bytes in any::<[u8; PAYLOAD_LEN]>(),
        network in prop_oneof![
            Just(Network::Mainnet),
            Just(Network::Testnet),
            Just(Network::Regtest),
        ],
        kind in prop_oneof![Just(OutputKind::SingleKey), Just(OutputKind::ConditionHash)],
    ) {
        let address = encode_address(network, kind, &bytes);
        prop_assert!(address.starts_with(network.hrp()));
        prop_assert_eq!(decode_address(network, &address), Ok((kind, bytes)));
        prop_assert_eq!(decode_address(network, &address.to_ascii_uppercase()), Ok((kind, bytes)));
    }
}
