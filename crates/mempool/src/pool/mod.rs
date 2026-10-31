//! Struktur [`Mempool`] dan aturan antreannya.
//!
//! - Transaksi baru diperiksa berurutan: duplikat, ukuran maksimum, sisa kapasitas, bentrok input dengan transaksi
//!   lain di mempool (tanpa penggantian), lalu T1 sampai T6 terhadap view chain + mempool.
//! - Setelah blok baru atau reorg, [`Mempool::revalidate`] memeriksa ulang semua transaksi sesuai urutan kedatangan.
//!   Transaksi yang sudah masuk blok, bentrok dengan blok, atau bergantung pada transaksi yang terbuang ikut terbuang.
//! - [`Mempool::select_for_block`] memilih transaksi sesuai urutan kedatangan, sehingga induk selalu sebelum anak.

use std::collections::{BTreeMap, BTreeSet};

use homeostasis_consensus::{TxError, UtxoLookup, check_transaction};
use homeostasis_primitives::amount::Amount;
use homeostasis_primitives::hash::Hash32;
use homeostasis_primitives::height::BlockHeight;
use homeostasis_primitives::params::MAX_BLOCK_SIZE;
use homeostasis_primitives::tx::{Transaction, TxBody};
use homeostasis_primitives::utxo::{OutPoint, UtxoEntry};
use thiserror::Error;

/// Batas bawaan total ukuran transaksi di mempool: 10_000_000 byte (sekitar 100 blok penuh).
pub const DEFAULT_MAX_BYTES: usize = 10_000_000;

/// Nama aturan konsensus untuk `data.reason` error RPC `1002` (Spesifikasi RPC v0.1 Bagian 4).
///
/// Galat struktur yang bukan bagian T1 sampai T6 (transaksi coinbase, jumlah witness, indeks input) bernama
/// `struktur`.
#[must_use]
pub const fn rule_name(error: &TxError) -> &'static str {
    match error {
        TxError::DuplicateInput { .. } | TxError::MissingUtxo { .. } => "T1",
        TxError::ImmatureCoinbase { .. } => "T2",
        TxError::WitnessKindMismatch { .. } => "T3",
        TxError::ConditionHashMismatch { .. } | TxError::ConditionLocked { .. } => "T4",
        TxError::AmountOverflow | TxError::OutputsExceedInputs => "T5",
        TxError::Sighash(_) | TxError::InvalidSignature { .. } => "T6",
        TxError::NotRegular | TxError::WitnessCountMismatch | TxError::IndexOverflow => "struktur",
    }
}

/// Alasan transaksi ditolak mempool.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum MempoolError {
    /// Transaksi dengan txid yang sama sudah ada di mempool.
    #[error("transaksi sudah ada di mempool")]
    Duplicate {
        /// Txid transaksi.
        txid: Hash32,
    },
    /// Input sudah dibelanjakan transaksi lain di mempool. Penggantian transaksi tidak didukung.
    #[error("input {outpoint:?} sudah dibelanjakan transaksi lain di mempool")]
    Conflict {
        /// Outpoint yang diperebutkan.
        outpoint: OutPoint,
        /// Txid transaksi yang lebih dulu membelanjakannya.
        existing: Hash32,
    },
    /// Ukuran transaksi melebihi ukuran blok maksimum, sehingga tidak akan pernah bisa ditambang.
    #[error("ukuran transaksi {size} byte melebihi ukuran blok maksimum")]
    TooLarge {
        /// Ukuran encoding transaksi dalam byte.
        size: usize,
    },
    /// Mempool penuh.
    #[error("mempool penuh: transaksi {size} byte tidak muat ({used} dari {limit} byte terpakai)")]
    Full {
        /// Ukuran transaksi dalam byte.
        size: usize,
        /// Byte yang sudah terpakai.
        used: usize,
        /// Batas byte mempool.
        limit: usize,
    },
    /// Transaksi melanggar aturan konsensus T1 sampai T6.
    #[error("transaksi tidak sah: {0}")]
    Invalid(#[from] TxError),
}

impl MempoolError {
    /// Nama aturan untuk error RPC `1002`, atau `None` untuk penolakan antrean yang bukan aturan konsensus.
    ///
    /// Bentrok input dengan mempool bernama `T1`, karena outpoint itu tidak ada lagi di view chain + mempool.
    #[must_use]
    pub const fn rule(&self) -> Option<&'static str> {
        match self {
            Self::Invalid(error) => Some(rule_name(error)),
            Self::Conflict { .. } => Some("T1"),
            Self::Duplicate { .. } | Self::TooLarge { .. } | Self::Full { .. } => None,
        }
    }
}

