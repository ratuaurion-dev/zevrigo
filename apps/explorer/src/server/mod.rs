//! Program explorer (Spesifikasi Explorer v0.1 Bagian 6): server HTTP axum di 127.0.0.1, sinkronisasi indeks dari node
//! setiap 2 detik, dan berhenti bersih dengan SIGINT atau SIGTERM.
//!
//! Satu koneksi SQLite dijaga mutex. Permintaan HTTP dan langkah sinkronisasi dijalankan di `spawn_blocking`, sehingga
//! akses SQLite dan RPC sinkron tidak menahan thread async. Tanpa cookie, analitik, atau log alamat IP, jalur, dan kata
//! pencarian; log hanya untuk start, kemajuan indeks, reorg, dan galat.

use std::future::Future;
use std::net::{Ipv4Addr, SocketAddr};
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

/// Router axum: semua jalur dan metode diteruskan ke [`Shared::respond`].
pub fn router(shared: Arc<Shared>) -> Router {
    Router::new().fallback(handle).with_state(shared)
}

async fn handle(State(shared): State<Arc<Shared>>, method: Method, uri: Uri) -> Response {
    let method = method.as_str().to_owned();
    let path = uri.path().to_owned();
    let query = uri.query().map(str::to_owned);
    let answer =
        tokio::task::spawn_blocking(move || shared.respond(&method, &path, query.as_deref()))
            .await
            .unwrap_or_else(|_| ApiResponse::internal());
    let status = StatusCode::from_u16(answer.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        answer.body,
    )
        .into_response()
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
    let served = axum::serve(listener, router(shared))
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
/// [`ExplorerError`] jika folder data, indeks, runtime, sinyal, klien RPC, atau port HTTP gagal.
pub fn run(config: &Config) -> Result<(), ExplorerError> {
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
