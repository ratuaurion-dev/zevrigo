//! Perintah IPC wallet desktop (MOD-15, DEC-31).
//!
//! Setiap perintah memakai pustaka MOD-12 `zevrigo-cli` dengan folder data [`AppState`]. Pekerjaan yang lama (Argon2id,
//! RPC ke node, pemindaian riwayat) berjalan di utas sendiri, tidak di utas jendela. Galat dikirim ke antarmuka sebagai
//! [`CommandError`] dengan kode tetap untuk diterjemahkan. Jumlah koin selalu teks angka quanta.

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::State;
use zeroize::Zeroizing;
use zevrigo_cli::{
    Balance, Cli, CliError, History, HistoryEntry, Protection, Sent, WalletFileError,
    check_new_password, default_rpc_url, load_network, network_name, next_address, os_random,
    show_phrase, wallet_path, wallet_restore,
};
use zevrigo_primitives::{Amount, Network, OutputKind};
use zevrigo_wallet_core::{
    KdfParams, Mnemonic, NONCE_LEN, RECEIVE_BRANCH, SALT_LEN, SendError, encode_address,
};

/// Variabel lingkungan untuk memilih folder data wallet (misalnya folder yang sama dengan `--datadir` CLI).
pub const DATADIR_ENV: &str = "ZEVRIGO_WALLET_DATADIR";

/// Gerbang RPC publik jaringan demo regtest (DEC-34, TSK-M53): alamat node bawaan wallet untuk regtest (TSK-M61).
pub const DEMO_RPC_URL: &str = "https://zevrigo-rpc.ratuaurion.store";

/// Alamat node bawaan wallet: regtest memakai gerbang demo publik [`DEMO_RPC_URL`], jaringan lain node di komputer sendiri
/// (alamat bawaan CLI). Pengguna bisa menggantinya di Pengaturan.
#[must_use]
pub fn wallet_default_rpc_url(network: Network) -> String {
    match network {
        Network::Regtest => DEMO_RPC_URL.to_owned(),
        Network::Testnet | Network::Mainnet => default_rpc_url(network),
    }
}

/// Folder data wallet. File wallet per jaringan ada di `<datadir>/<jaringan>/wallet.json`.
pub struct AppState {
    datadir: PathBuf,
}

impl AppState {
    /// Folder dari [`DATADIR_ENV`], atau bawaan: `%APPDATA%\Zevrigo` di Windows, `$HOME/.zevrigo` di sistem lain.
    /// `None` jika variabel lingkungan yang dibutuhkan tidak ada. Hanya desktop; Android memakai [`AppState::new`].
    #[cfg_attr(target_os = "android", allow(dead_code))]
    pub fn from_env() -> Option<Self> {
        let datadir = match std::env::var_os(DATADIR_ENV).filter(|dir| !dir.is_empty()) {
            Some(dir) => PathBuf::from(dir),
            None => default_datadir()?,
        };
        Some(Self::new(datadir))
    }

    /// Folder data tertentu, misalnya folder data aplikasi Android (TSK-M58).
    #[must_use]
    pub const fn new(datadir: PathBuf) -> Self {
        Self { datadir }
    }
}

#[cfg_attr(target_os = "android", allow(dead_code))]
fn default_datadir() -> Option<PathBuf> {
    let (variable, folder) = if cfg!(windows) {
        ("APPDATA", "Zevrigo")
    } else {
        ("HOME", ".zevrigo")
    };
    std::env::var_os(variable)
        .filter(|dir| !dir.is_empty())
        .map(|dir| PathBuf::from(dir).join(folder))
}

/// Galat perintah untuk antarmuka: `code` tetap (diterjemahkan antarmuka), `message` keterangan dari MOD-12.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CommandError {
    /// Kode galat, misalnya `wrong_password` atau `node_unreachable`.
    pub code: &'static str,
    /// Keterangan lengkap.
    pub message: String,
}

impl CommandError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl From<CliError> for CommandError {
    fn from(error: CliError) -> Self {
        Self::new(error_code(&error), error.to_string())
    }
}

impl From<WalletFileError> for CommandError {
    fn from(error: WalletFileError) -> Self {
        Self::from(CliError::from(error))
    }
}

