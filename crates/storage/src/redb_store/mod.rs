//! Implementasi penyimpanan persisten berbasis redb (`RedbStore`, DEC-03).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;
use std::path::Path;

use redb::backends::InMemoryBackend;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition, WriteTransaction};
use zevrigo_primitives::block::Block;
use zevrigo_primitives::encoding::Encode;
use zevrigo_primitives::hash::Hash32;
use zevrigo_primitives::height::BlockHeight;
use zevrigo_primitives::tx::TxBody;
use zevrigo_primitives::utxo::{OutPoint, UtxoEntry};

use crate::codec::{
    decode_block, decode_block_undo, decode_outpoint_key, decode_utxo_entry, encode_block_undo,
    encode_utxo_entry, outpoint_key,
};
use crate::store::{BlockUndo, ChainStore, StoreError};

/// Tabel meta: versi skema penyimpanan.
const META: TableDefinition<&str, u64> = TableDefinition::new("meta");
/// Tabel set UTXO: key `outpoint_key`, value `encode_utxo_entry`.
const UTXOS: TableDefinition<&[u8; 36], &[u8]> = TableDefinition::new("utxos");
/// Tabel badan blok chain aktif: key block_id, value encoding kanonis blok MOD-01.
const BLOCKS: TableDefinition<&[u8; 32], &[u8]> = TableDefinition::new("blocks");
/// Tabel chain aktif: key tinggi, value block_id.
const ACTIVE: TableDefinition<u64, &[u8; 32]> = TableDefinition::new("active");
/// Tabel undo data: key block_id, value `encode_block_undo`.
const UNDO: TableDefinition<&[u8; 32], &[u8]> = TableDefinition::new("undo");
/// Tabel badan blok cabang (TSK-M35): key block_id, value encoding kanonis blok MOD-01. Ditambahkan tanpa menaikkan
/// versi skema: file lama tanpa tabel ini tetap terbuka, tabelnya dibuat saat dibuka.
const SIDE: TableDefinition<&[u8; 32], &[u8]> = TableDefinition::new("side_blocks");
/// Key versi skema di tabel meta.
const SCHEMA_KEY: &str = "schema";
/// Versi skema yang ditulis dan diterima oleh kode ini.
const SCHEMA_VERSION: u64 = 1;

/// Mengubah error redb apa pun menjadi [`StoreError::Backend`].
fn redb_error(e: impl Display) -> StoreError {
    StoreError::Backend {
        message: e.to_string(),
    }
}

/// Penyimpanan chain aktif, set UTXO, undo data, dan badan blok cabang di database redb.
///
/// Setiap `apply_block`, `undo_tip`, dan operasi badan blok cabang adalah satu write transaction redb. Error di mana pun sebelum `commit`
/// membatalkan transaksi itu, sehingga isi store tidak berubah.
pub struct RedbStore {
    db: Database,
}

impl RedbStore {
    /// Membuat database di `path` (atau membuka jika sudah ada), lalu memeriksa versi skema.
    ///
    /// # Errors
    ///
    /// [`StoreError::Backend`] jika redb gagal, atau [`StoreError::UnsupportedSchema`] jika versi skema
    /// yang tersimpan bukan versi yang didukung.
    pub fn create(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let db = Database::create(path).map_err(redb_error)?;
        Self::init(db)
    }

