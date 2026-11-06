//! Encoding byte internal untuk penyimpanan ke backend (bukan bagian konsensus).

use zevrigo_primitives::block::Block;
use zevrigo_primitives::encoding::{Decode, Encode, Reader};
use zevrigo_primitives::hash::Hash32;
use zevrigo_primitives::height::BlockHeight;
use zevrigo_primitives::tx::Output;
use zevrigo_primitives::utxo::{OutPoint, UtxoEntry};

use crate::store::{BlockUndo, StoreError};

/// Key 36 byte untuk [`OutPoint`]: 32 byte txid, lalu index sebagai u32 big-endian.
/// Urutan byte key sama dengan urutan `Ord` pada [`OutPoint`].
pub(crate) fn outpoint_key(op: &OutPoint) -> [u8; 36] {
    let txid = op.txid();
    let index = op.index().to_be_bytes();
    let mut key = [0_u8; 36];
    for (dst, src) in key
        .iter_mut()
        .zip(txid.as_bytes().iter().chain(index.iter()))
    {
        *dst = *src;
    }
    key
}

/// Encoding [`UtxoEntry`]: tinggi u64 little-endian (8 byte), satu byte is_coinbase (0 atau 1),
/// lalu encoding kanonis [`Output`] dari MOD-01.
pub(crate) fn encode_utxo_entry(entry: &UtxoEntry) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&entry.height().get().to_le_bytes());
    buf.push(u8::from(entry.is_coinbase()));
    entry.output().encode_to(&mut buf);
    buf
}

/// Membaca satu [`UtxoEntry`] dari posisi saat ini pada [`Reader`], tanpa memeriksa sisa byte.
fn read_utxo_entry(reader: &mut Reader<'_>) -> Result<UtxoEntry, StoreError> {
    let height = reader.read_u64_le().map_err(|_| StoreError::CorruptData)?;
    let is_coinbase = match reader.read_u8().map_err(|_| StoreError::CorruptData)? {
        0 => false,
        1 => true,
        _ => return Err(StoreError::CorruptData),
    };
    let output = Output::decode_from(reader).map_err(|_| StoreError::CorruptData)?;
    Ok(UtxoEntry::new(
        output,
        BlockHeight::new(height),
        is_coinbase,
    ))
}

/// Kebalikan [`encode_utxo_entry`]. Data kurang, byte is_coinbase selain 0/1, atau byte sisa: `CorruptData`.
pub(crate) fn decode_utxo_entry(bytes: &[u8]) -> Result<UtxoEntry, StoreError> {
    let mut reader = Reader::new(bytes);
    let entry = read_utxo_entry(&mut reader)?;
    reader.finish().map_err(|_| StoreError::CorruptData)?;
    Ok(entry)
}

/// Encoding [`BlockUndo`]: jumlah spent u32 LE; tiap spent: 36 byte key lalu encoding entri UTXO;
/// jumlah created u32 LE; tiap created: 36 byte key. Jumlah yang tidak muat u32: `CorruptData`.
pub(crate) fn encode_block_undo(undo: &BlockUndo) -> Result<Vec<u8>, StoreError> {
    let spent_count = u32::try_from(undo.spent().len()).map_err(|_| StoreError::CorruptData)?;
    let created_count = u32::try_from(undo.created().len()).map_err(|_| StoreError::CorruptData)?;
    let mut buf = Vec::new();
    buf.extend_from_slice(&spent_count.to_le_bytes());
    for (op, entry) in undo.spent() {
        buf.extend_from_slice(&outpoint_key(op));
        buf.extend_from_slice(&encode_utxo_entry(entry));
    }
    buf.extend_from_slice(&created_count.to_le_bytes());
    for op in undo.created() {
        buf.extend_from_slice(&outpoint_key(op));
    }
    Ok(buf)
}

/// Membaca satu outpoint (32 byte txid, lalu index u32 big-endian) dari [`Reader`].
fn read_outpoint(reader: &mut Reader<'_>) -> Result<OutPoint, StoreError> {
    let txid = reader
        .read_bytes::<32>()
        .map_err(|_| StoreError::CorruptData)?;
    let index = reader
        .read_bytes::<4>()
        .map_err(|_| StoreError::CorruptData)?;
    Ok(OutPoint::new(
        Hash32::from_bytes(txid),
        u32::from_be_bytes(index),
    ))
}

/// Kebalikan [`outpoint_key`], dipakai saat membaca key tabel UTXO. Gagal: `CorruptData`.
pub(crate) fn decode_outpoint_key(key: &[u8; 36]) -> Result<OutPoint, StoreError> {
    let mut reader = Reader::new(key);
    let op = read_outpoint(&mut reader)?;
    reader.finish().map_err(|_| StoreError::CorruptData)?;
    Ok(op)
}

/// Membaca angka jumlah u32 little-endian dari [`Reader`].
fn read_count(reader: &mut Reader<'_>) -> Result<u32, StoreError> {
    let bytes = reader
        .read_bytes::<4>()
        .map_err(|_| StoreError::CorruptData)?;
    Ok(u32::from_le_bytes(bytes))
}

/// Kebalikan [`encode_block_undo`], lalu memastikan tidak ada byte sisa. Error apa pun: `CorruptData`.
/// Kapasitas vektor tidak diambil dari angka jumlah yang dibaca.
pub(crate) fn decode_block_undo(bytes: &[u8]) -> Result<BlockUndo, StoreError> {
    let mut reader = Reader::new(bytes);
    let spent_count = read_count(&mut reader)?;
    let mut spent = Vec::new();
    for _ in 0..spent_count {
        let op = read_outpoint(&mut reader)?;
        let entry = read_utxo_entry(&mut reader)?;
        spent.push((op, entry));
    }
    let created_count = read_count(&mut reader)?;
    let mut created = Vec::new();
    for _ in 0..created_count {
        created.push(read_outpoint(&mut reader)?);
    }
    reader.finish().map_err(|_| StoreError::CorruptData)?;
    Ok(BlockUndo::new(spent, created))
}

/// Decode blok dari encoding kanonis MOD-01 (`decode_from` lalu `finish`). Gagal: `CorruptData`.
pub(crate) fn decode_block(bytes: &[u8]) -> Result<Block, StoreError> {
    Block::from_bytes(bytes).map_err(|_| StoreError::CorruptData)
}

#[cfg(test)]
mod tests;
