//! Faucet jaringan demo regtest (Rancangan Ekosistem Demo v0.1 Bagian 4.7, TSK-M54 bagian A): server HTTP axum yang
//! mengirim [`Settings::amount`] (bawaan 100 ZVRG) dari wallet faucet ke alamat regtest pengunjung.
//!
//! - Kunci faucet: frasa 24 kata di `<datadir>/faucet-frasa.txt` (dibuat sekali, izin `0600`), jalur `m/44'/1'/0'/0/0`
//!   seperti wallet CLI. Wallet ini panas (kunci di server) dan hanya untuk regtest; isinya dari kantong Faucet treasury
//!   lewat isi ulang manual.
//! - Batas: satu kali per pengunjung per hari UTC, satu kali per alamat per hari UTC, dan [`Settings::daily_limit`]
//!   kiriman per hari. Batas alamat dan batas harian disimpan di `<datadir>/faucet-batas.json` (isi: hari, jumlah, alamat
//!   penerima hari itu) sehingga tetap berlaku setelah restart. Pengunjung hanya dikenali lewat hash alamat IP dengan
//!   kunci acak per hari di memori: alamat IP tidak disimpan, tidak dicatat, dan hilang saat restart.
//! - Kiriman berantai lewat mempool: kembalian kiriman yang belum masuk blok langsung bisa dipakai kiriman berikutnya.
//!   Outpoint yang sudah dibelanjakan kiriman sendiri dicatat di memori. Setelah start, faucet menunggu sampai mempool
//!   tidak lagi memuat transaksi faucet (paling lama satu blok) sebelum mengirim lagi ([`FaucetError::Busy`]).
//! - HTTP: `GET` dan `POST` [`FAUCET_PATH`], `OPTIONS` untuk CORS. Hanya origin di daftar izin yang mendapat kepala CORS.
//!   Batas laju per pengunjung seperti gerbang RPC ([`crate::limit`]). Di belakang Cloudflare Tunnel, alamat pengunjung
//!   dari [`CLIENT_IP_HEADER`] hanya dipakai untuk koneksi dari alamat proxy tepercaya ([`faucet_client_ip`]).
//! - Privasi: log hanya start, berhenti, txid kiriman, dan galat node. Tanpa alamat penerima dan tanpa alamat IP.

