use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::{
    ArgError, Command, DEFAULT_NODE_URL, DEFAULT_RATE, DEFAULT_RPC_PORT, RpcConfig, USAGE,
    parse_args,
};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|arg| (*arg).to_owned()).collect()
}

fn defaults() -> RpcConfig {
    RpcConfig {
        node_url: "http://127.0.0.1:28801".to_owned(),
        http_bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
        http_port: 28_804,
        rate: 20,
        trust_proxy: false,
    }
}

#[test]
fn parses_help_version_and_rpc_mode() {
    assert_eq!(parse_args(args(&[])), Ok(Command::Help));
    assert_eq!(parse_args(args(&["--help"])), Ok(Command::Help));
    assert_eq!(parse_args(args(&["-h"])), Ok(Command::Help));
    assert_eq!(parse_args(args(&["--version"])), Ok(Command::Version));
    assert_eq!(parse_args(args(&["rpc"])), Ok(Command::Rpc(defaults())));
    assert_eq!(
        (DEFAULT_NODE_URL, DEFAULT_RPC_PORT, DEFAULT_RATE),
        ("http://127.0.0.1:28801", 28_804, 20)
    );
    assert_eq!(
        parse_args(args(&[
            "rpc",
            "--trust-proxy",
            "--rate",
            "0",
            "--http-port",
            "0",
            "--http-bind",
            "::",
            "--node-url",
            "http://127.0.0.1:5000",
        ])),
        Ok(Command::Rpc(RpcConfig {
            node_url: "http://127.0.0.1:5000".to_owned(),
            http_bind: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
            http_port: 0,
            rate: 0,
            trust_proxy: true,
        }))
    );
    for word in [
        "get_block_template",
        "submit_block",
        "--trust-proxy",
        "--rate",
        "faucet",
    ] {
        assert!(USAGE.contains(word), "{word}");
    }
}

#[test]
fn rejects_bad_arguments() {
    let cases: Vec<(Vec<String>, ArgError)> = vec![
        (args(&["faucet"]), ArgError::NotYet("faucet")),
        (args(&["daftar", "--rate", "1"]), ArgError::NotYet("daftar")),
        (
            args(&["dompet"]),
            ArgError::UnknownMode("dompet".to_owned()),
        ),
        (args(&["--node-url", "x"]), ArgError::MissingMode),
        (args(&["--help", "rpc"]), ArgError::Exclusive),
        (args(&["rpc", "--version"]), ArgError::Exclusive),
        (args(&["rpc", "--rate"]), ArgError::MissingValue("--rate")),
        (
            args(&["rpc", "--node-url", "--trust-proxy"]),
            ArgError::MissingValue("--node-url"),
        ),
        (
            args(&["rpc", "--http-bind", ""]),
            ArgError::MissingValue("--http-bind"),
        ),
        (
            args(&["rpc", "--trust-proxy", "--trust-proxy"]),
            ArgError::Duplicate("--trust-proxy"),
        ),
        (
            args(&["rpc", "--rate", "1", "--rate", "2"]),
            ArgError::Duplicate("--rate"),
        ),
        (
            args(&["rpc", "--node-url", "a", "--node-url", "b"]),
            ArgError::Duplicate("--node-url"),
        ),
        (
            args(&["rpc", "--http-port", "65536"]),
            ArgError::InvalidPort("65536".to_owned()),
        ),
        (
            args(&["rpc", "--http-bind", "localhost"]),
            ArgError::InvalidBind("localhost".to_owned()),
        ),
        (
            args(&["rpc", "--rate", "-1"]),
            ArgError::InvalidRate("-1".to_owned()),
        ),
        (
            args(&["rpc", "--mainnet"]),
            ArgError::Unknown("--mainnet".to_owned()),
        ),
    ];
    for (list, error) in cases {
        assert_eq!(parse_args(list.clone()), Err(error), "{list:?}");
    }
    assert_eq!(
        ArgError::NotYet("faucet").to_string(),
        "mode faucet belum tersedia"
    );
}
