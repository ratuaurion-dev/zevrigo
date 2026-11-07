// Uji riwayat dengan blok buatan dan kunci akun publik W4 (Spesifikasi Wallet v0.1 Bagian 4, frasa W1). Txid diberikan
// langsung (node yang menghitungnya); blok tidak divalidasi konsensus.

use std::path::{Path, PathBuf};

use zevrigo_primitives::{
    Amount, Block, BlockHeight, Hash32, Header, Input, Network, Output, OutputKind, Transaction,
    Witness,
};
use zevrigo_rpc::{Hex32, KindNumber, MempoolOutput, MempoolSpend, Quanta};
use zevrigo_wallet_core::{CHANGE_BRANCH, ExtendedPubKey, RECEIVE_BRANCH};

use super::{
    HistoryEntry, HistoryError, HistoryState, HistoryTip, WalletKeys, history_path, identity, load,
    save,
};
use crate::wallet_file::NextIndex;

const ZVRG: u128 = 1_000_000_000_000_000_000;
const W4: &str = concat!(
    "03b9385a6fbee0caa6e79407f83b0053e2d864b6c7a8f32ce61c3f495b5dd5d027",
    "12ed17c3e72223beb38ea98ec1317581f02458f02b0ecf4abe02c6490eca170a"
);
/// Cap waktu blok tinggi 0; blok tinggi `h` bercap waktu `T0 + 120 h`.
const T0: u64 = 1_800_000_000;

fn account() -> ExtendedPubKey {
    ExtendedPubKey::from_account_hex(W4).unwrap()
}

fn payload(branch: u32, index: u32) -> [u8; 32] {
    account().derive_path(&[branch, index]).unwrap().x_only()
}

/// Payload yang bukan milik wallet.
const OTHER: [u8; 32] = [0x77; 32];

fn zvrg(tenths: u128) -> Amount {
    Amount::from_quanta(tenths * ZVRG / 10)
}

fn out(payload: [u8; 32], amount: Amount) -> Output {
    Output::new(amount, OutputKind::SingleKey, payload).unwrap()
}

fn txid(n: u8) -> Hex32 {
    Hex32::new([n; 32])
}

fn coinbase(height: u64, outputs: Vec<Output>) -> Transaction {
    Transaction::coinbase(BlockHeight::new(height), Vec::new(), outputs).unwrap()
}

fn spend(inputs: &[(Hex32, u32)], outputs: Vec<Output>) -> Transaction {
    Transaction::regular(
        inputs
            .iter()
            .map(|(id, index)| Input::new(id.to_hash32(), *index))
            .collect(),
        outputs,
        inputs
            .iter()
            .map(|_| Witness::single_key([0; 64]))
            .collect(),
    )
    .unwrap()
}

fn block(height: u64, txs: Vec<Transaction>) -> Block {
    let header = Header::new(
        BlockHeight::new(height),
        Hash32::ZERO,
        Hash32::ZERO,
        T0 + 120 * height,
        0x207f_ffff,
        0,
    );
    Block::new(header, txs).unwrap()
}

fn entry(
    id: Hex32,
    height: u64,
    coinbase: bool,
    received: Amount,
    spent: Amount,
    fee: Option<Amount>,
) -> HistoryEntry {
    HistoryEntry {
        txid: id,
        height: Some(height),
        timestamp: Some(T0 + 120 * height),
        coinbase,
        received,
        spent,
        fee,
    }
}

/// Folder sementara khusus test ini, dihapus saat selesai.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("zevrigo-q112-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Dua blok: coinbase 5 ZVRG ke alamat penerimaan 0, lalu transaksi yang membelanjakannya: 3 ZVRG ke pihak lain,
/// kembalian 1,9 ZVRG ke alamat kembalian 0, biaya 0,1 ZVRG.
fn two_blocks(state: &mut HistoryState, keys: &mut WalletKeys) {
    let first = block(
        1,
        vec![coinbase(1, vec![out(payload(RECEIVE_BRANCH, 0), zvrg(50))])],
    );
    state.apply_block(keys, &first, &[txid(1)], 1).unwrap();
    let second = block(
        2,
        vec![
            coinbase(2, vec![out(OTHER, zvrg(10))]),
            spend(
                &[(txid(1), 0)],
                vec![
                    out(OTHER, zvrg(30)),
                    out(payload(CHANGE_BRANCH, 0), zvrg(19)),
                ],
            ),
        ],
    );
    state
        .apply_block(keys, &second, &[txid(2), txid(3)], 2)
        .unwrap();
}