/// Kode galat tetap untuk galat MOD-12.
#[must_use]
pub fn error_code(error: &CliError) -> &'static str {
    match error {
        CliError::WrongPassword => "wrong_password",
        CliError::PasswordTooShort => "password_too_short",
        CliError::PasswordMismatch => "password_mismatch",
        CliError::WrongNetwork { .. } => "wrong_network",
        CliError::Rpc { .. } => "node_unreachable",
        CliError::Rejected { .. } => "rejected",
        CliError::Send(SendError::InsufficientFunds { .. }) => "insufficient_funds",
        CliError::Send(SendError::ZeroAmount) => "bad_amount",
        CliError::Send(_) => "send_failed",
        CliError::Address(_) => "bad_address",
        CliError::Mnemonic(_) | CliError::PassphraseMismatch => "bad_phrase",
        CliError::File(WalletFileError::Exists { .. }) => "wallet_exists",
        CliError::File(WalletFileError::Missing { .. }) => "wallet_missing",
        CliError::File(_) => "wallet_file",
        CliError::ChainChanged => "chain_changed",
        CliError::History(_) | CliError::HistoryFile { .. } => "history_failed",
        _ => "failed",
    }
}

/// Jaringan dari nama antarmuka: hanya `testnet` dan `regtest`.
///
/// # Errors
///
/// Kode `network` untuk nama lain.
pub fn parse_network(name: &str) -> Result<Network, CommandError> {
    match name {
        "testnet" => Ok(Network::Testnet),
        "regtest" => Ok(Network::Regtest),
        _ => Err(CommandError::new(
            "network",
            format!("jaringan tidak dikenal: {name} (pilih testnet atau regtest)"),
        )),
    }
}

/// Jumlah quanta dari teks angka desimal tanpa nol di depan, minimal 1.
///
/// # Errors
///
/// Kode `bad_amount` untuk teks lain atau nilai di atas `u128`.
pub fn parse_quanta(text: &str) -> Result<Amount, CommandError> {
    let bad = || CommandError::new("bad_amount", format!("jumlah quanta tidak sah: {text}"));
    let digits_only = !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
    if !digits_only || (text.len() > 1 && text.starts_with('0')) {
        return Err(bad());
    }
    let quanta: u128 = text.parse().map_err(|_| bad())?;
    if quanta == 0 {
        return Err(bad());
    }
    Ok(Amount::from_quanta(quanta))
}

fn quanta(amount: Amount) -> String {
    amount.as_quanta().to_string()
}

/// Keadaan file wallet satu jaringan.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletInfo {
    /// File wallet sudah ada.
    pub exists: bool,
    /// Path file wallet.
    pub path: String,
    /// Alamat RPC bawaan node jaringan ini ([`wallet_default_rpc_url`]).
    pub default_rpc_url: String,
}

/// Ringkasan chain dari node.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainStatus {
    /// Nama jaringan node.
    pub network: String,
    /// Tinggi ujung chain.
    pub height: u64,
    /// ID blok ujung (hex).
    pub best_block_id: String,
}

/// Saldo wallet ([`Balance`]) dengan jumlah sebagai teks quanta.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceView {
    /// Tinggi ujung chain.
    pub tip_height: u64,
    /// Semua UTXO milik wallet di chain.
    pub total: String,
    /// Bisa dipakai di blok berikutnya.
    pub spendable: String,
    /// Coinbase yang belum matang.
    pub immature: String,
    /// UTXO yang sedang dibelanjakan transaksi di mempool.
    pub pending_spent: String,
    /// Output transaksi di mempool ke alamat wallet.
    pub pending_received: String,
    /// Saldo setelah semua transaksi tertunda masuk blok.
    pub after_pending: String,
    /// Jumlah UTXO milik wallet.
    pub utxos: usize,
}

impl From<&Balance> for BalanceView {
    fn from(balance: &Balance) -> Self {
        Self {
            tip_height: balance.tip_height,
            total: quanta(balance.total),
            spendable: quanta(balance.spendable),
            immature: quanta(balance.immature),
            pending_spent: quanta(balance.pending_spent),
            pending_received: quanta(balance.pending_received),
            after_pending: quanta(balance.after_pending),
            utxos: balance.utxos,
        }
    }
}

