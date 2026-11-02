use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;

use homeostasis_consensus::regtest_network_id;
use homeostasis_crypto::txid;
use homeostasis_primitives::Network;
use homeostasis_rpc::{RpcClient, RpcServer};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

use super::{Shared, router};
use crate::index::Indexer;
use crate::rpc_source::RpcSource;
use crate::test_support::{Chain, FakeNode, genesis_output, out, spend};

/// Satu permintaan HTTP/1.1 mentah; mengembalikan status, kepala jawaban (huruf kecil), dan badan teks.
fn raw(addr: SocketAddr, method: &str, path: &str) -> (u16, String, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut text = String::new();
    stream.read_to_string(&mut text).unwrap();
    let (head, body) = text.split_once("\r\n\r\n").unwrap();
    let status = head.split(' ').nth(1).unwrap().parse().unwrap();
    (status, head.to_ascii_lowercase(), body.to_owned())
}

/// Seperti [`raw`], dengan badan JSON.
fn request(addr: SocketAddr, method: &str, path: &str) -> (u16, String, Value) {
    let (status, head, body) = raw(addr, method, path);
    (status, head, serde_json::from_str(&body).unwrap())
}

/// Kepala keamanan yang wajib ada di setiap jawaban.
fn assert_security_headers(head: &str) {
    assert!(head.contains("x-content-type-options: nosniff"), "{head}");
    assert!(head.contains("referrer-policy: no-referrer"), "{head}");
    assert!(
        head.contains("content-security-policy: default-src 'none'; script-src 'self';"),
        "{head}"
    );
    assert!(!head.contains("set-cookie"), "{head}");
}

/// Folder web sementara, dihapus saat selesai.
struct Site(PathBuf);

impl Drop for Site {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn http_server_answers_and_syncs_from_node() {
    let rt = Runtime::new().unwrap();
    let (genesis_txid, allocation, _) = genesis_output();
    let tx = spend(&[(genesis_txid, 0)], vec![out(allocation - 1_000, 3)]);
    let tx_id = txid(&tx);
    let mut chain = Chain::new();
    chain.push(2, Vec::new(), 0);
    chain.push(2, vec![tx], 1_000);
    let node = Arc::new(FakeNode::new(chain.clone()));
    let rpc = rt
        .block_on(RpcServer::start(
            SocketAddr::from(([127, 0, 0, 1], 0)),
            Arc::clone(&node),
        ))
        .unwrap();
    let url = format!("http://{}", rpc.local_addr());
    let client = rt.block_on(async { RpcClient::new(&url) }).unwrap();
    let source = RpcSource::new(client, rt.handle().clone());
    let index = Indexer::open_in_memory(regtest_network_id().unwrap()).unwrap();
    let shared = Arc::new(Shared::new(index, Network::Regtest));

    let listener = rt
        .block_on(TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))))
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let app = router(Arc::clone(&shared), None);
    let server = rt.spawn(async move { axum::serve(listener, app).await });

    // Sebelum sinkronisasi: genesis belum terindeks.
    let (status, head, body) = request(addr, "GET", "/api/v1/supply");
    assert_eq!((status, body["error"].as_str()), (503, Some("not_ready")));
    assert!(head.contains("content-type: application/json"), "{head}");
    assert!(head.contains("cache-control: no-store"), "{head}");
    assert_security_headers(&head);
    let (status, _, body) = request(addr, "GET", "/api/v1/status");
    assert_eq!((status, body["indexed_height"].clone()), (200, Value::Null));

    shared.sync_once(&source);
    let (status, _, body) = request(addr, "GET", "/api/v1/status");
    assert_eq!(status, 200);
    assert_eq!(body["indexed_height"], 2);
    assert_eq!(body["node_height"], 2);
    assert_eq!(body["node_best_block_id"], chain.id(2).to_string());
    assert!(body["chainwork"].as_str().unwrap().ends_with("03"));
    assert_eq!(body["index_error"], Value::Null);
    assert_eq!(body["last_error"], Value::Null);
    let (status, _, body) = request(addr, "GET", &format!("/api/v1/tx/{tx_id}"));
    assert_eq!((status, body["fee"].as_str()), (200, Some("1000")));
    let (_, _, body) = request(addr, "GET", "/api/v1/allocation?page=0");
    assert_eq!(body["unspent"], "0");
    assert_eq!(body["outputs"][0]["spent_by"]["txid"], tx_id.to_string());
    let (status, _, body) = request(addr, "POST", "/api/v1/status");
    assert_eq!(
        (status, body["error"].as_str()),
        (405, Some("method_not_allowed"))
    );
    let (status, _, body) = request(addr, "GET", "/");
    assert_eq!((status, body["error"].as_str()), (404, Some("not_found")));

    // Node mati: indeks lama tetap dilayani, galat terakhir dilaporkan.
    rt.block_on(rpc.stop());
    shared.sync_once(&source);
    let (status, _, body) = request(addr, "GET", "/api/v1/status");
    assert_eq!(status, 200);
    assert_eq!(body["indexed_height"], 2);
    assert!(body["last_error"].is_string());
    assert_eq!(body["index_error"], Value::Null);
    server.abort();
}

