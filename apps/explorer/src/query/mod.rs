//! Bacaan indeks untuk API explorer (Spesifikasi Explorer v0.1 Bagian 5): blok, transaksi, alamat, pasokan dibanding
//! jadwal rumus, alokasi genesis, dan mempool.
//!
//! Jumlah koin selalu [`Amount`] (`u128` quanta) dan dijumlahkan di Rust dengan `checked_add`, tidak pernah di SQL.
//! Jenis output disimpan sebagai angka `0` atau `1` (DEC-11).

use std::cmp::Reverse;
use std::collections::BTreeMap;

use homeostasis_consensus::{SupplySchedule, block_subsidy, total_supply_after};
use homeostasis_primitives::params::GENESIS_ALLOCATION;
use homeostasis_primitives::{Amount, BlockHeight, Hash32};
use rusqlite::{Connection, OptionalExtension, Params, Row, params};

use crate::db::amount_from_blob;
use crate::index::{IndexError, Indexer, indexed_block_id, indexed_tip, read_mined_total};

/// Jumlah blok terakhir untuk rata-rata waktu blok.
const AVERAGE_SPAN: u64 = 100;

const BLOCK_COLUMNS: &str = "height, block_id, prev_block_id, merkle_root, timestamp, target, nonce, size, tx_count, subsidy, fees";

const OUTPUT_COLUMNS: &str =
    "txid, idx, height, kind, payload, amount, spent_txid, spent_input, spent_height";

/// Jumlah konfirmasi blok setinggi `height` jika ujung indeks setinggi `tip`: `tip − height + 1`, `0` jika di atas ujung.
#[must_use]
pub const fn confirmations(tip: u64, height: u64) -> u64 {
    match tip.checked_sub(height) {
        Some(depth) => depth.saturating_add(1),
        None => 0,
    }
}

/// Satu blok di indeks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockRow {
    /// Tinggi blok.
    pub height: u64,
    /// block_id.
    pub block_id: Hash32,
    /// block_id induk.
    pub prev_block_id: Hash32,
    /// Merkle root header.
    pub merkle_root: Hash32,
    /// Cap waktu header (detik Unix).
    pub timestamp: u64,
    /// Target compact header.
    pub target: u32,
    /// Nonce header.
    pub nonce: u64,
    /// Ukuran encoding blok (byte).
    pub size: u64,
    /// Jumlah transaksi.
    pub tx_count: u64,
    /// Subsidy tambang blok ini (0 di genesis).
    pub subsidy: Amount,
    /// Total biaya transaksi biasa di blok ini.
    pub fees: Amount,
}

/// Blok beserta txid-nya dan block_id blok sesudahnya.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockDetail {
    /// Isi blok.
    pub block: BlockRow,
    /// Txid semua transaksi, urut seperti di blok.
    pub txids: Vec<Hash32>,
    /// block_id blok sesudahnya di indeks, `None` di ujung.
    pub next_block_id: Option<Hash32>,
}

/// Transaksi pembelanja sebuah output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpentBy {
    /// Txid pembelanja.
    pub txid: Hash32,
    /// Indeks input di transaksi pembelanja.
    pub input: u32,
    /// Tinggi blok pembelanja.
    pub height: u64,
}

/// Satu output di indeks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputRow {
    /// Txid transaksi pembuat output.
    pub txid: Hash32,
    /// Indeks output di transaksinya.
    pub index: u32,
    /// Tinggi blok pembuat output.
    pub height: u64,
    /// Jenis output: `0` kunci tunggal, `1` hash kondisi.
    pub kind: u8,
    /// Payload 32 byte.
    pub payload: [u8; 32],
    /// Jumlah koin.
    pub amount: Amount,
    /// Pembelanja, `None` jika belum dibelanjakan.
    pub spent_by: Option<SpentBy>,
}

/// Satu transaksi di indeks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxRow {
    /// Txid.
    pub txid: Hash32,
    /// Tinggi blok.
    pub height: u64,
    /// block_id blok.
    pub block_id: Hash32,
    /// Posisi di blok (coinbase = 0).
    pub position: u64,
    /// Ukuran encoding transaksi (byte).
    pub size: u64,
    /// `true` untuk coinbase.
    pub is_coinbase: bool,
    /// Biaya (0 untuk coinbase).
    pub fee: Amount,
}

/// Transaksi beserta output yang dibelanjakannya dan output yang dibuatnya.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxDetail {
    /// Isi transaksi.
    pub tx: TxRow,
    /// Output yang dibelanjakan, urut indeks input (kosong untuk coinbase).
    pub inputs: Vec<OutputRow>,
    /// Output yang dibuat, urut indeks output.
    pub outputs: Vec<OutputRow>,
}