/// Satu entri riwayat ([`HistoryEntry`]) dengan jumlah sebagai teks quanta.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntryView {
    /// Txid (hex).
    pub txid: String,
    /// Tinggi blok, `null` untuk transaksi tertunda.
    pub height: Option<u64>,
    /// Cap waktu blok (detik Unix), `null` untuk transaksi tertunda.
    pub timestamp: Option<u64>,
    /// Hadiah tambang.
    pub coinbase: bool,
    /// Jumlah output ke alamat wallet (termasuk kembalian).
    pub received: String,
    /// Jumlah koin wallet yang dibelanjakan.
    pub spent: String,
    /// Biaya, jika diketahui.
    pub fee: Option<String>,
}

impl From<&HistoryEntry> for HistoryEntryView {
    fn from(entry: &HistoryEntry) -> Self {
        Self {
            txid: String::from(entry.txid),
            height: entry.height,
            timestamp: entry.timestamp,
            coinbase: entry.coinbase,
            received: quanta(entry.received),
            spent: quanta(entry.spent),
            fee: entry.fee.map(quanta),
        }
    }
}

/// Riwayat wallet ([`History`]), entri terbaru dulu.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryView {
    /// Tinggi ujung chain saat dipindai.
    pub tip_height: u64,
    /// Entri riwayat.
    pub entries: Vec<HistoryEntryView>,
}

impl From<&History> for HistoryView {
    fn from(history: &History) -> Self {
        Self {
            tip_height: history.tip_height,
            entries: history.entries.iter().map(HistoryEntryView::from).collect(),
        }
    }
}

/// Hasil kirim ([`Sent`]) dengan jumlah sebagai teks quanta.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SentView {
    /// Txid dari node.
    pub txid: String,
    /// Jumlah yang diterima penerima.
    pub amount: String,
    /// Biaya.
    pub fee: String,
    /// Kembalian, jika ada.
    pub change: Option<String>,
    /// Jumlah input.
    pub inputs: usize,
    /// Ukuran transaksi dalam byte.
    pub size: usize,
}

impl From<&Sent> for SentView {
    fn from(sent: &Sent) -> Self {
        Self {
            txid: String::from(sent.txid),
            amount: quanta(sent.amount),
            fee: quanta(sent.fee),
            change: sent.change.map(quanta),
            inputs: sent.inputs,
            size: sent.size,
        }
    }
}

/// Keadaan file wallet `network` di `datadir`.
#[must_use]
pub fn wallet_info_in(datadir: &Path, network: Network) -> WalletInfo {
    let path = wallet_path(network, datadir);
    WalletInfo {
        exists: path.is_file(),
        path: path.display().to_string(),
        default_rpc_url: wallet_default_rpc_url(network),
    }
}

/// Frasa pemulihan baru 24 kata dari sumber acak sistem operasi. Belum disimpan: file wallet baru dibuat
/// [`restore_in`] setelah pengguna memeriksa catatannya.
///
/// # Errors
///
/// Sumber acak sistem operasi gagal dibaca.
pub fn new_phrase_words() -> Result<String, CommandError> {
    let entropy = Zeroizing::new(os_random()?);
    let mnemonic = Mnemonic::from_entropy(entropy.as_slice()).map_err(CliError::from)?;
    Ok(mnemonic.phrase().to_owned())
}

fn prefix<const N: usize>(bytes: &[u8; 32]) -> [u8; N] {
    let mut out = [0_u8; N];
    for (slot, byte) in out.iter_mut().zip(bytes) {
        *slot = *byte;
    }
    out
}

/// Membuat file wallet terenkripsi dari `phrase` (tanpa passphrase BIP39), lalu mengembalikan alamat penerimaan
/// pertama. Password minimal 8 karakter dan wajib sama dengan `confirm`.
///
/// # Errors
///
/// `password_too_short`, `password_mismatch`, `bad_phrase`, `wallet_exists`, atau galat file wallet.
pub fn restore_in(
    datadir: &Path,
    network: Network,
    phrase: &str,
    password: &str,
    confirm: &str,
) -> Result<String, CommandError> {
    check_new_password(password, confirm)?;
    let protection = Protection {
        password,
        kdf: KdfParams::DEFAULT,
        salt: prefix::<SALT_LEN>(&os_random()?),
        nonce: prefix::<NONCE_LEN>(&os_random()?),
    };
    wallet_restore(network, datadir, phrase, "", &protection)?;
    receive_address_in(datadir, network)
}