use std::collections::{BTreeSet, HashSet};
use std::hash::{BuildHasher, RandomState};
use std::io::{Read as _, Write as _};
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex as StdMutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{ConnectInfo, State};
use axum::http::{Extensions, HeaderMap, HeaderValue, Method, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use jsonrpsee::core::ClientError;
use serde_json::{Value, json};
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use zevrigo_primitives::{Amount, Encode, Hash32, Network, OutPoint, OutputKind};
use zevrigo_rpc::{Hex32, RpcClient, error_code, reject_reason};
use zevrigo_wallet_core::{
    Coin, ExtendedPrivKey, Mnemonic, RECEIVE_BRANCH, SendError, SendRequest, account_key,
    build_transaction, decode_address, encode_address,
};

use crate::cli::FaucetConfig;
use crate::gateway::{DemoError, shutdown_signal};
use crate::limit::{Limits, RateLimiter};
use crate::upacara::REGTEST_NETWORK_ID;

/// Nama file frasa wallet faucet di dalam `--datadir`.
pub const KEY_FILE: &str = "faucet-frasa.txt";

/// Nama file batas harian faucet di dalam `--datadir`.
pub const STATE_FILE: &str = "faucet-batas.json";

/// Panjang satu hari batas faucet (hari UTC).
pub const SECONDS_PER_DAY: u64 = 86_400;

/// Jalur HTTP faucet.
pub const FAUCET_PATH: &str = "/api/v1/faucet";

/// Badan `POST` paling besar (byte).
pub const MAX_BODY_BYTES: usize = 4_096;

/// Kepala alamat pengunjung dari Cloudflare Tunnel.
pub const CLIENT_IP_HEADER: &str = "cf-connecting-ip";

/// Selang resolusi ulang nama proxy tepercaya.
const RESOLVE_INTERVAL: Duration = Duration::from_secs(60);

/// Batas waktu menutup runtime setelah server berhenti.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// Sumber 32 byte acak: entropi frasa baru dan `aux_rand` tanda tangan BIP340.
pub type RandomSource = Box<dyn FnMut() -> std::io::Result<[u8; 32]> + Send>;

/// 32 byte acak dari `/dev/urandom` (Linux, image Docker). Di sistem tanpa `/dev/urandom` selalu galat.
///
/// # Errors
///
/// Galat I/O jika `/dev/urandom` tidak bisa dibaca.
pub fn os_random() -> std::io::Result<[u8; 32]> {
    let mut bytes = [0_u8; 32];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(bytes)
}

/// Waktu sekarang dalam detik Unix (UTC).
#[must_use]
pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

/// Galat faucet. [`FaucetError::code`] dan [`FaucetError::http_status`] dipakai untuk jawaban HTTP.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum FaucetError {
    /// Alamat bukan alamat regtest yang sah.
    #[error("alamat regtest tidak sah")]
    BadAddress,
    /// Alamat tujuan adalah alamat faucet sendiri.
    #[error("alamat faucet sendiri tidak bisa menerima")]
    OwnAddress,
    /// Batas kiriman harian sudah habis.
    #[error("batas harian faucet sudah habis, coba lagi besok (00.00 UTC)")]
    DailyLimit,
    /// Pengunjung ini sudah menerima koin hari ini.
    #[error("pengunjung ini sudah menerima koin hari ini, coba lagi besok (00.00 UTC)")]
    VisitorLimit,
    /// Alamat ini sudah menerima koin hari ini.
    #[error("alamat ini sudah menerima koin hari ini, coba lagi besok (00.00 UTC)")]
    AddressLimit,
    /// Saldo faucet yang bisa dipakai tidak cukup.
    #[error("saldo faucet habis, tunggu isi ulang")]
    Empty,
    /// Faucet baru start dan menunggu transaksi lama masuk blok.
    #[error("faucet sedang bersiap, coba lagi beberapa menit lagi")]
    Busy,
    /// Node tidak menjawab.
    #[error("node tidak bisa dihubungi: {0}")]
    NodeUnreachable(String),
    /// Node bukan jaringan demo regtest.
    #[error("node bukan jaringan demo regtest: {0}")]
    WrongNetwork(String),
    /// Node menolak permintaan atau transaksi.
    #[error("node menolak: {0}")]
    Rejected(String),
    /// File frasa tidak bisa dibuat atau dibaca.
    #[error("kunci faucet: {0}")]
    Key(String),
    /// File batas tidak bisa dibaca atau isinya tidak sah.
    #[error("file batas faucet: {0}")]
    State(String),
    /// Galat lain.
    #[error("galat internal: {0}")]
    Internal(String),
}

impl FaucetError {
    /// Kode galat untuk jawaban JSON (`{"error": kode}`).
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::BadAddress => "bad_address",
            Self::OwnAddress => "own_address",
            Self::DailyLimit => "daily_limit",
            Self::VisitorLimit => "visitor_limit",
            Self::AddressLimit => "address_limit",
            Self::Empty => "empty",
            Self::Busy => "busy",
            Self::NodeUnreachable(_) => "node_unreachable",
            Self::WrongNetwork(_) => "wrong_network",
            Self::Rejected(_) => "rejected",
            Self::Key(_) | Self::State(_) | Self::Internal(_) => "internal",
        }
    }

    /// Status HTTP untuk galat ini.
    #[must_use]
    pub const fn http_status(&self) -> u16 {
        match self {
            Self::BadAddress | Self::OwnAddress => 400,
            Self::DailyLimit | Self::VisitorLimit | Self::AddressLimit => 429,
            Self::Empty
            | Self::Busy
            | Self::NodeUnreachable(_)
            | Self::WrongNetwork(_)
            | Self::Rejected(_) => 503,
            Self::Key(_) | Self::State(_) | Self::Internal(_) => 500,
        }
    }

    /// Pesan untuk pengunjung. Rincian galat node dan galat internal tidak ditampilkan.
    #[must_use]
    pub fn public_message(&self) -> String {
        match self {
            Self::NodeUnreachable(_)
            | Self::WrongNetwork(_)
            | Self::Rejected(_)
            | Self::Key(_)
            | Self::State(_)
            | Self::Internal(_) => "faucet sedang bermasalah, coba lagi nanti".to_owned(),
            other => other.to_string(),
        }
    }
}

/// Pengaturan kiriman faucet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Jumlah per kiriman.
    pub amount: Amount,
    /// Tarif biaya quanta per byte.
    pub fee_rate: u128,
    /// Kiriman paling banyak per hari UTC.
    pub daily_limit: u32,
}

