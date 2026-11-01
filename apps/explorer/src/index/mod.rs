//! Indexer explorer (Spesifikasi Explorer v0.1 Bagian 3 dan 4): sinkronisasi dari [`ChainSource`], reorg tanpa batas
//! kedalaman, dan pemeriksaan sendiri I1 sampai I4.
//!
//! Satu langkah [`Indexer::sync_step`]: cari titik cabang (turun dari ujung indeks sampai block_id sama dengan node),
//! mundur dalam satu transaksi SQLite, lalu maju blok demi blok (setiap blok satu transaksi SQLite), lalu ganti isi
//! mempool. Blok yang gagal pemeriksaan tidak ditulis dan indexer berhenti ([`Indexer::halted`]); indeks terakhir yang
//! sah tetap bisa dibaca.

use std::path::Path;

use homeostasis_consensus::block_subsidy;
use homeostasis_crypto::{block_id, txid};
use homeostasis_primitives::params::GENESIS_ALLOCATION;
use homeostasis_primitives::{Amount, Block, BlockHeight, Decode, Encode, Hash32, Output, TxBody};
use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::db::{
    META_MINED_TOTAL, META_NETWORK_ID, META_SCHEMA_VERSION, SCHEMA, SCHEMA_VERSION,
    amount_from_blob, amount_to_blob, read_meta, write_meta,
};
use crate::source::{ChainSource, SourceBlock, SourceError};

/// Galat indexer.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum IndexError {
    /// Galat SQLite.
    #[error("galat SQLite: {0}")]
    Sql(String),
    /// Node tidak bisa dibaca; dicoba lagi pada langkah berikutnya.
    #[error("node tidak bisa dibaca: {0}")]
    Source(String),
    /// File indeks milik jaringan lain.
    #[error("indeks milik jaringan {found}, seharusnya {expected}")]
    WrongNetwork {
        /// `network_id` explorer.
        expected: Hash32,
        /// `network_id` di file indeks.
        found: Hash32,
    },
    /// Versi skema file indeks tidak didukung.
    #[error("versi skema indeks {found} tidak didukung (seharusnya 1)")]
    WrongSchema {
        /// Versi di file indeks.
        found: u32,
    },
    /// Genesis node tidak sama dengan genesis jaringan explorer. Indexer berhenti.
    #[error("genesis node tidak sama dengan genesis jaringan explorer")]
    GenesisMismatch,
    /// Blok dari node gagal pemeriksaan explorer (I1 sampai I4). Indexer berhenti.
    #[error("blok tinggi {height} gagal pemeriksaan {rule}: {reason}")]
    Check {
        /// Tinggi blok.
        height: u64,
        /// Nama pemeriksaan (`I1` sampai `I4`).
        rule: &'static str,
        /// Keterangan.
        reason: String,
    },
    /// Isi indeks tidak konsisten.
    #[error("isi indeks tidak konsisten: {0}")]
    Corrupt(String),
}

impl From<rusqlite::Error> for IndexError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sql(error.to_string())
    }
}

impl From<SourceError> for IndexError {
    fn from(error: SourceError) -> Self {
        Self::Source(error.0)
    }
}

/// Galat pemeriksaan blok di tinggi `height`.
fn check(height: u64, rule: &'static str, reason: impl Into<String>) -> IndexError {
    IndexError::Check {
        height,
        rule,
        reason: reason.into(),
    }
}

/// Ringkasan satu langkah sinkronisasi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncReport {
    /// Tinggi ujung node.
    pub node_height: u64,
    /// block_id ujung node.
    pub node_tip: Hash32,
    /// Jumlah blok yang dilepas dari indeks karena reorg.
    pub rolled_back: u64,
    /// Jumlah blok baru di indeks.
    pub added: u64,
    /// `true` jika node berubah di tengah langkah (blok hilang atau tidak menyambung); langkah berikutnya melanjutkan.
    pub interrupted: bool,
}

