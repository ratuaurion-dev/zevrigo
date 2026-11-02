//! Argumen program explorer (Spesifikasi Explorer v0.1 Bagian 6). Tanpa `clap` (DEC-10 bagian 2): argumen dibaca
//! sendiri dan tidak ada file konfigurasi.

use std::path::PathBuf;

use homeostasis_primitives::Network;
use homeostasis_rpc::transport::default_port;
use thiserror::Error;

use crate::db::DB_FILE;

/// Teks bantuan `--help`.
pub const USAGE: &str = concat!(
    "homeostasis-explorer (MOD-14): indexer dan API explorer Homeostasis Coin\n",
    "\n",
    "Pemakaian:\n",
    "  homeostasis-explorer --regtest --datadir <folder> [--rpc-url <url>] [--http-port <port>] [--web-dir <folder>]\n",
    "  homeostasis-explorer --help | --version\n",
    "\n",
    "Opsi:\n",
    "  --regtest           jaringan regtest (testnet dan mainnet menunggu DEC-01)\n",
    "  --datadir <folder>  folder data; indeks disimpan di <folder>/regtest/explorer.sqlite\n",
    "  --rpc-url <url>     node JSON-RPC (bawaan regtest http://127.0.0.1:28801)\n",
    "  --http-port <port>  port API HTTP di 127.0.0.1 (bawaan regtest 28802; 0 = port bebas)\n",
    "  --web-dir <folder>  antarmuka web hasil build apps/explorer/web (folder dist); tanpa opsi ini hanya API\n",
    "\n",
    "Setelah siap, alamat API dicetak ke stdout sebagai `http: http://<alamat>`; jalur API di /api/v1/.\n",
    "Log ditulis ke stderr. Explorer berhenti bersih dengan Ctrl-C (SIGINT) atau SIGTERM.",
);

/// Perintah hasil membaca argumen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Mencetak [`USAGE`].
    Help,
    /// Mencetak versi paket.
    Version,
    /// Menjalankan explorer.
    Run(Config),
}

/// Konfigurasi explorer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// Jaringan. Saat ini hanya regtest.
    pub network: Network,
    /// Folder data.
    pub datadir: PathBuf,
    /// URL JSON-RPC node.
    pub rpc_url: String,
    /// Port API HTTP di 127.0.0.1. `0` memilih port bebas.
    pub http_port: u16,
    /// Folder hasil build antarmuka web (`apps/explorer/web/dist`). `None` berarti hanya API.
    pub web_dir: Option<PathBuf>,
}

