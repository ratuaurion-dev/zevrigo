//! Uji asap lintas crate inti (MOD-01 sampai MOD-10).

#![forbid(unsafe_code)]

#[test]
fn core_crates_expose_expected_module_ids() {
    assert_eq!(zevrigo_primitives::MODULE_ID, "MOD-01");
    assert_eq!(zevrigo_crypto::MODULE_ID, "MOD-02");
    assert_eq!(zevrigo_pow::MODULE_ID, "MOD-03");
    assert_eq!(zevrigo_consensus::MODULE_ID, "MOD-04");
    assert_eq!(zevrigo_storage::MODULE_ID, "MOD-05");
    assert_eq!(zevrigo_network::MODULE_ID, "MOD-06");
    assert_eq!(zevrigo_mempool::MODULE_ID, "MOD-07");
    assert_eq!(zevrigo_rpc::MODULE_ID, "MOD-08");
    assert_eq!(zevrigo_wallet_core::MODULE_ID, "MOD-09");
    assert_eq!(zevrigo_merchant_core::MODULE_ID, "MOD-10");
}
