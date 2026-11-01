// Kunci akun publik W4 (Spesifikasi Wallet v0.1 Bagian 4, frasa W1).

use std::collections::BTreeMap;

use homeostasis_primitives::{Amount, OutputKind};
use homeostasis_rpc::{Hex32, KindNumber, Quanta, UtxoInfo};
use homeostasis_wallet_core::{CHANGE_BRANCH, ExtendedPubKey, RECEIVE_BRANCH};

use super::{GAP_LIMIT, scan_wallet};
use crate::wallet_file::NextIndex;

const W4: &str = concat!(
    "03b9385a6fbee0caa6e79407f83b0053e2d864b6c7a8f32ce61c3f495b5dd5d027",
    "12ed17c3e72223beb38ea98ec1317581f02458f02b0ecf4abe02c6490eca170a"
);

fn account() -> ExtendedPubKey {
    ExtendedPubKey::from_account_hex(W4).unwrap()
}

fn payload(branch: u32, index: u32) -> [u8; 32] {
    account().derive_path(&[branch, index]).unwrap().x_only()
}

fn utxo(payload: [u8; 32], kind: OutputKind, quanta: u128) -> UtxoInfo {
    UtxoInfo {
        txid: Hex32::new(payload),
        index: 0,
        amount: Quanta::new(Amount::from_quanta(quanta)),
        kind: KindNumber::new(kind),
        payload: Hex32::new(payload),
        height: 1,
        is_coinbase: false,
        confirmations: 1,
        spendable_at: 2,
    }
}

/// Node tiruan: UTXO per payload, mencatat payload setiap panggilan.
struct Fake {
    utxos: BTreeMap<[u8; 32], Vec<UtxoInfo>>,
    calls: Vec<Vec<[u8; 32]>>,
}

impl Fake {
    fn new(entries: &[(u32, u32, OutputKind)]) -> Self {
        let mut utxos: BTreeMap<[u8; 32], Vec<UtxoInfo>> = BTreeMap::new();
        for &(branch, index, kind) in entries {
            let key = payload(branch, index);
            utxos.entry(key).or_default().push(utxo(key, kind, 1_000));
        }
        Self {
            utxos,
            calls: Vec::new(),
        }
    }

    fn lookup(&mut self, payloads: &[Hex32]) -> Result<Vec<UtxoInfo>, String> {
        assert!(!payloads.is_empty() && payloads.len() <= 64);
        self.calls
            .push(payloads.iter().map(|p| *p.bytes()).collect());
        let mut out = Vec::new();
        for p in payloads {
            if let Some(list) = self.utxos.get(p.bytes()) {
                out.extend(list.iter().cloned());
            }
        }
        // UTXO milik orang lain di jawaban node tidak boleh terhitung.
        out.push(utxo([0xee; 32], OutputKind::SingleKey, 5));
        Ok(out)
    }
}

/// Indeks yang dipindai satu panggilan di `branch`, urut naik.
fn indices(call: &[[u8; 32]], branch: u32) -> Vec<u32> {
    let mut list: Vec<u32> = (0_u32..200)
        .filter(|&index| call.contains(&payload(branch, index)))
        .collect();
    list.sort_unstable();
    list
}

#[test]
fn empty_wallet_scans_gap_limit_on_both_branches() {
    let mut fake = Fake::new(&[]);
    let result = scan_wallet(&account(), NextIndex::default(), |p| fake.lookup(p)).unwrap();
    assert!(result.found.is_empty());
    assert_eq!(result.next_index, NextIndex::default());
    assert_eq!(fake.calls.len(), 2);
    let gap: Vec<u32> = (0..GAP_LIMIT).collect();
    assert_eq!(indices(&fake.calls[0], RECEIVE_BRANCH), gap);
    assert_eq!(indices(&fake.calls[1], CHANGE_BRANCH), gap);
}

#[test]
fn utxo_inside_gap_extends_scan() {
    // Indeks 19 ada di 20 indeks pertama, sehingga 20..40 ikut dipindai dan indeks 25 ditemukan; 26..46 juga.
    // Indeks 47 di luar gap (26 + 20 = 46), jadi tidak ditemukan.
    let mut fake = Fake::new(&[
        (RECEIVE_BRANCH, 19, OutputKind::SingleKey),
        (RECEIVE_BRANCH, 25, OutputKind::SingleKey),
        (RECEIVE_BRANCH, 47, OutputKind::SingleKey),
        (CHANGE_BRANCH, 0, OutputKind::SingleKey),
    ]);
    let result = scan_wallet(&account(), NextIndex::default(), |p| fake.lookup(p)).unwrap();
    let found: Vec<(u32, u32)> = result.found.iter().map(|f| (f.branch, f.index)).collect();
    assert_eq!(
        found,
        vec![
            (RECEIVE_BRANCH, 19),
            (RECEIVE_BRANCH, 25),
            (CHANGE_BRANCH, 0)
        ]
    );
    assert_eq!(
        result.next_index,
        NextIndex {
            receive: 26,
            change: 1
        }
    );
    // Penerimaan: 0..20, 20..40, 40..46. Kembalian: 0..20, lalu 20..21 karena indeks 0 terpakai.
    assert_eq!(fake.calls.len(), 5);
    assert_eq!(
        indices(&fake.calls[1], RECEIVE_BRANCH),
        (20_u32..40).collect::<Vec<_>>()
    );
    assert_eq!(
        indices(&fake.calls[2], RECEIVE_BRANCH),
        (40_u32..46).collect::<Vec<_>>()
    );
    assert_eq!(
        indices(&fake.calls[3], CHANGE_BRANCH),
        (0_u32..20).collect::<Vec<_>>()
    );
    assert_eq!(indices(&fake.calls[4], CHANGE_BRANCH), vec![20]);
}

#[test]
fn stored_next_index_and_batches_of_64() {
    // next_index penerimaan 100: indeks 0..120 dipindai dalam dua panggilan (64 + 56), dan tidak pernah turun.
    let mut fake = Fake::new(&[(RECEIVE_BRANCH, 3, OutputKind::SingleKey)]);
    let start = NextIndex {
        receive: 100,
        change: 5,
    };
    let result = scan_wallet(&account(), start, |p| fake.lookup(p)).unwrap();
    assert_eq!(result.next_index, start);
    assert_eq!(result.found.len(), 1);
    let sizes: Vec<usize> = fake.calls.iter().map(Vec::len).collect();
    assert_eq!(sizes, vec![64, 56, 25]);
}

#[test]
fn kind_one_utxo_with_wallet_payload_is_ignored() {
    let mut fake = Fake::new(&[(RECEIVE_BRANCH, 0, OutputKind::ConditionHash)]);
    let result = scan_wallet(&account(), NextIndex::default(), |p| fake.lookup(p)).unwrap();
    assert!(result.found.is_empty());
    assert_eq!(result.next_index, NextIndex::default());
}

#[test]
fn lookup_error_stops_scan() {
    let mut calls = 0_u32;
    let result = scan_wallet(&account(), NextIndex::default(), |_| {
        calls = calls.saturating_add(1);
        Err::<Vec<UtxoInfo>, _>("node mati")
    });
    assert_eq!(result, Err("node mati"));
    assert_eq!(calls, 1);
}
