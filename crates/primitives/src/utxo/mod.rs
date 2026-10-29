//! Tipe data OutPoint dan UtxoEntry untuk pengelolaan set UTXO.

use crate::hash::Hash32;
use crate::height::BlockHeight;
use crate::tx::{Input, Output};

/// Rujukan ke satu output transaksi tertentu berdasarkan txid dan indeks output.
///
/// Urutan field terdefinisi sebagai `txid` kemudian `index`.
/// Turunan (`derived`) trait `Ord` mengurutkan instans `OutPoint` berdasarkan `txid`
/// terlebih dahulu, kemudian berdasarkan `index`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutPoint {
    txid: Hash32,
    index: u32,
}

impl OutPoint {
    /// Membuat [`OutPoint`] baru dari `txid` dan indeks output (`index`).
    #[must_use]
    pub const fn new(txid: Hash32, index: u32) -> Self {
        Self { txid, index }
    }

    /// Membuat [`OutPoint`] dari referensi ke [`Input`].
    #[must_use]
    pub const fn from_input(input: &Input) -> Self {
        Self {
            txid: input.prev_txid(),
            index: input.prev_index(),
        }
    }

    /// Mengembalikan hash identitas transaksi (`txid`) dari output yang dirujuk.
    #[must_use]
    pub const fn txid(&self) -> Hash32 {
        self.txid
    }

    /// Mengembalikan indeks output (`index`) di dalam transaksi yang dirujuk.
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }
}

/// Output yang belum dibelanjakan (UTXO), beserta tinggi blok pembuatnya dan penanda coinbase.
///
/// Aturan konsensus seperti kematangan coinbase (100 blok) diperiksa oleh MOD-04,
/// bukan oleh tipe data ini.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UtxoEntry {
    output: Output,
    height: BlockHeight,
    is_coinbase: bool,
}

impl UtxoEntry {
    /// Membuat [`UtxoEntry`] baru.
    #[must_use]
    pub const fn new(output: Output, height: BlockHeight, is_coinbase: bool) -> Self {
        Self {
            output,
            height,
            is_coinbase,
        }
    }

    /// Mengembalikan referensi ke [`Output`].
    #[must_use]
    pub const fn output(&self) -> &Output {
        &self.output
    }

    /// Mengembalikan tinggi blok tempat output ini dibuat.
    #[must_use]
    pub const fn height(&self) -> BlockHeight {
        self.height
    }

    /// Mengembalikan apakah output ini berasal dari transaksi coinbase.
    #[must_use]
    pub const fn is_coinbase(&self) -> bool {
        self.is_coinbase
    }
}

#[cfg(test)]
mod tests;
