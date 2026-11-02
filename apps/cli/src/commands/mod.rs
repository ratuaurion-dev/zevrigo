//! Aksi CLI: ringkasan chain dan perintah wallet regtest (TSK-M19, DEC-21).
//!
//! `wallet-create`, `wallet-restore`, dan `address` berjalan tanpa node. `info`, `balance`, dan `send` memanggil
//! node lewat RPC (MOD-08) dengan runtime tokio satu thread. `balance` dan `send` memperhitungkan transaksi tertunda
//! di mempool node (`scan_mempool`, TSK-M36): UTXO yang sedang dibelanjakan tidak dihitung bisa dipakai dan tidak
//! dipilih lagi, output tertunda ke wallet dilaporkan sebagai masuk tertunda. Frasa, kunci, alamat, pemilihan koin, dan tanda tangan
//! seluruhnya dari MOD-09. Entropi frasa dan `aux_rand` tanda tangan diberikan pemanggil.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use homeostasis_primitives::{Amount, Encode, Network, OutPoint, OutputKind};
use homeostasis_rpc::{
    ChainInfo, Hex32, MAX_SCAN_PAYLOADS, MempoolOutput, MempoolScan, RpcClient, reject_reason,
};
use homeostasis_wallet_core::{
    AddressError, Bip32Error, CHANGE_BRANCH, Coin, ExtendedPrivKey, ExtendedPubKey, HARDENED,
    Mnemonic, MnemonicError, RECEIVE_BRANCH, SendError, SendRequest, account_key,
    build_transaction, decode_address, encode_address,
};
use thiserror::Error;
use tokio::runtime::Runtime;

use crate::args::{Action, Config};
use crate::scan::{ScanResult, scan_wallet, wallet_payloads};
use crate::wallet_file::{self, WalletFile, WalletFileError};

