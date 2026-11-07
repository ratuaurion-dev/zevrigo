//! Riwayat transaksi wallet (TSK-M41, DEC-31): dihitung di sisi wallet dari blok chain aktif, tanpa indeks alamat di
//! node (DEC-28).
//!
//! Wallet membaca blok satu per satu lewat RPC `get_block_hash` dan `get_block` mulai dari genesis, mencatat output
//! jenis 0 ke alamatnya (diturunkan dari kunci akun publik dengan gap limit 20, diperluas saat alamat terpakai
//! ditemukan) dan input yang membelanjakan output itu. Hasilnya disimpan di `<folder>/<jaringan>/history.json`
//! (tembolok, bukan sumber kebenaran): pemindaian berikutnya hanya membaca blok baru. Bila blok ujung tembolok tidak lagi
//! ada di chain aktif (reorg) atau file tidak sah, tembolok dibuang dan pemindaian diulang dari genesis.
//!
//! Satu entri per transaksi yang menyentuh wallet: jumlah masuk (output ke wallet), jumlah keluar (koin wallet yang
//! dibelanjakan), dan biaya jika semua input milik wallet. Transaksi di mempool dilaporkan sebagai entri tertunda
//! (tanpa tinggi dan tanpa biaya).

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zevrigo_primitives::{Amount, Block, Network, OutputKind, TxBody};
use zevrigo_rpc::{Hex32, MempoolOutput, MempoolSpend, Quanta};
use zevrigo_wallet_core::{CHANGE_BRANCH, ExtendedPubKey, HARDENED, RECEIVE_BRANCH};

use crate::scan::GAP_LIMIT;
use crate::wallet_file::{NextIndex, network_name};

/// Nama file tembolok riwayat di folder jaringan.
pub const HISTORY_FILE: &str = "history.json";

/// Versi format file tembolok riwayat.
pub const HISTORY_VERSION: u32 = 1;

/// Akhiran file sementara saat menulis tembolok.
const TEMP_SUFFIX: &str = ".tmp";

/// Galat menghitung riwayat dari blok.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum HistoryError {
    /// Jumlah txid dari node tidak sama dengan jumlah transaksi di blok.
    #[error("jumlah txid {actual} tidak sama dengan jumlah transaksi {expected}")]
    TxidCount {
        /// Jumlah transaksi di blok.
        expected: usize,
        /// Jumlah txid dari node.
        actual: usize,
    },
    /// Jumlah koin meluap `u128`.
    #[error("jumlah koin meluap")]
    AmountOverflow,
    /// Indeks output tidak muat `u32`.
    #[error("indeks output terlalu besar")]
    OutputIndex,
}

/// Satu transaksi yang menyentuh wallet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    /// Txid.
    pub txid: Hex32,
    /// Tinggi blok, `None` untuk transaksi tertunda di mempool.
    pub height: Option<u64>,
    /// Cap waktu blok (detik Unix), `None` untuk transaksi tertunda.
    pub timestamp: Option<u64>,
    /// Transaksi coinbase (hadiah tambang).
    pub coinbase: bool,
    /// Jumlah output ke alamat wallet (termasuk kembalian).
    pub received: Amount,
    /// Jumlah koin wallet yang dibelanjakan transaksi ini.
    pub spent: Amount,
    /// Biaya, hanya jika semua input milik wallet dan transaksi sudah masuk blok.
    pub fee: Option<Amount>,
}

/// Satu cabang jalur kunci: kunci cabang, indeks terpakai, dan batas indeks yang sudah diturunkan.
struct BranchKeys {
    branch: u32,
    key: Option<ExtendedPubKey>,
    used: u32,
    until: u32,
}

impl BranchKeys {
    fn new(account: &ExtendedPubKey, branch: u32, used: u32) -> Self {
        Self {
            branch,
            // Kunci cabang tidak sah (peluang sekitar 2^-127): cabang itu tidak punya alamat.
            key: account.derive_child(branch).ok(),
            used: used.min(HARDENED),
            until: 0,
        }
    }
}

