//! Argumen program explorer (Spesifikasi Explorer v0.1 Bagian 6). Tanpa `clap` (DEC-10 bagian 2): argumen dibaca
//! sendiri dan tidak ada file konfigurasi.

use std::net::{IpAddr, Ipv4Addr};
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
    "  homeostasis-explorer --regtest --datadir <folder> [--rpc-url <url>] [--http-bind <ip>] [--http-port <port>]\n",
    "                       [--api-rate <n>] [--web-dir <folder>]\n",
    "  homeostasis-explorer --testnet --datadir <folder> [opsi yang sama]\n",
    "  homeostasis-explorer --help | --version\n",
    "\n",
    "Opsi:\n",
    "  --regtest           jaringan regtest\n",
    "  --testnet           jaringan testnet (mainnet diluncurkan sesudah testnet)\n",
    "  --datadir <folder>  folder data; indeks disimpan di <folder>/<jaringan>/explorer.sqlite\n",
    "  --rpc-url <url>     node JSON-RPC (bawaan regtest http://127.0.0.1:28801, testnet http://127.0.0.1:18801)\n",
    "  --http-bind <ip>    alamat dengar API HTTP (bawaan 127.0.0.1); alamat lain hanya di dalam container atau di\n",
    "                      belakang reverse proxy dengan batas laju (TSK-M33)\n",
    "  --http-port <port>  port API HTTP (bawaan regtest 28802, testnet 18802; 0 = port bebas)\n",
    "  --api-rate <n>      batas laju API: n permintaan per detik per klien, kapasitas 4n; semua klien 20n per detik\n",
    "                      (bawaan 10; 0 = tanpa batas). Jawaban di atas batas: 429 rate_limited\n",
    "  --web-dir <folder>  antarmuka web hasil build apps/explorer/web (folder out); tanpa opsi ini hanya API\n",
    "\n",
    "Setelah siap, alamat API dicetak ke stdout sebagai `http: http://<alamat>`; jalur API di /api/v1/.\n",
    "Log ditulis ke stderr. Explorer berhenti bersih dengan Ctrl-C (SIGINT) atau SIGTERM.",
);

/// Batas laju API bawaan: permintaan per detik per klien (Spesifikasi Explorer v0.1 Bagian 6).
pub const DEFAULT_API_RATE: u32 = 10;

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
    /// Jaringan: regtest atau testnet.
    pub network: Network,
    /// Folder data.
    pub datadir: PathBuf,
    /// URL JSON-RPC node.
    pub rpc_url: String,
    /// Alamat dengar API HTTP. Bawaan `127.0.0.1`; alamat lain hanya untuk container (TSK-M33).
    pub http_bind: IpAddr,
    /// Port API HTTP. `0` memilih port bebas.
    pub http_port: u16,
    /// Batas laju API (permintaan per detik per klien). `0` berarti tanpa batas.
    pub api_rate: u32,
    /// Folder hasil build antarmuka web (`apps/explorer/web/out`). `None` berarti hanya API.
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
    /// Alamat dengar bukan alamat IPv4 atau IPv6.
    #[error("alamat dengar HTTP tidak sah: {0}")]
    InvalidBind(String),
    /// Batas laju bukan bilangan 0 sampai 4294967295.
    #[error("batas laju API tidak sah: {0}")]
    InvalidRate(String),
    /// Jaringan belum tersedia.
    #[error("jaringan {0} belum tersedia (mainnet diluncurkan sesudah testnet)")]
    NetworkUnavailable(&'static str),
    /// Jaringan tidak dipilih.
    #[error("jaringan wajib dipilih: --regtest atau --testnet")]
    MissingNetwork,
    /// Lebih dari satu jaringan dipilih.
    #[error("hanya satu jaringan boleh dipilih: --regtest atau --testnet")]
    NetworkConflict,
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

/// Mencatat pilihan jaringan dari `flag`. Opsi yang sama dua kali atau dua jaringan berbeda ditolak.
fn choose_network(
    slot: &mut Option<Network>,
    chosen: Network,
    flag: &'static str,
) -> Result<(), ArgError> {
    match *slot {
        Some(previous) if previous == chosen => Err(ArgError::Duplicate(flag)),
        Some(_) => Err(ArgError::NetworkConflict),
        None => {
            *slot = Some(chosen);
            Ok(())
        }
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
    let mut network: Option<Network> = None;
    let mut datadir: Option<PathBuf> = None;
    let mut rpc_url: Option<String> = None;
    let mut http_bind: Option<IpAddr> = None;
    let mut http_port: Option<u16> = None;
    let mut api_rate: Option<u32> = None;
    let mut web_dir: Option<PathBuf> = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--regtest" => choose_network(&mut network, Network::Regtest, "--regtest")?,
            "--testnet" => choose_network(&mut network, Network::Testnet, "--testnet")?,
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
            "--http-bind" => {
                if http_bind.is_some() {
                    return Err(ArgError::Duplicate("--http-bind"));
                }
                let value = value_of(&mut iter, "--http-bind")?;
                let address = value
                    .parse::<IpAddr>()
                    .map_err(|_| ArgError::InvalidBind(value.clone()))?;
                http_bind = Some(address);
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
            "--api-rate" => {
                if api_rate.is_some() {
                    return Err(ArgError::Duplicate("--api-rate"));
                }
                let value = value_of(&mut iter, "--api-rate")?;
                let rate = value
                    .parse::<u32>()
                    .map_err(|_| ArgError::InvalidRate(value.clone()))?;
                api_rate = Some(rate);
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
    let network = network.ok_or(ArgError::MissingNetwork)?;
    let datadir = datadir.ok_or(ArgError::MissingDatadir)?;
    Ok(Command::Run(Config {
        network,
        datadir,
        rpc_url: rpc_url.unwrap_or_else(|| default_rpc_url(network)),
        http_bind: http_bind.unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        http_port: http_port.unwrap_or(default_http_port(network)),
        api_rate: api_rate.unwrap_or(DEFAULT_API_RATE),
        web_dir,
    }))
}

#[cfg(test)]
mod tests;
