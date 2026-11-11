use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;

use zevrigo_primitives::Network;

use super::{
    ArgError, Command, Config, DEFAULT_API_RATE, USAGE, db_path, default_http_port,
    default_rpc_url, parse_args,
};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|arg| (*arg).to_owned()).collect()
}

fn run_config(datadir: &str, rpc_url: &str, http_port: u16) -> Command {
    Command::Run(Config {
        network: Network::Regtest,
        datadir: PathBuf::from(datadir),
        rpc_url: rpc_url.to_owned(),
        http_bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
        http_port,
        api_rate: 10,
        web_dir: None,
        trust_proxy: false,
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
        Ok(run_config("/tmp/zvrg", "http://127.0.0.1:28801", 28_802))
    );
    assert_eq!(
        parse_args(args(&[
            "--http-port",
            "0",
            "--rpc-url",
            "http://127.0.0.1:5000",
            "--datadir",
            "data",
            "--regtest"
        ])),
        Ok(run_config("data", "http://127.0.0.1:5000", 0))
    );
    assert_eq!(
        parse_args(args(&[
            "--regtest",
            "--web-dir",
            "apps/explorer/web/out",
            "--datadir",
            "data"
        ])),
        Ok(Command::Run(Config {
            network: Network::Regtest,
            datadir: PathBuf::from("data"),
            rpc_url: "http://127.0.0.1:28801".to_owned(),
            http_bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            http_port: 28_802,
            api_rate: DEFAULT_API_RATE,
            web_dir: Some(PathBuf::from("apps/explorer/web/out")),
            trust_proxy: false,
        }))
    );
    for (text, address) in [
        ("0.0.0.0", IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        ("::", IpAddr::V6(Ipv6Addr::UNSPECIFIED)),
        ("::1", IpAddr::V6(Ipv6Addr::LOCALHOST)),
    ] {
        let Ok(Command::Run(config)) = parse_args(args(&[
            "--regtest",
            "--http-bind",
            text,
            "--datadir",
            "data",
        ])) else {
            unreachable!("argumen sah: {text}");
        };
        assert_eq!(config.http_bind, address, "{text}");
        assert_eq!(config.http_port, 28_802, "{text}");
    }
    for (text, rate) in [("0", 0), ("25", 25), ("4294967295", u32::MAX)] {
        let Ok(Command::Run(config)) = parse_args(args(&[
            "--regtest",
            "--datadir",
            "data",
            "--api-rate",
            text,
        ])) else {
            unreachable!("argumen sah: {text}");
        };
        assert_eq!(config.api_rate, rate, "{text}");
    }
    assert_eq!(default_rpc_url(Network::Regtest), "http://127.0.0.1:28801");
    assert_eq!(default_http_port(Network::Regtest), 28_802);
    assert_eq!(default_http_port(Network::Testnet), 18_802);
    assert_eq!(default_http_port(Network::Mainnet), 8_802);
    let Ok(Command::Run(config)) = parse_args(args(&["--regtest", "--datadir", "data"])) else {
        unreachable!("argumen sah");
    };
    assert_eq!(
        db_path(&config),
        PathBuf::from("data")
            .join("regtest")
            .join("explorer.sqlite")
    );
    assert!(USAGE.contains("MOD-14"));
    assert!(USAGE.contains("--rpc-url"));
    assert!(USAGE.contains("--http-port"));
    assert!(USAGE.contains("--http-bind <ip>"));
    assert!(USAGE.contains("--api-rate <n>"));
    assert!(USAGE.contains("--web-dir <folder>"));
}

#[test]
fn rejects_bad_arguments() {
    let cases: [(&[&str], ArgError); 20] = [
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
            &["--regtest", "--datadir", "x", "--rpc-url", "--http-port"],
            ArgError::MissingValue("--rpc-url"),
        ),
        (
            &["--regtest", "--datadir", "x", "--http-port", "70000"],
            ArgError::InvalidPort("70000".to_owned()),
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
                "--rpc-url",
                "a",
                "--rpc-url",
                "b",
            ],
            ArgError::Duplicate("--rpc-url"),
        ),
        (
            &["--regtest", "--datadir", "x", "--web-dir"],
            ArgError::MissingValue("--web-dir"),
        ),
        (
            &[
                "--regtest",
                "--datadir",
                "x",
                "--web-dir",
                "a",
                "--web-dir",
                "b",
            ],
            ArgError::Duplicate("--web-dir"),
        ),
        (
            &["--regtest", "--datadir", "x", "--http-bind", "localhost"],
            ArgError::InvalidBind("localhost".to_owned()),
        ),
        (
            &["--regtest", "--datadir", "x", "--http-bind"],
            ArgError::MissingValue("--http-bind"),
        ),
        (
            &[
                "--regtest",
                "--datadir",
                "x",
                "--http-bind",
                "0.0.0.0",
                "--http-bind",
                "::",
            ],
            ArgError::Duplicate("--http-bind"),
        ),
        (
            &["--regtest", "--datadir", "x", "--api-rate", "4294967296"],
            ArgError::InvalidRate("4294967296".to_owned()),
        ),
        (
            &["--regtest", "--datadir", "x", "--api-rate"],
            ArgError::MissingValue("--api-rate"),
        ),
        (
            &[
                "--regtest",
                "--datadir",
                "x",
                "--api-rate",
                "1",
                "--api-rate",
                "2",
            ],
            ArgError::Duplicate("--api-rate"),
        ),
        (&["--regtest", "--version"], ArgError::Exclusive),
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
fn parses_testnet() {
    let Ok(Command::Run(config)) = parse_args(args(&["--testnet", "--datadir", "d"])) else {
        unreachable!("argumen sah");
    };
    assert_eq!(
        config,
        Config {
            network: Network::Testnet,
            datadir: PathBuf::from("d"),
            rpc_url: "http://127.0.0.1:18801".to_owned(),
            http_bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            http_port: 18_802,
            api_rate: DEFAULT_API_RATE,
            web_dir: None,
            trust_proxy: false,
        }
    );
    assert_eq!(default_rpc_url(Network::Testnet), "http://127.0.0.1:18801");
    assert_eq!(
        db_path(&config),
        PathBuf::from("d").join("testnet").join("explorer.sqlite")
    );
    assert_eq!(
        parse_args(args(&["--testnet", "--testnet", "--datadir", "d"])),
        Err(ArgError::Duplicate("--testnet"))
    );
    assert!(USAGE.contains("--testnet"));
}

#[test]
fn parses_trust_proxy() {
    let Ok(Command::Run(config)) = parse_args(args(&[
        "--testnet",
        "--datadir",
        "d",
        "--http-bind",
        "0.0.0.0",
        "--trust-proxy",
    ])) else {
        unreachable!("argumen sah");
    };
    assert!(config.trust_proxy);
    assert_eq!(config.http_bind, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    let Ok(Command::Run(config)) = parse_args(args(&["--testnet", "--datadir", "d"])) else {
        unreachable!("argumen sah");
    };
    assert!(!config.trust_proxy);
    assert_eq!(
        parse_args(args(&[
            "--regtest",
            "--datadir",
            "d",
            "--trust-proxy",
            "--trust-proxy"
        ])),
        Err(ArgError::Duplicate("--trust-proxy"))
    );
    assert!(USAGE.contains("--trust-proxy"));
    assert!(USAGE.contains("CF-Connecting-IP"));
}
