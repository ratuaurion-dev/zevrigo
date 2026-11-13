use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use zevrigo_primitives::{Amount, Network, OutputKind};
use zevrigo_wallet_core::encode_address;

use super::{
    ArgError, Command, DEFAULT_NODE_URL, DEFAULT_RATE, DEFAULT_RPC_PORT, RpcConfig, USAGE,
    UpacaraConfig, parse_args, parse_pocket,
};
use crate::upacara::Pocket;

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
        (args(&["faucet"]), ArgError::MissingOption("--datadir")),
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
        ArgError::NotYet("daftar").to_string(),
        "mode daftar belum tersedia"
    );
}

fn address(network: Network, kind: OutputKind, byte: u8) -> String {
    encode_address(network, kind, &[byte; 32])
}

#[test]
fn parses_upacara_mode() {
    let a = address(Network::Regtest, OutputKind::SingleKey, 0x11);
    let b = address(Network::Regtest, OutputKind::SingleKey, 0x22);
    let first = format!("{a}=9999999.5");
    let second = format!("{b}=0.5");
    assert_eq!(
        parse_args(args(&[
            "upacara",
            "--kantong",
            &first,
            "--kantong",
            &second
        ])),
        Ok(Command::Upacara(UpacaraConfig {
            node_url: "http://127.0.0.1:28801".to_owned(),
            pockets: vec![
                Pocket {
                    payload: [0x11; 32],
                    amount: Amount::from_zvrg_str("9999999.5").unwrap(),
                },
                Pocket {
                    payload: [0x22; 32],
                    amount: Amount::from_zvrg_str("0.5").unwrap(),
                },
            ],
            fee_rate: 1,
            send: false,
        }))
    );
    let Ok(Command::Upacara(config)) = parse_args(args(&[
        "upacara",
        "--kirim",
        "--fee-rate",
        "3",
        "--node-url",
        "http://127.0.0.1:38801",
        "--kantong",
        &first,
    ])) else {
        unreachable!("argumen upacara sah");
    };
    assert_eq!(
        (
            config.node_url.as_str(),
            config.fee_rate,
            config.send,
            config.pockets.len()
        ),
        ("http://127.0.0.1:38801", 3, true, 1)
    );
    assert!(USAGE.contains("upacara") && USAGE.contains("--kantong") && USAGE.contains("--kirim"));
}

#[test]
fn rejects_bad_pockets() {
    let good = address(Network::Regtest, OutputKind::SingleKey, 0x11);
    let testnet = address(Network::Testnet, OutputKind::SingleKey, 0x11);
    let condition = address(Network::Regtest, OutputKind::ConditionHash, 0x11);
    for (value, reason) in [
        (good.clone(), "format <alamat>=<ZVRG>"),
        (format!("{testnet}=1"), "alamat regtest tidak sah"),
        ("bukan-alamat=1".to_owned(), "alamat regtest tidak sah"),
        (
            format!("{condition}=1"),
            "alamat wajib jenis 0 (rzvrg1q...)",
        ),
        (format!("{good}=0"), "jumlah ZVRG tidak sah"),
        (format!("{good}=-1"), "jumlah ZVRG tidak sah"),
        (
            format!("{good}=1.1234567890123456789"),
            "jumlah ZVRG tidak sah",
        ),
    ] {
        assert_eq!(
            parse_pocket(&value),
            Err(ArgError::InvalidPocket {
                value: value.clone(),
                reason
            }),
            "{value}"
        );
    }
    assert_eq!(
        parse_args(args(&["upacara"])),
        Err(ArgError::MissingPockets)
    );
    let pocket = format!("{good}=1");
    for (list, error) in [
        (
            vec![
                "upacara",
                "--kantong",
                pocket.as_str(),
                "--kirim",
                "--kirim",
            ],
            ArgError::Duplicate("--kirim"),
        ),
        (
            vec!["upacara", "--kantong", pocket.as_str(), "--fee-rate", "x"],
            ArgError::InvalidFeeRate("x".to_owned()),
        ),
        (
            vec!["upacara", "--kantong", pocket.as_str(), "--rate", "1"],
            ArgError::Unknown("--rate".to_owned()),
        ),
        (
            vec!["upacara", "--kantong"],
            ArgError::MissingValue("--kantong"),
        ),
    ] {
        assert_eq!(parse_args(args(&list)), Err(error), "{list:?}");
    }
}

