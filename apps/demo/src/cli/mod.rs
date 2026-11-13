//! Argumen program `zevrigo-demo` (Rancangan Ekosistem Demo v0.1 Bagian 4.9). Tanpa `clap` (DEC-10 bagian 2): argumen
//! dibaca sendiri dan tidak ada file konfigurasi.

use std::net::{IpAddr, Ipv4Addr};

use thiserror::Error;

/// Teks bantuan `--help`.
pub const USAGE: &str = concat!(
    "zevrigo-demo (MOD-18): layanan jaringan demo regtest Zevrigo Coin (DEC-34)\n",
    "\n",
    "Pemakaian:\n",
    "  zevrigo-demo rpc [--node-url <url>] [--http-bind <ip>] [--http-port <port>] [--rate <n>] [--trust-proxy]\n",
    "  zevrigo-demo --help | --version\n",
    "\n",
    "Mode:\n",
    "  rpc                 gerbang RPC untuk wallet peserta: hanya get_chain_info, get_block_hash, get_block,\n",
    "                      get_mempool, scan_utxos, scan_mempool, dan send_raw_transaction yang diteruskan ke node;\n",
    "                      metode lain (termasuk get_block_template dan submit_block) dijawab -32601\n",
    "  faucet, daftar      belum tersedia (TSK-M54, TSK-M55)\n",
    "\n",
    "Opsi mode rpc:\n",
    "  --node-url <url>    node JSON-RPC (bawaan http://127.0.0.1:28801, node regtest)\n",
    "  --http-bind <ip>    alamat dengar (bawaan 127.0.0.1)\n",
    "  --http-port <port>  port dengar (bawaan 28804; 0 = port bebas)\n",
    "  --rate <n>          batas laju: n permintaan per detik per pengunjung, kapasitas 4n; semua pengunjung 20n per\n",
    "                      detik (bawaan 20; 0 = tanpa batas). Jawaban di atas batas: 429 rate_limited\n",
    "  --trust-proxy       di belakang Cloudflare Tunnel: untuk koneksi dari 127.0.0.1 atau ::1, batas laju memakai\n",
    "                      alamat pengunjung dari kepala CF-Connecting-IP; koneksi dari alamat lain tidak terpengaruh\n",
    "\n",
    "Setelah siap, alamat dengar dicetak ke stdout sebagai `http: http://<alamat>`. Log ditulis ke stderr tanpa alamat\n",
    "pengunjung, metode, atau parameter. Program berhenti bersih dengan Ctrl-C (SIGINT) atau SIGTERM.",
);

/// URL JSON-RPC node bawaan: node regtest di 127.0.0.1 (Spesifikasi RPC v0.1 Bagian 1).
pub const DEFAULT_NODE_URL: &str = "http://127.0.0.1:28801";

/// Port gerbang RPC bawaan (Rancangan Ekosistem Demo v0.1 Bagian 5).
pub const DEFAULT_RPC_PORT: u16 = 28_804;

/// Batas laju bawaan gerbang RPC (permintaan per detik per pengunjung). Lebih longgar dari explorer, karena riwayat wallet
/// memanggil `get_block` untuk setiap blok yang belum ada di temboloknya.
pub const DEFAULT_RATE: u32 = 20;

/// Perintah hasil membaca argumen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Mencetak [`USAGE`].
    Help,
    /// Mencetak versi paket.
    Version,
    /// Menjalankan gerbang RPC (mode `rpc`).
    Rpc(RpcConfig),
}

/// Konfigurasi mode `rpc`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RpcConfig {
    /// URL JSON-RPC node tujuan.
    pub node_url: String,
    /// Alamat dengar.
    pub http_bind: IpAddr,
    /// Port dengar. `0` memilih port bebas.
    pub http_port: u16,
    /// Batas laju (permintaan per detik per pengunjung). `0` berarti tanpa batas.
    pub rate: u32,
    /// Batas laju memakai alamat pengunjung dari kepala `CF-Connecting-IP` untuk koneksi dari loopback
    /// (`--trust-proxy`, Cloudflare Tunnel).
    pub trust_proxy: bool,
}