/// Satu kiriman yang sudah diterima node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Payout {
    /// Txid kiriman.
    pub txid: Hash32,
    /// Alamat penerima (bentuk kanonis).
    pub address: String,
    /// Jumlah yang diterima.
    pub amount: Amount,
    /// Biaya transaksi.
    pub fee: Amount,
}

impl Payout {
    /// Jawaban JSON `POST`.
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "txid": String::from(Hex32::from(self.txid)),
            "address": self.address,
            "amount": self.amount.as_quanta().to_string(),
            "amount_zvrg": self.amount.to_zvrg_string(),
            "fee": self.fee.as_quanta().to_string(),
        })
    }
}

/// Keadaan faucet untuk `GET`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaucetStatus {
    /// Alamat faucet.
    pub address: String,
    /// Jumlah per kiriman.
    pub amount: Amount,
    /// Saldo yang bisa dipakai sekarang (termasuk kembalian yang belum masuk blok).
    pub balance: Amount,
    /// Kiriman paling banyak per hari UTC.
    pub daily_limit: u32,
    /// Sisa kiriman hari ini.
    pub remaining_today: u32,
    /// Mempool sudah diperiksa setelah start (lihat [`FaucetError::Busy`]).
    pub synced: bool,
    /// Faucet siap mengirim: sudah sinkron, sisa harian ada, dan saldo lebih dari satu kiriman.
    pub ready: bool,
}

impl FaucetStatus {
    /// Jawaban JSON `GET`.
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "network": "regtest",
            "address": self.address,
            "amount": self.amount.as_quanta().to_string(),
            "amount_zvrg": self.amount.to_zvrg_string(),
            "balance": self.balance.as_quanta().to_string(),
            "balance_zvrg": self.balance.to_zvrg_string(),
            "daily_limit": self.daily_limit,
            "remaining_today": self.remaining_today,
            "synced": self.synced,
            "ready": self.ready,
            "resets_at_utc": "00:00",
        })
    }
}

/// Batas yang disimpan di [`STATE_FILE`]: hari UTC, jumlah kiriman, dan alamat penerima hari itu.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Ledger {
    day: u64,
    count: u32,
    addresses: BTreeSet<String>,
}

impl Ledger {
    const fn empty(day: u64) -> Self {
        Self {
            day,
            count: 0,
            addresses: BTreeSet::new(),
        }
    }
}

/// Pengunjung yang sudah menerima hari ini: hash alamat dengan kunci acak per hari, hanya di memori.
struct Visitors {
    day: u64,
    keys: RandomState,
    seen: HashSet<u64>,
}

impl Visitors {
    fn new(day: u64) -> Self {
        Self {
            day,
            keys: RandomState::new(),
            seen: HashSet::new(),
        }
    }
}

/// Koin faucet yang bisa dipilih dan tinggi ujung chain.
struct View {
    tip_height: u64,
    coins: Vec<Coin>,
    spendable: Amount,
}

/// Galat kunci faucet dari galat apa pun.
fn key(error: impl std::fmt::Display) -> FaucetError {
    FaucetError::Key(error.to_string())
}

/// Membaca frasa faucet dari `path`, atau membuatnya sekali dari 32 byte `random` (24 kata) dengan izin `0600`.
///
/// # Errors
///
/// [`FaucetError::Key`] jika file tidak bisa dibaca atau dibuat, atau isinya bukan frasa BIP39 yang sah.
pub fn load_or_create_phrase(
    path: &Path,
    random: &mut RandomSource,
) -> Result<Mnemonic, FaucetError> {
    match std::fs::read_to_string(path) {
        Ok(text) => Mnemonic::parse(text.trim()).map_err(key),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let entropy = random().map_err(key)?;
            let mnemonic = Mnemonic::from_entropy(&entropy).map_err(key)?;
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt as _;
                options.mode(0o600);
            }
            let mut file = options.open(path).map_err(key)?;
            file.write_all(mnemonic.phrase().as_bytes())
                .and_then(|()| file.write_all(b"\n"))
                .and_then(|()| file.sync_all())
                .map_err(key)?;
            Ok(mnemonic)
        }
        Err(error) => Err(key(&error)),
    }
}

