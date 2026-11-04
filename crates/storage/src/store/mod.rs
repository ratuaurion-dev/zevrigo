//! Antarmuka penyimpanan chain (`ChainStore`), tipe undo data (`BlockUndo`),
//! dan tipe error (`StoreError`). `ChainStore` juga menyimpan badan blok cabang (TSK-M35).

use homeostasis_primitives::block::Block;
use homeostasis_primitives::hash::Hash32;
use homeostasis_primitives::height::BlockHeight;
use homeostasis_primitives::utxo::{OutPoint, UtxoEntry};
use thiserror::Error;

/// Data pembatalan (undo) untuk satu blok di chain aktif.
///
/// Menyimpan entri UTXO yang dihabiskan oleh blok dan outpoint yang dibuat
/// oleh blok (yang masih ada di akhir blok).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockUndo {
    spent: Vec<(OutPoint, UtxoEntry)>,
    created: Vec<OutPoint>,
}

impl BlockUndo {
    /// Membuat objek `BlockUndo` baru.
    pub fn new(spent: Vec<(OutPoint, UtxoEntry)>, created: Vec<OutPoint>) -> Self {
        Self { spent, created }
    }

    /// Mengembalikan daftar entri UTXO yang dihabiskan oleh blok, sesuai urutan dihabiskan.
    pub fn spent(&self) -> &[(OutPoint, UtxoEntry)] {
        &self.spent
    }

    /// Mengembalikan daftar outpoint yang dibuat oleh blok dan masih ada di akhir blok, sesuai urutan dibuat.
    pub fn created(&self) -> &[OutPoint] {
        &self.created
    }
}

/// Kesalahan yang mungkin terjadi pada operasi penyimpanan chain.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum StoreError {
    /// Jumlah txid ({txids}) tidak sama dengan jumlah transaksi ({txs}).
    #[error("jumlah txid ({txids}) tidak sama dengan jumlah transaksi ({txs})")]
    TxidCountMismatch {
        /// Jumlah transaksi di dalam blok.
        txs: usize,
        /// Jumlah txid yang diberikan pemanggil.
        txids: usize,
    },

    /// Tinggi blok {actual} tidak menyambung, seharusnya {expected}.
    #[error("tinggi blok {actual} tidak menyambung, seharusnya {expected}")]
    UnexpectedHeight {
        /// Tinggi blok yang diharapkan.
        expected: u64,
        /// Tinggi blok aktual pada header blok.
        actual: u64,
    },

    /// Prev_block_id tidak sama dengan ID blok ujung chain.
    #[error("prev_block_id tidak sama dengan ID blok ujung chain")]
    PrevMismatch {
        /// ID blok yang diharapkan pada ujung chain.
        expected: Hash32,
        /// ID prev_block_id pada header blok.
        actual: Hash32,
    },

    /// UTXO yang dihabiskan tidak ditemukan.
    #[error("UTXO yang dihabiskan tidak ditemukan")]
    MissingUtxo {
        /// Outpoint yang tidak ditemukan.
        outpoint: OutPoint,
    },

    /// Outpoint baru sudah ada.
    #[error("outpoint baru sudah ada")]
    DuplicateOutpoint {
        /// Outpoint yang duplikat.
        outpoint: OutPoint,
    },

    /// Indeks output melebihi u32.
    #[error("indeks output melebihi u32")]
    OutputIndexOverflow,

    /// Tinggi blok melebihi u64.
    #[error("tinggi blok melebihi u64")]
    HeightOverflow,

    /// Chain kosong, tidak ada blok untuk di-undo.
    #[error("chain kosong, tidak ada blok untuk di-undo")]
    EmptyChain,

    /// Undo data blok ujung rusak atau tidak ada.
    #[error("undo data blok ujung rusak atau tidak ada")]
    CorruptUndo,

    /// Kesalahan dari backend redb; `message` berisi teks error aslinya.
    #[error("kesalahan backend penyimpanan: {message}")]
    Backend {
        /// Teks error dari redb.
        message: String,
    },

    /// Data penyimpanan rusak atau melebihi batas format.
    #[error("data penyimpanan rusak atau melebihi batas format")]
    CorruptData,

    /// Versi skema penyimpanan tidak didukung.
    #[error("versi skema penyimpanan {found} tidak didukung")]
    UnsupportedSchema {
        /// Versi skema yang tersimpan di tabel meta.
        found: u64,
    },
}