/// Hasil menulis satu blok.
enum Applied {
    /// Blok ditulis.
    Added,
    /// Blok tidak menyambung ke ujung indeks (node reorg di tengah langkah).
    Detached,
}

/// Tinggi dan block_id ujung indeks.
pub(crate) fn indexed_tip(conn: &Connection) -> Result<Option<(u64, Hash32)>, IndexError> {
    let tip = conn
        .query_row(
            "SELECT height, block_id FROM blocks ORDER BY height DESC LIMIT 1",
            (),
            |row| Ok((row.get::<_, u64>(0)?, Hash32::from_bytes(row.get(1)?))),
        )
        .optional()?;
    Ok(tip)
}

/// block_id indeks di tinggi `height`.
pub(crate) fn indexed_block_id(
    conn: &Connection,
    height: u64,
) -> Result<Option<Hash32>, IndexError> {
    let id = conn
        .query_row(
            "SELECT block_id FROM blocks WHERE height = ?1",
            params![height],
            |row| row.get::<_, [u8; 32]>(0),
        )
        .optional()?;
    Ok(id.map(Hash32::from_bytes))
}

/// Total subsidy tambang di indeks (`meta.mined_total`).
pub(crate) fn read_mined_total(conn: &Connection) -> Result<Amount, IndexError> {
    read_meta::<16>(conn, META_MINED_TOTAL)?
        .map(amount_from_blob)
        .ok_or_else(|| IndexError::Corrupt("mined_total tidak ada".to_owned()))
}