    /// Membuka database yang sudah ada di `path`, lalu memeriksa versi skema.
    ///
    /// # Errors
    ///
    /// [`StoreError::Backend`] jika redb gagal, atau [`StoreError::UnsupportedSchema`] jika versi skema
    /// yang tersimpan bukan versi yang didukung.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let db = Database::open(path).map_err(redb_error)?;
        Self::init(db)
    }

    /// Membuat database kosong di memori (`InMemoryBackend`), untuk test dan regtest.
    ///
    /// # Errors
    ///
    /// [`StoreError::Backend`] jika redb gagal.
    pub fn in_memory() -> Result<Self, StoreError> {
        let db = Database::builder()
            .create_with_backend(InMemoryBackend::new())
            .map_err(redb_error)?;
        Self::init(db)
    }

    /// Write transaction inisialisasi: membuka keenam tabel, lalu menulis atau memeriksa versi skema.
    /// Jika versi tidak didukung, kembali dengan `UnsupportedSchema` tanpa commit.
    fn init(db: Database) -> Result<Self, StoreError> {
        let write_txn = db.begin_write().map_err(redb_error)?;
        {
            let mut meta = write_txn.open_table(META).map_err(redb_error)?;
            let _ = write_txn.open_table(UTXOS).map_err(redb_error)?;
            let _ = write_txn.open_table(BLOCKS).map_err(redb_error)?;
            let _ = write_txn.open_table(ACTIVE).map_err(redb_error)?;
            let _ = write_txn.open_table(UNDO).map_err(redb_error)?;
            let _ = write_txn.open_table(SIDE).map_err(redb_error)?;
            let stored = meta
                .get(SCHEMA_KEY)
                .map_err(redb_error)?
                .map(|guard| guard.value());
            match stored {
                None => {
                    meta.insert(SCHEMA_KEY, SCHEMA_VERSION)
                        .map_err(redb_error)?;
                }
                Some(found) if found != SCHEMA_VERSION => {
                    return Err(StoreError::UnsupportedSchema { found });
                }
                Some(_) => {}
            }
        }
        write_txn.commit().map_err(redb_error)?;
        Ok(Self { db })
    }
}

/// Langkah 2 `apply_block`: store kosong menuntut tinggi 0 (prev tidak dicek); selain itu tinggi wajib
/// tinggi ujung + 1 dan `prev_block_id` wajib sama dengan block_id ujung. Mengembalikan tinggi header.
fn check_connection(write_txn: &WriteTransaction, block: &Block) -> Result<u64, StoreError> {
    let actual = block.header().height().get();
    let active = write_txn.open_table(ACTIVE).map_err(redb_error)?;
    let Some((height_guard, id_guard)) = active.last().map_err(redb_error)? else {
        if actual != 0 {
            return Err(StoreError::UnexpectedHeight {
                expected: 0,
                actual,
            });
        }
        return Ok(actual);
    };
    let expected = height_guard
        .value()
        .checked_add(1)
        .ok_or(StoreError::HeightOverflow)?;
    if actual != expected {
        return Err(StoreError::UnexpectedHeight { expected, actual });
    }
    let tip_id = Hash32::from_bytes(*id_guard.value());
    let prev = block.header().prev_block_id();
    if prev != tip_id {
        return Err(StoreError::PrevMismatch {
            expected: tip_id,
            actual: prev,
        });
    }
    Ok(actual)
}

/// Bagian akhir langkah 4 `apply_block`: menulis BLOCKS, ACTIVE, dan UNDO.
fn write_block_records(
    write_txn: &WriteTransaction,
    block: &Block,
    block_id: Hash32,
    height: u64,
    undo: &BlockUndo,
) -> Result<(), StoreError> {
    let undo_bytes = encode_block_undo(undo)?;
    let block_bytes = block.to_bytes();
    {
        let mut blocks = write_txn.open_table(BLOCKS).map_err(redb_error)?;
        blocks
            .insert(block_id.as_bytes(), block_bytes.as_slice())
            .map_err(redb_error)?;
    }
    {
        let mut active = write_txn.open_table(ACTIVE).map_err(redb_error)?;
        active
            .insert(height, block_id.as_bytes())
            .map_err(redb_error)?;
    }
    {
        let mut undo_table = write_txn.open_table(UNDO).map_err(redb_error)?;
        undo_table
            .insert(block_id.as_bytes(), undo_bytes.as_slice())
            .map_err(redb_error)?;
    }
    Ok(())
}