/// Alamat penerimaan terakhir yang sudah dibagikan (tanpa menaikkan indeks), atau alamat pertama untuk wallet yang
/// belum pernah membagikan alamat.
///
/// # Errors
///
/// Galat file wallet atau turunan kunci.
pub fn receive_address_in(datadir: &Path, network: Network) -> Result<String, CommandError> {
    let file = load_network(network, &wallet_path(network, datadir))?;
    if let (Some(index), Some(account)) = (file.next_index.receive.checked_sub(1), file.account()?)
        && let Ok(child) = account.derive_path(&[RECEIVE_BRANCH, index])
    {
        return Ok(encode_address(
            network,
            OutputKind::SingleKey,
            &child.x_only(),
        ));
    }
    Ok(next_address(network, datadir, "")?)
}

/// Alamat penerimaan baru (indeks berikutnya), sama dengan perintah CLI `address`.
///
/// # Errors
///
/// Galat file wallet atau turunan kunci.
pub fn new_address_in(datadir: &Path, network: Network) -> Result<String, CommandError> {
    Ok(next_address(network, datadir, "")?)
}

/// Frasa pemulihan wallet, dibuka dengan password.
///
/// # Errors
///
/// `wrong_password` atau galat file wallet.
pub fn reveal_in(datadir: &Path, network: Network, password: &str) -> Result<String, CommandError> {
    let phrase = show_phrase(network, datadir, password)?;
    Ok(phrase.as_str().to_owned())
}

fn chain_status_at(rpc_url: &str, network: Network) -> Result<ChainStatus, CommandError> {
    let info = Cli::connect(rpc_url, network)?.info()?;
    let actual = String::from(info.network);
    let expected = network_name(network);
    if actual != expected {
        return Err(CliError::WrongNetwork {
            expected: expected.to_owned(),
            actual,
        }
        .into());
    }
    Ok(ChainStatus {
        network: actual,
        height: info.height,
        best_block_id: String::from(info.best_block_id),
    })
}

fn send_at(
    datadir: &Path,
    network: Network,
    rpc_url: &str,
    password: &str,
    to: &str,
    amount: &str,
    fee_rate: u64,
) -> Result<SentView, CommandError> {
    let amount = parse_quanta(amount)?;
    if fee_rate == 0 {
        return Err(CommandError::new(
            "bad_fee",
            "tarif biaya minimal 1 quanta per byte",
        ));
    }
    let aux_rand = Zeroizing::new(os_random()?);
    let cli = Cli::connect(rpc_url, network)?;
    let sent = cli.send(
        datadir,
        "",
        password,
        to.trim(),
        amount,
        u128::from(fee_rate),
        &aux_rand,
    )?;
    Ok(SentView::from(&sent))
}

/// Menjalankan pekerjaan wallet di utas sendiri tanpa konteks runtime Tauri (MOD-12 memakai runtime tokio sendiri).
async fn run_blocking<T, F>(job: F) -> Result<T, CommandError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, CommandError> + Send + 'static,
{
    let stopped = || CommandError::new("failed", "pekerjaan wallet berhenti tak terduga");
    tauri::async_runtime::spawn_blocking(move || {
        std::thread::spawn(job)
            .join()
            .unwrap_or_else(|_| Err(stopped()))
    })
    .await
    .map_err(|_| stopped())?
}

/// `wallet_info`: apakah file wallet jaringan ini sudah ada, path-nya, dan alamat RPC bawaan.
///
/// # Errors
///
/// Kode `network`.
#[tauri::command]
pub fn wallet_info(
    state: State<'_, AppState>,
    network: String,
) -> Result<WalletInfo, CommandError> {
    let network = parse_network(&network)?;
    Ok(wallet_info_in(&state.datadir, network))
}

/// `new_phrase`: frasa baru 24 kata (belum disimpan).
///
/// # Errors
///
/// Sumber acak gagal.
#[tauri::command]
pub async fn new_phrase() -> Result<String, CommandError> {
    run_blocking(new_phrase_words).await
}

