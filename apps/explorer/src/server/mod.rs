//! Program explorer (Spesifikasi Explorer v0.1 Bagian 6): server HTTP axum di 127.0.0.1, sinkronisasi indeks dari node
//! setiap 2 detik, dan berhenti bersih dengan SIGINT atau SIGTERM. Dengan `--web-dir`, jalur di luar `/api/` menyajikan
//! antarmuka web (Spesifikasi Explorer Web v0.2 Bagian 7). Semua jawaban membawa kepala keamanan; halaman HTML
//! mendapat kebijakan keamanan konten dengan hash skrip inline-nya ([`web::page_policy`]).
//!
//! Satu koneksi SQLite dijaga mutex. Permintaan HTTP dan langkah sinkronisasi dijalankan di `spawn_blocking`, sehingga
//! akses SQLite dan RPC sinkron tidak menahan thread async. Tanpa cookie, analitik, atau log alamat IP, jalur, dan kata
//! pencarian; log hanya untuk start, kemajuan indeks, reorg, dan galat.

use std::future::Future;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::http::{Method, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use homeostasis_consensus::regtest_network_id;
use homeostasis_primitives::Network;
use homeostasis_rpc::RpcClient;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::runtime::Handle;
use tokio::time::MissedTickBehavior;

use crate::api::{ApiResponse, SyncStatus, respond};
use crate::cli::{Config, db_path};
use crate::index::{IndexError, Indexer};
use crate::rpc_source::RpcSource;
use crate::web::{self, CONTENT_SECURITY_POLICY, INDEX_FILE, WebResponse};

/// Selang langkah sinkronisasi (2 detik).
pub const SYNC_INTERVAL: Duration = Duration::from_secs(2);

/// Batas waktu menunggu thread blocking selesai saat explorer berhenti.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// Galat menjalankan explorer. Biner keluar dengan kode 1.
#[derive(Debug, Error)]
pub enum ExplorerError {
    /// Folder data, runtime, sinyal, atau port HTTP gagal.
    #[error("galat I/O: {0}")]
    Io(#[from] std::io::Error),
    /// File indeks tidak bisa dibuka (misalnya milik jaringan lain).
    #[error("indeks gagal dibuka: {0}")]
    Index(#[from] IndexError),
    /// Genesis jaringan tidak bisa dibentuk.
    #[error("genesis jaringan tidak tersedia: {0}")]
    Genesis(String),
    /// Klien RPC tidak bisa dibuat (URL tidak sah).
    #[error("klien RPC gagal dibuat: {0}")]
    Rpc(String),
    /// Folder `--web-dir` tidak berisi `index.html`.
    #[error("folder web tidak berisi index.html: {0}")]
    WebDir(String),
}

/// Isi explorer yang dibagi server HTTP dan sinkronisasi: indeks dan keadaan sinkronisasi terakhir.
pub struct Shared {
    indexer: Mutex<Indexer>,
    sync: Mutex<SyncStatus>,
    network: Network,
}

impl Shared {
    /// Membungkus `indexer` untuk jaringan `network`.
    #[must_use]
    pub fn new(indexer: Indexer, network: Network) -> Self {
        Self {
            indexer: Mutex::new(indexer),
            sync: Mutex::new(SyncStatus::default()),
            network,
        }
    }

    /// Menjawab satu permintaan API (lihat [`respond`]).
    #[must_use]
    pub fn respond(&self, method: &str, path: &str, query: Option<&str>) -> ApiResponse {
        let (Ok(index), Ok(sync)) = (self.indexer.lock(), self.sync.lock()) else {
            return ApiResponse::internal();
        };
        respond(&index, &sync, self.network, method, path, query)
    }

    /// Membaca indeks langsung dengan `read` (untuk operator dan uji), tidak lewat API HTTP. Dipakai untuk data yang
    /// sengaja tidak disajikan API (DEC-28): alamat, input dan output transaksi, alokasi genesis. `None` jika kunci
    /// indeks rusak.
    pub fn read_index<T>(&self, read: impl FnOnce(&Indexer) -> T) -> Option<T> {
        self.indexer.lock().ok().map(|index| read(&index))
    }

    /// Satu langkah sinkronisasi dengan node: ujung node untuk `/status`, lalu [`Indexer::sync_step`]. Galat dicatat
    /// di `last_error` dan di log (sekali per galat yang sama).
    pub fn sync_once(&self, source: &RpcSource) {
        let node = source.node_view();
        let outcome = match &node {
            Err(error) => Err(error.to_string()),
            Ok(_) => match self.indexer.lock() {
                Ok(mut index) => index.sync_step(source).map_err(|error| error.to_string()),
                Err(_) => Err("state indeks rusak".to_owned()),
            },
        };
        let Ok(mut status) = self.sync.lock() else {
            return;
        };
        if let Ok(view) = node {
            status.node = Some(view);
        }
        match outcome {
            Ok(report) => {
                if report.rolled_back > 0 {
                    tracing::info!(
                        rolled_back = report.rolled_back,
                        "reorg: blok dilepas dari indeks"
                    );
                }
                if report.added > 0 {
                    tracing::info!(
                        added = report.added,
                        node_height = report.node_height,
                        "blok baru terindeks"
                    );
                }
                status.last_error = None;
            }
            Err(message) => {
                if status.last_error.as_deref() != Some(message.as_str()) {
                    tracing::warn!(%message, "sinkronisasi indeks gagal");
                }
                status.last_error = Some(message);
            }
        }
    }
}

/// Isi router: data explorer dan folder antarmuka web (jika ada).
struct App {
    shared: Arc<Shared>,
    web_dir: Option<Arc<PathBuf>>,
}

/// Router axum. Jalur `/api` dan `/api/…` diteruskan ke [`Shared::respond`]; jalur lain menyajikan file dari `web_dir`
/// lewat [`web::serve`] (metode `GET` dan `HEAD`), atau `404` JSON API jika `web_dir` tidak ada.
pub fn router(shared: Arc<Shared>, web_dir: Option<PathBuf>) -> Router {
    let app = App {
        shared,
        web_dir: web_dir.map(Arc::new),
    };
    Router::new().fallback(handle).with_state(Arc::new(app))
}

/// Jalur API: `/api` atau diawali `/api/`.
fn is_api_path(path: &str) -> bool {
    path == "/api" || path.starts_with("/api/")
}

/// Jawaban HTTP dengan kepala keamanan explorer: `nosniff`, tanpa referrer, dan kebijakan keamanan konten `policy`
/// ([`CONTENT_SECURITY_POLICY`], atau [`web::page_policy`] untuk halaman HTML).
fn reply(
    status: u16,
    content_type: &'static str,
    cache: &'static str,
    policy: &str,
    body: Vec<u8>,
) -> Response {
    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (
        status,
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, cache),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::REFERRER_POLICY, "no-referrer"),
            (header::CONTENT_SECURITY_POLICY, policy),
        ],
        body,
    )
        .into_response()
}