impl ChainStore for RedbStore {
    fn tip(&self) -> Result<Option<(BlockHeight, Hash32)>, StoreError> {
        let read_txn = self.db.begin_read().map_err(redb_error)?;
        let table = read_txn.open_table(ACTIVE).map_err(redb_error)?;
        let Some((height_guard, id_guard)) = table.last().map_err(redb_error)? else {
            return Ok(None);
        };
        Ok(Some((
            BlockHeight::new(height_guard.value()),
            Hash32::from_bytes(*id_guard.value()),
        )))
    }

    fn block_id_at(&self, height: BlockHeight) -> Result<Option<Hash32>, StoreError> {
        let read_txn = self.db.begin_read().map_err(redb_error)?;
        let table = read_txn.open_table(ACTIVE).map_err(redb_error)?;
        let Some(id_guard) = table.get(height.get()).map_err(redb_error)? else {
            return Ok(None);
        };
        Ok(Some(Hash32::from_bytes(*id_guard.value())))
    }

    fn block(&self, block_id: &Hash32) -> Result<Option<Block>, StoreError> {
        let read_txn = self.db.begin_read().map_err(redb_error)?;
        let table = read_txn.open_table(BLOCKS).map_err(redb_error)?;
        let Some(guard) = table.get(block_id.as_bytes()).map_err(redb_error)? else {
            return Ok(None);
        };
        decode_block(guard.value()).map(Some)
    }

    fn utxo(&self, outpoint: &OutPoint) -> Result<Option<UtxoEntry>, StoreError> {
        let read_txn = self.db.begin_read().map_err(redb_error)?;
        let table = read_txn.open_table(UTXOS).map_err(redb_error)?;
        let Some(guard) = table.get(&outpoint_key(outpoint)).map_err(redb_error)? else {
            return Ok(None);
        };
        decode_utxo_entry(guard.value()).map(Some)
    }

    fn utxos_by_payloads(
        &self,
        payloads: &[[u8; 32]],
    ) -> Result<Vec<(OutPoint, UtxoEntry)>, StoreError> {
        let wanted: BTreeSet<[u8; 32]> = payloads.iter().copied().collect();
        let mut found = Vec::new();
        if wanted.is_empty() {
            return Ok(found);
        }
        let read_txn = self.db.begin_read().map_err(redb_error)?;
        let table = read_txn.open_table(UTXOS).map_err(redb_error)?;
        for item in table.iter().map_err(redb_error)? {
            let (key_guard, value_guard) = item.map_err(redb_error)?;
            let entry = decode_utxo_entry(value_guard.value())?;
            if wanted.contains(entry.output().payload()) {
                found.push((decode_outpoint_key(key_guard.value())?, entry));
            }
        }
        // Urutan key sudah sama dengan urutan OutPoint (lihat `outpoint_key`); pengurutan ini menjaga kontrak
        // trait tanpa bergantung pada urutan internal backend.
        found.sort_unstable_by_key(|(op, _)| *op);
        Ok(found)
    }

    fn undo_data(&self, block_id: &Hash32) -> Result<Option<BlockUndo>, StoreError> {
        let read_txn = self.db.begin_read().map_err(redb_error)?;
        let table = read_txn.open_table(UNDO).map_err(redb_error)?;
        let Some(guard) = table.get(block_id.as_bytes()).map_err(redb_error)? else {
            return Ok(None);
        };
        decode_block_undo(guard.value()).map(Some)
    }