/// Argumen tidak sah. Biner keluar dengan kode 2.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ArgError {
    /// Mode tidak diisi.
    #[error("mode wajib diisi: rpc")]
    MissingMode,
    /// Mode tidak dikenali.
    #[error("mode tidak dikenali: {0}")]
    UnknownMode(String),
    /// Mode sudah direncanakan tetapi belum tersedia.
    #[error("mode {0} belum tersedia")]
    NotYet(&'static str),
    /// Argumen tidak dikenali.
    #[error("argumen tidak dikenali: {0}")]
    Unknown(String),
    /// Opsi butuh nilai, tetapi nilainya tidak ada, kosong, atau berupa opsi lain.
    #[error("opsi {0} butuh nilai")]
    MissingValue(&'static str),
    /// Opsi disebut lebih dari sekali.
    #[error("opsi {0} disebut lebih dari sekali")]
    Duplicate(&'static str),
    /// Port bukan bilangan 0 sampai 65535.
    #[error("port HTTP tidak sah: {0}")]
    InvalidPort(String),
    /// Alamat dengar bukan alamat IPv4 atau IPv6.
    #[error("alamat dengar HTTP tidak sah: {0}")]
    InvalidBind(String),
    /// Batas laju bukan bilangan 0 sampai 4294967295.
    #[error("batas laju tidak sah: {0}")]
    InvalidRate(String),
    /// `--help` atau `--version` digabung dengan argumen lain.
    #[error("--help dan --version tidak boleh digabung dengan argumen lain")]
    Exclusive,
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

/// Membaca argumen (tanpa nama program). Tanpa argumen sama dengan `--help`. Argumen pertama adalah mode.
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
    let mut iter = args.into_iter();
    let mode = iter.next().ok_or(ArgError::MissingMode)?;
    match mode.as_str() {
        "rpc" => parse_rpc(iter).map(Command::Rpc),
        "faucet" => Err(ArgError::NotYet("faucet")),
        "daftar" => Err(ArgError::NotYet("daftar")),
        "--help" | "-h" | "--version" => Err(ArgError::Exclusive),
        option if option.starts_with("--") => Err(ArgError::MissingMode),
        _ => Err(ArgError::UnknownMode(mode)),
    }
}

/// Membaca opsi mode `rpc`.
fn parse_rpc(mut iter: impl Iterator<Item = String>) -> Result<RpcConfig, ArgError> {
    let mut node_url: Option<String> = None;
    let mut http_bind: Option<IpAddr> = None;
    let mut http_port: Option<u16> = None;
    let mut rate: Option<u32> = None;
    let mut trust_proxy = false;
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--node-url" => {
                if node_url.is_some() {
                    return Err(ArgError::Duplicate("--node-url"));
                }
                node_url = Some(value_of(&mut iter, "--node-url")?);
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
            "--rate" => {
                if rate.is_some() {
                    return Err(ArgError::Duplicate("--rate"));
                }
                let value = value_of(&mut iter, "--rate")?;
                let parsed = value
                    .parse::<u32>()
                    .map_err(|_| ArgError::InvalidRate(value.clone()))?;
                rate = Some(parsed);
            }
            "--trust-proxy" => {
                if trust_proxy {
                    return Err(ArgError::Duplicate("--trust-proxy"));
                }
                trust_proxy = true;
            }
            "--help" | "-h" | "--version" => return Err(ArgError::Exclusive),
            _ => return Err(ArgError::Unknown(arg)),
        }
    }
    Ok(RpcConfig {
        node_url: node_url.unwrap_or_else(|| DEFAULT_NODE_URL.to_owned()),
        http_bind: http_bind.unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        http_port: http_port.unwrap_or(DEFAULT_RPC_PORT),
        rate: rate.unwrap_or(DEFAULT_RATE),
        trust_proxy,
    })
}

#[cfg(test)]
mod tests;
