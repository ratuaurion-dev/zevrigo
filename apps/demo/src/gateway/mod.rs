//! Gerbang RPC untuk wallet peserta (Rancangan Ekosistem Demo v0.1 Bagian 4.9, TSK-M53): server HTTP axum yang menerima
//! satu permintaan JSON-RPC 2.0 per `POST` dan meneruskan hanya metode di [`ALLOWED_METHODS`] ke node lewat [`RpcClient`]
//! (MOD-08). Hasil atau galat node (kode, pesan, dan `data`) dikembalikan apa adanya dengan `id` permintaan, sehingga
//! wallet yang memakai MOD-08 bekerja sama seperti ke node sendiri.
//!
//! - Metode lain, termasuk `get_block_template` dan `submit_block`, dijawab `-32601` tanpa dikirim ke node: menambang
//!   tetap lewat node sendiri.
//! - Batch (array JSON) tidak dilayani (`-32600`). Badan di atas [`MAX_BODY_BYTES`] dijawab `413`, metode HTTP selain
//!   `POST` dijawab `405`.
//! - Batas laju per pengunjung dan bersama ([`crate::limit`]), diperiksa sebelum badan dibaca. Di belakang Cloudflare
//!   Tunnel dengan `--trust-proxy`, alamat pengunjung dari kepala [`CLIENT_IP_HEADER`] hanya dipakai untuk koneksi dari
//!   loopback ([`client_ip`]).
//! - Privasi: tanpa cookie dan tanpa log alamat, metode, atau parameter. Log hanya untuk start, berhenti, dan node yang
//!   tidak menjawab.

use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{ConnectInfo, State};
use axum::http::{Extensions, HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use jsonrpsee::core::ClientError;
use serde_json::{Map, Value, json};
use thiserror::Error;
use tokio::net::TcpListener;
use zevrigo_rpc::RpcClient;
use zevrigo_rpc::error::{INTERNAL_ERROR, INVALID_REQUEST, METHOD_NOT_FOUND, PARSE_ERROR};
use zevrigo_rpc::types::{
    METHOD_GET_BLOCK, METHOD_GET_BLOCK_HASH, METHOD_GET_CHAIN_INFO, METHOD_GET_MEMPOOL,
    METHOD_SCAN_MEMPOOL, METHOD_SCAN_UTXOS, METHOD_SEND_RAW_TRANSACTION,
};

use crate::cli::RpcConfig;
use crate::limit::{Limits, RateLimiter};

/// Metode yang diteruskan ke node: membaca chain dan mempool, memindai saldo, dan mengirim transaksi yang sudah
/// ditandatangani wallet. `get_block_template` dan `submit_block` tidak termasuk.
pub const ALLOWED_METHODS: [&str; 7] = [
    METHOD_GET_CHAIN_INFO,
    METHOD_GET_BLOCK_HASH,
    METHOD_GET_BLOCK,
    METHOD_GET_MEMPOOL,
    METHOD_SCAN_UTXOS,
    METHOD_SCAN_MEMPOOL,
    METHOD_SEND_RAW_TRANSACTION,
];

/// Ukuran badan permintaan paling besar (256 KiB): cukup untuk transaksi sebesar blok maksimum (100_000 byte) dalam hex.
pub const MAX_BODY_BYTES: usize = 262_144;

/// Kepala HTTP berisi alamat IP pengunjung yang dipasang Cloudflare (huruf kecil). Hanya dibaca dengan `--trust-proxy`
/// untuk koneksi dari loopback ([`client_ip`]).
pub const CLIENT_IP_HEADER: &str = "cf-connecting-ip";

/// Batas waktu menunggu tugas selesai saat gerbang berhenti.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// Badan jawaban `429`.
const RATE_LIMITED_BODY: &[u8] = b"{\"error\":\"rate_limited\"}";

/// Badan jawaban `405`.
const METHOD_NOT_ALLOWED_BODY: &[u8] = b"{\"error\":\"method_not_allowed\"}";

/// Badan jawaban `413`.
const TOO_LARGE_BODY: &[u8] = b"{\"error\":\"payload_too_large\"}";

/// Galat menjalankan layanan demo. Biner keluar dengan kode 1.
#[derive(Debug, Error)]
pub enum DemoError {
    /// Runtime, sinyal, atau port HTTP gagal.
    #[error("galat I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Klien RPC ke node tidak bisa dibuat (URL tidak sah).
    #[error("klien RPC gagal dibuat: {0}")]
    Rpc(String),
}

/// Satu permintaan JSON-RPC yang lolos pemeriksaan gerbang dan boleh diteruskan ke node.
#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    /// `id` permintaan (angka atau teks), dikembalikan apa adanya.
    pub id: Value,
    /// Nama metode, salah satu dari [`ALLOWED_METHODS`].
    pub method: String,
    /// Parameter berupa objek atau array; tanpa parameter atau `null` menjadi objek kosong.
    pub params: Value,
}

