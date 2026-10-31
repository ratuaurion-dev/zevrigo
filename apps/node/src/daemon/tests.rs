use std::path::PathBuf;

use homeostasis_primitives::Network;

use super::{ArgError, Command, Config, USAGE, chain_path, parse_args};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|arg| (*arg).to_owned()).collect()
}

fn run_config(datadir: &str, rpc_port: u16) -> Command {
    Command::Run(Config {
        network: Network::Regtest,
        datadir: PathBuf::from(datadir),
        rpc_port,
    })
}

#[test]
fn parses_help_version_and_run() {
    assert_eq!(parse_args(args(&[])), Ok(Command::Help));
    assert_eq!(parse_args(args(&["--help"])), Ok(Command::Help));
    assert_eq!(parse_args(args(&["-h"])), Ok(Command::Help));
    assert_eq!(parse_args(args(&["--version"])), Ok(Command::Version));
    assert_eq!(
        parse_args(args(&["--regtest", "--datadir", "/tmp/hsis"])),
        Ok(run_config("/tmp/hsis", 28_801))
    );
    assert_eq!(
        parse_args(args(&["--rpc-port", "0", "--datadir", "data", "--regtest"])),
        Ok(run_config("data", 0))
    );
    assert!(USAGE.contains("--datadir"));
    assert!(USAGE.contains("--regtest"));
    assert!(USAGE.contains("MOD-11"));
}

#[test]
fn rejects_bad_arguments() {
    let cases: [(&[&str], ArgError); 13] = [
        (&["--datadir", "x"], ArgError::MissingNetwork),
        (&["--regtest"], ArgError::MissingDatadir),
        (
            &["--testnet", "--datadir", "x"],
            ArgError::NetworkUnavailable("testnet"),
        ),
        (&["--mainnet"], ArgError::NetworkUnavailable("mainnet")),
        (
            &["--regtest", "--datadir"],
            ArgError::MissingValue("--datadir"),
        ),
        (
            &["--regtest", "--datadir", "--rpc-port"],
            ArgError::MissingValue("--datadir"),
        ),
        (
            &["--regtest", "--datadir", "x", "--rpc-port", "65536"],
            ArgError::InvalidPort("65536".to_owned()),
        ),
        (
            &["--regtest", "--datadir", "x", "--rpc-port", "abc"],
            ArgError::InvalidPort("abc".to_owned()),
        ),
        (
            &["--regtest", "--regtest", "--datadir", "x"],
            ArgError::Duplicate("--regtest"),
        ),
        (
            &["--regtest", "--datadir", "x", "--datadir", "y"],
            ArgError::Duplicate("--datadir"),
        ),
        (
            &[
                "--regtest",
                "--datadir",
                "x",
                "--rpc-port",
                "1",
                "--rpc-port",
                "2",
            ],
            ArgError::Duplicate("--rpc-port"),
        ),
        (&["--regtest", "--help"], ArgError::Exclusive),
        (
            &["--regtest", "--datadir", "x", "--foo"],
            ArgError::Unknown("--foo".to_owned()),
        ),
    ];
    for (list, expected) in cases {
        assert_eq!(parse_args(args(list)), Err(expected), "{list:?}");
    }
}

#[test]
fn chain_file_lives_under_network_folder() {
    let Ok(Command::Run(config)) = parse_args(args(&["--regtest", "--datadir", "data"])) else {
        unreachable!("argumen sah");
    };
    assert_eq!(
        chain_path(&config),
        PathBuf::from("data").join("regtest").join("chain.redb")
    );
}
