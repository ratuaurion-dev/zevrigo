use homeostasis_primitives::amount::Amount;
use homeostasis_primitives::hash::Hash32;
use homeostasis_primitives::params::Network;

use crate::error::{
    INTERNAL_ERROR, INVALID_ENCODING, INVALID_PARAMS, INVALID_REQUEST, INVALID_TRANSACTION,
    METHOD_NOT_FOUND, NOT_FOUND, PARSE_ERROR,
};
use crate::transport::{DEFAULT_HOST, MAX_REQUEST_BYTES, default_port};

#[test]
fn module_id_matches() {
    assert_eq!(super::MODULE_ID, "MOD-08");
}

#[test]
fn error_codes_match_spec() {
    assert_eq!(
        [
            PARSE_ERROR,
            INVALID_REQUEST,
            METHOD_NOT_FOUND,
            INVALID_PARAMS,
            INTERNAL_ERROR
        ],
        [-32_700, -32_600, -32_601, -32_602, -32_603]
    );
    assert_eq!(
        [NOT_FOUND, INVALID_TRANSACTION, INVALID_ENCODING],
        [1_001, 1_002, 1_003]
    );
    let data = crate::RejectData {
        reason: "T6".to_owned(),
    };
    assert_eq!(
        serde_json::to_string(&data).expect("json"),
        "{\"reason\":\"T6\"}"
    );
}

#[test]
fn transport_defaults_match_spec() {
    assert_eq!(DEFAULT_HOST, "127.0.0.1");
    assert_eq!(MAX_REQUEST_BYTES, 1_048_576);
    assert_eq!(default_port(Network::Regtest), 28_801);
    assert_eq!(default_port(Network::Testnet), 18_801);
    assert_eq!(default_port(Network::Mainnet), 8_801);
}

#[test]
fn crate_root_reexports_are_accessible() {
    assert_eq!(
        crate::METHODS,
        [
            "get_chain_info",
            "get_block_hash",
            "get_block",
            "get_block_template",
            "submit_block",
            "send_raw_transaction",
            "get_mempool",
            "scan_utxos"
        ]
    );
    assert_eq!(crate::MAX_SCAN_PAYLOADS, 64);
    assert_eq!(
        crate::Quanta::try_from("0").map(crate::Quanta::amount),
        Ok(Amount::ZERO)
    );
    assert_eq!(crate::Hex32::from(Hash32::ZERO).to_hash32(), Hash32::ZERO);
}
