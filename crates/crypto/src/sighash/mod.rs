//! Pesan tanda tangan `sighash v1` (DEC-13, `docs/reference/spesifikasi-format-v0.1.md` Bagian 9).
//!
//! Satu mode saja: setiap input menandatangani seluruh transaksi (lewat `txid`), indeks inputnya sendiri,
//! ID jaringan, dan encoding kanonis semua output yang dibelanjakan transaksi itu.

use homeostasis_primitives::encoding::Encode;
use homeostasis_primitives::hash::Hash32;
use homeostasis_primitives::tx::{Output, Transaction, TxBody};

use crate::hash::{Domain, hash};
use crate::ids::txid;

/// Kesalahan saat menyusun pesan `sighash v1`.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SighashError {
    /// Coinbase tidak punya input, jadi tidak ada yang ditandatangani.
    #[error("coinbase tidak punya input untuk ditandatangani")]
    Coinbase,
    /// `input_index` tidak menunjuk input yang ada.
    #[error("indeks input {index} di luar jumlah input ({inputs})")]
    InputIndexOutOfRange {
        /// Indeks input yang diminta.
        index: u32,
        /// Jumlah input transaksi.
        inputs: usize,
    },
    /// Jumlah output yang dibelanjakan tidak sama dengan jumlah input.
    #[error("jumlah output yang dibelanjakan ({spent}) tidak sama dengan jumlah input ({inputs})")]
    SpentCountMismatch {
        /// Jumlah input transaksi.
        inputs: usize,
        /// Jumlah output yang diberikan pemanggil.
        spent: usize,
    },
}

/// Menghitung pesan 32 byte yang ditandatangani input ke-`input_index` dengan BIP340.
///
/// `msg = H(sighash v1, network_id ‖ txid ‖ input_index (u32le) ‖ encode(spent_outputs[0]) ‖ … ‖ encode(spent_outputs[n-1]))`
///
/// `spent_outputs` adalah output yang dibelanjakan setiap input, dengan urutan yang sama seperti input.
/// Untuk output jenis 1, setiap tanda tangan dari m kunci menandatangani pesan yang sama.
///
/// # Errors
///
/// [`SighashError::Coinbase`] untuk coinbase, [`SighashError::InputIndexOutOfRange`] jika indeks tidak ada,
/// dan [`SighashError::SpentCountMismatch`] jika jumlah `spent_outputs` tidak sama dengan jumlah input.
pub fn sighash_v1(
    network_id: &Hash32,
    tx: &Transaction,
    input_index: u32,
    spent_outputs: &[Output],
) -> Result<Hash32, SighashError> {
    let TxBody::Regular(body) = tx.body() else {
        return Err(SighashError::Coinbase);
    };
    let inputs = body.inputs().len();
    let in_range = usize::try_from(input_index).is_ok_and(|index| index < inputs);
    if !in_range {
        return Err(SighashError::InputIndexOutOfRange {
            index: input_index,
            inputs,
        });
    }
    if spent_outputs.len() != inputs {
        return Err(SighashError::SpentCountMismatch {
            inputs,
            spent: spent_outputs.len(),
        });
    }
    let mut preimage = Vec::new();
    preimage.extend_from_slice(network_id.as_bytes());
    preimage.extend_from_slice(txid(tx).as_bytes());
    preimage.extend_from_slice(&input_index.to_le_bytes());
    for output in spent_outputs {
        output.encode_to(&mut preimage);
    }
    Ok(hash(Domain::Sighash, &preimage))
}

#[cfg(test)]
mod tests;
