// Uji perintah wallet desktop tanpa node: nama jaringan, jumlah quanta, kode galat, bentuk JSON untuk antarmuka, dan
// alur file wallet di folder sementara (frasa W1 dan alamat W5, W6: Spesifikasi Wallet v0.1 Bagian 4). Argon2id memakai
// parameter penuh seperti aplikasi.

use std::path::PathBuf;

use zevrigo_cli::{Balance, CliError, History, HistoryEntry};
use zevrigo_primitives::{Amount, Network};
use zevrigo_rpc::Hex32;
use zevrigo_wallet_core::{Mnemonic, SendError};

use super::{
    BalanceView, CommandError, HistoryView, error_code, new_address_in, new_phrase_words,
    parse_network, parse_quanta, receive_address_in, restore_in, reveal_in, wallet_info_in,
};

const W1: &str = concat!(
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ",
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
);
const W5: &str = "rzvrg1qtfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939mseq59qlxg";
const W6: &str = "rzvrg1q3uesn8pvcdeeufxr98v794e07l8ghd85fyd3wclknj5xj04rfqrqyeht59";
const PASSWORD: &str = "kata sandi uji";

fn temp_datadir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zevrigo-wallet-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn code<T>(result: Result<T, CommandError>) -> Option<&'static str> {
    result.err().map(|error| error.code)
}

#[test]
fn parses_only_testnet_and_regtest() {
    assert_eq!(parse_network("testnet"), Ok(Network::Testnet));
    assert_eq!(parse_network("regtest"), Ok(Network::Regtest));
    for name in ["mainnet", "Testnet", "", "regtest "] {
        assert_eq!(code(parse_network(name)), Some("network"), "{name:?}");
    }
}

#[test]
fn parses_quanta_text_without_fractions() {
    assert_eq!(parse_quanta("1"), Ok(Amount::from_quanta(1)));
    assert_eq!(
        parse_quanta("1000000000000000000"),
        Ok(Amount::from_quanta(1_000_000_000_000_000_000))
    );
    assert_eq!(
        parse_quanta("340282366920938463463374607431768211455"),
        Ok(Amount::from_quanta(u128::MAX))
    );
    for text in [
        "",
        "0",
        "00",
        "01",
        "1.5",
        "-1",
        "+1",
        " 1",
        "1e3",
        "340282366920938463463374607431768211456",
    ] {
        assert_eq!(code(parse_quanta(text)), Some("bad_amount"), "{text:?}");
    }
}