/// Jawaban galat JSON-RPC 2.0. `data` hanya disertakan jika ada.
fn error_response(id: Value, code: i32, message: &str, data: Option<Value>) -> Value {
    let mut error = Map::new();
    error.insert("code".to_owned(), Value::from(code));
    error.insert("message".to_owned(), Value::from(message));
    if let Some(data) = data {
        error.insert("data".to_owned(), data);
    }
    json!({"jsonrpc": "2.0", "error": Value::Object(error), "id": id})
}

/// Memeriksa badan satu permintaan JSON-RPC 2.0.
///
/// # Errors
///
/// Jawaban galat JSON-RPC yang langsung dikirim ke pengunjung, tanpa menghubungi node: `-32700` jika JSON rusak;
/// `-32600` untuk batch, badan yang bukan objek, `id` yang bukan angka atau teks, `jsonrpc` yang bukan `"2.0"`, `method`
/// yang bukan teks, atau parameter yang bukan objek atau array; `-32601` untuk metode di luar [`ALLOWED_METHODS`].
pub fn parse_call(body: &[u8]) -> Result<Call, Value> {
    let request: Value = serde_json::from_slice(body)
        .map_err(|_| error_response(Value::Null, PARSE_ERROR, "Parse error", None))?;
    let batch = request.is_array();
    let Value::Object(mut object) = request else {
        let message = if batch {
            "batch tidak dilayani gerbang"
        } else {
            "Invalid request"
        };
        return Err(error_response(Value::Null, INVALID_REQUEST, message, None));
    };
    let id = match object.remove("id") {
        Some(id @ (Value::Number(_) | Value::String(_))) => id,
        _ => {
            return Err(error_response(
                Value::Null,
                INVALID_REQUEST,
                "id wajib berupa angka atau teks",
                None,
            ));
        }
    };
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(error_response(id, INVALID_REQUEST, "Invalid request", None));
    }
    let method = match object.remove("method") {
        Some(Value::String(method)) => method,
        _ => return Err(error_response(id, INVALID_REQUEST, "Invalid request", None)),
    };
    if !ALLOWED_METHODS.contains(&method.as_str()) {
        return Err(error_response(
            id,
            METHOD_NOT_FOUND,
            "metode tidak tersedia di gerbang publik",
            None,
        ));
    }
    let params = match object.remove("params") {
        None | Some(Value::Null) => Value::Object(Map::new()),
        Some(params @ (Value::Object(_) | Value::Array(_))) => params,
        Some(_) => {
            return Err(error_response(
                id,
                INVALID_REQUEST,
                "params wajib berupa objek atau array",
                None,
            ));
        }
    };
    Ok(Call { id, method, params })
}