/// Payload alamat wallet di kedua cabang, sampai `GAP_LIMIT` indeks di atas indeks terpakai tertinggi.
pub struct WalletKeys {
    receive: BranchKeys,
    change: BranchKeys,
    map: BTreeMap<[u8; 32], (u32, u32)>,
}

impl WalletKeys {
    /// Menurunkan payload dari kunci akun publik mulai indeks 0 sampai `GAP_LIMIT` indeks di atas `next_index`.
    #[must_use]
    pub fn new(account: &ExtendedPubKey, next_index: NextIndex) -> Self {
        let mut keys = Self {
            receive: BranchKeys::new(account, RECEIVE_BRANCH, next_index.receive),
            change: BranchKeys::new(account, CHANGE_BRANCH, next_index.change),
            map: BTreeMap::new(),
        };
        keys.extend();
        keys
    }

    /// Menurunkan payload sampai batas gap limit setiap cabang.
    fn extend(&mut self) {
        for branch in [&mut self.receive, &mut self.change] {
            let target = branch.used.saturating_add(GAP_LIMIT).min(HARDENED);
            let Some(key) = &branch.key else {
                continue;
            };
            while branch.until < target {
                // Indeks yang turunannya tidak sah dilewati, sesuai BIP32.
                if let Ok(child) = key.derive_child(branch.until) {
                    self.map
                        .insert(child.x_only(), (branch.branch, branch.until));
                }
                branch.until = branch.until.saturating_add(1);
            }
        }
    }

    /// `true` jika `payload` milik wallet, tanpa memperluas batas.
    #[must_use]
    pub fn contains(&self, payload: &[u8; 32]) -> bool {
        self.map.contains_key(payload)
    }

    /// `true` jika `payload` milik wallet. Indeks payload ditandai terpakai, dan batas gap limit cabangnya diperluas.
    pub fn owns(&mut self, payload: &[u8; 32]) -> bool {
        let Some(&(branch, index)) = self.map.get(payload) else {
            return false;
        };
        let used = index.saturating_add(1);
        let slot = if branch == RECEIVE_BRANCH {
            &mut self.receive
        } else {
            &mut self.change
        };
        if used > slot.used {
            slot.used = used;
            self.extend();
        }
        true
    }

    /// Indeks berikutnya yang belum terpakai per cabang.
    #[must_use]
    pub const fn next_index(&self) -> NextIndex {
        NextIndex {
            receive: self.receive.used,
            change: self.change.used,
        }
    }

    /// Semua payload yang sudah diturunkan, urut byte.
    #[must_use]
    pub fn payloads(&self) -> Vec<Hex32> {
        self.map
            .keys()
            .map(|payload| Hex32::new(*payload))
            .collect()
    }
}

/// Blok terakhir yang sudah dipindai.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryTip {
    /// Tinggi blok.
    pub height: u64,
    /// ID blok.
    pub block_id: Hex32,
}

/// Keadaan pemindaian riwayat: blok terakhir, koin wallet yang belum dibelanjakan, dan entri yang sudah masuk blok.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HistoryState {
    /// Blok terakhir yang sudah dipindai, `None` sebelum genesis.
    pub tip: Option<HistoryTip>,
    coins: BTreeMap<(Hex32, u32), Amount>,
    entries: Vec<HistoryEntry>,
}

impl HistoryState {
    /// Keadaan kosong, sebelum genesis.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Entri yang sudah masuk blok, urut seperti di chain (lama dulu).
    #[must_use]
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Jumlah koin wallet yang belum dibelanjakan menurut blok yang sudah dipindai.
    #[must_use]
    pub fn unspent_count(&self) -> usize {
        self.coins.len()
    }

