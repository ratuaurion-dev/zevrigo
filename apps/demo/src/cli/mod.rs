//! Argumen program `zevrigo-demo` (Rancangan Ekosistem Demo v0.1 Bagian 4.9). Tanpa `clap` (DEC-10 bagian 2): argumen
//! dibaca sendiri dan tidak ada file konfigurasi.

use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

use thiserror::Error;
use zevrigo_primitives::{Amount, Network, OutputKind, QUANTA_PER_ZVRG};
use zevrigo_wallet_core::decode_address;

use crate::upacara::{DEFAULT_FEE_RATE, Pocket};

/// Teks bantuan `--help`.
pub const USAGE: &str = concat!(
    "zevrigo-demo (MOD-18): layanan jaringan demo regtest Zevrigo Coin (DEC-34)\n",
    "\n",
    "Pemakaian:\n",
    "  zevrigo-demo rpc [--node-url <url>] [--http-bind <ip>] [--http-port <port>] [--rate <n>] [--trust-proxy]\n",
    "  zevrigo-demo upacara [--node-url <url>] --kantong <alamat>=<ZVRG>... [--fee-rate <n>] [--kirim]\n",
    "  zevrigo-demo faucet --datadir <dir> [--node-url <url>] [--http-bind <ip>] [--http-port <port>] [--rate <n>]\n",
    "                      [--jumlah <ZVRG>] [--batas-harian <n>] [--fee-rate <n>] [--trust-proxy-host <nama>]\n",
    "                      [--allow-origin <origin>]...\n",
    "  zevrigo-demo --help | --version\n",
    "\n",
    "Mode:\n",
    "  rpc                 gerbang RPC untuk wallet peserta: hanya get_chain_info, get_block_hash, get_block,\n",
    "                      get_mempool, scan_utxos, scan_mempool, dan send_raw_transaction yang diteruskan ke node;\n",
    "                      metode lain (termasuk get_block_template dan submit_block) dijawab -32601\n",
    "  upacara             upacara genesis regtest: membelanjakan alokasi genesis (10.000.000 ZVRG, kunci rahasia 1) ke\n",
    "                      kantong treasury dalam satu transaksi; jumlah semua kantong wajib tepat 10000000 ZVRG, biaya\n",
    "                      diambil dari kantong terakhir; tanpa --kirim hanya rencana dan txid\n",
    "  faucet              faucet demo: mengirim --jumlah ZVRG dari wallet faucet ke alamat regtest pengunjung, sekali\n",
    "                      per pengunjung dan sekali per alamat per hari UTC, paling banyak --batas-harian kiriman per\n",
    "                      hari; GET/POST /api/v1/faucet dengan badan {\"address\": \"rzvrg1...\"}\n",
    "  daftar              belum tersedia (TSK-M55)\n",
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
    "Opsi mode upacara:\n",
    "  --node-url <url>    node JSON-RPC regtest (bawaan http://127.0.0.1:28801)\n",
    "  --kantong <a>=<z>   alamat regtest jenis 0 (rzvrg1q...) dan jumlah ZVRG; 1 sampai 16 kali, urut seperti ditulis\n",
    "  --fee-rate <n>      tarif biaya quanta per byte (bawaan 1)\n",
    "  --kirim             kirim transaksi ke node (tanpa opsi ini: rencana saja)\n",
    "\n",
    "Opsi mode faucet:\n",
    "  --datadir <dir>     folder wallet faucet (wajib): faucet-frasa.txt (frasa 24 kata, dibuat sekali, izin 0600) dan\n",
    "                      faucet-batas.json (batas hari ini)\n",
    "  --node-url <url>    node JSON-RPC regtest (bawaan http://127.0.0.1:28801)\n",
    "  --http-bind <ip>    alamat dengar (bawaan 127.0.0.1)\n",
    "  --http-port <port>  port dengar (bawaan 28805; 0 = port bebas)\n",
    "  --rate <n>          batas laju per pengunjung seperti mode rpc (bawaan 2; 0 = tanpa batas)\n",
    "  --jumlah <ZVRG>     jumlah per kiriman (bawaan 100)\n",
    "  --batas-harian <n>  kiriman paling banyak per hari UTC (bawaan 100)\n",
    "  --fee-rate <n>      tarif biaya quanta per byte (bawaan 1)\n",
    "  --trust-proxy-host <nama>\n",
    "                      nama host proxy tepercaya (misalnya cloudflared); untuk koneksi dari alamatnya, pengunjung\n",
    "                      dikenali dari kepala CF-Connecting-IP (di-resolve ulang tiap 60 detik)\n",
    "  --allow-origin <o>  origin peramban yang boleh memanggil faucet (CORS); boleh berulang (bawaan\n",
    "                      https://zevrigo.ratuaurion.store)\n",
    "\n",
    "Setelah siap, alamat dengar dicetak ke stdout sebagai `http: http://<alamat>` (mode faucet juga\n",
    "`alamat faucet: <alamat>`). Log ditulis ke stderr tanpa alamat\n",
    "pengunjung, metode, atau parameter. Program berhenti bersih dengan Ctrl-C (SIGINT) atau SIGTERM.",
);

