use zevrigo_consensus::regtest_network_id;
use zevrigo_crypto::txid;
use zevrigo_primitives::{Amount, Hash32};

use super::{IndexError, Indexer, SyncReport};
use crate::query::SpentBy;
use crate::test_support::{Chain, Fault, genesis_output, out, payload, spend};

fn network() -> Hash32 {
    regtest_network_id().unwrap()
}

fn tip_height(index: &Indexer) -> Option<u64> {
    index.tip().unwrap().map(|(height, _)| height)
}

#[test]
fn sync_indexes_genesis_new_blocks_and_mempool() {
    let mut chain = Chain::new();
    chain.push(2, Vec::new(), 0);
    chain.push(2, Vec::new(), 0);
    let mut index = Indexer::open_in_memory(network()).unwrap();
    assert_eq!(index.tip().unwrap(), None);
    assert_eq!(index.supply().unwrap(), None);
    assert_eq!(index.allocation().unwrap(), None);

    let pending = Hash32::from_bytes([5; 32]);
    let mut source = chain.source();
    source.mempool = vec![pending];
    assert_eq!(
        index.sync_step(&source).unwrap(),
        SyncReport {
            node_height: 2,
            node_tip: chain.id(2),
            rolled_back: 0,
            added: 3,
            interrupted: false,
        }
    );
    assert_eq!(index.tip().unwrap(), Some((2, chain.id(2))));
    assert_eq!(index.mempool().unwrap(), vec![pending]);
    assert!(index.in_mempool(&pending).unwrap());

    // Tanpa blok baru: tidak ada yang berubah, mempool diganti isi terbaru.
    let again = index.sync_step(&chain.source()).unwrap();
    assert_eq!((again.rolled_back, again.added), (0, 0));
    assert!(index.mempool().unwrap().is_empty());

    chain.push(4, Vec::new(), 0);
    assert_eq!(index.sync_step(&chain.source()).unwrap().added, 1);

    // Blok 4 belum bisa diambil: langkah berhenti di tengah tanpa galat, lalu dilanjutkan.
    let mut longer = chain.clone();
    longer.push(4, Vec::new(), 0);
    let mut missing = longer.source();
    missing.fault = Fault::Missing(4);
    let report = index.sync_step(&missing).unwrap();
    assert!(report.interrupted);
    assert_eq!((report.node_height, report.added), (4, 0));
    assert_eq!(tip_height(&index), Some(3));

    // Node tidak bisa dihubungi: galat sumber, indexer tidak berhenti.
    let mut offline = longer.source();
    offline.fault = Fault::Offline;
    assert!(matches!(
        index.sync_step(&offline),
        Err(IndexError::Source(_))
    ));
    assert_eq!(index.halted(), None);
    assert_eq!(index.sync_step(&longer.source()).unwrap().added, 1);
    assert_eq!(index.tip().unwrap(), Some((4, longer.id(4))));
}

#[test]
fn reorg_rolls_back_and_follows_new_chain() {
    let (genesis_txid, allocation, _) = genesis_output();
    let tx = spend(&[(genesis_txid, 0)], vec![out(allocation - 1_000, 3)]);
    let tx_id = txid(&tx);
    let mut a = Chain::new();
    a.push(2, Vec::new(), 0);
    a.push(2, vec![tx], 1_000);
    a.push(2, Vec::new(), 0);
    let mut index = Indexer::open_in_memory(network()).unwrap();
    index.sync_step(&a.source()).unwrap();
    assert!(index.tx(&tx_id).unwrap().is_some());

    // Cabang B dari tinggi 1, tiga blok tanpa transaksi itu: dua blok dilepas, tiga blok baru.
    let mut b = a.fork_at(1);
    for _ in 0..3 {
        b.push(4, Vec::new(), 0);
    }
    let report = index.sync_step(&b.source()).unwrap();
    assert_eq!(
        (report.rolled_back, report.added, report.interrupted),
        (2, 3, false)
    );
    assert_eq!(index.tip().unwrap(), Some((4, b.id(4))));
    assert_eq!(index.tx(&tx_id).unwrap(), None);
    assert_eq!(index.block_at(1).unwrap().unwrap().block.block_id, a.id(1));
    let allocation_info = index.allocation().unwrap().unwrap();
    assert_eq!(allocation_info.unspent, Amount::from_quanta(allocation));
    assert_eq!(allocation_info.outputs[0].spent_by, None);
    assert_eq!(index.address(0, &payload(3), 0, 25).unwrap().tx_count, 0);
    let supply = index.supply().unwrap().unwrap();
    assert!(supply.matches_schedule);
    assert_eq!(supply.mined_total, b.mined_total());
    assert_eq!(supply.fees_total, Amount::ZERO);

    // Kembali ke chain A yang kini lebih panjang: B dilepas sedalam 3 blok, transaksi itu terindeks lagi.
    let mut a2 = a.clone();
    a2.push(2, Vec::new(), 0);
    a2.push(2, Vec::new(), 0);
    let report = index.sync_step(&a2.source()).unwrap();
    assert_eq!((report.rolled_back, report.added), (3, 4));
    assert_eq!(index.tip().unwrap(), Some((5, a2.id(5))));
    let detail = index.tx(&tx_id).unwrap().unwrap();
    assert_eq!(detail.tx.height, 2);
    assert_eq!(detail.tx.fee, Amount::from_quanta(1_000));
    assert_eq!(
        index.allocation().unwrap().unwrap().outputs[0].spent_by,
        Some(SpentBy {
            txid: tx_id,
            input: 0,
            height: 2
        })
    );
    let supply = index.supply().unwrap().unwrap();
    assert!(supply.matches_schedule);
    assert_eq!(supply.mined_total, a2.mined_total());
    assert_eq!(supply.fees_total, Amount::from_quanta(1_000));
}