/// Kunci akun regtest dan payload alamat penerimaan pertama (`0/0`) dari frasa faucet tanpa passphrase.
fn faucet_keys(mnemonic: &Mnemonic) -> Result<(ExtendedPrivKey, [u8; 32]), FaucetError> {
    let seed = mnemonic.to_seed("").map_err(key)?;
    let account = account_key(&seed, Network::Regtest).map_err(key)?;
    let payload = account
        .derive_path(&[RECEIVE_BRANCH, 0])
        .and_then(|child| child.to_public())
        .map_err(key)?
        .x_only();
    Ok((account, payload))
}

/// Membaca [`STATE_FILE`]. File yang tidak ada atau berisi hari lain berarti batas kosong untuk `day`.
fn load_ledger(path: &Path, day: u64) -> Result<Ledger, FaucetError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Ledger::empty(day));
        }
        Err(error) => return Err(FaucetError::State(error.to_string())),
    };
    let value: Value =
        serde_json::from_str(&text).map_err(|error| FaucetError::State(error.to_string()))?;
    let version = value.get("versi").and_then(Value::as_u64);
    let stored_day = value.get("hari").and_then(Value::as_u64);
    let count = value
        .get("jumlah")
        .and_then(Value::as_u64)
        .and_then(|count| u32::try_from(count).ok());
    let addresses = value
        .get("alamat")
        .and_then(Value::as_array)
        .and_then(|list| {
            list.iter()
                .map(|item| item.as_str().map(str::to_owned))
                .collect::<Option<BTreeSet<String>>>()
        });
    match (version, stored_day, count, addresses) {
        (Some(1), Some(stored), Some(count), Some(addresses)) => Ok(if stored == day {
            Ledger {
                day,
                count,
                addresses,
            }
        } else {
            Ledger::empty(day)
        }),
        _ => Err(FaucetError::State(
            "isi tidak sah (hapus file ini untuk memulai batas hari ini dari nol)".to_owned(),
        )),
    }
}

/// Menulis [`STATE_FILE`] lewat file sementara lalu rename.
fn save_ledger(path: &Path, ledger: &Ledger) -> Result<(), FaucetError> {
    let value = json!({
        "versi": 1,
        "hari": ledger.day,
        "jumlah": ledger.count,
        "alamat": ledger.addresses.iter().collect::<Vec<_>>(),
    });
    let temporary = path.with_extension("json.baru");
    std::fs::write(&temporary, format!("{value}\n"))
        .and_then(|()| std::fs::rename(&temporary, path))
        .map_err(|error| FaucetError::State(error.to_string()))
}

/// Galat klien RPC: penolakan node menjadi [`FaucetError::Rejected`], node yang tidak menjawab menjadi
/// [`FaucetError::NodeUnreachable`].
fn node_error(error: &ClientError) -> FaucetError {
    if let Some(reason) = reject_reason(error) {
        return FaucetError::Rejected(reason);
    }
    match error_code(error) {
        Some(code) => FaucetError::Rejected(format!("kode {code}: {error}")),
        None => FaucetError::NodeUnreachable(error.to_string()),
    }
}

/// Wallet faucet, batas harian, dan klien ke node.
pub struct Faucet {
    client: RpcClient,
    account: ExtendedPrivKey,
    payload: [u8; 32],
    address: String,
    settings: Settings,
    state_path: PathBuf,
    ledger: Ledger,
    visitors: Visitors,
    pending: BTreeSet<OutPoint>,
    synced: bool,
    genesis_checked: bool,
    random: RandomSource,
}

impl Faucet {
    /// Membuka faucet di `datadir`: membuat atau membaca frasa ([`load_or_create_phrase`]) dan batas hari ini. Node belum
    /// dihubungi; jaringan node diperiksa pada permintaan pertama.
    ///
    /// # Errors
    ///
    /// [`FaucetError::Key`] atau [`FaucetError::State`].
    pub fn open(
        client: RpcClient,
        datadir: &Path,
        settings: Settings,
        mut random: RandomSource,
        now: u64,
    ) -> Result<Self, FaucetError> {
        std::fs::create_dir_all(datadir).map_err(|error| FaucetError::State(error.to_string()))?;
        let mnemonic = load_or_create_phrase(&datadir.join(KEY_FILE), &mut random)?;
        let (account, payload) = faucet_keys(&mnemonic)?;
        let state_path = datadir.join(STATE_FILE);
        let day = now / SECONDS_PER_DAY;
        let ledger = load_ledger(&state_path, day)?;
        Ok(Self {
            client,
            account,
            payload,
            address: encode_address(Network::Regtest, OutputKind::SingleKey, &payload),
            settings,
            state_path,
            ledger,
            visitors: Visitors::new(day),
            pending: BTreeSet::new(),
            synced: false,
            genesis_checked: false,
            random,
        })
    }