/// Meneruskan `call` ke node. Galat dari node (kode, pesan, `data`) diteruskan apa adanya; node yang tidak menjawab
/// menjadi `-32603` tanpa rincian.
async fn forward(client: &RpcClient, call: Call) -> Value {
    let Call { id, method, params } = call;
    match client.raw_request(&method, params).await {
        Ok(result) => json!({"jsonrpc": "2.0", "result": result, "id": id}),
        Err(ClientError::Call(error)) => {
            let data = error
                .data()
                .and_then(|raw| serde_json::from_str::<Value>(raw.get()).ok());
            error_response(id, error.code(), error.message(), data)
        }
        Err(error) => {
            tracing::warn!(%error, "node tidak menjawab");
            error_response(id, INTERNAL_ERROR, "node tidak bisa dihubungi", None)
        }
    }
}

/// Alamat pengunjung untuk batas laju. `connect` adalah alamat koneksi TCP. Dengan `trust_proxy` (opsi
/// `--trust-proxy`), koneksi dari loopback (`127.0.0.0/8`, `::1`, juga bentuk IPv4-mapped) dianggap datang dari
/// `cloudflared` di container yang sama, sehingga alamat pengunjung dibaca dari kepala [`CLIENT_IP_HEADER`]. Kepala yang
/// tidak ada atau bukan satu alamat IP membuat alamat koneksi tetap dipakai. Koneksi dari alamat lain tidak pernah memakai
/// kepala itu, sehingga klien yang tersambung langsung tidak bisa memalsukan alamatnya. Alamat tidak disimpan dan tidak
/// dicatat.
#[must_use]
pub fn client_ip(
    connect: Option<IpAddr>,
    headers: &HeaderMap,
    trust_proxy: bool,
) -> Option<IpAddr> {
    let forwarded = connect
        .filter(|address| trust_proxy && address.to_canonical().is_loopback())
        .and_then(|_| headers.get(CLIENT_IP_HEADER))
        .and_then(|value| value.to_str().ok())
        .and_then(|text| text.trim().parse::<IpAddr>().ok());
    forwarded.or(connect)
}

/// Isi router: klien ke node dan pembatas laju (jika ada).
struct App {
    client: RpcClient,
    limiter: Option<Mutex<RateLimiter>>,
    trust_proxy: bool,
    started: Instant,
}

impl App {
    /// `true` jika permintaan boleh dilayani. Alamat pengunjung dari [`ConnectInfo`] (jika server dipasang dengan
    /// `into_make_service_with_connect_info`), atau dari kepala Cloudflare lewat [`client_ip`], hanya diteruskan ke
    /// [`RateLimiter`], tidak disimpan atau dicatat.
    fn allow(&self, extensions: &Extensions, headers: &HeaderMap) -> bool {
        let Some(limiter) = &self.limiter else {
            return true;
        };
        let connect = extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|info| info.0.ip());
        let client = client_ip(connect, headers, self.trust_proxy);
        let now_ms = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);
        limiter
            .lock()
            .map(|mut limiter| limiter.allow(client, now_ms))
            .unwrap_or(false)
    }
}

/// Router axum gerbang RPC: setiap jalur dilayani [`handle`] dengan batas laju `limits` (jika ada). Batas per pengunjung
/// butuh `into_make_service_with_connect_info::<SocketAddr>()`; tanpa itu semua pengunjung berbagi satu ember. Dengan
/// `trust_proxy`, koneksi dari loopback memakai alamat pengunjung dari kepala Cloudflare ([`client_ip`]).
pub fn router(client: RpcClient, limits: Option<Limits>, trust_proxy: bool) -> Router {
    let app = App {
        client,
        limiter: limits.map(|limits| Mutex::new(RateLimiter::new(limits))),
        trust_proxy,
        started: Instant::now(),
    };
    Router::new().fallback(handle).with_state(Arc::new(app))
}

/// Jawaban JSON dengan `no-store` dan `nosniff`.
fn json_reply(status: StatusCode, body: Vec<u8>) -> Response {
    (
        status,
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-store"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        body,
    )
        .into_response()
}