/// Jumlah subsidy semua blok di indeks.
fn sum_subsidies(conn: &Connection) -> Result<Amount, IndexError> {
    let mut statement = conn.prepare("SELECT subsidy FROM blocks ORDER BY height")?;
    let subsidies = statement
        .query_map((), |row| row.get::<_, [u8; 16]>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Amount::checked_sum(subsidies.into_iter().map(amount_from_blob))
        .ok_or_else(|| IndexError::Corrupt("total subsidy meluap".to_owned()))
}

/// Menulis satu blok yang sudah lolos I1 dan I2, sambil memeriksa I3 dan I4. Dipanggil di dalam transaksi SQLite.
fn write_block(
    conn: &Connection,
    height: u64,
    id: Hash32,
    block: &Block,
    size: usize,
    txids: &[Hash32],
) -> Result<(), IndexError> {
    let mut fees = Amount::ZERO;
    let mut coinbase_total = Amount::ZERO;
    for (position, (tx, tx_id)) in block.txs().iter().zip(txids).enumerate() {
        let known = conn
            .query_row(
                "SELECT 1 FROM txs WHERE txid = ?1",
                params![tx_id.as_bytes()],
                |_| Ok(()),
            )
            .optional()?;
        if known.is_some() {
            return Err(check(
                height,
                "I2",
                format!("txid {tx_id} sudah ada di indeks"),
            ));
        }
        let outputs = tx.body().outputs();
        let output_total =
            Amount::checked_sum(outputs.iter().map(Output::amount)).ok_or_else(|| {
                check(
                    height,
                    "I3",
                    format!("jumlah output transaksi {tx_id} meluap"),
                )
            })?;
        let fee = match tx.body() {
            TxBody::Coinbase(_) => {
                coinbase_total = output_total;
                Amount::ZERO
            }
            TxBody::Regular(body) => {
                let mut input_total = Amount::ZERO;
                for (input_index, input) in body.inputs().iter().enumerate() {
                    let prev = input.prev_txid();
                    let prev_index = input.prev_index();
                    let amount = conn
                        .query_row(
                            "SELECT amount FROM outputs WHERE txid = ?1 AND idx = ?2 AND spent_txid IS NULL",
                            params![prev.as_bytes(), prev_index],
                            |row| row.get::<_, [u8; 16]>(0),
                        )
                        .optional()?;
                    let Some(amount) = amount else {
                        return Err(check(
                            height,
                            "I3",
                            format!(
                                "input {input_index} transaksi {tx_id} merujuk output {prev}:{prev_index} yang tidak ada atau sudah dibelanjakan"
                            ),
                        ));
                    };
                    conn.execute(
                        "UPDATE outputs SET spent_txid = ?1, spent_input = ?2, spent_height = ?3 WHERE txid = ?4 AND idx = ?5",
                        params![tx_id.as_bytes(), input_index, height, prev.as_bytes(), prev_index],
                    )?;
                    input_total = input_total
                        .checked_add(amount_from_blob(amount))
                        .ok_or_else(|| {
                            check(
                                height,
                                "I3",
                                format!("jumlah input transaksi {tx_id} meluap"),
                            )
                        })?;
                }
                let fee = input_total.checked_sub(output_total).ok_or_else(|| {
                    check(
                        height,
                        "I3",
                        format!("output transaksi {tx_id} melebihi input"),
                    )
                })?;
                fees = fees
                    .checked_add(fee)
                    .ok_or_else(|| check(height, "I4", "total biaya blok meluap"))?;
                fee
            }
        };
        let tx_size = tx.to_bytes().len();
        conn.execute(
            "INSERT INTO txs (txid, height, position, size, is_coinbase, fee) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                tx_id.as_bytes(),
                height,
                position,
                tx_size,
                tx.is_coinbase(),
                amount_to_blob(fee)
            ],
        )?;
        for (index, output) in outputs.iter().enumerate() {
            conn.execute(
                "INSERT INTO outputs (txid, idx, height, kind, payload, amount) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    tx_id.as_bytes(),
                    index,
                    height,
                    output.kind().to_u8(),
                    output.payload(),
                    amount_to_blob(output.amount())
                ],
            )?;
        }
    }

    let mined_before = read_mined_total(conn)?;
    let (subsidy, expected) = if height == 0 {
        (Amount::ZERO, GENESIS_ALLOCATION)
    } else {
        let subsidy = block_subsidy(BlockHeight::new(height), mined_before)
            .map_err(|error| check(height, "I4", error.to_string()))?;
        let expected = subsidy
            .checked_add(fees)
            .ok_or_else(|| check(height, "I4", "subsidy ditambah biaya meluap"))?;
        (subsidy, expected)
    };
    if coinbase_total != expected {
        return Err(check(
            height,
            "I4",
            format!(
                "coinbase {} quanta, seharusnya {} quanta",
                coinbase_total.as_quanta(),
                expected.as_quanta()
            ),
        ));
    }
    let mined_after = mined_before
        .checked_add(subsidy)
        .ok_or_else(|| check(height, "I4", "total tambang meluap"))?;

    let header = block.header();
    let prev = header.prev_block_id();
    let merkle = header.merkle_root();
    let tx_count = block.txs().len();
    conn.execute(
        "INSERT INTO blocks (height, block_id, prev_block_id, merkle_root, timestamp, target, nonce, size, tx_count, subsidy, fees) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            height,
            id.as_bytes(),
            prev.as_bytes(),
            merkle.as_bytes(),
            header.timestamp(),
            header.target(),
            header.nonce().to_be_bytes(),
            size,
            tx_count,
            amount_to_blob(subsidy),
            amount_to_blob(fees)
        ],
    )?;
    write_meta(conn, META_MINED_TOTAL, &amount_to_blob(mined_after))?;
    Ok(())
}

/// Indeks explorer di atas satu koneksi SQLite.
pub struct Indexer {
    pub(crate) conn: Connection,
    network_id: Hash32,
    halted: Option<IndexError>,
}