async fn handle(State(app): State<Arc<App>>, method: Method, uri: Uri) -> Response {
    let path = uri.path().to_owned();
    let web_dir = if is_api_path(&path) {
        None
    } else {
        app.web_dir.clone()
    };
    if let Some(dir) = web_dir {
        // HEAD dilayani seperti GET (prefetch Next.js); hyper tidak mengirim badan untuk HEAD.
        if method != Method::GET && method != Method::HEAD {
            return reply(
                405,
                "text/plain; charset=utf-8",
                "no-store",
                CONTENT_SECURITY_POLICY,
                b"method not allowed\n".to_vec(),
            );
        }
        let answer = tokio::task::spawn_blocking(move || web::serve(&dir, &path))
            .await
            .unwrap_or_else(|_| WebResponse::internal());
        return reply(
            answer.status,
            answer.content_type,
            answer.cache_control,
            &answer.content_security_policy,
            answer.body,
        );
    }
    let shared = Arc::clone(&app.shared);
    let method = method.as_str().to_owned();
    let query = uri.query().map(str::to_owned);
    let answer =
        tokio::task::spawn_blocking(move || shared.respond(&method, &path, query.as_deref()))
            .await
            .unwrap_or_else(|_| ApiResponse::internal());
    reply(
        answer.status,
        "application/json",
        "no-store",
        CONTENT_SECURITY_POLICY,
        answer.body.into_bytes(),
    )
}

async fn sync_loop(shared: Arc<Shared>, source: Arc<RpcSource>) {
    let mut tick = tokio::time::interval(SYNC_INTERVAL);
    tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        tick.tick().await;
        let shared = Arc::clone(&shared);
        let source = Arc::clone(&source);
        if tokio::task::spawn_blocking(move || shared.sync_once(&source))
            .await
            .is_err()
        {
            tracing::warn!("langkah sinkronisasi berhenti tidak normal");
        }
    }
}

/// Memasang logger ke stderr (level INFO, tanpa warna). Jika logger global sudah ada, logger lama tetap dipakai.
pub fn init_logging() {
    // Hanya gagal jika subscriber global sudah dipasang sebelumnya; itu tidak menghalangi explorer.
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
        // Galat berarti sinyal tidak bisa dipasang; explorer berhenti seperti menerima Ctrl-C.
        let _ = tokio::signal::ctrl_c().await;
    })
}

async fn serve(indexer: Indexer, config: &Config) -> Result<(), ExplorerError> {
    let shutdown = shutdown_signal()?;
    let client =
        RpcClient::new(&config.rpc_url).map_err(|error| ExplorerError::Rpc(error.to_string()))?;
    let source = Arc::new(RpcSource::new(client, Handle::current()));
    let shared = Arc::new(Shared::new(indexer, config.network));
    let listener =
        TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, config.http_port))).await?;
    let local = listener.local_addr()?;
    tracing::info!(%local, rpc = %config.rpc_url, "explorer berjalan");
    println!("http: http://{local}");
    let sync = tokio::spawn(sync_loop(Arc::clone(&shared), source));
    let served = axum::serve(listener, router(shared, config.web_dir.clone()))
        .with_graceful_shutdown(shutdown)
        .await;
    sync.abort();
    tracing::info!("explorer berhenti");
    Ok(served?)
}

/// Menjalankan explorer sampai menerima SIGINT atau SIGTERM.
///
/// # Errors
///
/// [`ExplorerError`] jika folder web tidak berisi `index.html`, atau folder data, indeks, runtime, sinyal, klien RPC,
/// atau port HTTP gagal.
pub fn run(config: &Config) -> Result<(), ExplorerError> {
    let missing_index = config
        .web_dir
        .as_ref()
        .filter(|dir| !dir.join(INDEX_FILE).is_file());
    if let Some(dir) = missing_index {
        return Err(ExplorerError::WebDir(dir.display().to_string()));
    }
    let path = db_path(config);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let network_id =
        regtest_network_id().map_err(|error| ExplorerError::Genesis(error.to_string()))?;
    let indexer = Indexer::open(&path, network_id)?;
    tracing::info!(path = %path.display(), "indeks explorer dibuka");
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let result = runtime.block_on(serve(indexer, config));
    runtime.shutdown_timeout(SHUTDOWN_TIMEOUT);
    result
}

#[cfg(test)]
mod tests;