/// Sinkron dengan `chain` yang diganggu `fault`: wajib berhenti dengan galat pemeriksaan di `height` aturan `rule`,
/// ujung indeks tetap `kept`, dan langkah berikutnya (walaupun dengan sumber jujur) mengembalikan galat yang sama.
fn assert_halts(chain: &Chain, fault: Fault, height: u64, rule: &str, kept: u64) {
    let mut index = Indexer::open_in_memory(network()).unwrap();
    let mut source = chain.source();
    source.fault = fault;
    let error = index.sync_step(&source).unwrap_err();
    match &error {
        IndexError::Check {
            height: failed,
            rule: broken,
            ..
        } => assert_eq!((*failed, *broken), (height, rule), "{error}"),
        other => unreachable!("galat pemeriksaan diharapkan, didapat {other:?}"),
    }
    assert_eq!(index.halted(), Some(&error));
    assert_eq!(tip_height(&index), Some(kept));
    assert_eq!(index.sync_step(&chain.source()).unwrap_err(), error);
    assert_eq!(tip_height(&index), Some(kept));
}

#[test]
fn bad_blocks_halt_indexer_and_keep_last_valid_index() {
    let (genesis_txid, allocation, _) = genesis_output();
    let mut good = Chain::new();
    good.push(2, Vec::new(), 0);
    good.push(2, Vec::new(), 0);
    // I1: block_id dari node bukan hash header. I2: txid dari node salah.
    assert_halts(&good, Fault::WrongId(2), 2, "I1", 1);
    assert_halts(&good, Fault::WrongTxids(2), 2, "I2", 1);

    // I3: input merujuk output yang tidak ada.
    let mut missing = Chain::new();
    missing.push(2, Vec::new(), 0);
    missing.push(
        2,
        vec![spend(&[(Hash32::from_bytes([9; 32]), 0)], vec![out(5, 3)])],
        0,
    );
    assert_halts(&missing, Fault::None, 2, "I3", 1);

    // I3: output yang sama dibelanjakan dua kali di blok berbeda.
    let mut double = Chain::new();
    double.push(
        2,
        vec![spend(&[(genesis_txid, 0)], vec![out(allocation - 10, 3)])],
        10,
    );
    double.push(
        2,
        vec![spend(&[(genesis_txid, 0)], vec![out(allocation - 20, 4)])],
        20,
    );
    assert_halts(&double, Fault::None, 2, "I3", 1);

    // I3: output melebihi input.
    let mut inflated = Chain::new();
    inflated.push(
        2,
        vec![spend(&[(genesis_txid, 0)], vec![out(allocation + 1, 3)])],
        0,
    );
    assert_halts(&inflated, Fault::None, 1, "I3", 0);

    // I4: coinbase 1 quanta di atas subsidy, atau tidak mengklaim biaya yang ada (DEC-17: wajib tepat).
    let mut greedy = Chain::new();
    greedy.push_with(2, Vec::new(), 0, 1);
    assert_halts(&greedy, Fault::None, 1, "I4", 0);
    let mut modest = Chain::new();
    modest.push(
        2,
        vec![spend(
            &[(genesis_txid, 0)],
            vec![out(allocation - 1_000, 3)],
        )],
        0,
    );
    assert_halts(&modest, Fault::None, 1, "I4", 0);

    // Genesis node bukan genesis jaringan indeks.
    let mut foreign = Indexer::open_in_memory(Hash32::from_bytes([1; 32])).unwrap();
    assert_eq!(
        foreign.sync_step(&good.source()).unwrap_err(),
        IndexError::GenesisMismatch
    );
    assert_eq!(foreign.halted(), Some(&IndexError::GenesisMismatch));
    assert_eq!(foreign.tip().unwrap(), None);
}