    /// Memproses satu blok setinggi `height` dengan `txids` dari node (urut seperti di blok). Blok tidak divalidasi:
    /// node hanya mengirim blok chain aktif yang sudah diperiksa MOD-04.
    ///
    /// # Errors
    ///
    /// [`HistoryError::TxidCount`] jika jumlah txid berbeda, [`HistoryError::AmountOverflow`], atau
    /// [`HistoryError::OutputIndex`].
    pub fn apply_block(
        &mut self,
        keys: &mut WalletKeys,
        block: &Block,
        txids: &[Hex32],
        height: u64,
    ) -> Result<(), HistoryError> {
        let txs = block.txs();
        if txs.len() != txids.len() {
            return Err(HistoryError::TxidCount {
                expected: txs.len(),
                actual: txids.len(),
            });
        }
        let timestamp = block.header().timestamp();
        for (tx, txid) in txs.iter().zip(txids) {
            let mut spent = Amount::ZERO;
            let mut ours: usize = 0;
            let mut inputs: usize = 0;
            if let TxBody::Regular(body) = tx.body() {
                inputs = body.inputs().len();
                for input in body.inputs() {
                    let key = (Hex32::from(input.prev_txid()), input.prev_index());
                    if let Some(amount) = self.coins.remove(&key) {
                        spent = spent
                            .checked_add(amount)
                            .ok_or(HistoryError::AmountOverflow)?;
                        ours = ours.saturating_add(1);
                    }
                }
            }
            let mut received = Amount::ZERO;
            let mut outputs_total = Amount::ZERO;
            for (index, output) in tx.body().outputs().iter().enumerate() {
                outputs_total = outputs_total
                    .checked_add(output.amount())
                    .ok_or(HistoryError::AmountOverflow)?;
                if output.kind() == OutputKind::SingleKey && keys.owns(output.payload()) {
                    let index = u32::try_from(index).map_err(|_| HistoryError::OutputIndex)?;
                    self.coins.insert((*txid, index), output.amount());
                    received = received
                        .checked_add(output.amount())
                        .ok_or(HistoryError::AmountOverflow)?;
                }
            }
            if ours == 0 && received == Amount::ZERO {
                continue;
            }
            let fee = if inputs > 0 && ours == inputs {
                spent.checked_sub(outputs_total)
            } else {
                None
            };
            self.entries.push(HistoryEntry {
                txid: *txid,
                height: Some(height),
                timestamp: Some(timestamp),
                coinbase: tx.is_coinbase(),
                received,
                spent,
                fee,
            });
        }
        Ok(())
    }

    /// Entri tertunda dari hasil `scan_mempool`: `spends` (UTXO yang sedang dibelanjakan) dan `outputs` (output ke
    /// wallet, sudah disaring pemanggil), satu entri per txid mempool, urut txid.
    ///
    /// # Errors
    ///
    /// [`HistoryError::AmountOverflow`].
    pub fn pending_entries(
        &self,
        spends: &[MempoolSpend],
        outputs: &[MempoolOutput],
    ) -> Result<Vec<HistoryEntry>, HistoryError> {
        let mut totals: BTreeMap<Hex32, (Amount, Amount)> = BTreeMap::new();
        for spend in spends {
            let Some(&amount) = self.coins.get(&(spend.txid, spend.index)) else {
                continue;
            };
            let slot = totals
                .entry(spend.spent_by)
                .or_insert((Amount::ZERO, Amount::ZERO));
            slot.0 = slot
                .0
                .checked_add(amount)
                .ok_or(HistoryError::AmountOverflow)?;
        }
        for output in outputs {
            let slot = totals
                .entry(output.txid)
                .or_insert((Amount::ZERO, Amount::ZERO));
            slot.1 = slot
                .1
                .checked_add(output.amount.amount())
                .ok_or(HistoryError::AmountOverflow)?;
        }
        Ok(totals
            .into_iter()
            .map(|(txid, (spent, received))| HistoryEntry {
                txid,
                height: None,
                timestamp: None,
                coinbase: false,
                received,
                spent,
                fee: None,
            })
            .collect())
    }
}

/// Riwayat wallet untuk ditampilkan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct History {
    /// Tinggi ujung chain saat dipindai.
    pub tip_height: u64,
    /// Entri terbaru dulu: tertunda, lalu yang sudah masuk blok dari tinggi terbesar.
    pub entries: Vec<HistoryEntry>,
}

/// Path tembolok riwayat: `<datadir>/<jaringan>/history.json`.
#[must_use]
pub fn history_path(network: Network, datadir: &Path) -> PathBuf {
    datadir.join(network_name(network)).join(HISTORY_FILE)
}