/// Satu transaksi dalam riwayat alamat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddressTx {
    /// Txid.
    pub txid: Hash32,
    /// Tinggi blok.
    pub height: u64,
    /// Jumlah yang diterima alamat di transaksi ini.
    pub received: Amount,
    /// Jumlah yang dibelanjakan alamat di transaksi ini.
    pub sent: Amount,
}

/// Ringkasan alamat (jenis + payload) beserta satu halaman riwayat transaksinya, terbaru dulu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddressSummary {
    /// Jenis output: `0` kunci tunggal, `1` hash kondisi.
    pub kind: u8,
    /// Payload 32 byte.
    pub payload: [u8; 32],
    /// Jumlah output yang belum dibelanjakan.
    pub balance: Amount,
    /// Jumlah semua output yang pernah diterima.
    pub received: Amount,
    /// Jumlah output yang sudah dibelanjakan.
    pub sent: Amount,
    /// Banyaknya output yang belum dibelanjakan.
    pub utxo_count: u64,
    /// Banyaknya transaksi yang menyentuh alamat.
    pub tx_count: u64,
    /// Halaman riwayat, terbaru dulu.
    pub txs: Vec<AddressTx>,
}

/// Pasokan di ujung indeks dibanding jadwal rumus MOD-04.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupplyInfo {
    /// Tinggi ujung indeks.
    pub height: u64,
    /// Jumlah subsidy semua blok di indeks.
    pub mined_total: Amount,
    /// `mined_total_after(height)` menurut jadwal pasokan MOD-04.
    pub scheduled_mined_total: Amount,
    /// `mined_total == scheduled_mined_total`.
    pub matches_schedule: bool,
    /// Alokasi genesis (4.380.000 HSIS).
    pub genesis_allocation: Amount,
    /// Alokasi genesis ditambah `mined_total`.
    pub total_supply: Amount,
    /// Jumlah biaya semua blok di indeks.
    pub fees_total: Amount,
    /// Subsidy blok berikutnya.
    pub next_subsidy: Amount,
    /// Target compact blok ujung.
    pub tip_target: u32,
    /// Rata-rata waktu blok (bilangan bulat detik) atas paling banyak 100 blok terakhir; `None` jika hanya genesis.
    pub average_block_seconds: Option<u64>,
}

/// Alokasi genesis: output coinbase genesis dan statusnya.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllocationInfo {
    /// block_id genesis.
    pub genesis_block_id: Hash32,
    /// Jumlah semua output genesis.
    pub allocated: Amount,
    /// Jumlah output genesis yang belum dibelanjakan.
    pub unspent: Amount,
    /// Output genesis beserta pembelanjanya.
    pub outputs: Vec<OutputRow>,
}

fn block_row(row: &Row<'_>) -> rusqlite::Result<BlockRow> {
    Ok(BlockRow {
        height: row.get(0)?,
        block_id: Hash32::from_bytes(row.get(1)?),
        prev_block_id: Hash32::from_bytes(row.get(2)?),
        merkle_root: Hash32::from_bytes(row.get(3)?),
        timestamp: row.get(4)?,
        target: row.get(5)?,
        nonce: u64::from_be_bytes(row.get(6)?),
        size: row.get(7)?,
        tx_count: row.get(8)?,
        subsidy: amount_from_blob(row.get(9)?),
        fees: amount_from_blob(row.get(10)?),
    })
}

fn output_row(row: &Row<'_>) -> rusqlite::Result<OutputRow> {
    let spent_txid: Option<[u8; 32]> = row.get(6)?;
    let spent_input: Option<u32> = row.get(7)?;
    let spent_height: Option<u64> = row.get(8)?;
    let spent_by = match (spent_txid, spent_input, spent_height) {
        (Some(txid), Some(input), Some(height)) => Some(SpentBy {
            txid: Hash32::from_bytes(txid),
            input,
            height,
        }),
        _ => None,
    };
    Ok(OutputRow {
        txid: Hash32::from_bytes(row.get(0)?),
        index: row.get(1)?,
        height: row.get(2)?,
        kind: row.get(3)?,
        payload: row.get(4)?,
        amount: amount_from_blob(row.get(5)?),
        spent_by,
    })
}

fn txid_row(row: &Row<'_>) -> rusqlite::Result<Hash32> {
    row.get::<_, [u8; 32]>(0).map(Hash32::from_bytes)
}