impl Indexer {
    /// Membuka atau membuat file indeks di `path` untuk jaringan `network_id`. Folder induk wajib sudah ada.
    ///
    /// # Errors
    ///
    /// [`IndexError::WrongNetwork`] atau [`IndexError::WrongSchema`] untuk file milik jaringan atau versi lain,
    /// [`IndexError::Sql`] untuk galat SQLite.
    pub fn open(path: &Path, network_id: Hash32) -> Result<Self, IndexError> {
        Self::init(Connection::open(path)?, network_id)
    }

    /// Indeks di memori (untuk test).
    ///
    /// # Errors
    ///
    /// [`IndexError::Sql`] untuk galat SQLite.
    pub fn open_in_memory(network_id: Hash32) -> Result<Self, IndexError> {
        Self::init(Connection::open_in_memory()?, network_id)
    }

    fn init(conn: Connection, network_id: Hash32) -> Result<Self, IndexError> {
        conn.execute_batch(SCHEMA)?;
        match read_meta::<4>(&conn, META_SCHEMA_VERSION)? {
            None => {
                write_meta(&conn, META_SCHEMA_VERSION, &SCHEMA_VERSION.to_le_bytes())?;
                write_meta(&conn, META_NETWORK_ID, network_id.as_bytes())?;
                write_meta(&conn, META_MINED_TOTAL, &amount_to_blob(Amount::ZERO))?;
            }
            Some(version) => {
                let found = u32::from_le_bytes(version);
                if found != SCHEMA_VERSION {
                    return Err(IndexError::WrongSchema { found });
                }
                let stored = read_meta::<32>(&conn, META_NETWORK_ID)?
                    .ok_or_else(|| IndexError::Corrupt("network_id tidak ada".to_owned()))?;
                let found = Hash32::from_bytes(stored);
                if found != network_id {
                    return Err(IndexError::WrongNetwork {
                        expected: network_id,
                        found,
                    });
                }
            }
        }
        Ok(Self {
            conn,
            network_id,
            halted: None,
        })
    }

    /// `network_id` jaringan indeks.
    #[must_use]
    pub const fn network_id(&self) -> Hash32 {
        self.network_id
    }

    /// Galat yang menghentikan indexer (blok gagal I1 sampai I4, atau genesis berbeda), jika ada.
    #[must_use]
    pub const fn halted(&self) -> Option<&IndexError> {
        self.halted.as_ref()
    }

    /// Satu langkah sinkronisasi dengan `source` (Spesifikasi Explorer Bagian 3).
    ///
    /// # Errors
    ///
    /// [`IndexError::Source`] jika node tidak bisa dibaca (indexer tidak berhenti), [`IndexError::Check`] atau
    /// [`IndexError::GenesisMismatch`] (indexer berhenti; langkah berikutnya mengembalikan galat yang sama), atau
    /// [`IndexError::Sql`].
    pub fn sync_step(&mut self, source: &impl ChainSource) -> Result<SyncReport, IndexError> {
        if let Some(error) = &self.halted {
            return Err(error.clone());
        }
        let result = self.sync_inner(source);
        if let Err(error @ (IndexError::Check { .. } | IndexError::GenesisMismatch)) = &result {
            self.halted = Some(error.clone());
        }
        result
    }

    fn sync_inner(&mut self, source: &impl ChainSource) -> Result<SyncReport, IndexError> {
        let (node_height, node_tip) = source.tip()?;
        let mut rolled_back = 0_u64;
        let mut next = 0_u64;
        if let Some((tip_height, _)) = indexed_tip(&self.conn)? {
            let fork = self.find_fork(source, tip_height.min(node_height))?;
            if fork < tip_height {
                self.rollback(fork)?;
                rolled_back = tip_height.saturating_sub(fork);
            }
            next = fork
                .checked_add(1)
                .ok_or_else(|| IndexError::Corrupt("tinggi indeks meluap".to_owned()))?;
        }
        let mut added = 0_u64;
        let mut interrupted = false;
        let mut height = next;
        while height <= node_height {
            let Some(id) = source.block_id_at(height)? else {
                interrupted = true;
                break;
            };
            let Some(block) = source.block(&id)? else {
                interrupted = true;
                break;
            };
            match self.apply(height, id, &block)? {
                Applied::Added => added = added.saturating_add(1),
                Applied::Detached => {
                    interrupted = true;
                    break;
                }
            }
            let Some(following) = height.checked_add(1) else {
                break;
            };
            height = following;
        }
        let mempool = source.mempool()?;
        self.replace_mempool(&mempool)?;
        Ok(SyncReport {
            node_height,
            node_tip,
            rolled_back,
            added,
            interrupted,
        })
    }