    /// Alamat faucet (alamat isi ulang).
    #[must_use]
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Pengaturan kiriman.
    #[must_use]
    pub const fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Mengosongkan batas jika hari UTC sudah berganti.
    fn roll(&mut self, now: u64) {
        let day = now / SECONDS_PER_DAY;
        if self.ledger.day != day {
            self.ledger = Ledger::empty(day);
        }
        if self.visitors.day != day {
            self.visitors = Visitors::new(day);
        }
    }

    /// Memeriksa node (jaringan regtest, genesis regtest) lalu mengumpulkan koin faucet: UTXO yang belum dibelanjakan
    /// mempool, ditambah output mempool ke faucet yang belum dibelanjakan kiriman sendiri.
    async fn view(&mut self) -> Result<View, FaucetError> {
        let info = self
            .client
            .get_chain_info()
            .await
            .map_err(|error| node_error(&error))?;
        if info.network.network() != Network::Regtest {
            return Err(FaucetError::WrongNetwork(String::from(info.network)));
        }
        if !self.genesis_checked {
            let id = self
                .client
                .get_block_hash(0)
                .await
                .map_err(|error| node_error(&error))?;
            if *id.bytes() != REGTEST_NETWORK_ID {
                return Err(FaucetError::WrongNetwork(String::from(id)));
            }
            self.genesis_checked = true;
        }
        let mine = [Hex32::new(self.payload)];
        let utxos = self
            .client
            .scan_utxos(&mine)
            .await
            .map_err(|error| node_error(&error))?;
        let mempool = self
            .client
            .scan_mempool(&mine)
            .await
            .map_err(|error| node_error(&error))?;
        if !self.synced && mempool.outputs.is_empty() && mempool.spent.is_empty() {
            self.synced = true;
        }
        let in_mempool: BTreeSet<OutPoint> = mempool
            .outputs
            .iter()
            .map(|output| OutPoint::new(output.txid.to_hash32(), output.index))
            .collect();
        // Outpoint yang sudah masuk blok (atau hilang) tidak perlu dicatat lagi: setelah itu `scan_utxos` dan
        // `scan_mempool.spent` dari node yang menentukan.
        self.pending
            .retain(|outpoint| in_mempool.contains(outpoint));
        let spent: BTreeSet<OutPoint> = mempool
            .spent
            .iter()
            .map(|spend| OutPoint::new(spend.txid.to_hash32(), spend.index))
            .collect();
        let next = info.height.saturating_add(1);
        let mut coins = Vec::new();
        for utxo in &utxos {
            let outpoint = OutPoint::new(utxo.txid.to_hash32(), utxo.index);
            if utxo.kind.kind() == OutputKind::SingleKey
                && *utxo.payload.bytes() == self.payload
                && !spent.contains(&outpoint)
            {
                coins.push(Coin::new(
                    outpoint,
                    utxo.amount.amount(),
                    self.payload,
                    utxo.spendable_at,
                    RECEIVE_BRANCH,
                    0,
                ));
            }
        }
        // Sebelum sinkron, output mempool ke faucet mungkin sudah dibelanjakan kiriman lama yang tidak dikenal proses ini
        // (`scan_mempool.spent` hanya memuat UTXO terkonfirmasi), jadi tidak dihitung sebagai koin atau saldo.
        for output in &mempool.outputs {
            let outpoint = OutPoint::new(output.txid.to_hash32(), output.index);
            if self.synced
                && output.kind.kind() == OutputKind::SingleKey
                && *output.payload.bytes() == self.payload
                && !self.pending.contains(&outpoint)
            {
                coins.push(Coin::new(
                    outpoint,
                    output.amount.amount(),
                    self.payload,
                    next,
                    RECEIVE_BRANCH,
                    0,
                ));
            }
        }
        let spendable = Amount::checked_sum(
            coins
                .iter()
                .filter(|coin| coin.spendable_at() <= next)
                .map(Coin::amount),
        )
        .ok_or_else(|| FaucetError::Internal("jumlah saldo meluap".to_owned()))?;
        Ok(View {
            tip_height: info.height,
            coins,
            spendable,
        })
    }

