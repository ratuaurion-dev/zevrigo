use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::http::{HeaderMap, HeaderValue};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use zevrigo_primitives::{Amount, Network};
use zevrigo_rpc::error::{
    INTERNAL_ERROR, INVALID_REQUEST, INVALID_TRANSACTION, METHOD_NOT_FOUND, NOT_FOUND, PARSE_ERROR,
};
use zevrigo_rpc::types::{METHOD_GET_BLOCK_TEMPLATE, METHOD_SUBMIT_BLOCK, METHODS};
use zevrigo_rpc::{
    BlockInfo, BlockTemplate, ChainInfo, CompactHex, GetBlockTemplateParams, Hex32, MempoolInfo,
    MempoolScan, NetworkName, Quanta, RpcBackend, RpcClient, RpcError, RpcServer,
    SendRawTransactionResult, SubmitBlockResult, UtxoInfo, error_code, reject_reason,
};

use super::{
    ALLOWED_METHODS, CLIENT_IP_HEADER, Call, MAX_BODY_BYTES, client_ip, parse_call, router,
};
use crate::limit::{Limits, Rate};

fn h(byte: u8) -> Hex32 {
    Hex32::new([byte; 32])
}

fn chain_info() -> ChainInfo {
    ChainInfo {
        network: NetworkName::new(Network::Regtest),
        height: 7,
        best_block_id: h(0xb7),
        chainwork: h(0x00),
        next_target: CompactHex::new(0x207f_ffff),
        median_time_past: 1_793_145_600,
        mined_total: Quanta::new(Amount::from_quanta(5)),
        total_supply: Quanta::new(Amount::from_quanta(6)),
        mempool_count: 1,
    }
}

/// Node tiruan dengan data tetap. `forbidden` menghitung panggilan metode yang tidak boleh sampai lewat gerbang.
#[derive(Default)]
struct Mock {
    forbidden: AtomicUsize,
}

impl RpcBackend for Mock {
    fn chain_info(&self) -> Result<ChainInfo, RpcError> {
        Ok(chain_info())
    }

    fn block_hash(&self, height: u64) -> Result<Hex32, RpcError> {
        if height == 0 {
            Ok(h(0x8b))
        } else {
            Err(RpcError::NotFound)
        }
    }

    fn block(&self, _block_id: Hex32) -> Result<BlockInfo, RpcError> {
        Err(RpcError::NotFound)
    }

    fn block_template(&self, _params: &GetBlockTemplateParams) -> Result<BlockTemplate, RpcError> {
        self.forbidden.fetch_add(1, Ordering::SeqCst);
        Err(RpcError::Internal {
            message: "tidak boleh sampai".to_owned(),
        })
    }

    fn submit_block(&self, _block: &[u8]) -> Result<SubmitBlockResult, RpcError> {
        self.forbidden.fetch_add(1, Ordering::SeqCst);
        Err(RpcError::Internal {
            message: "tidak boleh sampai".to_owned(),
        })
    }

    fn send_raw_transaction(&self, tx: &[u8]) -> Result<SendRawTransactionResult, RpcError> {
        if tx == [0xaa] {
            Ok(SendRawTransactionResult { txid: h(0xaa) })
        } else {
            Err(RpcError::InvalidTransaction {
                reason: "T6".to_owned(),
            })
        }
    }

    fn mempool(&self) -> Result<MempoolInfo, RpcError> {
        Ok(MempoolInfo {
            txids: vec![h(0x0a)],
        })
    }

    fn scan_utxos(&self, _payloads: &[Hex32]) -> Result<Vec<UtxoInfo>, RpcError> {
        Ok(Vec::new())
    }

    fn scan_mempool(&self, _payloads: &[Hex32]) -> Result<MempoolScan, RpcError> {
        Ok(MempoolScan {
            spent: Vec::new(),
            outputs: Vec::new(),
        })
    }
}

/// Node tiruan di port bebas dan gerbang di depannya.
struct Setup {
    rt: Runtime,
    mock: Arc<Mock>,
    node: RpcServer,
    server: JoinHandle<std::io::Result<()>>,
    gateway: SocketAddr,
}