    /// Tinggi tertinggi `<= start` yang block_id-nya sama di indeks dan di node. Tanpa batas kedalaman.
    fn find_fork(&self, source: &impl ChainSource, start: u64) -> Result<u64, IndexError> {
        let mut height = start;
        loop {
            let node_id = source.block_id_at(height)?;
            if node_id.is_some() && node_id == indexed_block_id(&self.conn, height)? {
                return Ok(height);
            }
            if height == 0 {
                return Err(IndexError::GenesisMismatch);
            }
            height = height.saturating_sub(1);
        }
    }

    /// Melepas semua blok di atas `fork` dalam satu transaksi SQLite.
    fn rollback(&mut self, fork: u64) -> Result<(), IndexError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE outputs SET spent_txid = NULL, spent_input = NULL, spent_height = NULL WHERE spent_height > ?1",
            params![fork],
        )?;
        tx.execute("DELETE FROM outputs WHERE height > ?1", params![fork])?;
        tx.execute("DELETE FROM txs WHERE height > ?1", params![fork])?;
        tx.execute("DELETE FROM blocks WHERE height > ?1", params![fork])?;
        let mined = sum_subsidies(&tx)?;
        write_meta(&tx, META_MINED_TOTAL, &amount_to_blob(mined))?;
        tx.commit()?;
        Ok(())
    }

    /// Memeriksa I1 dan I2, lalu menulis blok setinggi `height` dalam satu transaksi SQLite (I3 dan I4 di dalamnya).
    fn apply(
        &mut self,
        height: u64,
        id: Hash32,
        source_block: &SourceBlock,
    ) -> Result<Applied, IndexError> {
        let block = Block::from_bytes(&source_block.bytes)
            .map_err(|error| check(height, "I1", format!("encoding blok tidak sah: {error}")))?;
        let header = *block.header();
        if block_id(&header) != id {
            return Err(check(
                height,
                "I1",
                "block_id dari node tidak sama dengan hash header",
            ));
        }
        if header.height().get() != height {
            return Err(check(
                height,
                "I1",
                format!(
                    "tinggi header {} tidak sama dengan tinggi chain",
                    header.height()
                ),
            ));
        }
        if height == 0 {
            if id != self.network_id {
                return Err(IndexError::GenesisMismatch);
            }
        } else if indexed_tip(&self.conn)?.map(|(_, tip_id)| tip_id) != Some(header.prev_block_id())
        {
            return Ok(Applied::Detached);
        }
        let txids: Vec<Hash32> = block.txs().iter().map(txid).collect();
        if txids != source_block.txids {
            return Err(check(
                height,
                "I2",
                "txid dari node tidak sama dengan hasil hitung ulang",
            ));
        }
        let tx = self.conn.transaction()?;
        write_block(&tx, height, id, &block, source_block.bytes.len(), &txids)?;
        tx.commit()?;
        Ok(Applied::Added)
    }

    /// Mengganti isi tabel `mempool`.
    fn replace_mempool(&mut self, txids: &[Hash32]) -> Result<(), IndexError> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM mempool", ())?;
        for id in txids {
            tx.execute(
                "INSERT OR IGNORE INTO mempool (txid) VALUES (?1)",
                params![id.as_bytes()],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
