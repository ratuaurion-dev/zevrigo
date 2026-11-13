use super::{RpcClient, error_code};

#[tokio::test]
async fn new_accepts_http_and_https_urls() {
    for url in [
        "http://127.0.0.1:28801",
        "https://zevrigo-rpc.ratuaurion.store",
        "https://zevrigo-rpc.ratuaurion.store/",
    ] {
        assert!(RpcClient::new(url).is_ok(), "{url}");
    }
    for url in [
        "ftp://127.0.0.1:28801",
        "ws://127.0.0.1:28801",
        "127.0.0.1:28801",
        "bukan url",
    ] {
        assert!(RpcClient::new(url).is_err(), "{url}");
    }
}

#[tokio::test]
async fn https_to_closed_port_is_transport_error() {
    // Jalur TLS dijalankan sampai koneksi TCP (penyedia kripto dipasang tanpa panik); port tertutup menjadi galat
    // transport, bukan galat JSON-RPC.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("https://{}", listener.local_addr().unwrap());
    drop(listener);
    let client = RpcClient::new(&url).unwrap();
    let error = client.get_chain_info().await.unwrap_err();
    assert_eq!(error_code(&error), None);
}
