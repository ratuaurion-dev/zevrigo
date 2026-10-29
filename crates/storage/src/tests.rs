use crate::{BlockUndo, ChainStore, MemoryStore, StoreError};

#[test]
fn module_id_matches() {
    assert_eq!(super::MODULE_ID, "MOD-05");
}

#[test]
fn crate_root_reexports_are_accessible() {
    let store = MemoryStore::default();
    assert_eq!(store.tip(), Ok(None));
    assert!(BlockUndo::new(Vec::new(), Vec::new()).spent().is_empty());
    assert_eq!(
        StoreError::EmptyChain.to_string(),
        "chain kosong, tidak ada blok untuk di-undo"
    );
}