/// Satu transaksi di mempool.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MempoolEntry {
    tx: Transaction,
    txid: Hash32,
    fee: Amount,
    size: usize,
    sequence: u64,
}

impl MempoolEntry {
    /// Transaksi lengkap.
    #[must_use]
    pub const fn tx(&self) -> &Transaction {
        &self.tx
    }

    /// Txid transaksi (diberikan pemanggil).
    #[must_use]
    pub const fn txid(&self) -> Hash32 {
        self.txid
    }

    /// Biaya transaksi (`Σ input − Σ output`).
    #[must_use]
    pub const fn fee(&self) -> Amount {
        self.fee
    }

    /// Ukuran encoding transaksi dalam byte.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.size
    }

    /// Nomor urut kedatangan. Makin kecil makin awal.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
}

/// View UTXO untuk `check_transaction`: output transaksi di mempool, lalu set UTXO chain.
struct PoolView<'a, U> {
    chain: &'a U,
    entries: &'a BTreeMap<Hash32, MempoolEntry>,
    height: BlockHeight,
}

impl<U: UtxoLookup> UtxoLookup for PoolView<'_, U> {
    fn lookup(&self, outpoint: &OutPoint) -> Option<UtxoEntry> {
        if let Some(entry) = self.entries.get(&outpoint.txid()) {
            let index = usize::try_from(outpoint.index()).ok()?;
            let output = entry.tx.body().outputs().get(index)?;
            return Some(UtxoEntry::new(output.clone(), self.height, false));
        }
        self.chain.lookup(outpoint)
    }
}

/// Antrean transaksi belum terkonfirmasi.
///
/// Invarian: setiap input transaksi di mempool tercatat tepat sekali di indeks pembelanja, dan setiap transaksi lolos
/// T1 sampai T6 terhadap view chain + transaksi yang datang lebih dulu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mempool {
    entries: BTreeMap<Hash32, MempoolEntry>,
    order: BTreeMap<u64, Hash32>,
    spends: BTreeMap<OutPoint, Hash32>,
    total_bytes: usize,
    max_bytes: usize,
    next_sequence: u64,
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_BYTES)
    }
}