#[test]
fn receive_spend_change_and_fee() {
    let mut keys = WalletKeys::new(&account(), NextIndex::default());
    let mut state = HistoryState::new();
    two_blocks(&mut state, &mut keys);
    assert_eq!(
        state.entries(),
        &[
            entry(txid(1), 1, true, zvrg(50), Amount::ZERO, None),
            entry(txid(3), 2, false, zvrg(19), zvrg(50), Some(zvrg(1))),
        ]
    );
    assert_eq!(
        keys.next_index(),
        NextIndex {
            receive: 1,
            change: 1
        }
    );
    assert_eq!(state.unspent_count(), 1);
}

#[test]
fn foreign_input_has_no_fee_and_unrelated_tx_is_skipped() {
    let mut keys = WalletKeys::new(&account(), NextIndex::default());
    let mut state = HistoryState::new();
    let first = block(
        1,
        vec![coinbase(1, vec![out(payload(RECEIVE_BRANCH, 0), zvrg(20))])],
    );
    state.apply_block(&mut keys, &first, &[txid(1)], 1).unwrap();
    let second = block(
        2,
        vec![
            coinbase(2, vec![out(OTHER, zvrg(10))]),
            spend(&[(txid(1), 0), (txid(9), 0)], vec![out(OTHER, zvrg(50))]),
            spend(&[(txid(8), 0)], vec![out(OTHER, zvrg(10))]),
        ],
    );
    state
        .apply_block(&mut keys, &second, &[txid(2), txid(3), txid(4)], 2)
        .unwrap();
    assert_eq!(
        state.entries(),
        &[
            entry(txid(1), 1, true, zvrg(20), Amount::ZERO, None),
            entry(txid(3), 2, false, Amount::ZERO, zvrg(20), None),
        ]
    );
    assert_eq!(state.unspent_count(), 0);
}

#[test]
fn gap_limit_grows_with_used_addresses() {
    let mut keys = WalletKeys::new(&account(), NextIndex::default());
    let mut state = HistoryState::new();
    assert!(keys.contains(&payload(RECEIVE_BRANCH, 19)));
    assert!(!keys.contains(&payload(RECEIVE_BRANCH, 20)));
    assert!(!keys.contains(&payload(RECEIVE_BRANCH, 35)));
    let blocks = [
        block(
            1,
            vec![coinbase(
                1,
                vec![out(payload(RECEIVE_BRANCH, 19), zvrg(10))],
            )],
        ),
        block(
            2,
            vec![coinbase(
                2,
                vec![out(payload(RECEIVE_BRANCH, 35), zvrg(10))],
            )],
        ),
        // Indeks 60 di atas batas 36 + 20: tidak dikenali.
        block(
            3,
            vec![coinbase(
                3,
                vec![out(payload(RECEIVE_BRANCH, 60), zvrg(10))],
            )],
        ),
    ];
    for (height, item) in (1_u64..).zip(blocks.iter()) {
        let id = txid(u8::try_from(height).unwrap());
        state.apply_block(&mut keys, item, &[id], height).unwrap();
    }
    assert_eq!(state.entries().len(), 2);
    assert_eq!(
        keys.next_index(),
        NextIndex {
            receive: 36,
            change: 0
        }
    );
    assert!(keys.contains(&payload(RECEIVE_BRANCH, 55)));
    assert!(!keys.contains(&payload(RECEIVE_BRANCH, 56)));
    assert_eq!(keys.payloads().len(), 56 + 20);
}