/// `restore_wallet`: membuat file wallet terenkripsi dari frasa (wallet baru atau pemulihan).
///
/// # Errors
///
/// Lihat [`restore_in`].
#[tauri::command]
pub async fn restore_wallet(
    state: State<'_, AppState>,
    network: String,
    phrase: String,
    password: String,
    confirm: String,
) -> Result<String, CommandError> {
    let network = parse_network(&network)?;
    let datadir = state.datadir.clone();
    let phrase = Zeroizing::new(phrase);
    let password = Zeroizing::new(password);
    let confirm = Zeroizing::new(confirm);
    run_blocking(move || restore_in(&datadir, network, &phrase, &password, &confirm)).await
}

/// `receive_address`: alamat penerimaan terakhir.
///
/// # Errors
///
/// Lihat [`receive_address_in`].
#[tauri::command]
pub async fn receive_address(
    state: State<'_, AppState>,
    network: String,
) -> Result<String, CommandError> {
    let network = parse_network(&network)?;
    let datadir = state.datadir.clone();
    run_blocking(move || receive_address_in(&datadir, network)).await
}

/// `new_address`: alamat penerimaan baru.
///
/// # Errors
///
/// Lihat [`new_address_in`].
#[tauri::command]
pub async fn new_address(
    state: State<'_, AppState>,
    network: String,
) -> Result<String, CommandError> {
    let network = parse_network(&network)?;
    let datadir = state.datadir.clone();
    run_blocking(move || new_address_in(&datadir, network)).await
}

/// `chain_status`: ringkasan chain node di `rpc_url`, dengan syarat node berjalan di jaringan yang sama.
///
/// # Errors
///
/// `node_unreachable` atau `wrong_network`.
#[tauri::command]
pub async fn chain_status(network: String, rpc_url: String) -> Result<ChainStatus, CommandError> {
    let network = parse_network(&network)?;
    run_blocking(move || chain_status_at(&rpc_url, network)).await
}

/// `balance`: saldo wallet dari node.
///
/// # Errors
///
/// `node_unreachable`, `wrong_network`, atau galat file wallet.
#[tauri::command]
pub async fn balance(
    state: State<'_, AppState>,
    network: String,
    rpc_url: String,
) -> Result<BalanceView, CommandError> {
    let network = parse_network(&network)?;
    let datadir = state.datadir.clone();
    run_blocking(move || {
        let balance = Cli::connect(&rpc_url, network)?.balance(&datadir, "")?;
        Ok(BalanceView::from(&balance))
    })
    .await
}

/// `history`: riwayat transaksi wallet dari blok (tembolok `history.json` sama dengan CLI).
///
/// # Errors
///
/// `node_unreachable`, `wrong_network`, `chain_changed`, atau `history_failed`.
#[tauri::command]
pub async fn history(
    state: State<'_, AppState>,
    network: String,
    rpc_url: String,
) -> Result<HistoryView, CommandError> {
    let network = parse_network(&network)?;
    let datadir = state.datadir.clone();
    run_blocking(move || {
        let history = Cli::connect(&rpc_url, network)?.history(&datadir, "")?;
        Ok(HistoryView::from(&history))
    })
    .await
}

/// `send`: menyusun, menandatangani (password wallet), dan mengirim transaksi `amount` quanta ke `to`.
///
/// # Errors
///
/// `bad_amount`, `bad_fee`, `bad_address`, `insufficient_funds`, `wrong_password`, `rejected`, atau
/// `node_unreachable`.
#[tauri::command]
pub async fn send(
    state: State<'_, AppState>,
    network: String,
    rpc_url: String,
    password: String,
    to: String,
    amount: String,
    fee_rate: u64,
) -> Result<SentView, CommandError> {
    let network = parse_network(&network)?;
    let datadir = state.datadir.clone();
    let password = Zeroizing::new(password);
    run_blocking(move || {
        send_at(
            &datadir, network, &rpc_url, &password, &to, &amount, fee_rate,
        )
    })
    .await
}

/// `reveal_phrase`: frasa pemulihan, dibuka dengan password.
///
/// # Errors
///
/// `wrong_password` atau galat file wallet.
#[tauri::command]
pub async fn reveal_phrase(
    state: State<'_, AppState>,
    network: String,
    password: String,
) -> Result<String, CommandError> {
    let network = parse_network(&network)?;
    let datadir = state.datadir.clone();
    let password = Zeroizing::new(password);
    run_blocking(move || reveal_in(&datadir, network, &password)).await
}

#[cfg(test)]
mod tests;
