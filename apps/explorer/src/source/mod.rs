//! Sumber data chain untuk indexer (Spesifikasi Explorer v0.1 Bagian 1 dan 3).
//!
//! Di program explorer (TSK-M30), sumbernya node lewat JSON-RPC v0.1 (`get_chain_info`, `get_block_hash`, `get_block`,
//! `get_mempool`). Test memakai chain buatan di memori.

use thiserror::Error;
use zevrigo_primitives::Hash32;

/// Galat membaca sumber data (misalnya node tidak bisa dihubungi). Bukan pelanggaran: indexer mencoba lagi nanti.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("{0}")]
pub struct SourceError(pub String);

/// Satu blok dari sumber data: encoding kanonis dan daftar txid menurut sumber itu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlock {
    /// Encoding kanonis blok (Spesifikasi Format v0.1).
    pub bytes: Vec<u8>,
    /// Txid semua transaksi menurut sumber, urut seperti di blok (diperiksa ulang, I2).
    pub txids: Vec<Hash32>,
}

/// Chain aktif sebuah node, dilihat dari luar.
pub trait ChainSource {
    /// Tinggi dan block_id ujung chain aktif.
    ///
    /// # Errors
    ///
    /// [`SourceError`] jika sumber tidak bisa dibaca.
    fn tip(&self) -> Result<(u64, Hash32), SourceError>;

    /// block_id di chain aktif pada `height`, `None` jika belum ada.
    ///
    /// # Errors
    ///
    /// [`SourceError`] jika sumber tidak bisa dibaca.
    fn block_id_at(&self, height: u64) -> Result<Option<Hash32>, SourceError>;

    /// Blok di chain aktif dengan ID `id`, `None` jika tidak ada.
    ///
    /// # Errors
    ///
    /// [`SourceError`] jika sumber tidak bisa dibaca.
    fn block(&self, id: &Hash32) -> Result<Option<SourceBlock>, SourceError>;

    /// Txid di mempool.
    ///
    /// # Errors
    ///
    /// [`SourceError`] jika sumber tidak bisa dibaca.
    fn mempool(&self) -> Result<Vec<Hash32>, SourceError>;
}
