use zevrigo_consensus::TxError;
use zevrigo_primitives::Hash32;

#[test]
fn module_id_matches() {
    assert_eq!(super::MODULE_ID, "MOD-07");
}

#[test]
fn crate_root_reexports_are_accessible() {
    let pool = crate::Mempool::default();
    assert!(pool.is_empty());
    assert_eq!(pool.max_bytes(), crate::DEFAULT_MAX_BYTES);
    assert_eq!(crate::rule_name(&TxError::NotRegular), "struktur");
    assert_eq!(
        crate::MempoolError::Duplicate { txid: Hash32::ZERO }.rule(),
        None
    );
}