/// Antarmuka penyimpanan chain aktif, set UTXO, dan undo data, serta badan blok cabang menurut ID.
pub trait ChainStore {
    /// Mengembalikan tinggi dan ID blok pada ujung chain aktif, atau `None` jika chain kosong.
    fn tip(&self) -> Result<Option<(BlockHeight, Hash32)>, StoreError>;

    /// Mengembalikan ID blok pada tinggi tertentu di chain aktif, atau `None` jika belum ada.
    fn block_id_at(&self, height: BlockHeight) -> Result<Option<Hash32>, StoreError>;

    /// Mengambil blok berdasarkan ID blok dari chain aktif, atau `None` jika tidak ada.
    fn block(&self, block_id: &Hash32) -> Result<Option<Block>, StoreError>;

    /// Mengambil entri UTXO berdasarkan outpoint, atau `None` jika tidak ada di set UTXO.
    fn utxo(&self, outpoint: &OutPoint) -> Result<Option<UtxoEntry>, StoreError>;

    /// Mengembalikan semua UTXO yang `payload` output-nya ada di `payloads`, urut naik [`OutPoint`]
    /// (`txid`, lalu `index`). `payloads` kosong menghasilkan daftar kosong; payload ganda tidak menggandakan hasil.
    /// Dipakai RPC `scan_utxos` (Spesifikasi RPC 3.8, TSK-M26). Batas 1 sampai 64 payload diperiksa MOD-08.
    ///
    /// Implementasi membaca seluruh set UTXO tanpa indeks payload, jadi biayanya sebanding dengan ukuran set UTXO.
    fn utxos_by_payloads(
        &self,
        payloads: &[[u8; 32]],
    ) -> Result<Vec<(OutPoint, UtxoEntry)>, StoreError>;

    /// Mengambil undo data untuk blok tertentu di chain aktif, atau `None` jika tidak ada.
    fn undo_data(&self, block_id: &Hash32) -> Result<Option<BlockUndo>, StoreError>;

    /// Menerapkan blok baru ke ujung chain aktif secara atomik.
    fn apply_block(
        &mut self,
        block: &Block,
        block_id: Hash32,
        txids: &[Hash32],
    ) -> Result<(), StoreError>;

    /// Membatalkan blok paling ujung dari chain aktif dan mengembalikan ID serta blok yang dibatalkan.
    fn undo_tip(&mut self) -> Result<(Hash32, Block), StoreError>;

    /// Menyimpan badan blok cabang `block_id` (blok yang tidak di chain aktif, TSK-M35); ID yang sama ditimpa.
    ///
    /// Terpisah dari chain aktif: tidak mengubah ujung, set UTXO, blok chain aktif, maupun undo data. Indeks dan
    /// pemilihan cabang diurus pemanggil (MOD-11).
    fn put_side_block(&mut self, block_id: Hash32, block: &Block) -> Result<(), StoreError>;

    /// Mengambil badan blok cabang berdasarkan ID, atau `None` jika tidak ada.
    fn side_block(&self, block_id: &Hash32) -> Result<Option<Block>, StoreError>;

    /// Menghapus badan blok cabang berdasarkan ID; ID yang tidak ada bukan galat.
    fn remove_side_block(&mut self, block_id: &Hash32) -> Result<(), StoreError>;

    /// Menghapus semua badan blok cabang.
    fn clear_side_blocks(&mut self) -> Result<(), StoreError>;
}