/// URL JSON-RPC node bawaan: node regtest di 127.0.0.1 (Spesifikasi RPC v0.1 Bagian 1).
pub const DEFAULT_NODE_URL: &str = "http://127.0.0.1:28801";

/// Port gerbang RPC bawaan (Rancangan Ekosistem Demo v0.1 Bagian 5).
pub const DEFAULT_RPC_PORT: u16 = 28_804;

/// Batas laju bawaan gerbang RPC (permintaan per detik per pengunjung). Lebih longgar dari explorer, karena riwayat wallet
/// memanggil `get_block` untuk setiap blok yang belum ada di temboloknya.
pub const DEFAULT_RATE: u32 = 20;

/// Port faucet bawaan (Rancangan Ekosistem Demo v0.1 Bagian 5).
pub const DEFAULT_FAUCET_PORT: u16 = 28_805;

/// Batas laju bawaan faucet (permintaan per detik per pengunjung).
pub const DEFAULT_FAUCET_RATE: u32 = 2;

/// Jumlah bawaan per kiriman faucet: 100 ZVRG (Rancangan Ekosistem Demo v0.1 Bagian 4.7).
pub const DEFAULT_FAUCET_AMOUNT: Amount = Amount::from_quanta(100 * QUANTA_PER_ZVRG);

/// Kiriman faucet paling banyak per hari UTC (bawaan).
pub const DEFAULT_DAILY_LIMIT: u32 = 100;

/// Origin bawaan yang boleh memanggil faucet dari peramban: explorer publik.
pub const DEFAULT_ORIGIN: &str = "https://zevrigo.ratuaurion.store";

/// Perintah hasil membaca argumen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Mencetak [`USAGE`].
    Help,
    /// Mencetak versi paket.
    Version,
    /// Menjalankan gerbang RPC (mode `rpc`).
    Rpc(RpcConfig),
    /// Upacara genesis regtest (mode `upacara`).
    Upacara(UpacaraConfig),
    /// Menjalankan faucet (mode `faucet`).
    Faucet(FaucetConfig),
}

/// Konfigurasi mode `faucet`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaucetConfig {
    /// URL JSON-RPC node regtest.
    pub node_url: String,
    /// Folder wallet faucet (frasa dan batas harian).
    pub datadir: PathBuf,
    /// Alamat dengar.
    pub http_bind: IpAddr,
    /// Port dengar. `0` memilih port bebas.
    pub http_port: u16,
    /// Batas laju (permintaan per detik per pengunjung). `0` berarti tanpa batas.
    pub rate: u32,
    /// Jumlah per kiriman.
    pub amount: Amount,
    /// Kiriman paling banyak per hari UTC.
    pub daily_limit: u32,
    /// Tarif biaya quanta per byte.
    pub fee_rate: u128,
    /// Nama host proxy tepercaya untuk kepala `CF-Connecting-IP`.
    pub trust_proxy_host: Option<String>,
    /// Origin peramban yang boleh memanggil faucet (CORS).
    pub origins: Vec<String>,
}

