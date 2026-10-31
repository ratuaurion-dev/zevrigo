use jsonrpsee::server::types::ErrorObjectOwned;

use super::{
    INTERNAL_ERROR, INVALID_ENCODING, INVALID_TRANSACTION, NOT_FOUND, RejectData, RpcError,
};

#[test]
fn rpc_error_maps_to_spec_codes() {
    let not_found = ErrorObjectOwned::from(RpcError::NotFound);
    assert_eq!(not_found.code(), NOT_FOUND);
    assert_eq!(not_found.message(), "tidak ditemukan");
    assert!(not_found.data().is_none());

    let rejected = ErrorObjectOwned::from(RpcError::InvalidTransaction {
        reason: "T6".to_owned(),
    });
    assert_eq!(rejected.code(), INVALID_TRANSACTION);
    assert_eq!(rejected.message(), "transaksi tidak sah: T6");
    let data = rejected.data().map(|raw| raw.get().to_owned());
    assert_eq!(data.as_deref(), Some("{\"reason\":\"T6\"}"));
    let parsed: RejectData = serde_json::from_str(data.as_deref().unwrap_or("")).expect("data");
    assert_eq!(parsed.reason, "T6");

    let encoding = ErrorObjectOwned::from(RpcError::InvalidEncoding);
    assert_eq!(encoding.code(), INVALID_ENCODING);
    assert!(encoding.data().is_none());

    let internal = ErrorObjectOwned::from(RpcError::Internal {
        message: "disk".to_owned(),
    });
    assert_eq!(internal.code(), INTERNAL_ERROR);
    assert_eq!(internal.message(), "galat internal: disk");
    assert_eq!(RpcError::NotFound.code(), 1_001);
}