#[test]
fn maps_cli_errors_to_fixed_codes() {
    let cases = [
        (CliError::WrongPassword, "wrong_password"),
        (CliError::PasswordTooShort, "password_too_short"),
        (CliError::PasswordMismatch, "password_mismatch"),
        (
            CliError::Rpc {
                message: String::from("koneksi ditolak"),
            },
            "node_unreachable",
        ),
        (
            CliError::WrongNetwork {
                expected: String::from("testnet"),
                actual: String::from("regtest"),
            },
            "wrong_network",
        ),
        (
            CliError::Rejected {
                reason: String::from("double-spend"),
            },
            "rejected",
        ),
        (
            CliError::Send(SendError::InsufficientFunds {
                available: Amount::from_quanta(5),
                needed: Amount::from_quanta(9),
            }),
            "insufficient_funds",
        ),
        (CliError::Send(SendError::ZeroAmount), "bad_amount"),
        (CliError::Send(SendError::Signing), "send_failed"),
        (CliError::ChainChanged, "chain_changed"),
        (CliError::PassphraseMismatch, "bad_phrase"),
        (CliError::IndexExhausted, "failed"),
    ];
    for (error, expected) in cases {
        assert_eq!(error_code(&error), expected, "{error}");
        let message = error.to_string();
        let converted = CommandError::from(error);
        assert_eq!(converted.code, expected);
        assert_eq!(converted.message, message);
    }
    let json = serde_json::to_string(&CommandError::from(CliError::WrongPassword));
    assert_eq!(
        json.ok().as_deref(),
        Some(r#"{"code":"wrong_password","message":"password salah atau file wallet rusak"}"#)
    );
}

#[test]
fn balance_view_sends_quanta_as_text() {
    let balance = Balance {
        tip_height: 120,
        total: Amount::from_quanta(43_959_096_939_479_752_879),
        spendable: Amount::from_quanta(2_000_000_000_000_000_000),
        immature: Amount::from_quanta(41_959_096_939_479_752_879),
        pending_spent: Amount::ZERO,
        pending_received: Amount::from_quanta(7),
        after_pending: Amount::from_quanta(43_959_096_939_479_752_886),
        utxos: 3,
    };
    let json = serde_json::to_value(BalanceView::from(&balance)).ok();
    let expected = serde_json::json!({
        "tipHeight": 120,
        "total": "43959096939479752879",
        "spendable": "2000000000000000000",
        "immature": "41959096939479752879",
        "pendingSpent": "0",
        "pendingReceived": "7",
        "afterPending": "43959096939479752886",
        "utxos": 3,
    });
    assert_eq!(json, Some(expected));
}

#[test]
fn history_view_keeps_pending_entries_and_fees() {
    let history = History {
        tip_height: 9,
        entries: vec![
            HistoryEntry {
                txid: Hex32::new([0xab; 32]),
                height: None,
                timestamp: None,
                coinbase: false,
                received: Amount::from_quanta(40),
                spent: Amount::from_quanta(100),
                fee: None,
            },
            HistoryEntry {
                txid: Hex32::new([0x01; 32]),
                height: Some(9),
                timestamp: Some(1_788_000_000),
                coinbase: true,
                received: Amount::from_quanta(4_361_021_521_773_785),
                spent: Amount::ZERO,
                fee: Some(Amount::from_quanta(300)),
            },
        ],
    };
    let json = serde_json::to_value(HistoryView::from(&history)).ok();
    let expected = serde_json::json!({
        "tipHeight": 9,
        "entries": [
            {
                "txid": "ab".repeat(32),
                "height": null,
                "timestamp": null,
                "coinbase": false,
                "received": "40",
                "spent": "100",
                "fee": null,
            },
            {
                "txid": "01".repeat(32),
                "height": 9,
                "timestamp": 1_788_000_000_u64,
                "coinbase": true,
                "received": "4361021521773785",
                "spent": "0",
                "fee": "300",
            },
        ],
    });
    assert_eq!(json, Some(expected));
}

#[test]
fn new_phrase_has_24_valid_words_and_is_fresh() {
    let first = new_phrase_words();
    let second = new_phrase_words();
    assert!(first.is_ok() && second.is_ok());
    let (first, second) = (first.unwrap_or_default(), second.unwrap_or_default());
    assert_ne!(first, second);
    for phrase in [&first, &second] {
        assert_eq!(phrase.split(' ').count(), 24);
        assert!(Mnemonic::parse(phrase).is_ok());
    }
}

#[test]
fn restores_wallet_and_keeps_last_receive_address() {
    let datadir = temp_datadir("pulih");
    let info = wallet_info_in(&datadir, Network::Regtest);
    assert!(!info.exists);
    assert_eq!(
        info.default_rpc_url,
        zevrigo_cli::default_rpc_url(Network::Regtest)
    );
    assert!(info.path.ends_with("wallet.json"));

    assert_eq!(
        code(restore_in(
            &datadir,
            Network::Regtest,
            W1,
            PASSWORD,
            "kata sandi lain"
        )),
        Some("password_mismatch")
    );
    assert_eq!(
        code(restore_in(
            &datadir,
            Network::Regtest,
            W1,
            "pendek",
            "pendek"
        )),
        Some("password_too_short")
    );
    assert_eq!(
        code(restore_in(
            &datadir,
            Network::Regtest,
            "abandon art",
            PASSWORD,
            PASSWORD
        )),
        Some("bad_phrase")
    );
    assert!(!wallet_info_in(&datadir, Network::Regtest).exists);

    assert_eq!(
        restore_in(&datadir, Network::Regtest, W1, PASSWORD, PASSWORD),
        Ok(String::from(W5))
    );
    assert!(wallet_info_in(&datadir, Network::Regtest).exists);
    assert!(!wallet_info_in(&datadir, Network::Testnet).exists);
    assert_eq!(
        receive_address_in(&datadir, Network::Regtest),
        Ok(String::from(W5))
    );
    assert_eq!(
        new_address_in(&datadir, Network::Regtest),
        Ok(String::from(W6))
    );
    assert_eq!(
        receive_address_in(&datadir, Network::Regtest),
        Ok(String::from(W6))
    );

    assert_eq!(
        reveal_in(&datadir, Network::Regtest, PASSWORD),
        Ok(String::from(W1))
    );
    assert_eq!(
        code(reveal_in(&datadir, Network::Regtest, "kata sandi salah")),
        Some("wrong_password")
    );
    assert_eq!(
        code(restore_in(
            &datadir,
            Network::Regtest,
            W1,
            PASSWORD,
            PASSWORD
        )),
        Some("wallet_exists")
    );
    assert_eq!(
        code(receive_address_in(&datadir, Network::Testnet)),
        Some("wallet_missing")
    );
    let _ = std::fs::remove_dir_all(&datadir);
}