/// Konfigurasi mode `upacara`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpacaraConfig {
    /// URL JSON-RPC node regtest.
    pub node_url: String,
    /// Kantong tujuan, urut seperti ditulis.
    pub pockets: Vec<Pocket>,
    /// Tarif biaya quanta per byte.
    pub fee_rate: u128,
    /// Kirim transaksi ke node (`--kirim`).
    pub send: bool,
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
    /// Nilai `--kantong` tidak sah.
    #[error("kantong tidak sah ({reason}): {value}")]
    InvalidPocket {
        /// Nilai yang ditulis.
        value: String,
        /// Alasan.
        reason: &'static str,
    },
    /// Mode `upacara` tanpa `--kantong`.
    #[error("mode upacara wajib memakai --kantong paling sedikit sekali")]
    MissingPockets,
    /// Tarif biaya bukan bilangan bulat tak bertanda.
    #[error("tarif biaya tidak sah: {0}")]
    InvalidFeeRate(String),
    /// Opsi wajib tidak disebut.
    #[error("opsi {0} wajib diisi")]
    MissingOption(&'static str),
    /// Jumlah ZVRG bukan bilangan desimal lebih dari nol.
    #[error("jumlah ZVRG tidak sah: {0}")]
    InvalidAmount(String),
    /// Batas harian bukan bilangan 1 sampai 4294967295.
    #[error("batas harian tidak sah: {0}")]
    InvalidLimit(String),
    /// Origin bukan `http://...` atau `https://...` tanpa garis miring di akhir.
    #[error("origin tidak sah: {0}")]
    InvalidOrigin(String),
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
        "upacara" => parse_upacara(iter).map(Command::Upacara),
        "faucet" => parse_faucet(iter).map(Command::Faucet),
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

/// Membaca satu `--kantong <alamat>=<ZVRG>`: alamat regtest jenis 0 dan jumlah ZVRG lebih dari nol.
///
/// # Errors
///
/// [`ArgError::InvalidPocket`] dengan alasan yang dilanggar.
pub fn parse_pocket(value: &str) -> Result<Pocket, ArgError> {
    let bad = |reason| ArgError::InvalidPocket {
        value: value.to_owned(),
        reason,
    };
    let (address, amount) = value
        .split_once('=')
        .ok_or_else(|| bad("format <alamat>=<ZVRG>"))?;
    let (kind, payload) = decode_address(Network::Regtest, address.trim())
        .map_err(|_| bad("alamat regtest tidak sah"))?;
    if kind != OutputKind::SingleKey {
        return Err(bad("alamat wajib jenis 0 (rzvrg1q...)"));
    }
    let amount = Amount::from_zvrg_str(amount.trim())
        .ok()
        .filter(|amount| *amount != Amount::ZERO)
        .ok_or_else(|| bad("jumlah ZVRG tidak sah"))?;
    Ok(Pocket { payload, amount })
}

/// Membaca opsi mode `upacara`.
fn parse_upacara(mut iter: impl Iterator<Item = String>) -> Result<UpacaraConfig, ArgError> {
    let mut node_url: Option<String> = None;
    let mut pockets = Vec::new();
    let mut fee_rate: Option<u128> = None;
    let mut send = false;
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--node-url" => {
                if node_url.is_some() {
                    return Err(ArgError::Duplicate("--node-url"));
                }
                node_url = Some(value_of(&mut iter, "--node-url")?);
            }
            "--kantong" => {
                let value = value_of(&mut iter, "--kantong")?;
                pockets.push(parse_pocket(&value)?);
            }
            "--fee-rate" => {
                if fee_rate.is_some() {
                    return Err(ArgError::Duplicate("--fee-rate"));
                }
                let value = value_of(&mut iter, "--fee-rate")?;
                let rate = value
                    .parse::<u128>()
                    .map_err(|_| ArgError::InvalidFeeRate(value.clone()))?;
                fee_rate = Some(rate);
            }
            "--kirim" => {
                if send {
                    return Err(ArgError::Duplicate("--kirim"));
                }
                send = true;
            }
            "--help" | "-h" | "--version" => return Err(ArgError::Exclusive),
            _ => return Err(ArgError::Unknown(arg)),
        }
    }
    if pockets.is_empty() {
        return Err(ArgError::MissingPockets);
    }
    Ok(UpacaraConfig {
        node_url: node_url.unwrap_or_else(|| DEFAULT_NODE_URL.to_owned()),
        pockets,
        fee_rate: fee_rate.unwrap_or(DEFAULT_FEE_RATE),
        send,
    })
}

/// Mengisi opsi yang hanya boleh disebut sekali.
fn once<T>(slot: &mut Option<T>, value: T, option: &'static str) -> Result<(), ArgError> {
    if slot.is_some() {
        return Err(ArgError::Duplicate(option));
    }
    *slot = Some(value);
    Ok(())
}