    /// Keadaan faucet pada waktu `now` (detik Unix).
    ///
    /// # Errors
    ///
    /// [`FaucetError`] dari node.
    pub async fn status(&mut self, now: u64) -> Result<FaucetStatus, FaucetError> {
        self.roll(now);
        let view = self.view().await?;
        let remaining_today = self.settings.daily_limit.saturating_sub(self.ledger.count);
        Ok(FaucetStatus {
            address: self.address.clone(),
            amount: self.settings.amount,
            balance: view.spendable,
            daily_limit: self.settings.daily_limit,
            remaining_today,
            synced: self.synced,
            ready: self.synced && remaining_today > 0 && view.spendable > self.settings.amount,
        })
    }

    /// Mengirim [`Settings::amount`] ke `address` untuk pengunjung `visitor` pada waktu `now` (detik Unix).
    ///
    /// Urutan pemeriksaan: alamat, batas harian, pengunjung, alamat hari ini, node, saldo. Batas hanya terpakai jika node
    /// menerima transaksi.
    ///
    /// # Errors
    ///
    /// [`FaucetError`] sesuai pemeriksaan yang gagal.
    pub async fn request(
        &mut self,
        address: &str,
        visitor: Option<IpAddr>,
        now: u64,
    ) -> Result<Payout, FaucetError> {
        self.roll(now);
        let (kind, payload) = decode_address(Network::Regtest, address.trim())
            .map_err(|_| FaucetError::BadAddress)?;
        if payload == self.payload {
            return Err(FaucetError::OwnAddress);
        }
        let canonical = encode_address(Network::Regtest, kind, &payload);
        if self.ledger.count >= self.settings.daily_limit {
            return Err(FaucetError::DailyLimit);
        }
        let visitor_key = self.visitors.keys.hash_one(visitor);
        if self.visitors.seen.contains(&visitor_key) {
            return Err(FaucetError::VisitorLimit);
        }
        if self.ledger.addresses.contains(&canonical) {
            return Err(FaucetError::AddressLimit);
        }
        let view = self.view().await?;
        if !self.synced {
            return Err(FaucetError::Busy);
        }
        let request = SendRequest {
            recipient_kind: kind,
            recipient_payload: payload,
            amount: self.settings.amount,
            fee_rate: self.settings.fee_rate,
            change_payload: self.payload,
            tip_height: view.tip_height,
        };
        let aux_rand = (self.random)().map_err(|error| FaucetError::Internal(error.to_string()))?;
        let network_id = Hash32::from_bytes(REGTEST_NETWORK_ID);
        let (plan, tx) =
            build_transaction(&view.coins, &request, &self.account, &network_id, &aux_rand)
                .map_err(|error| match error {
                    SendError::InsufficientFunds { .. } => FaucetError::Empty,
                    other => FaucetError::Internal(other.to_string()),
                })?;
        let sent = self
            .client
            .send_raw_transaction(&tx.to_bytes())
            .await
            .map_err(|error| node_error(&error))?;
        self.pending.extend(plan.coins().iter().map(Coin::outpoint));
        self.visitors.seen.insert(visitor_key);
        self.ledger.count = self.ledger.count.saturating_add(1);
        self.ledger.addresses.insert(canonical.clone());
        if let Err(error) = save_ledger(&self.state_path, &self.ledger) {
            tracing::warn!(%error, "file batas faucet gagal disimpan");
        }
        Ok(Payout {
            txid: sent.txid.to_hash32(),
            address: canonical,
            amount: self.settings.amount,
            fee: plan.fee(),
        })
    }
}

/// Alamat pengunjung: dari kepala [`CLIENT_IP_HEADER`] jika koneksi datang dari salah satu alamat `trusted` (proxy
/// tepercaya, misalnya container `cloudflared`), selain itu alamat koneksi. Alamat tidak disimpan dan tidak dicatat.
#[must_use]
pub fn faucet_client_ip(
    connect: Option<IpAddr>,
    headers: &HeaderMap,
    trusted: &[IpAddr],
) -> Option<IpAddr> {
    let forwarded = connect
        .filter(|address| {
            let address = address.to_canonical();
            trusted.iter().any(|proxy| proxy.to_canonical() == address)
        })
        .and_then(|_| headers.get(CLIENT_IP_HEADER))
        .and_then(|value| value.to_str().ok())
        .and_then(|text| text.trim().parse::<IpAddr>().ok());
    forwarded.or(connect)
}