#[test]
fn http_server_serves_web_files_and_api() {
    let rt = Runtime::new().unwrap();
    let site = Site(
        std::env::temp_dir().join(format!("homeostasis-q85-web-server-{}", std::process::id())),
    );
    let _ = std::fs::remove_dir_all(&site.0);
    std::fs::create_dir_all(site.0.join("assets")).unwrap();
    std::fs::write(
        site.0.join("index.html"),
        "<!doctype html><div id=\"root\"></div>",
    )
    .unwrap();
    std::fs::write(
        site.0.join("assets").join("index-a1b2c3.js"),
        "console.log(1)",
    )
    .unwrap();
    let index = Indexer::open_in_memory(regtest_network_id().unwrap()).unwrap();
    let shared = Arc::new(Shared::new(index, Network::Regtest));
    let listener = rt
        .block_on(TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))))
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let app = router(shared, Some(site.0.clone()));
    let server = rt.spawn(async move { axum::serve(listener, app).await });

    let (status, head, body) = raw(addr, "GET", "/");
    assert_eq!(status, 200);
    assert_eq!(body, "<!doctype html><div id=\"root\"></div>");
    assert!(
        head.contains("content-type: text/html; charset=utf-8"),
        "{head}"
    );
    assert!(head.contains("cache-control: no-cache"), "{head}");
    assert_security_headers(&head);
    let (status, head, body) = raw(addr, "GET", "/assets/index-a1b2c3.js");
    assert_eq!((status, body.as_str()), (200, "console.log(1)"));
    assert!(
        head.contains("content-type: text/javascript; charset=utf-8"),
        "{head}"
    );
    assert!(head.contains("immutable"), "{head}");
    assert_security_headers(&head);
    for path in [
        "/missing.js",
        "/assets/../index.html",
        "/.hidden",
        "/block/1",
    ] {
        let (status, head, body) = raw(addr, "GET", path);
        assert_eq!((status, body.as_str()), (404, "not found\n"), "{path}");
        assert_security_headers(&head);
    }
    let (status, _, _) = raw(addr, "POST", "/");
    assert_eq!(status, 405);

    // Jalur API tetap dilayani API, juga dengan folder web.
    let (status, head, body) = request(addr, "GET", "/api/v1/status");
    assert_eq!((status, body["indexed_height"].clone()), (200, Value::Null));
    assert!(head.contains("content-type: application/json"), "{head}");
    assert_security_headers(&head);
    let (status, _, body) = request(addr, "GET", "/api/v1/blocks");
    assert_eq!((status, body["error"].as_str()), (503, Some("not_ready")));
    let (status, _, body) = request(addr, "GET", "/api");
    assert_eq!((status, body["error"].as_str()), (404, Some("not_found")));
    let (status, _, body) = request(addr, "POST", "/api/v1/status");
    assert_eq!(
        (status, body["error"].as_str()),
        (405, Some("method_not_allowed"))
    );
    server.abort();
}