/// Membaca opsi mode `faucet`.
fn parse_faucet(mut iter: impl Iterator<Item = String>) -> Result<FaucetConfig, ArgError> {
    let mut node_url: Option<String> = None;
    let mut datadir: Option<PathBuf> = None;
    let mut http_bind: Option<IpAddr> = None;
    let mut http_port: Option<u16> = None;
    let mut rate: Option<u32> = None;
    let mut amount: Option<Amount> = None;
    let mut daily_limit: Option<u32> = None;
    let mut fee_rate: Option<u128> = None;
    let mut trust_proxy_host: Option<String> = None;
    let mut origins: Vec<String> = Vec::new();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--node-url" => once(
                &mut node_url,
                value_of(&mut iter, "--node-url")?,
                "--node-url",
            )?,
            "--datadir" => once(
                &mut datadir,
                PathBuf::from(value_of(&mut iter, "--datadir")?),
                "--datadir",
            )?,
            "--http-bind" => {
                let value = value_of(&mut iter, "--http-bind")?;
                let address = value
                    .parse::<IpAddr>()
                    .map_err(|_| ArgError::InvalidBind(value.clone()))?;
                once(&mut http_bind, address, "--http-bind")?;
            }
            "--http-port" => {
                let value = value_of(&mut iter, "--http-port")?;
                let port = value
                    .parse::<u16>()
                    .map_err(|_| ArgError::InvalidPort(value.clone()))?;
                once(&mut http_port, port, "--http-port")?;
            }
            "--rate" => {
                let value = value_of(&mut iter, "--rate")?;
                let parsed = value
                    .parse::<u32>()
                    .map_err(|_| ArgError::InvalidRate(value.clone()))?;
                once(&mut rate, parsed, "--rate")?;
            }
            "--jumlah" => {
                let value = value_of(&mut iter, "--jumlah")?;
                let parsed = Amount::from_zvrg_str(&value)
                    .ok()
                    .filter(|amount| *amount != Amount::ZERO)
                    .ok_or_else(|| ArgError::InvalidAmount(value.clone()))?;
                once(&mut amount, parsed, "--jumlah")?;
            }
            "--batas-harian" => {
                let value = value_of(&mut iter, "--batas-harian")?;
                let parsed = value
                    .parse::<u32>()
                    .ok()
                    .filter(|limit| *limit > 0)
                    .ok_or_else(|| ArgError::InvalidLimit(value.clone()))?;
                once(&mut daily_limit, parsed, "--batas-harian")?;
            }
            "--fee-rate" => {
                let value = value_of(&mut iter, "--fee-rate")?;
                let parsed = value
                    .parse::<u128>()
                    .map_err(|_| ArgError::InvalidFeeRate(value.clone()))?;
                once(&mut fee_rate, parsed, "--fee-rate")?;
            }
            "--trust-proxy-host" => once(
                &mut trust_proxy_host,
                value_of(&mut iter, "--trust-proxy-host")?,
                "--trust-proxy-host",
            )?,
            "--allow-origin" => {
                let value = value_of(&mut iter, "--allow-origin")?;
                let valid = (value.starts_with("https://") || value.starts_with("http://"))
                    && !value.ends_with('/')
                    && value.is_ascii()
                    && !value.contains(char::is_whitespace);
                if !valid {
                    return Err(ArgError::InvalidOrigin(value));
                }
                origins.push(value);
            }
            "--help" | "-h" | "--version" => return Err(ArgError::Exclusive),
            _ => return Err(ArgError::Unknown(arg)),
        }
    }
    let datadir = datadir.ok_or(ArgError::MissingOption("--datadir"))?;
    if origins.is_empty() {
        origins.push(DEFAULT_ORIGIN.to_owned());
    }
    Ok(FaucetConfig {
        node_url: node_url.unwrap_or_else(|| DEFAULT_NODE_URL.to_owned()),
        datadir,
        http_bind: http_bind.unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        http_port: http_port.unwrap_or(DEFAULT_FAUCET_PORT),
        rate: rate.unwrap_or(DEFAULT_FAUCET_RATE),
        amount: amount.unwrap_or(DEFAULT_FAUCET_AMOUNT),
        daily_limit: daily_limit.unwrap_or(DEFAULT_DAILY_LIMIT),
        fee_rate: fee_rate.unwrap_or(DEFAULT_FEE_RATE),
        trust_proxy_host,
        origins,
    })
}

#[cfg(test)]
mod tests;