/// `429 {"error": "rate_limited"}` dengan `Retry-After: 1`.
fn rate_limited() -> Response {
    let mut response = json_reply(StatusCode::TOO_MANY_REQUESTS, RATE_LIMITED_BODY.to_vec());
    response
        .headers_mut()
        .insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
    response
}

/// Satu permintaan HTTP: batas laju, lalu metode `POST`, lalu badan paling banyak [`MAX_BODY_BYTES`], lalu
/// [`parse_call`] dan [`forward`]. Jawaban JSON-RPC (hasil atau galat) selalu berstatus `200`.
async fn handle(
    State(app): State<Arc<App>>,
    method: Method,
    headers: HeaderMap,
    extensions: Extensions,
    body: Body,
) -> Response {
    if !app.allow(&extensions, &headers) {
        return rate_limited();
    }
    if method != Method::POST {
        let mut response = json_reply(
            StatusCode::METHOD_NOT_ALLOWED,
            METHOD_NOT_ALLOWED_BODY.to_vec(),
        );
        response
            .headers_mut()
            .insert(header::ALLOW, HeaderValue::from_static("POST"));
        return response;
    }
    let Ok(bytes) = to_bytes(body, MAX_BODY_BYTES).await else {
        return json_reply(StatusCode::PAYLOAD_TOO_LARGE, TOO_LARGE_BODY.to_vec());
    };
    let answer = match parse_call(&bytes) {
        Ok(call) => forward(&app.client, call).await,
        Err(answer) => answer,
    };
    json_reply(StatusCode::OK, answer.to_string().into_bytes())
}

/// Memasang logger ke stderr (level INFO, tanpa warna). Jika logger global sudah ada, logger lama tetap dipakai.
pub fn init_logging() {
    // Hanya gagal jika subscriber global sudah dipasang sebelumnya; itu tidak menghalangi gerbang.
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .try_init();
}

/// Menunggu SIGINT atau SIGTERM. Penangkap sinyal dipasang saat fungsi ini dipanggil. Wajib di dalam runtime tokio.
#[cfg(unix)]
fn shutdown_signal() -> std::io::Result<impl Future<Output = ()> + Send + 'static> {
    use tokio::signal::unix::{SignalKind, signal};
    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut terminate = signal(SignalKind::terminate())?;
    Ok(async move {
        tokio::select! {
            _ = interrupt.recv() => {}
            _ = terminate.recv() => {}
        }
    })
}

/// Menunggu Ctrl-C (bukan Unix).
#[cfg(not(unix))]
fn shutdown_signal() -> std::io::Result<impl Future<Output = ()> + Send + 'static> {
    Ok(async {
        // Galat berarti sinyal tidak bisa dipasang; gerbang berhenti seperti menerima Ctrl-C.
        let _ = tokio::signal::ctrl_c().await;
    })
}

async fn serve(config: &RpcConfig) -> Result<(), DemoError> {
    let shutdown = shutdown_signal()?;
    let client =
        RpcClient::new(&config.node_url).map_err(|error| DemoError::Rpc(error.to_string()))?;
    let listener =
        TcpListener::bind(SocketAddr::from((config.http_bind, config.http_port))).await?;
    let local = listener.local_addr()?;
    tracing::info!(
        %local,
        node = %config.node_url,
        rate = config.rate,
        trust_proxy = config.trust_proxy,
        "gerbang RPC berjalan"
    );
    println!("http: http://{local}");
    let app = router(client, Limits::from_rate(config.rate), config.trust_proxy);
    let served = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await;
    tracing::info!("gerbang RPC berhenti");
    Ok(served?)
}

/// Menjalankan gerbang RPC (mode `rpc`) sampai menerima SIGINT atau SIGTERM.
///
/// # Errors
///
/// [`DemoError`] jika runtime, sinyal, klien RPC, atau port HTTP gagal.
pub fn run_rpc(config: &RpcConfig) -> Result<(), DemoError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let result = runtime.block_on(serve(config));
    runtime.shutdown_timeout(SHUTDOWN_TIMEOUT);
    result
}

#[cfg(test)]
mod tests;