impl Setup {
    fn start(limits: Option<Limits>) -> Self {
        let rt = Runtime::new().unwrap();
        let mock = Arc::new(Mock::default());
        let node = rt
            .block_on(RpcServer::start(
                SocketAddr::from(([127, 0, 0, 1], 0)),
                Arc::clone(&mock),
            ))
            .unwrap();
        let url = format!("http://{}", node.local_addr());
        let client = rt.block_on(async { RpcClient::new(&url) }).unwrap();
        let listener = rt
            .block_on(TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))))
            .unwrap();
        let gateway = listener.local_addr().unwrap();
        let app = router(client, limits, false);
        let server = rt.spawn(async move { axum::serve(listener, app).await });
        Self {
            rt,
            mock,
            node,
            server,
            gateway,
        }
    }

    /// Klien MOD-08 yang memakai gerbang seperti node.
    fn client(&self) -> RpcClient {
        let url = format!("http://{}", self.gateway);
        self.rt.block_on(async { RpcClient::new(&url) }).unwrap()
    }

    fn finish(self) {
        let Self {
            rt, node, server, ..
        } = self;
        server.abort();
        rt.block_on(node.stop());
    }
}

/// Badan JSON sebagai byte.
fn body(value: &Value) -> Vec<u8> {
    value.to_string().into_bytes()
}

/// Kode galat JSON-RPC di jawaban, jika ada.
fn code(answer: &Value) -> Option<i64> {
    answer["error"]["code"].as_i64()
}