/// Galat menjalankan aksi. Biner keluar dengan kode 1.
#[derive(Debug, Error)]
pub enum CliError {
    /// Runtime tokio tidak bisa dibuat.
    #[error("runtime gagal dibuat: {0}")]
    Runtime(std::io::Error),
    /// Sumber acak sistem operasi tidak bisa dibaca.
    #[error("sumber acak sistem operasi gagal dibaca: {0}")]
    Random(std::io::Error),
    /// Panggilan RPC gagal (node tidak bisa dihubungi atau membalas error).
    #[error("RPC gagal: {message}")]
    Rpc {
        /// Keterangan dari klien RPC.
        message: String,
    },
    /// Node menolak transaksi (error 1002).
    #[error("node menolak transaksi: aturan {reason}")]
    Rejected {
        /// Nama aturan yang dilanggar, dari node.
        reason: String,
    },
    /// Node bukan regtest.
    #[error("node berjalan di jaringan {network}, CLI v0.1 hanya untuk regtest")]
    WrongNetwork {
        /// Nama jaringan node.
        network: String,
    },
    /// File wallet.
    #[error(transparent)]
    File(#[from] WalletFileError),
    /// Frasa atau passphrase tidak sah.
    #[error("frasa atau passphrase tidak sah: {0}")]
    Mnemonic(#[from] MnemonicError),
    /// Turunan kunci gagal.
    #[error("turunan kunci gagal: {0}")]
    Key(#[from] Bip32Error),
    /// Alamat penerima tidak sah.
    #[error("alamat penerima tidak sah: {0}")]
    Address(#[from] AddressError),
    /// Transaksi tidak bisa disusun.
    #[error("transaksi tidak bisa disusun: {0}")]
    Send(#[from] SendError),
    /// Semua indeks non-hardened di satu cabang sudah terpakai.
    #[error("indeks alamat habis")]
    IndexExhausted,
    /// Aksi wallet tanpa folder data (tidak terjadi untuk konfigurasi dari `parse_args`).
    #[error("folder data wajib untuk perintah wallet")]
    MissingDatadir,
}

/// Galat RPC sebagai teks, agar tipe galat klien tidak ikut terekspos.
fn rpc_error(error: impl core::fmt::Display) -> CliError {
    CliError::Rpc {
        message: error.to_string(),
    }
}

/// Saldo wallet di ujung chain, beserta transaksi tertunda di mempool.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Balance {
    /// Tinggi ujung chain.
    pub tip_height: u64,
    /// Jumlah semua UTXO milik wallet di chain (`bisa dipakai` + `belum matang` + `keluar tertunda`).
    pub total: Amount,
    /// Bagian yang boleh dibelanjakan di blok berikutnya dan belum dibelanjakan transaksi di mempool.
    pub spendable: Amount,
    /// Bagian yang belum matang (coinbase di bawah 100 konfirmasi).
    pub immature: Amount,
    /// UTXO wallet yang sedang dibelanjakan transaksi di mempool.
    pub pending_spent: Amount,
    /// Output transaksi di mempool ke alamat wallet (termasuk kembalian).
    pub pending_received: Amount,
    /// Saldo setelah semua transaksi tertunda masuk blok: `total − pending_spent + pending_received`.
    pub after_pending: Amount,
    /// Jumlah UTXO milik wallet di chain.
    pub utxos: usize,
}

/// Hasil `send`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sent {
    /// Txid dari node.
    pub txid: Hex32,
    /// Jumlah yang diterima penerima.
    pub amount: Amount,
    /// Biaya untuk penambang.
    pub fee: Amount,
    /// Kembalian, jika ada.
    pub change: Option<Amount>,
    /// Jumlah input.
    pub inputs: usize,
    /// Ukuran transaksi dalam byte.
    pub size: usize,
}

/// Transaksi tertunda di mempool node yang menyentuh wallet.
struct Pending {
    /// Outpoint UTXO wallet yang sedang dibelanjakan.
    spent: BTreeSet<(Hex32, u32)>,
    /// Output tertunda jenis 0 ke alamat wallet.
    outputs: Vec<MempoolOutput>,
}

/// Wallet yang sudah dibuka: path file, isi file, dan kunci akun `m/44'/1'/0'`.
struct OpenWallet {
    path: PathBuf,
    file: WalletFile,
    account: ExtendedPrivKey,
    public: ExtendedPubKey,
}

fn open_wallet(datadir: &Path, passphrase: &str) -> Result<OpenWallet, CliError> {
    let path = wallet_file::wallet_path(datadir);
    let file = wallet_file::load(&path)?;
    let seed = file.mnemonic()?.to_seed(passphrase)?;
    let account = account_key(&seed, Network::Regtest)?;
    let public = account.to_public()?;
    Ok(OpenWallet {
        path,
        file,
        account,
        public,
    })
}

/// Alamat pertama yang sah di `branch` mulai indeks `start`: `(indeks, x-only)`.
fn derive_unused(
    account: &ExtendedPubKey,
    branch: u32,
    start: u32,
) -> Result<(u32, [u8; 32]), CliError> {
    let branch_key = account.derive_child(branch)?;
    let mut index = start;
    while index < HARDENED {
        match branch_key.derive_child(index) {
            Ok(child) => return Ok((index, child.x_only())),
            // Indeks yang turunannya tidak sah dilewati, sesuai BIP32.
            Err(Bip32Error::InvalidChild { .. }) => {
                index = index.checked_add(1).ok_or(CliError::IndexExhausted)?;
            }
            Err(error) => return Err(error.into()),
        }
    }
    Err(CliError::IndexExhausted)
}

/// `wallet-create`: wallet baru dari 32 byte entropi. Mengembalikan frasa dan path file.
///
/// # Errors
///
/// [`CliError::File`] jika wallet sudah ada atau tidak bisa ditulis.
pub fn wallet_create(datadir: &Path, entropy: &[u8; 32]) -> Result<(Mnemonic, PathBuf), CliError> {
    let mnemonic = Mnemonic::from_entropy(entropy)?;
    let path = wallet_file::wallet_path(datadir);
    wallet_file::create_new(&path, &WalletFile::new(&mnemonic))?;
    Ok((mnemonic, path))
}

/// `wallet-restore`: wallet dari frasa 12 sampai 24 kata. Mengembalikan path file.
///
/// # Errors
///
/// [`CliError::Mnemonic`] jika frasa tidak sah, [`CliError::File`] jika wallet sudah ada atau tidak bisa ditulis.
pub fn wallet_restore(datadir: &Path, phrase: &str) -> Result<PathBuf, CliError> {
    let mnemonic = Mnemonic::parse(phrase)?;
    let path = wallet_file::wallet_path(datadir);
    wallet_file::create_new(&path, &WalletFile::new(&mnemonic))?;
    Ok(path)
}

/// `address`: alamat penerimaan berikutnya. Indeks penerimaan di file wallet dinaikkan.
///
/// # Errors
///
/// [`CliError::File`], [`CliError::Mnemonic`], [`CliError::Key`], atau [`CliError::IndexExhausted`].
pub fn next_address(datadir: &Path, passphrase: &str) -> Result<String, CliError> {
    let mut wallet = open_wallet(datadir, passphrase)?;
    let (index, payload) = derive_unused(
        &wallet.public,
        RECEIVE_BRANCH,
        wallet.file.next_index.receive,
    )?;
    wallet.file.next_index.receive = index.checked_add(1).ok_or(CliError::IndexExhausted)?;
    wallet_file::save(&wallet.path, &wallet.file)?;
    Ok(encode_address(
        Network::Regtest,
        OutputKind::SingleKey,
        &payload,
    ))
}

/// Klien ke satu node regtest.
pub struct Cli {
    client: RpcClient,
    runtime: Runtime,
}

impl Cli {
    /// Menyiapkan klien untuk node di `rpc_url`. Belum ada koneksi sampai panggilan pertama.
    ///
    /// # Errors
    ///
    /// [`CliError::Runtime`] jika runtime gagal dibuat, [`CliError::Rpc`] jika `rpc_url` tidak sah.
    pub fn connect(rpc_url: &str) -> Result<Self, CliError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(CliError::Runtime)?;
        let client = runtime
            .block_on(async { RpcClient::new(rpc_url) })
            .map_err(rpc_error)?;
        Ok(Self { client, runtime })
    }

    /// `info`: ringkasan chain dari node.
    ///
    /// # Errors
    ///
    /// [`CliError::Rpc`].
    pub fn info(&self) -> Result<ChainInfo, CliError> {
        self.runtime
            .block_on(self.client.get_chain_info())
            .map_err(rpc_error)
    }

    /// Ringkasan chain, dengan syarat node berjalan di regtest.
    fn regtest_info(&self) -> Result<ChainInfo, CliError> {
        let info = self.info()?;
        if info.network.network() != Network::Regtest {
            return Err(CliError::WrongNetwork {
                network: String::from(info.network),
            });
        }
        Ok(info)
    }

    /// Memindai wallet, lalu menyimpan indeks berikutnya jika pemindaian menemukan indeks terpakai yang lebih tinggi.
    fn scan(&self, wallet: &mut OpenWallet) -> Result<ScanResult, CliError> {
        let result = scan_wallet(&wallet.public, wallet.file.next_index, |payloads| {
            self.runtime
                .block_on(self.client.scan_utxos(payloads))
                .map_err(rpc_error)
        })?;
        if result.next_index != wallet.file.next_index {
            wallet.file.next_index = result.next_index;
            wallet_file::save(&wallet.path, &wallet.file)?;
        }
        Ok(result)
    }

    /// Transaksi tertunda di mempool node yang menyentuh alamat wallet sampai batas pemindaian `scanned`: outpoint
    /// UTXO wallet yang sedang dibelanjakan, dan output tertunda jenis 0 ke alamat wallet.
    fn pending(&self, wallet: &OpenWallet, scanned: &ScanResult) -> Result<Pending, CliError> {
        let payloads = wallet_payloads(&wallet.public, scanned.next_index);
        let mut scan = MempoolScan::default();
        for chunk in payloads.chunks(MAX_SCAN_PAYLOADS) {
            let part = self
                .runtime
                .block_on(self.client.scan_mempool(chunk))
                .map_err(rpc_error)?;
            scan.spent.extend(part.spent);
            scan.outputs.extend(part.outputs);
        }
        let mine: BTreeSet<Hex32> = payloads.into_iter().collect();
        let spent = scan
            .spent
            .iter()
            .map(|spend| (spend.txid, spend.index))
            .collect();
        let outputs = scan
            .outputs
            .into_iter()
            .filter(|output| {
                output.kind.kind() == OutputKind::SingleKey && mine.contains(&output.payload)
            })
            .collect();
        Ok(Pending { spent, outputs })
    }

    /// `balance`: saldo wallet dari node, termasuk transaksi tertunda di mempool.
    ///
    /// # Errors
    ///
    /// [`CliError::Rpc`], [`CliError::WrongNetwork`], galat file wallet, atau [`CliError::Send`] jika jumlah meluap.
    pub fn balance(&self, datadir: &Path, passphrase: &str) -> Result<Balance, CliError> {
        let info = self.regtest_info()?;
        let mut wallet = open_wallet(datadir, passphrase)?;
        let scanned = self.scan(&mut wallet)?;
        let pending = self.pending(&wallet, &scanned)?;
        // Sengaja saturating: pada tinggi u64::MAX semua UTXO dianggap sudah bisa dipakai.
        let limit = info.height.saturating_add(1);
        let overflow = || CliError::Send(SendError::AmountOverflow);
        let mut balance = Balance {
            tip_height: info.height,
            total: Amount::ZERO,
            spendable: Amount::ZERO,
            immature: Amount::ZERO,
            pending_spent: Amount::ZERO,
            pending_received: Amount::ZERO,
            after_pending: Amount::ZERO,
            utxos: scanned.found.len(),
        };
        for found in &scanned.found {
            let amount = found.utxo.amount.amount();
            balance.total = balance.total.checked_add(amount).ok_or_else(overflow)?;
            if pending.spent.contains(&(found.utxo.txid, found.utxo.index)) {
                balance.pending_spent = balance
                    .pending_spent
                    .checked_add(amount)
                    .ok_or_else(overflow)?;
            } else if found.utxo.spendable_at <= limit {
                balance.spendable = balance.spendable.checked_add(amount).ok_or_else(overflow)?;
            } else {
                balance.immature = balance.immature.checked_add(amount).ok_or_else(overflow)?;
            }
        }
        for output in &pending.outputs {
            balance.pending_received = balance
                .pending_received
                .checked_add(output.amount.amount())
                .ok_or_else(overflow)?;
        }
        balance.after_pending = balance
            .total
            .checked_sub(balance.pending_spent)
            .and_then(|rest| rest.checked_add(balance.pending_received))
            .ok_or_else(overflow)?;
        Ok(balance)
    }

    /// `send`: memilih koin, menandatangani dengan `aux_rand`, lalu mengirim ke mempool node. UTXO yang sedang
    /// dibelanjakan transaksi di mempool tidak dipilih. Indeks kembalian di file wallet dinaikkan jika transaksi
    /// memakai output kembalian.
    ///
    /// # Errors
    ///
    /// [`CliError::Address`], [`CliError::Send`], [`CliError::Rejected`], [`CliError::Rpc`],
    /// [`CliError::WrongNetwork`], atau galat file wallet.
    pub fn send(
        &self,
        datadir: &Path,
        passphrase: &str,
        to: &str,
        amount: Amount,
        fee_rate: u128,
        aux_rand: &[u8; 32],
    ) -> Result<Sent, CliError> {
        let (recipient_kind, recipient_payload) = decode_address(Network::Regtest, to)?;
        let info = self.regtest_info()?;
        let network_id = self
            .runtime
            .block_on(self.client.get_block_hash(0))
            .map_err(rpc_error)?
            .to_hash32();
        let mut wallet = open_wallet(datadir, passphrase)?;
        let scanned = self.scan(&mut wallet)?;
        let pending = self.pending(&wallet, &scanned)?;
        let coins: Vec<Coin> = scanned
            .found
            .iter()
            .filter(|found| !pending.spent.contains(&(found.utxo.txid, found.utxo.index)))
            .map(|found| {
                Coin::new(
                    OutPoint::new(found.utxo.txid.to_hash32(), found.utxo.index),
                    found.utxo.amount.amount(),
                    *found.utxo.payload.bytes(),
                    found.utxo.spendable_at,
                    found.branch,
                    found.index,
                )
            })
            .collect();
        let (change_index, change_payload) =
            derive_unused(&wallet.public, CHANGE_BRANCH, wallet.file.next_index.change)?;
        let request = SendRequest {
            recipient_kind,
            recipient_payload,
            amount,
            fee_rate,
            change_payload,
            tip_height: info.height,
        };
        let (plan, tx) =
            build_transaction(&coins, &request, &wallet.account, &network_id, aux_rand)?;
        let result = self
            .runtime
            .block_on(self.client.send_raw_transaction(&tx.to_bytes()))
            .map_err(|error| match reject_reason(&error) {
                Some(reason) => CliError::Rejected { reason },
                None => rpc_error(error),
            })?;
        if plan.change().is_some() {
            wallet.file.next_index.change = change_index
                .checked_add(1)
                .ok_or(CliError::IndexExhausted)?;
            wallet_file::save(&wallet.path, &wallet.file)?;
        }
        Ok(Sent {
            txid: result.txid,
            amount,
            fee: plan.fee(),
            change: plan.change(),
            inputs: plan.coins().len(),
            size: plan.size(),
        })
    }
}

/// Teks keluaran `info`.
#[must_use]
pub fn format_info(info: &ChainInfo) -> String {
    format!(
        "jaringan: {}\ntinggi: {}\nujung: {}\nhasil tambang: {}\ntotal pasokan: {}\nmempool: {} transaksi\n",
        String::from(info.network),
        info.height,
        String::from(info.best_block_id),
        info.mined_total.amount(),
        info.total_supply.amount(),
        info.mempool_count,
    )
}

/// Teks keluaran `balance`.
#[must_use]
pub fn format_balance(balance: &Balance) -> String {
    format!(
        "tinggi ujung: {}\nsaldo: {}\nbisa dipakai: {}\nbelum matang: {}\nkeluar tertunda: {}\nmasuk tertunda: {}\nsaldo setelah tertunda masuk blok: {}\nutxo: {}\n",
        balance.tip_height,
        balance.total,
        balance.spendable,
        balance.immature,
        balance.pending_spent,
        balance.pending_received,
        balance.after_pending,
        balance.utxos,
    )
}

/// Teks keluaran `send`.
#[must_use]
pub fn format_sent(sent: &Sent) -> String {
    let change = sent
        .change
        .map_or_else(|| "tidak ada".to_owned(), |amount| amount.to_string());
    format!(
        "txid: {}\ndikirim: {}\nbiaya: {}\nkembalian: {}\ninput: {}\nukuran: {} byte\n",
        String::from(sent.txid),
        sent.amount,
        sent.fee,
        change,
        sent.inputs,
        sent.size,
    )
}

/// Menjalankan satu aksi dan mengembalikan teks keluarannya. `random` dipanggil untuk entropi frasa
/// (`wallet-create`) dan `aux_rand` tanda tangan (`send`).
///
/// # Errors
///
/// [`CliError`] dari aksi itu.
pub fn run(
    config: &Config,
    mut random: impl FnMut() -> Result<[u8; 32], CliError>,
) -> Result<String, CliError> {
    let datadir = || config.datadir.as_deref().ok_or(CliError::MissingDatadir);
    match &config.action {
        Action::Info => Ok(format_info(&Cli::connect(&config.rpc_url)?.info()?)),
        Action::WalletCreate => {
            let (mnemonic, path) = wallet_create(datadir()?, &random()?)?;
            Ok(format!(
                "wallet dibuat: {}\nfrasa pemulihan ({} kata, simpan baik-baik; file regtest tanpa enkripsi):\n{}\n",
                path.display(),
                mnemonic.word_count(),
                mnemonic.phrase(),
            ))
        }
        Action::WalletRestore { mnemonic } => {
            let path = wallet_restore(datadir()?, mnemonic)?;
            Ok(format!(
                "wallet dipulihkan: {}\njalankan balance agar alamat yang sudah terpakai ikut terpindai\n",
                path.display(),
            ))
        }
        Action::Address => {
            let address = next_address(datadir()?, &config.passphrase)?;
            let (_kind, payload) = decode_address(Network::Regtest, &address)?;
            Ok(format!(
                "alamat: {address}\npayload: {}\n",
                String::from(Hex32::new(payload))
            ))
        }
        Action::Balance => {
            let cli = Cli::connect(&config.rpc_url)?;
            Ok(format_balance(
                &cli.balance(datadir()?, &config.passphrase)?,
            ))
        }
        Action::Send {
            to,
            amount,
            fee_rate,
        } => {
            let cli = Cli::connect(&config.rpc_url)?;
            let aux_rand = random()?;
            let sent = cli.send(
                datadir()?,
                &config.passphrase,
                to,
                *amount,
                *fee_rate,
                &aux_rand,
            )?;
            Ok(format_sent(&sent))
        }
    }
}

/// Membaca 32 byte dari `/dev/urandom` (Unix).
///
/// # Errors
///
/// [`CliError::Random`] jika sumber acak tidak bisa dibaca.
pub fn os_random() -> Result<[u8; 32], CliError> {
    use std::io::Read;
    let mut bytes = [0_u8; 32];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .map_err(CliError::Random)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests;