fn amount_row(row: &Row<'_>) -> rusqlite::Result<Amount> {
    row.get::<_, [u8; 16]>(0).map(amount_from_blob)
}

/// Menjalankan `sql` dan mengubah setiap baris dengan `map`.
fn collect_rows<T, P: Params>(
    conn: &Connection,
    sql: &str,
    params: P,
    map: fn(&Row<'_>) -> rusqlite::Result<T>,
) -> Result<Vec<T>, IndexError> {
    let mut statement = conn.prepare(sql)?;
    let rows = statement
        .query_map(params, map)?
        .collect::<Result<Vec<T>, _>>()?;
    Ok(rows)
}

fn sum(amounts: impl IntoIterator<Item = Amount>) -> Result<Amount, IndexError> {
    Amount::checked_sum(amounts).ok_or_else(|| IndexError::Corrupt("jumlah koin meluap".to_owned()))
}

/// Satu output yang diterima alamat, untuk menyusun riwayatnya.
struct Touch {
    txid: Hash32,
    height: u64,
    position: u64,
    amount: Amount,
    spender: Option<(Hash32, u64, u64)>,
}

fn touch_row(row: &Row<'_>) -> rusqlite::Result<Touch> {
    let spent_txid: Option<[u8; 32]> = row.get(4)?;
    let spent_height: Option<u64> = row.get(5)?;
    let spent_position: Option<u64> = row.get(6)?;
    let spender = match (spent_txid, spent_height, spent_position) {
        (Some(txid), Some(height), Some(position)) => {
            Some((Hash32::from_bytes(txid), height, position))
        }
        _ => None,
    };
    Ok(Touch {
        txid: Hash32::from_bytes(row.get(0)?),
        height: row.get(1)?,
        position: row.get(2)?,
        amount: amount_from_blob(row.get(3)?),
        spender,
    })
}

/// Riwayat satu transaksi untuk alamat yang sedang disusun.
struct Pending {
    height: u64,
    position: u64,
    received: Amount,
    sent: Amount,
}

impl Indexer {
    /// Tinggi dan block_id ujung indeks, `None` sebelum genesis terindeks.
    ///
    /// # Errors
    ///
    /// [`IndexError::Sql`] untuk galat SQLite.
    pub fn tip(&self) -> Result<Option<(u64, Hash32)>, IndexError> {
        indexed_tip(&self.conn)
    }

    fn block_detail(&self, block: BlockRow) -> Result<BlockDetail, IndexError> {
        let txids = collect_rows(
            &self.conn,
            "SELECT txid FROM txs WHERE height = ?1 ORDER BY position",
            params![block.height],
            txid_row,
        )?;
        let next_block_id = match block.height.checked_add(1) {
            Some(next) => indexed_block_id(&self.conn, next)?,
            None => None,
        };
        Ok(BlockDetail {
            block,
            txids,
            next_block_id,
        })
    }

    /// Blok di tinggi `height`.
    ///
    /// # Errors
    ///
    /// [`IndexError::Sql`] untuk galat SQLite.
    pub fn block_at(&self, height: u64) -> Result<Option<BlockDetail>, IndexError> {
        let sql = format!("SELECT {BLOCK_COLUMNS} FROM blocks WHERE height = ?1");
        let block = self
            .conn
            .query_row(&sql, params![height], block_row)
            .optional()?;
        block.map(|block| self.block_detail(block)).transpose()
    }

    /// Blok dengan block_id `id`.
    ///
    /// # Errors
    ///
    /// [`IndexError::Sql`] untuk galat SQLite.
    pub fn block_by_id(&self, id: &Hash32) -> Result<Option<BlockDetail>, IndexError> {
        let sql = format!("SELECT {BLOCK_COLUMNS} FROM blocks WHERE block_id = ?1");
        let block = self
            .conn
            .query_row(&sql, params![id.as_bytes()], block_row)
            .optional()?;
        block.map(|block| self.block_detail(block)).transpose()
    }

    /// Paling banyak `limit` blok dengan tinggi `< before` (tanpa batas jika `None`), terbaru dulu.
    ///
    /// # Errors
    ///
    /// [`IndexError::Sql`] untuk galat SQLite.
    pub fn blocks_before(
        &self,
        before: Option<u64>,
        limit: usize,
    ) -> Result<Vec<BlockRow>, IndexError> {
        let bound = before.map_or(i64::MAX, |height| i64::try_from(height).unwrap_or(i64::MAX));
        let sql = format!(
            "SELECT {BLOCK_COLUMNS} FROM blocks WHERE height < ?1 ORDER BY height DESC LIMIT ?2"
        );
        collect_rows(&self.conn, &sql, params![bound, limit], block_row)
    }

    /// Transaksi dengan txid `id`.
    ///
    /// # Errors
    ///
    /// [`IndexError::Sql`] untuk galat SQLite.
    pub fn tx(&self, id: &Hash32) -> Result<Option<TxDetail>, IndexError> {
        let tx = self
            .conn
            .query_row(
                "SELECT t.txid, t.height, b.block_id, t.position, t.size, t.is_coinbase, t.fee \
                 FROM txs t JOIN blocks b ON b.height = t.height WHERE t.txid = ?1",
                params![id.as_bytes()],
                |row| {
                    Ok(TxRow {
                        txid: Hash32::from_bytes(row.get(0)?),
                        height: row.get(1)?,
                        block_id: Hash32::from_bytes(row.get(2)?),
                        position: row.get(3)?,
                        size: row.get(4)?,
                        is_coinbase: row.get(5)?,
                        fee: amount_from_blob(row.get(6)?),
                    })
                },
            )
            .optional()?;
        let Some(tx) = tx else {
            return Ok(None);
        };
        let inputs = collect_rows(
            &self.conn,
            &format!(
                "SELECT {OUTPUT_COLUMNS} FROM outputs WHERE spent_txid = ?1 ORDER BY spent_input"
            ),
            params![id.as_bytes()],
            output_row,
        )?;
        let outputs = collect_rows(
            &self.conn,
            &format!("SELECT {OUTPUT_COLUMNS} FROM outputs WHERE txid = ?1 ORDER BY idx"),
            params![id.as_bytes()],
            output_row,
        )?;
        Ok(Some(TxDetail {
            tx,
            inputs,
            outputs,
        }))
    }

    /// Ringkasan alamat `(kind, payload)` dan halaman `page` (mulai 0) riwayatnya, `page_size` transaksi per halaman.
    ///
    /// # Errors
    ///
    /// [`IndexError::Sql`] untuk galat SQLite, [`IndexError::Corrupt`] jika jumlah meluap.
    pub fn address(
        &self,
        kind: u8,
        payload: &[u8; 32],
        page: usize,
        page_size: usize,
    ) -> Result<AddressSummary, IndexError> {
        let touches = collect_rows(
            &self.conn,
            "SELECT o.txid, o.height, t.position, o.amount, o.spent_txid, o.spent_height, s.position \
             FROM outputs o JOIN txs t ON t.txid = o.txid LEFT JOIN txs s ON s.txid = o.spent_txid \
             WHERE o.kind = ?1 AND o.payload = ?2",
            params![kind, payload],
            touch_row,
        )?;
        let overflow = || IndexError::Corrupt("jumlah koin alamat meluap".to_owned());
        let mut received = Amount::ZERO;
        let mut sent = Amount::ZERO;
        let mut utxo_count = 0_u64;
        let mut by_tx: BTreeMap<Hash32, Pending> = BTreeMap::new();
        for touch in touches {
            received = received.checked_add(touch.amount).ok_or_else(overflow)?;
            let created = by_tx.entry(touch.txid).or_insert(Pending {
                height: touch.height,
                position: touch.position,
                received: Amount::ZERO,
                sent: Amount::ZERO,
            });
            created.received = created
                .received
                .checked_add(touch.amount)
                .ok_or_else(overflow)?;
            match touch.spender {
                Some((spender, height, position)) => {
                    sent = sent.checked_add(touch.amount).ok_or_else(overflow)?;
                    let spending = by_tx.entry(spender).or_insert(Pending {
                        height,
                        position,
                        received: Amount::ZERO,
                        sent: Amount::ZERO,
                    });
                    spending.sent = spending
                        .sent
                        .checked_add(touch.amount)
                        .ok_or_else(overflow)?;
                }
                None => utxo_count = utxo_count.saturating_add(1),
            }
        }
        let balance = received.checked_sub(sent).ok_or_else(overflow)?;
        let mut history: Vec<(Hash32, Pending)> = by_tx.into_iter().collect();
        history.sort_by_key(|(_, pending)| Reverse((pending.height, pending.position)));
        let tx_count = u64::try_from(history.len()).unwrap_or(u64::MAX);
        let txs = history
            .into_iter()
            .skip(page.saturating_mul(page_size))
            .take(page_size)
            .map(|(txid, pending)| AddressTx {
                txid,
                height: pending.height,
                received: pending.received,
                sent: pending.sent,
            })
            .collect();
        Ok(AddressSummary {
            kind,
            payload: *payload,
            balance,
            received,
            sent,
            utxo_count,
            tx_count,
            txs,
        })
    }

    /// Pasokan di ujung indeks dibanding jadwal rumus MOD-04, `None` sebelum genesis terindeks.
    ///
    /// # Errors
    ///
    /// [`IndexError::Sql`] untuk galat SQLite, [`IndexError::Corrupt`] jika isi indeks tidak konsisten.
    pub fn supply(&self) -> Result<Option<SupplyInfo>, IndexError> {
        let Some((height, _)) = self.tip()? else {
            return Ok(None);
        };
        let corrupt = |what: &str| IndexError::Corrupt(what.to_owned());
        let mined_total = read_mined_total(&self.conn)?;
        let scheduled_mined_total = usize::try_from(height)
            .ok()
            .and_then(|index| SupplySchedule::new().nth(index))
            .map(|entry| entry.mined_total_after)
            .ok_or_else(|| corrupt("jadwal pasokan tidak mencapai tinggi ujung"))?;
        let fees_total = sum(collect_rows(
            &self.conn,
            "SELECT fees FROM blocks",
            (),
            amount_row,
        )?)?;
        let next_height = height
            .checked_add(1)
            .ok_or_else(|| corrupt("tinggi meluap"))?;
        let next_subsidy = block_subsidy(BlockHeight::new(next_height), mined_total)
            .map_err(|error| corrupt(&error.to_string()))?;
        let total_supply =
            total_supply_after(mined_total).ok_or_else(|| corrupt("total pasokan meluap"))?;
        let tip = self
            .block_at(height)?
            .ok_or_else(|| corrupt("blok ujung tidak ada"))?;
        let span = height.min(AVERAGE_SPAN);
        let average_block_seconds = if span == 0 {
            None
        } else {
            let start = self
                .block_at(height.saturating_sub(span))?
                .ok_or_else(|| corrupt("blok awal rentang rata-rata tidak ada"))?;
            tip.block
                .timestamp
                .checked_sub(start.block.timestamp)
                .and_then(|elapsed| elapsed.checked_div(span))
        };
        Ok(Some(SupplyInfo {
            height,
            mined_total,
            scheduled_mined_total,
            matches_schedule: mined_total == scheduled_mined_total,
            genesis_allocation: GENESIS_ALLOCATION,
            total_supply,
            fees_total,
            next_subsidy,
            tip_target: tip.block.target,
            average_block_seconds,
        }))
    }

    /// Output coinbase genesis beserta statusnya (halaman alokasi genesis), `None` sebelum genesis terindeks.
    ///
    /// # Errors
    ///
    /// [`IndexError::Sql`] untuk galat SQLite, [`IndexError::Corrupt`] jika jumlah meluap.
    pub fn allocation(&self) -> Result<Option<AllocationInfo>, IndexError> {
        let Some(genesis_block_id) = indexed_block_id(&self.conn, 0)? else {
            return Ok(None);
        };
        let outputs = collect_rows(
            &self.conn,
            &format!("SELECT {OUTPUT_COLUMNS} FROM outputs WHERE height = 0 ORDER BY txid, idx"),
            (),
            output_row,
        )?;
        let allocated = sum(outputs.iter().map(|output| output.amount))?;
        let unspent = sum(outputs
            .iter()
            .filter(|output| output.spent_by.is_none())
            .map(|output| output.amount))?;
        Ok(Some(AllocationInfo {
            genesis_block_id,
            allocated,
            unspent,
            outputs,
        }))
    }

    /// Txid di mempool node menurut langkah sinkronisasi terakhir, urut naik.
    ///
    /// # Errors
    ///
    /// [`IndexError::Sql`] untuk galat SQLite.
    pub fn mempool(&self) -> Result<Vec<Hash32>, IndexError> {
        collect_rows(
            &self.conn,
            "SELECT txid FROM mempool ORDER BY txid",
            (),
            txid_row,
        )
    }

    /// `true` jika txid `id` ada di mempool menurut langkah sinkronisasi terakhir.
    ///
    /// # Errors
    ///
    /// [`IndexError::Sql`] untuk galat SQLite.
    pub fn in_mempool(&self, id: &Hash32) -> Result<bool, IndexError> {
        let found = self
            .conn
            .query_row(
                "SELECT 1 FROM mempool WHERE txid = ?1",
                params![id.as_bytes()],
                |_| Ok(()),
            )
            .optional()?;
        Ok(found.is_some())
    }
}

#[cfg(test)]
mod tests;