/// Satu permintaan HTTP/1.1 mentah ke gerbang; mengembalikan status, kepala jawaban (huruf kecil), dan badan teks.
fn raw(addr: SocketAddr, method: &str, payload: &[u8]) -> (u16, String, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    write!(
        stream,
        "{method} / HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    )
    .unwrap();
    stream.write_all(payload).unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    let (head, rest) = text.split_once("\r\n\r\n").unwrap();
    let status = head.split(' ').nth(1).unwrap().parse().unwrap();
    (status, head.to_ascii_lowercase(), rest.to_owned())
}

/// Seperti [`raw`] dengan `POST`, badan jawaban dibaca sebagai JSON.
fn post(addr: SocketAddr, payload: &[u8]) -> (u16, String, Value) {
    let (status, head, text) = raw(addr, "POST", payload);
    (status, head, serde_json::from_str(&text).unwrap())
}

#[test]
fn allowed_methods_are_the_safe_part_of_rpc_v0_1() {
    for method in ALLOWED_METHODS {
        assert!(METHODS.contains(&method), "{method}");
    }
    for method in [METHOD_GET_BLOCK_TEMPLATE, METHOD_SUBMIT_BLOCK] {
        assert!(!ALLOWED_METHODS.contains(&method), "{method}");
    }
    assert_eq!(ALLOWED_METHODS.len() + 2, METHODS.len());
}

#[test]
fn parse_call_accepts_only_single_safe_requests() {
    let request =
        json!({"jsonrpc": "2.0", "id": 3, "method": "scan_utxos", "params": {"payloads": ["ab"]}});
    assert_eq!(
        parse_call(&body(&request)),
        Ok(Call {
            id: json!(3),
            method: "scan_utxos".to_owned(),
            params: json!({"payloads": ["ab"]}),
        })
    );
    // Tanpa parameter atau `null` menjadi objek kosong; array dan id teks diteruskan apa adanya.
    for params in [None, Some(Value::Null)] {
        let mut request = json!({"jsonrpc": "2.0", "id": "a", "method": "get_chain_info"});
        if let Some(params) = params {
            request["params"] = params;
        }
        let call = parse_call(&body(&request)).unwrap();
        assert_eq!((call.id, call.params), (json!("a"), json!({})));
    }
    let request = json!({"jsonrpc": "2.0", "id": 1, "method": "get_block_hash", "params": [5]});
    assert_eq!(parse_call(&body(&request)).unwrap().params, json!([5]));

    /// Jawaban galat `parse_call` untuk `payload`.
    fn reject(payload: &[u8]) -> Value {
        parse_call(payload).unwrap_err()
    }
    let answer = reject("{".as_bytes());
    assert_eq!(
        (code(&answer), answer["id"].clone()),
        (Some(i64::from(PARSE_ERROR)), Value::Null)
    );
    let batch = json!([{"jsonrpc": "2.0", "id": 1, "method": "get_chain_info"}]);
    let answer = reject(&body(&batch));
    assert_eq!(
        (code(&answer), answer["id"].clone()),
        (Some(i64::from(INVALID_REQUEST)), Value::Null)
    );
    assert_eq!(answer["jsonrpc"], "2.0");
    for request in [
        json!("teks"),
        json!({"jsonrpc": "2.0", "method": "get_chain_info"}),
        json!({"jsonrpc": "2.0", "id": null, "method": "get_chain_info"}),
        json!({"jsonrpc": "2.0", "id": {"a": 1}, "method": "get_chain_info"}),
    ] {
        let answer = reject(&body(&request));
        assert_eq!(code(&answer), Some(i64::from(INVALID_REQUEST)), "{request}");
        assert_eq!(answer["id"], Value::Null, "{request}");
    }
    for (request, expected) in [
        (
            json!({"jsonrpc": "1.0", "id": 9, "method": "get_chain_info"}),
            INVALID_REQUEST,
        ),
        (
            json!({"id": 9, "method": "get_chain_info"}),
            INVALID_REQUEST,
        ),
        (
            json!({"jsonrpc": "2.0", "id": 9, "method": 5}),
            INVALID_REQUEST,
        ),
        (
            json!({"jsonrpc": "2.0", "id": 9, "method": "get_chain_info", "params": "x"}),
            INVALID_REQUEST,
        ),
        (
            json!({"jsonrpc": "2.0", "id": 9, "method": "get_block_template", "params": {}}),
            METHOD_NOT_FOUND,
        ),
        (
            json!({"jsonrpc": "2.0", "id": 9, "method": "submit_block", "params": {"hex": "01"}}),
            METHOD_NOT_FOUND,
        ),
        (
            json!({"jsonrpc": "2.0", "id": 9, "method": "stop"}),
            METHOD_NOT_FOUND,
        ),
    ] {
        let answer = reject(&body(&request));
        assert_eq!(code(&answer), Some(i64::from(expected)), "{request}");
        assert_eq!(answer["id"], 9, "{request}");
    }
}

#[test]
fn gateway_forwards_safe_methods_and_blocks_mining() {
    let setup = Setup::start(None);
    let client = setup.client();
    let rt = &setup.rt;
    assert_eq!(rt.block_on(client.get_chain_info()).unwrap(), chain_info());
    assert_eq!(rt.block_on(client.get_block_hash(0)).unwrap(), h(0x8b));
    let missing = rt.block_on(client.get_block_hash(9)).unwrap_err();
    assert_eq!(error_code(&missing), Some(NOT_FOUND));
    let missing = rt.block_on(client.get_block(h(0x01))).unwrap_err();
    assert_eq!(error_code(&missing), Some(NOT_FOUND));
    assert_eq!(
        rt.block_on(client.send_raw_transaction(&[0xaa]))
            .unwrap()
            .txid,
        h(0xaa)
    );
    // Galat transaksi diteruskan lengkap dengan `data.reason`, seperti dari node sendiri.
    let rejected = rt
        .block_on(client.send_raw_transaction(&[0xbb]))
        .unwrap_err();
    assert_eq!(error_code(&rejected), Some(INVALID_TRANSACTION));
    assert_eq!(reject_reason(&rejected).as_deref(), Some("T6"));
    assert_eq!(
        rt.block_on(client.get_mempool()).unwrap().txids,
        vec![h(0x0a)]
    );
    assert!(
        rt.block_on(client.scan_utxos(&[h(0x01)]))
            .unwrap()
            .is_empty()
    );
    assert!(
        rt.block_on(client.scan_mempool(&[h(0x01)]))
            .unwrap()
            .spent
            .is_empty()
    );
    // Menambang tidak lewat gerbang: node tidak pernah dipanggil.
    for method in [METHOD_GET_BLOCK_TEMPLATE, METHOD_SUBMIT_BLOCK] {
        let blocked = rt
            .block_on(client.raw_request(method, json!({})))
            .unwrap_err();
        assert_eq!(error_code(&blocked), Some(METHOD_NOT_FOUND), "{method}");
    }
    assert_eq!(setup.mock.forbidden.load(Ordering::SeqCst), 0);
    setup.finish();
}

#[test]
fn gateway_answers_bad_http_requests() {
    let setup = Setup::start(None);
    let addr = setup.gateway;
    let request = json!({"jsonrpc": "2.0", "id": "a", "method": "get_chain_info"});
    let (status, head, answer) = post(addr, &body(&request));
    assert_eq!(status, 200);
    assert_eq!(
        (answer["id"].clone(), answer["result"]["height"].clone()),
        (json!("a"), json!(7))
    );
    assert_eq!(answer["jsonrpc"], "2.0");
    for expected in [
        "content-type: application/json",
        "cache-control: no-store",
        "x-content-type-options: nosniff",
    ] {
        assert!(head.contains(expected), "{head}");
    }
    assert!(!head.contains("set-cookie"), "{head}");
    let (status, _, answer) = post(addr, b"{");
    assert_eq!((status, code(&answer)), (200, Some(i64::from(PARSE_ERROR))));
    let batch = json!([{"jsonrpc": "2.0", "id": 1, "method": "get_chain_info"}]);
    let (status, _, answer) = post(addr, &body(&batch));
    assert_eq!(
        (status, code(&answer)),
        (200, Some(i64::from(INVALID_REQUEST)))
    );
    let (status, head, text) = raw(addr, "GET", b"");
    assert_eq!(status, 405);
    assert!(head.contains("allow: post"), "{head}");
    assert_eq!(text, "{\"error\":\"method_not_allowed\"}");
    let (status, _, text) = raw(addr, "POST", &vec![b' '; MAX_BODY_BYTES + 1]);
    assert_eq!(
        (status, text.as_str()),
        (413, "{\"error\":\"payload_too_large\"}")
    );
    // Badan tepat sebesar batas masih dibaca (JSON kosong: -32700).
    let (status, _, answer) = post(addr, &vec![b' '; MAX_BODY_BYTES]);
    assert_eq!((status, code(&answer)), (200, Some(i64::from(PARSE_ERROR))));
    setup.finish();
}

#[test]
fn gateway_limits_rate_before_reading_requests() {
    let limits = Limits {
        client: Rate {
            per_second: 1,
            burst: 2,
        },
        global: Rate {
            per_second: 1_000,
            burst: 1_000,
        },
    };
    let setup = Setup::start(Some(limits));
    let addr = setup.gateway;
    // Tanpa ConnectInfo semua permintaan berbagi ember limpahan berkapasitas 2.
    assert_eq!(raw(addr, "POST", b"").0, 200);
    assert_eq!(raw(addr, "GET", b"").0, 405);
    let (status, head, text) = raw(addr, "POST", b"");
    assert_eq!(status, 429);
    assert!(head.contains("retry-after: 1"), "{head}");
    assert_eq!(text, "{\"error\":\"rate_limited\"}");
    setup.finish();
}

#[test]
fn unreachable_node_becomes_internal_error_without_details() {
    let rt = Runtime::new().unwrap();
    let closed = std::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
    let url = format!("http://{}", closed.local_addr().unwrap());
    drop(closed);
    let client = rt.block_on(async { RpcClient::new(&url) }).unwrap();
    let listener = rt
        .block_on(TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))))
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let app = router(client, None, false);
    let server = rt.spawn(async move { axum::serve(listener, app).await });
    let request = json!({"jsonrpc": "2.0", "id": 4, "method": "get_chain_info"});
    let (status, _, answer) = post(addr, &body(&request));
    assert_eq!(
        (status, code(&answer)),
        (200, Some(i64::from(INTERNAL_ERROR)))
    );
    assert_eq!(answer["error"]["message"], "node tidak bisa dihubungi");
    assert_eq!(answer["id"], 4);
    server.abort();
}

