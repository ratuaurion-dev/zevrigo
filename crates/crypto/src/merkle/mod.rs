//! Perhitungan Merkle root biner secara iteratif sesuai Bagian 6.3 spesifikasi format.

use homeostasis_primitives::Hash32;

use crate::hash::{Domain, DomainHasher, hash};

/// Menghitung Merkle root dari daftar item 32 byte (`items`).
///
/// Aturan konsensus (`docs/reference/spesifikasi-format-v0.1.md` Bagian 6.3):
/// - Jika `items` kosong, mengembalikan `None`.
/// - Setiap item dipetakan menjadi daun `hash(Domain::MerkleLeaf, item)`.
/// - Pasangan berurutan `(kiri, kanan)` digabung menjadi `hash(Domain::MerkleNode, kiri || kanan)`.
/// - Jika jumlah simpul pada suatu tingkat ganjil, simpul terakhir dinaikkan apa adanya
///   ke tingkat berikutnya (**tidak digandakan**, mencegah kerentanan sejenis CVE-2012-2459).
/// - Untuk satu item, root sama dengan daun item tersebut.
#[must_use]
pub fn merkle_root(items: &[Hash32]) -> Option<Hash32> {
    if items.is_empty() {
        return None;
    }

    let mut current: Vec<Hash32> = items
        .iter()
        .map(|item| hash(Domain::MerkleLeaf, item.as_bytes()))
        .collect();

    while current.len() > 1 {
        let mut next = Vec::with_capacity(current.len().div_ceil(2));

        for chunk in current.chunks(2) {
            match chunk {
                [left, right] => {
                    let mut node_hasher = DomainHasher::new(Domain::MerkleNode);
                    node_hasher.update(left.as_bytes());
                    node_hasher.update(right.as_bytes());
                    next.push(node_hasher.finalize());
                }
                [odd_last] => {
                    next.push(*odd_last);
                }
                _ => {}
            }
        }

        current = next;
    }

    current.into_iter().next()
}

#[cfg(test)]
mod tests;