/// Penanda wallet di tembolok: payload alamat penerimaan pertama (`m/…/0/0`). Tembolok wallet lain dibuang.
#[must_use]
pub fn identity(account: &ExtendedPubKey) -> Hex32 {
    account
        .derive_child(RECEIVE_BRANCH)
        .and_then(|branch| branch.derive_child(0))
        .map_or(Hex32::new([0_u8; 32]), |child| Hex32::new(child.x_only()))
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CacheCoin {
    txid: Hex32,
    index: u32,
    amount: Quanta,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CacheEntry {
    txid: Hex32,
    height: u64,
    timestamp: u64,
    coinbase: bool,
    received: Quanta,
    spent: Quanta,
    fee: Option<Quanta>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CacheFile {
    version: u32,
    network: String,
    identity: Hex32,
    tip: Option<HistoryTip>,
    coins: Vec<CacheCoin>,
    entries: Vec<CacheEntry>,
}

/// Teks JSON tembolok untuk `state`.
fn to_cache_json(
    state: &HistoryState,
    network: Network,
    identity: Hex32,
) -> Result<String, serde_json::Error> {
    let file = CacheFile {
        version: HISTORY_VERSION,
        network: network_name(network).to_owned(),
        identity,
        tip: state.tip,
        coins: state
            .coins
            .iter()
            .map(|(&(txid, index), &amount)| CacheCoin {
                txid,
                index,
                amount: Quanta::new(amount),
            })
            .collect(),
        entries: state
            .entries
            .iter()
            .filter_map(|entry| {
                Some(CacheEntry {
                    txid: entry.txid,
                    height: entry.height?,
                    timestamp: entry.timestamp?,
                    coinbase: entry.coinbase,
                    received: Quanta::new(entry.received),
                    spent: Quanta::new(entry.spent),
                    fee: entry.fee.map(Quanta::new),
                })
            })
            .collect(),
    };
    serde_json::to_string_pretty(&file)
}

/// Keadaan dari teks tembolok, atau `None` jika tidak sah, versi lain, jaringan lain, atau wallet lain.
fn from_cache_json(text: &str, network: Network, identity: Hex32) -> Option<HistoryState> {
    let file: CacheFile = serde_json::from_str(text).ok()?;
    if file.version != HISTORY_VERSION
        || file.network != network_name(network)
        || file.identity != identity
    {
        return None;
    }
    let mut coins = BTreeMap::new();
    for coin in file.coins {
        if coins
            .insert((coin.txid, coin.index), coin.amount.amount())
            .is_some()
        {
            return None;
        }
    }
    let entries = file
        .entries
        .into_iter()
        .map(|entry| HistoryEntry {
            txid: entry.txid,
            height: Some(entry.height),
            timestamp: Some(entry.timestamp),
            coinbase: entry.coinbase,
            received: entry.received.amount(),
            spent: entry.spent.amount(),
            fee: entry.fee.map(Quanta::amount),
        })
        .collect();
    Some(HistoryState {
        tip: file.tip,
        coins,
        entries,
    })
}

/// Membaca tembolok riwayat. File yang tidak ada, tidak sah, atau milik wallet atau jaringan lain menghasilkan
/// keadaan kosong (pemindaian dari genesis).
#[must_use]
pub fn load(path: &Path, network: Network, identity: Hex32) -> HistoryState {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| from_cache_json(&text, network, identity))
        .unwrap_or_default()
}

/// Menyimpan tembolok riwayat: tulis ke `<path>.tmp` (izin `0600` di Unix), lalu ganti nama menjadi `path`.
///
/// # Errors
///
/// Galat baca, tulis, atau ganti nama dari sistem operasi.
pub fn save(
    path: &Path,
    network: Network,
    identity: Hex32,
    state: &HistoryState,
) -> Result<(), std::io::Error> {
    let text = to_cache_json(state, network, identity).map_err(std::io::Error::other)?;
    let mut temp = path.as_os_str().to_owned();
    temp.push(TEMP_SUFFIX);
    let temp = PathBuf::from(temp);
    match fs::remove_file(&temp) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temp)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temp, path)
}

#[cfg(test)]
mod tests;
