//! Identitas data blockchain: `txid`, `wtxid`, `block_id`, `condition_hash`, `block_merkle_root`, dan `network_id`.
//!
//! Sesuai `docs/reference/spesifikasi-format-v0.1.md` Bagian 3, 4.1, 5.3, 6.1, 6.3, dan 7,
//! modul ini adalah satu-satunya tempat di proyek yang menghitung identitas data chain.

use homeostasis_primitives::block::{Block, Header};
use homeostasis_primitives::encoding::Encode;
use homeostasis_primitives::hash::Hash32;
use homeostasis_primitives::tx::{Condition, Transaction};

use crate::hash::{Domain, hash};
use crate::merkle::merkle_root;

/// Menghitung ID transaksi (`txid`) dari serialisasi biner badan transaksi saja (tanpa witness).
///
/// Preimage: `Transaction::encode_body_to(&tx)` di bawah domain [`Domain::Txid`].
#[must_use]
pub fn txid(tx: &Transaction) -> Hash32 {
    let mut buf = Vec::new();
    tx.encode_body_to(&mut buf);
    hash(Domain::Txid, &buf)
}

/// Menghitung ID transaksi lengkap (`wtxid`) dari serialisasi biner transaksi utuh beserta witness.
///
/// Preimage: `Transaction::encode_to(&tx)` di bawah domain [`Domain::Wtxid`].
/// Untuk coinbase, wtxid dihitung dari badan yang sama karena witness coinbase selalu kosong.
#[must_use]
pub fn wtxid(tx: &Transaction) -> Hash32 {
    let mut buf = Vec::new();
    tx.encode_to(&mut buf);
    hash(Domain::Wtxid, &buf)
}

/// Menghitung ID blok (`block_id`) dari tepat 96 byte biner header blok.
///
/// Preimage: `Header::to_array(&header)` di bawah domain [`Domain::BlockId`].
#[must_use]
pub fn block_id(header: &Header) -> Hash32 {
    hash(Domain::BlockId, &header.to_array())
}

/// Menghitung hash kondisi (`condition_hash`) dari serialisasi kanonis `Condition`.
///
/// Nilai ini menjadi `payload` pada output jenis 1 (`OutputKind::ConditionHash`).
/// Preimage: `Condition::encode_to(&condition)` di bawah domain [`Domain::Condition`].
#[must_use]
pub fn condition_hash(condition: &Condition) -> Hash32 {
    let mut buf = Vec::new();
    condition.encode_to(&mut buf);
    hash(Domain::Condition, &buf)
}

/// Menghitung Merkle root dari daftar `wtxid` semua transaksi di dalam blok sesuai urutannya.
///
/// Meneruskan daftar `wtxid` apa adanya ke [`merkle_root`] (karena `merkle_root` sudah menerapkan
/// domain `MerkleLeaf` pada setiap elemen).
///
/// Mengembalikan `None` jika blok tidak memuat transaksi.
#[must_use]
pub fn block_merkle_root(block: &Block) -> Option<Hash32> {
    let wtxids: Vec<Hash32> = block.txs().iter().map(wtxid).collect();
    merkle_root(&wtxids)
}

/// Menghitung ID jaringan (`network_id`) dari header blok genesis.
///
/// Mengembalikan `block_id(genesis_header)`. Fungsi ini hanya bermakna untuk blok genesis (`height == 0`).
#[must_use]
pub fn network_id(genesis_header: &Header) -> Hash32 {
    block_id(genesis_header)
}

#[cfg(test)]
mod tests;
