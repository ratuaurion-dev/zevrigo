use std::path::PathBuf;

use homeostasis_primitives::{Amount, Network};

use super::{Action, ArgError, Command, Config, USAGE, default_rpc_url, parse_args};

const ADDRESS: &str = "rhsis1qtfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939msequa3ft3";

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|arg| (*arg).to_owned()).collect()
}

fn run(list: &[&str]) -> Config {
    match parse_args(args(list)) {
        Ok(Command::Run(config)) => config,
        other => unreachable!("hasil bukan Run: {other:?}"),
    }
}

#[test]
fn parses_help_version_and_info() {
    assert_eq!(parse_args(args(&[])), Ok(Command::Help));
    assert_eq!(parse_args(args(&["--help"])), Ok(Command::Help));
    assert_eq!(parse_args(args(&["-h"])), Ok(Command::Help));
    assert_eq!(parse_args(args(&["--version"])), Ok(Command::Version));
    assert_eq!(default_rpc_url(Network::Regtest), "http://127.0.0.1:28801");
    assert_eq!(default_rpc_url(Network::Testnet), "http://127.0.0.1:18801");
    assert_eq!(
        run(&["info"]),
        Config {
            network: Network::Regtest,
            rpc_url: "http://127.0.0.1:28801".to_owned(),
            datadir: None,
            passphrase: String::new(),
            action: Action::Info,
        }
    );
    assert_eq!(
        run(&["info", "--rpc-url", "http://127.0.0.1:9"]).rpc_url,
        "http://127.0.0.1:9"
    );
    assert!(USAGE.contains("wallet-restore --mnemonic"));
    assert!(USAGE.contains("wallet-encrypt"));
    assert!(USAGE.contains("wallet-show-phrase"));
    assert!(USAGE.contains("HOMEOSTASIS_WALLET_PASSWORD"));
}

#[test]
fn parses_wallet_actions_with_options_in_any_order() {
    let create = run(&["--datadir", "/tmp/w", "wallet-create"]);
    assert_eq!(create.datadir, Some(PathBuf::from("/tmp/w")));
    assert_eq!(create.action, Action::WalletCreate);

    let restore = run(&[
        "wallet-restore",
        "--mnemonic",
        "a b c",
        "--datadir",
        "/tmp/w",
    ]);
    assert_eq!(
        restore.action,
        Action::WalletRestore {
            mnemonic: "a b c".to_owned()
        }
    );

    let address = run(&["address", "--datadir", "/tmp/w", "--passphrase", "TREZOR"]);
    assert_eq!(address.action, Action::Address);
    assert_eq!(address.passphrase, "TREZOR");
    assert_eq!(run(&["--datadir", "d", "balance"]).action, Action::Balance);
    assert_eq!(
        run(&["wallet-encrypt", "--datadir", "d"]).action,
        Action::WalletEncrypt
    );
    assert_eq!(
        run(&["--datadir", "d", "wallet-show-phrase"]).action,
        Action::WalletShowPhrase
    );
    assert_eq!(
        format!("{:?}", Action::WalletShowPhrase),
        "wallet-show-phrase"
    );

    let send = run(&["send", "--to", ADDRESS, "--amount", "1.5", "--datadir", "d"]);
    assert_eq!(
        send.action,
        Action::Send {
            to: ADDRESS.to_owned(),
            amount: Amount::from_quanta(1_500_000_000_000_000_000),
            fee_rate: 1,
        }
    );
    let with_rate = run(&[
        "--fee-rate",
        "0",
        "--datadir",
        "d",
        "send",
        "--amount",
        "0.000000000000000001",
        "--to",
        ADDRESS,
    ]);
    assert_eq!(
        with_rate.action,
        Action::Send {
            to: ADDRESS.to_owned(),
            amount: Amount::from_quanta(1),
            fee_rate: 0,
        }
    );
}

#[test]
fn rejects_bad_arguments() {
    let cases: [(&[&str], ArgError); 14] = [
        (
            &["--tidak-dikenal"],
            ArgError::Unknown("--tidak-dikenal".to_owned()),
        ),
        (&["mine"], ArgError::Unknown("mine".to_owned())),
        (&["--datadir", "d"], ArgError::MissingAction),
        (
            &["info", "balance"],
            ArgError::ExtraAction("balance".to_owned()),
        ),
        (&["info", "--help"], ArgError::Exclusive),
        (
            &["balance", "--datadir"],
            ArgError::MissingValue("--datadir"),
        ),
        (
            &["balance", "--datadir", "--to"],
            ArgError::MissingValue("--datadir"),
        ),
        (
            &["balance", "--datadir", "a", "--datadir", "b"],
            ArgError::Duplicate("--datadir"),
        ),
        (
            &["balance"],
            ArgError::MissingOption {
                option: "--datadir",
                action: "balance",
            },
        ),
        (
            &["info", "--datadir", "d"],
            ArgError::NotFor {
                option: "--datadir",
                action: "info",
            },
        ),
        (
            &["balance", "--datadir", "d", "--to", ADDRESS],
            ArgError::NotFor {
                option: "--to",
                action: "balance",
            },
        ),
        (
            &["wallet-restore", "--datadir", "d"],
            ArgError::MissingOption {
                option: "--mnemonic",
                action: "wallet-restore",
            },
        ),
        (
            &["send", "--datadir", "d", "--to", ADDRESS],
            ArgError::MissingOption {
                option: "--amount",
                action: "send",
            },
        ),
        (
            &["send", "--datadir", "d", "--fee-rate", "-1"],
            ArgError::InvalidFeeRate("-1".to_owned()),
        ),
    ];
    for (list, expected) in cases {
        assert_eq!(parse_args(args(list)), Err(expected), "{list:?}");
    }
    for amount in ["0", "1.", "abc", "1.0000000000000000001"] {
        assert_eq!(
            parse_args(args(&["send", "--amount", amount])),
            Err(ArgError::InvalidAmount(amount.to_owned())),
            "{amount}"
        );
    }
}

#[test]
fn debug_hides_passphrase_and_mnemonic() {
    let config = run(&[
        "wallet-restore",
        "--mnemonic",
        "kata rahasia",
        "--passphrase",
        "sandi rahasia",
        "--datadir",
        "d",
    ]);
    let text = format!("{config:?}");
    assert!(!text.contains("kata rahasia"), "{text}");
    assert!(!text.contains("sandi rahasia"), "{text}");
    assert!(text.contains("<rahasia>"), "{text}");
}

#[test]
fn parses_network_options() {
    let info = run(&["--testnet", "info"]);
    assert_eq!(info.network, Network::Testnet);
    assert_eq!(info.rpc_url, "http://127.0.0.1:18801");
    let custom = run(&["info", "--rpc-url", "http://127.0.0.1:9", "--testnet"]);
    assert_eq!(custom.rpc_url, "http://127.0.0.1:9");
    let address = run(&["address", "--datadir", "d", "--testnet"]);
    assert_eq!(address.network, Network::Testnet);
    assert_eq!(run(&["--regtest", "info"]).network, Network::Regtest);
    assert_eq!(run(&["info"]).network, Network::Regtest);
    assert_eq!(
        parse_args(args(&["--testnet", "--testnet", "info"])),
        Err(ArgError::Duplicate("--testnet"))
    );
    assert_eq!(
        parse_args(args(&["--regtest", "info", "--testnet"])),
        Err(ArgError::NetworkConflict)
    );
    assert!(USAGE.contains("--testnet"));
    assert!(format!("{address:?}").contains("Testnet"));
}
