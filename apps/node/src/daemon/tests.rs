use std::path::PathBuf;

use zevrigo_network::Multiaddr;
use zevrigo_primitives::Network;

use super::{
    ArgError, Command, Config, USAGE, chain_path, default_p2p_port, p2p_key_path,
    p2p_listen_address, parse_args,
};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|arg| (*arg).to_owned()).collect()
}

fn run_config(datadir: &str, rpc_port: u16) -> Command {
    Command::Run(Config {
        network: Network::Regtest,
        datadir: PathBuf::from(datadir),
        rpc_port,
        p2p_port: 28_800,
        connect: Vec::new(),
    })
}

#[test]
fn parses_help_version_and_run() {
    assert_eq!(parse_args(args(&[])), Ok(Command::Help));
    assert_eq!(parse_args(args(&["--help"])), Ok(Command::Help));
    assert_eq!(parse_args(args(&["-h"])), Ok(Command::Help));
    assert_eq!(parse_args(args(&["--version"])), Ok(Command::Version));
    assert_eq!(
        parse_args(args(&["--regtest", "--datadir", "/tmp/zvrg"])),
        Ok(run_config("/tmp/zvrg", 28_801))
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
            &["--testnet", "--regtest", "--datadir", "x"],
            ArgError::NetworkConflict,
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

#[test]
fn parses_p2p_options() {
    let first = "/ip4/127.0.0.1/tcp/28800";
    let second = "/ip4/10.0.0.2/tcp/1/p2p/12D3KooWQK1wnefoLrcVHbbnf5tLzbopUd3K3bFAoJpA7YJgL5pV";
    let Ok(Command::Run(config)) = parse_args(args(&[
        "--regtest",
        "--datadir",
        "d",
        "--p2p-port",
        "0",
        "--connect",
        first,
        "--connect",
        second,
    ])) else {
        unreachable!("argumen sah");
    };
    assert_eq!(config.p2p_port, 0);
    assert_eq!(
        config.connect,
        vec![
            first.parse::<Multiaddr>().unwrap(),
            second.parse::<Multiaddr>().unwrap()
        ]
    );
    assert_eq!(
        p2p_key_path(&config),
        PathBuf::from("d").join("regtest").join("p2p_key")
    );
    assert_eq!(
        p2p_listen_address(28_800).unwrap(),
        first.parse::<Multiaddr>().unwrap()
    );
    assert_eq!(default_p2p_port(Network::Regtest), 28_800);
    assert_eq!(default_p2p_port(Network::Testnet), 18_800);
    assert_eq!(default_p2p_port(Network::Mainnet), 8_800);
    assert!(USAGE.contains("--p2p-port"));
    assert!(USAGE.contains("--connect"));

    let cases: [(&[&str], ArgError); 4] = [
        (
            &["--regtest", "--datadir", "x", "--p2p-port", "70000"],
            ArgError::InvalidP2pPort("70000".to_owned()),
        ),
        (
            &[
                "--regtest",
                "--datadir",
                "x",
                "--p2p-port",
                "1",
                "--p2p-port",
                "2",
            ],
            ArgError::Duplicate("--p2p-port"),
        ),
        (
            &[
                "--regtest",
                "--datadir",
                "x",
                "--connect",
                "127.0.0.1:28800",
            ],
            ArgError::InvalidAddress("127.0.0.1:28800".to_owned()),
        ),
        (
            &["--regtest", "--datadir", "x", "--connect"],
            ArgError::MissingValue("--connect"),
        ),
    ];
    for (list, expected) in cases {
        assert_eq!(parse_args(args(list)), Err(expected), "{list:?}");
    }
}

#[test]
fn parses_testnet() {
    let Ok(Command::Run(config)) = parse_args(args(&["--testnet", "--datadir", "d"])) else {
        unreachable!("argumen sah");
    };
    assert_eq!(
        config,
        Config {
            network: Network::Testnet,
            datadir: PathBuf::from("d"),
            rpc_port: 18_801,
            p2p_port: 18_800,
            connect: Vec::new(),
        }
    );
    assert_eq!(
        chain_path(&config),
        PathBuf::from("d").join("testnet").join("chain.redb")
    );
    assert_eq!(
        p2p_key_path(&config),
        PathBuf::from("d").join("testnet").join("p2p_key")
    );
    assert_eq!(
        parse_args(args(&["--testnet", "--testnet", "--datadir", "d"])),
        Err(ArgError::Duplicate("--testnet"))
    );
    assert_eq!(
        parse_args(args(&["--regtest", "--testnet", "--datadir", "d"])),
        Err(ArgError::NetworkConflict)
    );
    assert!(USAGE.contains("--testnet"));
    assert_eq!(
        ArgError::MissingNetwork.to_string(),
        "jaringan wajib dipilih: --regtest atau --testnet"
    );
}