/// Pilihan router faucet.
pub struct RouterOptions {
    /// Batas laju per pengunjung (`None` = tanpa batas).
    pub limits: Option<Limits>,
    /// Alamat proxy tepercaya, diperbarui dari luar.
    pub trusted: Arc<RwLock<Vec<IpAddr>>>,
    /// Origin yang boleh memanggil faucet dari peramban (CORS), persis seperti kepala `Origin`.
    pub origins: Vec<String>,
    /// Jam (detik Unix), untuk pergantian hari.
    pub clock: fn() -> u64,
}

struct App {
    faucet: Mutex<Faucet>,
    limiter: Option<StdMutex<RateLimiter>>,
    trusted: Arc<RwLock<Vec<IpAddr>>>,
    origins: Vec<String>,
    clock: fn() -> u64,
    started: Instant,
}

impl App {
    fn allow(&self, visitor: Option<IpAddr>) -> bool {
        let Some(limiter) = &self.limiter else {
            return true;
        };
        let now_ms = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);
        limiter
            .lock()
            .map(|mut limiter| limiter.allow(visitor, now_ms))
            .unwrap_or(false)
    }
}

/// Router faucet: semua jalur ke satu handler; selain [`FAUCET_PATH`] dijawab `404`.
pub fn router(faucet: Faucet, options: RouterOptions) -> Router {
    let app = App {
        faucet: Mutex::new(faucet),
        limiter: options
            .limits
            .map(|limits| StdMutex::new(RateLimiter::new(limits))),
        trusted: options.trusted,
        origins: options.origins,
        clock: options.clock,
        started: Instant::now(),
    };
    Router::new().fallback(handle).with_state(Arc::new(app))
}

/// Jawaban JSON dengan `no-store` dan `nosniff`.
fn reply(status: StatusCode, value: &Value) -> Response {
    (
        status,
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-store"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        value.to_string(),
    )
        .into_response()
}

fn error_reply(error: &FaucetError) -> Response {
    let status =
        StatusCode::from_u16(error.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    reply(
        status,
        &json!({"error": error.code(), "message": error.public_message()}),
    )
}

/// Kepala `Origin` jika ada di daftar izin.
fn allowed_origin(origins: &[String], headers: &HeaderMap) -> Option<HeaderValue> {
    headers
        .get(header::ORIGIN)
        .filter(|value| {
            value
                .to_str()
                .is_ok_and(|text| origins.iter().any(|origin| origin == text))
        })
        .cloned()
}

async fn handle(
    State(app): State<Arc<App>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    extensions: Extensions,
    body: Body,
) -> Response {
    let origin = allowed_origin(&app.origins, &headers);
    let mut response = if uri.path() != FAUCET_PATH {
        reply(StatusCode::NOT_FOUND, &json!({"error": "not_found"}))
    } else if method == Method::OPTIONS {
        let mut preflight = StatusCode::NO_CONTENT.into_response();
        if origin.is_some() {
            let head = preflight.headers_mut();
            head.insert(
                header::ACCESS_CONTROL_ALLOW_METHODS,
                HeaderValue::from_static("GET, POST"),
            );
            head.insert(
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                HeaderValue::from_static("content-type"),
            );
            head.insert(
                header::ACCESS_CONTROL_MAX_AGE,
                HeaderValue::from_static("600"),
            );
        }
        preflight
    } else {
        respond(&app, &method, &headers, &extensions, body).await
    };
    if let Some(origin) = origin {
        let head = response.headers_mut();
        head.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        head.insert(header::VARY, HeaderValue::from_static("Origin"));
    }
    response
}

/// `GET` (keadaan) atau `POST` (kiriman) setelah batas laju.
async fn respond(
    app: &App,
    method: &Method,
    headers: &HeaderMap,
    extensions: &Extensions,
    body: Body,
) -> Response {
    let connect = extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip());
    let trusted = app
        .trusted
        .read()
        .map(|list| list.to_vec())
        .unwrap_or_default();
    let visitor = faucet_client_ip(connect, headers, &trusted);
    if !app.allow(visitor) {
        let mut response = reply(
            StatusCode::TOO_MANY_REQUESTS,
            &json!({"error": "rate_limited"}),
        );
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
        return response;
    }
    let now = (app.clock)();
    if *method == Method::GET {
        let result = app.faucet.lock().await.status(now).await;
        return match result {
            Ok(status) => reply(StatusCode::OK, &status.to_json()),
            Err(error) => {
                log_failure(&error);
                error_reply(&error)
            }
        };
    }
    if *method != Method::POST {
        let mut response = reply(
            StatusCode::METHOD_NOT_ALLOWED,
            &json!({"error": "method_not_allowed"}),
        );
        response.headers_mut().insert(
            header::ALLOW,
            HeaderValue::from_static("GET, POST, OPTIONS"),
        );
        return response;
    }
    let Ok(bytes) = to_bytes(body, MAX_BODY_BYTES).await else {
        return reply(
            StatusCode::PAYLOAD_TOO_LARGE,
            &json!({"error": "payload_too_large"}),
        );
    };
    let address = serde_json::from_slice::<Value>(&bytes)
        .ok()
        .and_then(|value| {
            value
                .get("address")
                .and_then(Value::as_str)
                .map(str::to_owned)
        });
    let Some(address) = address else {
        return reply(StatusCode::BAD_REQUEST, &json!({"error": "bad_request"}));
    };
    let result = app
        .faucet
        .lock()
        .await
        .request(&address, visitor, now)
        .await;
    match result {
        Ok(payout) => {
            tracing::info!(txid = %String::from(Hex32::from(payout.txid)), "faucet mengirim");
            reply(StatusCode::OK, &payout.to_json())
        }
        Err(error) => {
            log_failure(&error);
            error_reply(&error)
        }
    }
}

