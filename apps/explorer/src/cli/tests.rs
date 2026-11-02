use std::path::PathBuf;

use homeostasis_primitives::Network;

use super::{
    ArgError, Command, Config, USAGE, db_path, default_http_port, default_rpc_url, parse_args,
};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|arg| (*arg).to_owned()).collect()
}

fn run_config(datadir: &str, rpc_url: &str, http_port: u16) -> Command {
    Command::Run(Config {
        network: Network::Regtest,
        datadir: PathBuf::from(datadir),
        rpc_url: rpc_url.to_owned(),
        http_port,
        web_dir: None,
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
        Ok(run_config("/tmp/hsis", "http://127.0.0.1:28801", 28_802))
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
            "apps/explorer/web/dist",
            "--datadir",
            "data"
        ])),
        Ok(Command::Run(Config {
            network: Network::Regtest,
            datadir: PathBuf::from("data"),
            rpc_url: "http://127.0.0.1:28801".to_owned(),
            http_port: 28_802,
            web_dir: Some(PathBuf::from("apps/explorer/web/dist")),
        }))
    );
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
    assert!(USAGE.contains("--web-dir <folder>"));
}

#[test]
fn rejects_bad_arguments() {
    let cases: [(&[&str], ArgError); 14] = [
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