#[test]
fn starts_from_stored_next_index_and_rejects_txid_count() {
    let mut keys = WalletKeys::new(
        &account(),
        NextIndex {
            receive: 5,
            change: 2,
        },
    );
    assert!(keys.contains(&payload(RECEIVE_BRANCH, 24)));
    assert!(!keys.contains(&payload(RECEIVE_BRANCH, 25)));
    assert!(keys.contains(&payload(CHANGE_BRANCH, 21)));
    assert_eq!(
        keys.next_index(),
        NextIndex {
            receive: 5,
            change: 2
        }
    );
    let mut state = HistoryState::new();
    let first = block(
        1,
        vec![coinbase(1, vec![out(payload(RECEIVE_BRANCH, 0), zvrg(10))])],
    );
    assert_eq!(
        state.apply_block(&mut keys, &first, &[], 1),
        Err(HistoryError::TxidCount {
            expected: 1,
            actual: 0
        })
    );
}

#[test]
fn pending_entries_from_mempool_scan() {
    let mut keys = WalletKeys::new(&account(), NextIndex::default());
    let mut state = HistoryState::new();
    let first = block(
        1,
        vec![coinbase(1, vec![out(payload(RECEIVE_BRANCH, 0), zvrg(50))])],
    );
    state.apply_block(&mut keys, &first, &[txid(1)], 1).unwrap();
    let spends = [
        MempoolSpend {
            txid: txid(1),
            index: 0,
            spent_by: txid(5),
        },
        // Outpoint yang bukan koin wallet: diabaikan.
        MempoolSpend {
            txid: txid(8),
            index: 0,
            spent_by: txid(6),
        },
    ];
    let output = |id: Hex32, index: u32, amount: Amount, key: [u8; 32]| MempoolOutput {
        txid: id,
        index,
        amount: Quanta::new(amount),
        kind: KindNumber::new(OutputKind::SingleKey),
        payload: Hex32::new(key),
    };
    let outputs = [
        output(txid(5), 1, zvrg(40), payload(CHANGE_BRANCH, 0)),
        output(txid(7), 0, zvrg(10), payload(RECEIVE_BRANCH, 1)),
    ];
    let pending = |id: Hex32, received: Amount, spent: Amount| HistoryEntry {
        txid: id,
        height: None,
        timestamp: None,
        coinbase: false,
        received,
        spent,
        fee: None,
    };
    assert_eq!(
        state.pending_entries(&spends, &outputs),
        Ok(vec![
            pending(txid(5), zvrg(40), zvrg(50)),
            pending(txid(7), zvrg(10), Amount::ZERO),
        ])
    );
}

#[test]
fn cache_round_trip_and_rejections() {
    let dir = TempDir::new("cache");
    let path = dir.0.join("history.json");
    let wallet = identity(&account());
    assert_eq!(wallet, Hex32::new(payload(RECEIVE_BRANCH, 0)));
    assert_eq!(load(&path, Network::Regtest, wallet), HistoryState::new());

    let mut keys = WalletKeys::new(&account(), NextIndex::default());
    let mut state = HistoryState::new();
    two_blocks(&mut state, &mut keys);
    state.tip = Some(HistoryTip {
        height: 2,
        block_id: txid(0xb2),
    });
    save(&path, Network::Regtest, wallet, &state).unwrap();
    assert_eq!(load(&path, Network::Regtest, wallet), state);
    // Menyimpan ulang menimpa tembolok lama tanpa sisa file sementara.
    save(&path, Network::Regtest, wallet, &state).unwrap();
    assert!(!dir.0.join("history.json.tmp").exists());
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"network\": \"regtest\""), "{text}");
    assert!(
        text.contains("\"received\": \"1900000000000000000\""),
        "{text}"
    );

    assert_eq!(load(&path, Network::Testnet, wallet), HistoryState::new());
    assert_eq!(load(&path, Network::Regtest, txid(9)), HistoryState::new());
    std::fs::write(&path, text.replace("\"version\": 1", "\"version\": 2")).unwrap();
    assert_eq!(load(&path, Network::Regtest, wallet), HistoryState::new());
    std::fs::write(&path, "bukan json").unwrap();
    assert_eq!(load(&path, Network::Regtest, wallet), HistoryState::new());
}

#[test]
fn history_file_lives_beside_wallet_file() {
    assert_eq!(
        history_path(Network::Testnet, Path::new("/data")),
        Path::new("/data").join("testnet").join("history.json")
    );
    assert_eq!(
        history_path(Network::Regtest, Path::new("/data")),
        Path::new("/data").join("regtest").join("history.json")
    );
}