/// Argumen tidak sah. Biner keluar dengan kode 2.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ArgError {
    /// Argumen tidak dikenali.
    #[error("argumen tidak dikenali: {0}")]
    Unknown(String),
    /// Opsi butuh nilai, tetapi nilainya tidak ada, kosong, atau berupa opsi lain.
    #[error("opsi {0} butuh nilai")]
    MissingValue(&'static str),
    /// Opsi disebut lebih dari sekali.
    #[error("opsi {0} disebut lebih dari sekali")]
    Duplicate(&'static str),
    /// Port HTTP bukan bilangan 0 sampai 65535.
    #[error("port HTTP tidak sah: {0}")]
    InvalidPort(String),
    /// Jaringan belum tersedia.
    #[error("jaringan {0} belum tersedia (menunggu DEC-01)")]
    NetworkUnavailable(&'static str),
    /// Jaringan tidak dipilih.
    #[error("jaringan wajib dipilih: --regtest")]
    MissingNetwork,
    /// Folder data tidak diisi.
    #[error("folder data wajib diisi: --datadir <folder>")]
    MissingDatadir,
    /// `--help` atau `--version` digabung dengan opsi lain.
    #[error("--help dan --version tidak boleh digabung dengan opsi lain")]
    Exclusive,
}

/// Port API HTTP bawaan per jaringan (Spesifikasi Explorer v0.1 Bagian 6).
#[must_use]
pub const fn default_http_port(network: Network) -> u16 {
    match network {
        Network::Mainnet => 8_802,
        Network::Testnet => 18_802,
        Network::Regtest => 28_802,
    }
}

/// URL JSON-RPC bawaan: node di 127.0.0.1 dengan port RPC bawaan jaringan.
#[must_use]
pub fn default_rpc_url(network: Network) -> String {
    let port = default_port(network);
    format!("http://127.0.0.1:{port}")
}

/// Nama folder jaringan di dalam folder data.
const fn network_dir(network: Network) -> &'static str {
    match network {
        Network::Mainnet => "mainnet",
        Network::Testnet => "testnet",
        Network::Regtest => "regtest",
    }
}

/// Path file indeks: `<datadir>/<jaringan>/explorer.sqlite`.
#[must_use]
pub fn db_path(config: &Config) -> PathBuf {
    config
        .datadir
        .join(network_dir(config.network))
        .join(DB_FILE)
}

/// Membaca nilai opsi berikutnya. Nilai kosong atau yang diawali `--` ditolak.
fn value_of(
    iter: &mut impl Iterator<Item = String>,
    option: &'static str,
) -> Result<String, ArgError> {
    match iter.next() {
        Some(value) if !value.is_empty() && !value.starts_with("--") => Ok(value),
        _ => Err(ArgError::MissingValue(option)),
    }
}

/// Membaca argumen (tanpa nama program). Tanpa argumen sama dengan `--help`.
///
/// # Errors
///
/// [`ArgError`] untuk argumen yang tidak sah.
pub fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Result<Command, ArgError> {
    let args: Vec<String> = args.into_iter().collect();
    match args.as_slice() {
        [] => return Ok(Command::Help),
        [only] if only.as_str() == "--help" || only.as_str() == "-h" => return Ok(Command::Help),
        [only] if only.as_str() == "--version" => return Ok(Command::Version),
        _ => {}
    }
    let mut regtest = false;
    let mut datadir: Option<PathBuf> = None;
    let mut rpc_url: Option<String> = None;
    let mut http_port: Option<u16> = None;
    let mut web_dir: Option<PathBuf> = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--regtest" => {
                if regtest {
                    return Err(ArgError::Duplicate("--regtest"));
                }
                regtest = true;
            }
            "--testnet" => return Err(ArgError::NetworkUnavailable("testnet")),
            "--mainnet" => return Err(ArgError::NetworkUnavailable("mainnet")),
            "--datadir" => {
                if datadir.is_some() {
                    return Err(ArgError::Duplicate("--datadir"));
                }
                datadir = Some(PathBuf::from(value_of(&mut iter, "--datadir")?));
            }
            "--rpc-url" => {
                if rpc_url.is_some() {
                    return Err(ArgError::Duplicate("--rpc-url"));
                }
                rpc_url = Some(value_of(&mut iter, "--rpc-url")?);
            }
            "--http-port" => {
                if http_port.is_some() {
                    return Err(ArgError::Duplicate("--http-port"));
                }
                let value = value_of(&mut iter, "--http-port")?;
                let port = value
                    .parse::<u16>()
                    .map_err(|_| ArgError::InvalidPort(value.clone()))?;
                http_port = Some(port);
            }
            "--web-dir" => {
                if web_dir.is_some() {
                    return Err(ArgError::Duplicate("--web-dir"));
                }
                web_dir = Some(PathBuf::from(value_of(&mut iter, "--web-dir")?));
            }
            "--help" | "-h" | "--version" => return Err(ArgError::Exclusive),
            _ => return Err(ArgError::Unknown(arg)),
        }
    }
    if !regtest {
        return Err(ArgError::MissingNetwork);
    }
    let datadir = datadir.ok_or(ArgError::MissingDatadir)?;
    Ok(Command::Run(Config {
        network: Network::Regtest,
        datadir,
        rpc_url: rpc_url.unwrap_or_else(|| default_rpc_url(Network::Regtest)),
        http_port: http_port.unwrap_or(default_http_port(Network::Regtest)),
        web_dir,
    }))
}

#[cfg(test)]
mod tests;
