//! Pemindaian saldo dengan gap limit 20 (Spesifikasi Wallet v0.1 Bagian 6).
//!
//! Untuk cabang penerimaan dan kembalian, alamat diturunkan dari kunci akun publik (CKDpub, tanpa kunci rahasia)
//! mulai indeks 0 sampai 20 indeks berturut-turut di atas indeks terpakai tertinggi tanpa UTXO. Indeks terpakai
//! adalah gabungan `next_index` di file wallet dan indeks yang punya UTXO. Pencarian ke node dilakukan lewat
//! `lookup`, paling banyak 64 payload sekali panggil (`scan_utxos`).
//!
//! [`wallet_payloads`] menurunkan semua payload sampai batas yang sama, untuk mencari transaksi tertunda di mempool
//! (`scan_mempool`).

use std::collections::BTreeMap;

use zevrigo_primitives::OutputKind;
use zevrigo_rpc::{Hex32, MAX_SCAN_PAYLOADS, UtxoInfo};
use zevrigo_wallet_core::{CHANGE_BRANCH, ExtendedPubKey, HARDENED, RECEIVE_BRANCH};

use crate::wallet_file::NextIndex;

/// Gap limit: jumlah indeks berturut-turut tanpa UTXO di atas indeks terpakai sebelum pemindaian berhenti.
pub const GAP_LIMIT: u32 = 20;

/// Satu UTXO milik wallet beserta jalur kuncinya.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    /// Cabang jalur kunci.
    pub branch: u32,
    /// Indeks jalur kunci.
    pub index: u32,
    /// UTXO dari node.
    pub utxo: UtxoInfo,
}

/// Hasil pemindaian: UTXO milik wallet dan indeks berikutnya yang belum terpakai per cabang.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanResult {
    /// UTXO milik wallet, cabang penerimaan dulu, lalu kembalian.
    pub found: Vec<Found>,
    /// Indeks berikutnya per cabang, tidak pernah lebih kecil dari `next_index` di file wallet.
    pub next_index: NextIndex,
}

/// Memindai kedua cabang.
///
/// # Errors
///
/// Galat pertama dari `lookup`.
pub fn scan_wallet<E>(
    account: &ExtendedPubKey,
    next_index: NextIndex,
    mut lookup: impl FnMut(&[Hex32]) -> Result<Vec<UtxoInfo>, E>,
) -> Result<ScanResult, E> {
    let mut found = Vec::new();
    let receive = scan_branch(
        account,
        RECEIVE_BRANCH,
        next_index.receive,
        &mut lookup,
        &mut found,
    )?;
    let change = scan_branch(
        account,
        CHANGE_BRANCH,
        next_index.change,
        &mut lookup,
        &mut found,
    )?;
    Ok(ScanResult {
        found,
        next_index: NextIndex { receive, change },
    })
}

/// Payload kedua cabang dari indeks 0 sampai `GAP_LIMIT` indeks di atas `next_index` (batas akhir pemindaian
/// [`scan_wallet`] bila `next_index` adalah hasilnya), cabang penerimaan dulu, lalu kembalian. Indeks yang turunannya
/// tidak sah dilewati.
#[must_use]
pub fn wallet_payloads(account: &ExtendedPubKey, next_index: NextIndex) -> Vec<Hex32> {
    let mut payloads = Vec::new();
    for (branch, used) in [
        (RECEIVE_BRANCH, next_index.receive),
        (CHANGE_BRANCH, next_index.change),
    ] {
        let Ok(branch_key) = account.derive_child(branch) else {
            continue;
        };
        let limit = used.min(HARDENED).saturating_add(GAP_LIMIT).min(HARDENED);
        for index in 0..limit {
            if let Ok(child) = branch_key.derive_child(index) {
                payloads.push(Hex32::new(child.x_only()));
            }
        }
    }
    payloads
}

/// Memindai satu cabang dan mengembalikan indeks berikutnya yang belum terpakai.
fn scan_branch<E>(
    account: &ExtendedPubKey,
    branch: u32,
    start_used: u32,
    lookup: &mut impl FnMut(&[Hex32]) -> Result<Vec<UtxoInfo>, E>,
    found: &mut Vec<Found>,
) -> Result<u32, E> {
    let mut used = start_used.min(HARDENED);
    // Kunci cabang tidak sah (peluang sekitar 2^-127): cabang itu tidak punya alamat.
    let Ok(branch_key) = account.derive_child(branch) else {
        return Ok(used);
    };
    let batch_len = u32::try_from(MAX_SCAN_PAYLOADS).unwrap_or(1);
    let mut next: u32 = 0;
    loop {
        let limit = used.saturating_add(GAP_LIMIT).min(HARDENED);
        if next >= limit {
            return Ok(used);
        }
        let end = next.saturating_add(batch_len).min(limit);
        let mut batch: BTreeMap<[u8; 32], u32> = BTreeMap::new();
        for index in next..end {
            // Indeks yang turunannya tidak sah dilewati, sesuai BIP32.
            if let Ok(child) = branch_key.derive_child(index) {
                batch.insert(child.x_only(), index);
            }
        }
        if !batch.is_empty() {
            let payloads: Vec<Hex32> = batch.keys().map(|payload| Hex32::new(*payload)).collect();
            for utxo in lookup(&payloads)? {
                if utxo.kind.kind() != OutputKind::SingleKey {
                    continue;
                }
                let Some(&index) = batch.get(utxo.payload.bytes()) else {
                    continue;
                };
                used = used.max(index.saturating_add(1));
                found.push(Found {
                    branch,
                    index,
                    utxo,
                });
            }
        }
        next = end;
    }
}

#[cfg(test)]
mod tests;
