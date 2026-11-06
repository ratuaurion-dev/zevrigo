use std::path::PathBuf;

use zevrigo_consensus::regtest_network_id;
use zevrigo_primitives::{Amount, Hash32};

use super::{DB_FILE, amount_from_blob, amount_to_blob};
use crate::index::{IndexError, Indexer};
use crate::query::confirmations;
use crate::test_support::Chain;

/// Folder sementara khusus satu test, dihapus saat selesai.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("zevrigo-q82-{name}-{}", std::process::id()));
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

#[test]
fn index_file_survives_reopen_and_rejects_other_network_or_schema() {
    let dir = TempDir::new("reopen");
    let path = dir.0.join(DB_FILE);
    let network = regtest_network_id().unwrap();
    let mut chain = Chain::new();
    chain.push(2, Vec::new(), 0);
    chain.push(2, Vec::new(), 0);
    {
        let mut index = Indexer::open(&path, network).unwrap();
        assert_eq!(index.sync_step(&chain.source()).unwrap().added, 3);
    }
    let index = Indexer::open(&path, network).unwrap();
    assert_eq!(index.network_id(), network);
    assert_eq!(index.tip().unwrap(), Some((2, chain.id(2))));
    assert_eq!(
        index.supply().unwrap().unwrap().mined_total,
        chain.mined_total()
    );
    drop(index);

    let other = Hash32::from_bytes([1; 32]);
    assert_eq!(
        Indexer::open(&path, other).err(),
        Some(IndexError::WrongNetwork {
            expected: other,
            found: network
        })
    );
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute(
        "UPDATE meta SET value = ?1 WHERE key = 'schema_version'",
        rusqlite::params![2_u32.to_le_bytes()],
    )
    .unwrap();
    drop(conn);
    assert_eq!(
        Indexer::open(&path, network).err(),
        Some(IndexError::WrongSchema { found: 2 })
    );
}

#[test]
fn amount_blobs_and_confirmations() {
    for quanta in [0, 1, 1_910_127_426_536_917, u128::MAX] {
        let amount = Amount::from_quanta(quanta);
        assert_eq!(amount_from_blob(amount_to_blob(amount)), amount);
    }
    let mut expected = [0_u8; 16];
    expected[14] = 1;
    expected[15] = 2;
    assert_eq!(amount_to_blob(Amount::from_quanta(258)), expected);
    assert_eq!(confirmations(10, 10), 1);
    assert_eq!(confirmations(10, 3), 8);
    assert_eq!(confirmations(3, 10), 0);
}