#[test]
fn parses_faucet_mode() {
    use std::path::PathBuf;

    use super::{
        DEFAULT_DAILY_LIMIT, DEFAULT_FAUCET_AMOUNT, DEFAULT_FAUCET_PORT, DEFAULT_FAUCET_RATE,
        DEFAULT_ORIGIN, FaucetConfig,
    };

    assert_eq!(
        parse_args(args(&["faucet", "--datadir", "/data"])),
        Ok(Command::Faucet(FaucetConfig {
            node_url: DEFAULT_NODE_URL.to_owned(),
            datadir: PathBuf::from("/data"),
            http_bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            http_port: DEFAULT_FAUCET_PORT,
            rate: DEFAULT_FAUCET_RATE,
            amount: DEFAULT_FAUCET_AMOUNT,
            daily_limit: DEFAULT_DAILY_LIMIT,
            fee_rate: 1,
            trust_proxy_host: None,
            origins: vec![DEFAULT_ORIGIN.to_owned()],
        }))
    );
    assert_eq!(DEFAULT_FAUCET_AMOUNT, Amount::from_zvrg_str("100").unwrap());
    assert_eq!(
        parse_args(args(&[
            "faucet",
            "--node-url",
            "http://node:28801",
            "--datadir",
            "/data",
            "--http-bind",
            "0.0.0.0",
            "--http-port",
            "0",
            "--rate",
            "0",
            "--jumlah",
            "2.5",
            "--batas-harian",
            "3",
            "--fee-rate",
            "2",
            "--trust-proxy-host",
            "cloudflared",
            "--allow-origin",
            "https://a.example",
            "--allow-origin",
            "http://localhost:28802",
        ])),
        Ok(Command::Faucet(FaucetConfig {
            node_url: "http://node:28801".to_owned(),
            datadir: PathBuf::from("/data"),
            http_bind: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            http_port: 0,
            rate: 0,
            amount: Amount::from_zvrg_str("2.5").unwrap(),
            daily_limit: 3,
            fee_rate: 2,
            trust_proxy_host: Some("cloudflared".to_owned()),
            origins: vec![
                "https://a.example".to_owned(),
                "http://localhost:28802".to_owned()
            ],
        }))
    );
    for word in [
        "--datadir",
        "--batas-harian",
        "--jumlah",
        "--allow-origin",
        "--trust-proxy-host",
    ] {
        assert!(USAGE.contains(word), "{word}");
    }
}

#[test]
fn rejects_bad_faucet_arguments() {
    let base = |extra: &[&str]| {
        let mut list = args(&["faucet", "--datadir", "/data"]);
        list.extend(extra.iter().map(|item| (*item).to_owned()));
        list
    };
    let cases: Vec<(Vec<String>, ArgError)> = vec![
        (
            args(&["faucet", "--rate", "1"]),
            ArgError::MissingOption("--datadir"),
        ),
        (
            base(&["--datadir", "/lain"]),
            ArgError::Duplicate("--datadir"),
        ),
        (
            base(&["--jumlah", "0"]),
            ArgError::InvalidAmount("0".to_owned()),
        ),
        (
            base(&["--jumlah", "satu"]),
            ArgError::InvalidAmount("satu".to_owned()),
        ),
        (
            base(&["--batas-harian", "0"]),
            ArgError::InvalidLimit("0".to_owned()),
        ),
        (
            base(&["--batas-harian", "-1"]),
            ArgError::InvalidLimit("-1".to_owned()),
        ),
        (
            base(&["--allow-origin", "https://a.example/"]),
            ArgError::InvalidOrigin("https://a.example/".to_owned()),
        ),
        (
            base(&["--allow-origin", "a.example"]),
            ArgError::InvalidOrigin("a.example".to_owned()),
        ),
        (
            base(&["--http-port", "70000"]),
            ArgError::InvalidPort("70000".to_owned()),
        ),
        (
            base(&["--trust-proxy"]),
            ArgError::Unknown("--trust-proxy".to_owned()),
        ),
        (
            base(&["--rate", "1", "--rate", "2"]),
            ArgError::Duplicate("--rate"),
        ),
        (base(&["--help"]), ArgError::Exclusive),
    ];
    for (list, error) in cases {
        assert_eq!(parse_args(list.clone()), Err(error), "{list:?}");
    }
}
