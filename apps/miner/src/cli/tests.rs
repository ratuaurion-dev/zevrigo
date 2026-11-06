use zevrigo_primitives::{Network, OutputKind};
use zevrigo_rpc::{Hex32, KindNumber};

use super::{ArgError, Command, MinerConfig, USAGE, default_rpc_url, parse_args};

const KEY: &str = "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|arg| (*arg).to_owned()).collect()
}

fn key() -> Hex32 {
    Hex32::try_from(KEY).unwrap()
}

#[test]
fn parses_help_version_and_run() {
    assert_eq!(parse_args(args(&[])), Ok(Command::Help));
    assert_eq!(parse_args(args(&["--help"])), Ok(Command::Help));
    assert_eq!(parse_args(args(&["--version"])), Ok(Command::Version));
    assert_eq!(default_rpc_url(Network::Regtest), "http://127.0.0.1:28801");
    assert_eq!(default_rpc_url(Network::Testnet), "http://127.0.0.1:18801");
    assert_eq!(
        parse_args(args(&["--payout", KEY])),
        Ok(Command::Run(MinerConfig {
            network: Network::Regtest,
            light: false,
            rpc_url: "http://127.0.0.1:28801".to_owned(),
            payout_kind: KindNumber::new(OutputKind::SingleKey),
            payout_payload: key(),
            blocks: 0,
        }))
    );
    assert_eq!(
        parse_args(args(&[
            "--blocks",
            "5",
            "--rpc-url",
            "http://127.0.0.1:9",
            "--payout-kind",
            "1",
            "--payout",
            KEY,
        ])),
        Ok(Command::Run(MinerConfig {
            network: Network::Regtest,
            light: false,
            rpc_url: "http://127.0.0.1:9".to_owned(),
            payout_kind: KindNumber::new(OutputKind::ConditionHash),
            payout_payload: key(),
            blocks: 5,
        }))
    );
    assert!(USAGE.contains("MOD-13"));
    assert!(USAGE.contains("--payout"));
}

#[test]
fn rejects_bad_arguments() {
    let upper = KEY.to_uppercase();
    let short = &KEY[..62];
    let cases: [(Vec<&str>, ArgError); 13] = [
        (vec!["--blocks", "1"], ArgError::MissingPayout),
        (vec!["--payout"], ArgError::MissingValue("--payout")),
        (
            vec!["--payout", upper.as_str()],
            ArgError::InvalidPayout(upper.clone()),
        ),
        (
            vec!["--payout", short],
            ArgError::InvalidPayout(short.to_owned()),
        ),
        (
            vec!["--payout", KEY, "--payout-kind", "2"],
            ArgError::InvalidKind("2".to_owned()),
        ),
        (
            vec!["--payout", KEY, "--blocks", "-1"],
            ArgError::InvalidBlocks("-1".to_owned()),
        ),
        (
            vec!["--payout", KEY, "--blocks", "x"],
            ArgError::InvalidBlocks("x".to_owned()),
        ),
        (
            vec!["--payout", KEY, "--payout", KEY],
            ArgError::Duplicate("--payout"),
        ),
        (vec!["--payout", KEY, "--version"], ArgError::Exclusive),
        (
            vec!["--payout", KEY, "--mainnet"],
            ArgError::Unknown("--mainnet".to_owned()),
        ),
        (
            vec!["--payout", KEY, "--testnet", "--regtest"],
            ArgError::NetworkConflict,
        ),
        (
            vec!["--payout", KEY, "--light"],
            ArgError::LightWithoutRandomx,
        ),
        (
            vec!["--testnet", "--light", "--light", "--payout", KEY],
            ArgError::Duplicate("--light"),
        ),
    ];
    for (list, expected) in cases {
        assert_eq!(parse_args(args(&list)), Err(expected), "{list:?}");
    }
}

#[test]
fn parses_testnet_and_light() {
    assert_eq!(
        parse_args(args(&["--testnet", "--payout", KEY])),
        Ok(Command::Run(MinerConfig {
            network: Network::Testnet,
            light: false,
            rpc_url: "http://127.0.0.1:18801".to_owned(),
            payout_kind: KindNumber::new(OutputKind::SingleKey),
            payout_payload: key(),
            blocks: 0,
        }))
    );
    assert_eq!(
        parse_args(args(&[
            "--payout",
            KEY,
            "--light",
            "--testnet",
            "--blocks",
            "2"
        ])),
        Ok(Command::Run(MinerConfig {
            network: Network::Testnet,
            light: true,
            rpc_url: "http://127.0.0.1:18801".to_owned(),
            payout_kind: KindNumber::new(OutputKind::SingleKey),
            payout_payload: key(),
            blocks: 2,
        }))
    );
    assert_eq!(
        parse_args(args(&["--regtest", "--payout", KEY])),
        parse_args(args(&["--payout", KEY]))
    );
    assert!(USAGE.contains("--testnet"));
    assert!(USAGE.contains("--light"));
}