#[test]
fn client_ip_uses_proxy_header_only_from_loopback_with_option() {
    let loopback = Some(IpAddr::V4(Ipv4Addr::LOCALHOST));
    let loopback_v6 = Some(IpAddr::V6(Ipv6Addr::LOCALHOST));
    let mapped = Some(IpAddr::V6(Ipv4Addr::LOCALHOST.to_ipv6_mapped()));
    let remote = Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)));
    let visitor = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 9));
    let mut headers = HeaderMap::new();
    headers.insert(CLIENT_IP_HEADER, HeaderValue::from_static(" 198.51.100.9 "));
    for connect in [loopback, loopback_v6, mapped] {
        assert_eq!(
            client_ip(connect, &headers, true),
            Some(visitor),
            "{connect:?}"
        );
    }
    // Tanpa opsi, atau koneksi bukan dari loopback: kepala diabaikan.
    assert_eq!(client_ip(loopback, &headers, false), loopback);
    assert_eq!(client_ip(remote, &headers, true), remote);
    assert_eq!(client_ip(None, &headers, true), None);
    for text in ["198.51.100.9, 10.0.0.1", "unknown", ""] {
        let mut bad = HeaderMap::new();
        bad.insert(CLIENT_IP_HEADER, HeaderValue::from_static(text));
        assert_eq!(client_ip(loopback, &bad, true), loopback, "{text}");
    }
}