impl Mempool {
    /// Mempool kosong dengan batas total `max_bytes` byte.
    #[must_use]
    pub const fn new(max_bytes: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            order: BTreeMap::new(),
            spends: BTreeMap::new(),
            total_bytes: 0,
            max_bytes,
            next_sequence: 0,
        }
    }

    /// Memasukkan transaksi dengan txid `txid` (dihitung pemanggil). `height` adalah tinggi blok berikutnya
    /// (ujung chain + 1), `chain` adalah set UTXO di ujung chain. Mengembalikan biaya transaksi.
    ///
    /// # Errors
    ///
    /// Berurutan: [`MempoolError::Duplicate`], [`MempoolError::TooLarge`], [`MempoolError::Full`],
    /// [`MempoolError::Conflict`], lalu [`MempoolError::Invalid`]. Jika gagal, isi mempool tidak berubah.
    pub fn add<U: UtxoLookup>(
        &mut self,
        tx: Transaction,
        txid: Hash32,
        height: BlockHeight,
        network_id: &Hash32,
        chain: &U,
    ) -> Result<Amount, MempoolError> {
        if self.entries.contains_key(&txid) {
            return Err(MempoolError::Duplicate { txid });
        }
        let size = tx.encoded_size().unwrap_or(usize::MAX);
        if size > MAX_BLOCK_SIZE {
            return Err(MempoolError::TooLarge { size });
        }
        let used = self.total_bytes;
        if !used
            .checked_add(size)
            .is_some_and(|total| total <= self.max_bytes)
        {
            return Err(MempoolError::Full {
                size,
                used,
                limit: self.max_bytes,
            });
        }
        if let TxBody::Regular(body) = tx.body() {
            for input in body.inputs() {
                let outpoint = OutPoint::from_input(input);
                if let Some(&existing) = self.spends.get(&outpoint) {
                    return Err(MempoolError::Conflict { outpoint, existing });
                }
            }
        }
        let view = PoolView {
            chain,
            entries: &self.entries,
            height,
        };
        let fee = check_transaction(&tx, height, network_id, &view)?;
        self.insert(tx, txid, fee, size);
        Ok(fee)
    }

    /// Mencatat transaksi yang sudah lolos pemeriksaan.
    fn insert(&mut self, tx: Transaction, txid: Hash32, fee: Amount, size: usize) {
        if let TxBody::Regular(body) = tx.body() {
            for input in body.inputs() {
                self.spends.insert(OutPoint::from_input(input), txid);
            }
        }
        let sequence = self.next_sequence;
        // Sengaja saturating: 2^64 transaksi tidak tercapai; jika pun tercapai, urutan tetap tidak turun.
        self.next_sequence = sequence.saturating_add(1);
        // Sengaja saturating: `add` sudah memastikan total tidak melebihi `max_bytes`.
        self.total_bytes = self.total_bytes.saturating_add(size);
        self.order.insert(sequence, txid);
        self.entries.insert(
            txid,
            MempoolEntry {
                tx,
                txid,
                fee,
                size,
                sequence,
            },
        );
    }

    /// Memeriksa ulang semua transaksi sesuai urutan kedatangan setelah ujung chain berubah (blok baru atau reorg).
    /// `height` dan `chain` seperti di [`Mempool::add`], untuk ujung chain yang baru. Mengembalikan txid yang
    /// terbuang, sesuai urutan kedatangan.
    ///
    /// Biayanya sebanding dengan isi mempool (termasuk verifikasi tanda tangan ulang); cukup untuk regtest.
    pub fn revalidate<U: UtxoLookup>(
        &mut self,
        height: BlockHeight,
        network_id: &Hash32,
        chain: &U,
    ) -> Vec<Hash32> {
        let mut fresh = Self::new(self.max_bytes);
        fresh.next_sequence = self.next_sequence;
        let old = std::mem::replace(self, fresh);
        let Self {
            mut entries, order, ..
        } = old;
        let mut dropped = Vec::new();
        for txid in order.into_values() {
            let Some(entry) = entries.remove(&txid) else {
                continue;
            };
            if self.add(entry.tx, txid, height, network_id, chain).is_err() {
                dropped.push(txid);
            }
        }
        dropped
    }

    /// Memilih transaksi untuk blok berikutnya, paling banyak `max_bytes` byte, sesuai urutan kedatangan.
    /// Transaksi yang induknya di mempool tidak terpilih ikut dilewati.
    #[must_use]
    pub fn select_for_block(&self, max_bytes: usize) -> Vec<&MempoolEntry> {
        let mut chosen = Vec::new();
        let mut chosen_ids = BTreeSet::new();
        let mut used: usize = 0;
        for entry in self.in_arrival_order() {
            let parents_chosen = match entry.tx.body() {
                TxBody::Regular(body) => body.inputs().iter().all(|input| {
                    let parent = OutPoint::from_input(input).txid();
                    !self.entries.contains_key(&parent) || chosen_ids.contains(&parent)
                }),
                TxBody::Coinbase(_) => false,
            };
            if !parents_chosen {
                continue;
            }
            let Some(next) = used.checked_add(entry.size) else {
                continue;
            };
            if next > max_bytes {
                continue;
            }
            used = next;
            chosen_ids.insert(entry.txid);
            chosen.push(entry);
        }
        chosen
    }

    /// Transaksi sesuai urutan kedatangan.
    pub fn in_arrival_order(&self) -> impl Iterator<Item = &MempoolEntry> {
        self.order
            .values()
            .filter_map(|txid| self.entries.get(txid))
    }

    /// Txid semua transaksi, urut naik bytewise (hasil RPC `get_mempool`).
    #[must_use]
    pub fn txids(&self) -> Vec<Hash32> {
        self.entries.keys().copied().collect()
    }

    /// Transaksi dengan txid `txid`, jika ada.
    #[must_use]
    pub fn get(&self, txid: &Hash32) -> Option<&MempoolEntry> {
        self.entries.get(txid)
    }

    /// Apakah transaksi `txid` ada di mempool.
    #[must_use]
    pub fn contains(&self, txid: &Hash32) -> bool {
        self.entries.contains_key(txid)
    }

    /// Txid transaksi di mempool yang membelanjakan `outpoint`, jika ada.
    #[must_use]
    pub fn spender(&self, outpoint: &OutPoint) -> Option<Hash32> {
        self.spends.get(outpoint).copied()
    }

    /// Jumlah transaksi.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Apakah mempool kosong.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Total ukuran transaksi dalam byte.
    #[must_use]
    pub const fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Batas total ukuran transaksi dalam byte.
    #[must_use]
    pub const fn max_bytes(&self) -> usize {
        self.max_bytes
    }
}

#[cfg(test)]
mod tests;