    fn apply_block(
        &mut self,
        block: &Block,
        block_id: Hash32,
        txids: &[Hash32],
    ) -> Result<(), StoreError> {
        // langkah 1: jumlah txid wajib sama dengan jumlah transaksi
        let txs = block.txs().len();
        if txids.len() != txs {
            return Err(StoreError::TxidCountMismatch {
                txs,
                txids: txids.len(),
            });
        }
        let write_txn = self.db.begin_write().map_err(redb_error)?;

        // langkah 2: tinggi dan prev_block_id wajib menyambung ke ujung chain
        let height = check_connection(&write_txn, block)?;

        // langkah 3: overlay lokal; UTXO lama dibaca dari tabel UTXOS di write transaction yang sama
        let mut utxos = write_txn.open_table(UTXOS).map_err(redb_error)?;
        let mut created: BTreeMap<OutPoint, UtxoEntry> = BTreeMap::new();
        let mut created_order: Vec<OutPoint> = Vec::new();
        let mut created_all: BTreeSet<OutPoint> = BTreeSet::new();
        let mut spent: Vec<(OutPoint, UtxoEntry)> = Vec::new();
        let mut spent_set: BTreeSet<OutPoint> = BTreeSet::new();
        for (tx, txid) in block.txs().iter().zip(txids.iter()) {
            // langkah 3a: input transaksi biasa
            if let TxBody::Regular(body) = tx.body() {
                for input in body.inputs() {
                    let op = OutPoint::from_input(input);
                    if created.remove(&op).is_some() {
                        created_order.retain(|x| x != &op);
                    } else if spent_set.contains(&op) {
                        return Err(StoreError::MissingUtxo { outpoint: op });
                    } else {
                        let Some(guard) = utxos.get(&outpoint_key(&op)).map_err(redb_error)? else {
                            return Err(StoreError::MissingUtxo { outpoint: op });
                        };
                        let entry = decode_utxo_entry(guard.value())?;
                        spent.push((op, entry));
                        spent_set.insert(op);
                    }
                }
            }
            // langkah 3b: output
            for (j, output) in tx.body().outputs().iter().enumerate() {
                let index = u32::try_from(j).map_err(|_| StoreError::OutputIndexOverflow)?;
                let op = OutPoint::new(*txid, index);
                if created_all.contains(&op)
                    || utxos.get(&outpoint_key(&op)).map_err(redb_error)?.is_some()
                {
                    return Err(StoreError::DuplicateOutpoint { outpoint: op });
                }
                created.insert(
                    op,
                    UtxoEntry::new(output.clone(), block.header().height(), tx.is_coinbase()),
                );
                created_order.push(op);
                created_all.insert(op);
            }
        }

        // langkah 4: terapkan; error setelah titik ini membatalkan seluruh write transaction
        for (op, _) in &spent {
            utxos.remove(&outpoint_key(op)).map_err(redb_error)?;
        }
        for (op, entry) in &created {
            let value = encode_utxo_entry(entry);
            utxos
                .insert(&outpoint_key(op), value.as_slice())
                .map_err(redb_error)?;
        }
        drop(utxos);
        let undo = BlockUndo::new(spent, created_order);
        write_block_records(&write_txn, block, block_id, height, &undo)?;
        write_txn.commit().map_err(redb_error)?;
        Ok(())
    }