/// Mencatat galat node atau internal (tanpa alamat penerima atau pengunjung). Galat batas dan alamat tidak dicatat.
fn log_failure(error: &FaucetError) {
    if error.http_status() >= 500 && *error != FaucetError::Busy && *error != FaucetError::Empty {
        tracing::warn!(%error, "faucet gagal");
    }
}

/// Alamat IP untuk `host` (misalnya nama container `cloudflared`), atau `None` jika gagal.
async fn resolve(host: &str) -> Option<Vec<IpAddr>> {
    tokio::net::lookup_host((host, 0))
        .await
        .ok()
        .map(|addresses| addresses.map(|address| address.ip()).collect())
}

async fn serve(config: &FaucetConfig) -> Result<(), DemoError> {
    let shutdown = shutdown_signal()?;
    let client =
        RpcClient::new(&config.node_url).map_err(|error| DemoError::Rpc(error.to_string()))?;
    let settings = Settings {
        amount: config.amount,
        fee_rate: config.fee_rate,
        daily_limit: config.daily_limit,
    };
    let faucet = Faucet::open(
        client,
        &config.datadir,
        settings,
        Box::new(os_random),
        unix_now(),
    )
    .map_err(|error| DemoError::Faucet(error.to_string()))?;
    let address = faucet.address().to_owned();
    let trusted = Arc::new(RwLock::new(Vec::new()));
    if let Some(host) = &config.trust_proxy_host {
        match resolve(host).await {
            Some(list) if !list.is_empty() => {
                if let Ok(mut guard) = trusted.write() {
                    *guard = list;
                }
            }
            _ => tracing::warn!(%host, "nama proxy tepercaya belum bisa di-resolve"),
        }
        let host = host.clone();
        let shared = Arc::clone(&trusted);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(RESOLVE_INTERVAL).await;
                let Some(list) = resolve(&host).await else {
                    continue;
                };
                if let Ok(mut guard) = shared.write() {
                    *guard = list;
                }
            }
        });
    }
    let listener =
        TcpListener::bind(SocketAddr::from((config.http_bind, config.http_port))).await?;
    let local = listener.local_addr()?;
    tracing::info!(
        %local,
        node = %config.node_url,
        rate = config.rate,
        amount = %config.amount,
        daily_limit = config.daily_limit,
        "faucet berjalan"
    );
    println!("alamat faucet: {address}");
    println!("http: http://{local}");
    let app = router(
        faucet,
        RouterOptions {
            limits: Limits::from_rate(config.rate),
            trusted,
            origins: config.origins.clone(),
            clock: unix_now,
        },
    );
    let served = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await;
    tracing::info!("faucet berhenti");
    Ok(served?)
}

/// Menjalankan faucet (mode `faucet`) sampai menerima SIGINT atau SIGTERM.
///
/// # Errors
///
/// [`DemoError`] jika runtime, sinyal, klien RPC, file faucet, atau port HTTP gagal.
pub fn run_faucet(config: &FaucetConfig) -> Result<(), DemoError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let result = runtime.block_on(serve(config));
    runtime.shutdown_timeout(SHUTDOWN_TIMEOUT);
    result
}

#[cfg(test)]
mod tests;