    fn undo_tip(&mut self) -> Result<(Hash32, Block), StoreError> {
        let write_txn = self.db.begin_write().map_err(redb_error)?;

        // langkah 1: chain kosong tidak bisa di-undo
        let (tip_height, tip_id) = {
            let active = write_txn.open_table(ACTIVE).map_err(redb_error)?;
            let Some((height_guard, id_guard)) = active.last().map_err(redb_error)? else {
                return Err(StoreError::EmptyChain);
            };
            (height_guard.value(), Hash32::from_bytes(*id_guard.value()))
        };

        // langkah 2: undo data dan blok ujung wajib ada
        let undo = {
            let table = write_txn.open_table(UNDO).map_err(redb_error)?;
            let guard = table
                .get(tip_id.as_bytes())
                .map_err(redb_error)?
                .ok_or(StoreError::CorruptUndo)?;
            decode_block_undo(guard.value())?
        };
        let block = {
            let table = write_txn.open_table(BLOCKS).map_err(redb_error)?;
            let guard = table
                .get(tip_id.as_bytes())
                .map_err(redb_error)?
                .ok_or(StoreError::CorruptUndo)?;
            decode_block(guard.value())?
        };

        // langkah 3: semua created wajib ada dan semua spent wajib tidak ada; jika tidak, CorruptUndo
        let mut utxos = write_txn.open_table(UTXOS).map_err(redb_error)?;
        for op in undo.created() {
            if utxos.get(&outpoint_key(op)).map_err(redb_error)?.is_none() {
                return Err(StoreError::CorruptUndo);
            }
        }
        for (op, _) in undo.spent() {
            if utxos.get(&outpoint_key(op)).map_err(redb_error)?.is_some() {
                return Err(StoreError::CorruptUndo);
            }
        }

        // langkah 4: hapus created, kembalikan spent, hapus ACTIVE[tinggi], UNDO[id], BLOCKS[id], lalu commit
        for op in undo.created() {
            utxos.remove(&outpoint_key(op)).map_err(redb_error)?;
        }
        for (op, entry) in undo.spent() {
            let value = encode_utxo_entry(entry);
            utxos
                .insert(&outpoint_key(op), value.as_slice())
                .map_err(redb_error)?;
        }
        drop(utxos);
        {
            let mut active = write_txn.open_table(ACTIVE).map_err(redb_error)?;
            active.remove(tip_height).map_err(redb_error)?;
        }
        {
            let mut undo_table = write_txn.open_table(UNDO).map_err(redb_error)?;
            undo_table.remove(tip_id.as_bytes()).map_err(redb_error)?;
        }
        {
            let mut blocks = write_txn.open_table(BLOCKS).map_err(redb_error)?;
            blocks.remove(tip_id.as_bytes()).map_err(redb_error)?;
        }
        write_txn.commit().map_err(redb_error)?;
        Ok((tip_id, block))
    }

    fn put_side_block(&mut self, block_id: Hash32, block: &Block) -> Result<(), StoreError> {
        let bytes = block.to_bytes();
        let write_txn = self.db.begin_write().map_err(redb_error)?;
        {
            let mut table = write_txn.open_table(SIDE).map_err(redb_error)?;
            table
                .insert(block_id.as_bytes(), bytes.as_slice())
                .map_err(redb_error)?;
        }
        write_txn.commit().map_err(redb_error)?;
        Ok(())
    }

    fn side_block(&self, block_id: &Hash32) -> Result<Option<Block>, StoreError> {
        let read_txn = self.db.begin_read().map_err(redb_error)?;
        let table = read_txn.open_table(SIDE).map_err(redb_error)?;
        let Some(guard) = table.get(block_id.as_bytes()).map_err(redb_error)? else {
            return Ok(None);
        };
        decode_block(guard.value()).map(Some)
    }

    fn remove_side_block(&mut self, block_id: &Hash32) -> Result<(), StoreError> {
        let write_txn = self.db.begin_write().map_err(redb_error)?;
        {
            let mut table = write_txn.open_table(SIDE).map_err(redb_error)?;
            table.remove(block_id.as_bytes()).map_err(redb_error)?;
        }
        write_txn.commit().map_err(redb_error)?;
        Ok(())
    }

    fn clear_side_blocks(&mut self) -> Result<(), StoreError> {
        let write_txn = self.db.begin_write().map_err(redb_error)?;
        {
            let mut table = write_txn.open_table(SIDE).map_err(redb_error)?;
            let mut keys: Vec<[u8; 32]> = Vec::new();
            for item in table.iter().map_err(redb_error)? {
                let (key_guard, _) = item.map_err(redb_error)?;
                keys.push(*key_guard.value());
            }
            for key in &keys {
                table.remove(key).map_err(redb_error)?;
            }
        }
        write_txn.commit().map_err(redb_error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
