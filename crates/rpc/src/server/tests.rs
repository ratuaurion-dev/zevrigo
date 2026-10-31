use std::net::SocketAddr;
use std::sync::Arc;

use homeostasis_primitives::amount::Amount;
use homeostasis_primitives::params::Network;
use homeostasis_primitives::tx::OutputKind;
use jsonrpsee::core::client::ClientT;
use jsonrpsee_http_client::HttpClientBuilder;
use serde_json::{Map, Value, json};

use super::{RpcBackend, RpcServer};
use crate::client::{RpcClient, error_code, reject_reason};
use crate::error::{
    INTERNAL_ERROR, INVALID_ENCODING, INVALID_PARAMS, INVALID_TRANSACTION, METHOD_NOT_FOUND,
    NOT_FOUND, RpcError,
};
use crate::types::{
    BlockInfo, BlockTemplate, ChainInfo, GetBlockTemplateParams, MempoolInfo,
    SendRawTransactionResult, SubmitBlockResult, SubmitStatus, UtxoInfo,
};
use crate::value::{CompactHex, DecimalU64, Hex32, HexBytes, KindNumber, NetworkName, Quanta};

fn h(byte: u8) -> Hex32 {
    Hex32::new([byte; 32])
}

fn chain_info() -> ChainInfo {
    ChainInfo {
        network: NetworkName::new(Network::Regtest),
        height: 1,
        best_block_id: h(0xb1),
        chainwork: h(0x00),
        next_target: CompactHex::new(0x207f_ffff),
        median_time_past: 1_793_145_600,
        mined_total: Quanta::new(Amount::from_quanta(1_910_127_426_536_917)),
        total_supply: Quanta::new(Amount::from_quanta(4_380_000_001_910_127_426_536_917)),
        mempool_count: 2,
    }
}

fn block_info() -> BlockInfo {
    BlockInfo {
        block_id: h(0xb1),
        height: 1,
        prev_block_id: h(0xb0),
        merkle_root: h(0xcd),
        timestamp: 1_800_000_000,
        target: CompactHex::new(0x207f_ffff),
        nonce: DecimalU64::new(u64::MAX),
        size: 2,
        txids: vec![h(0x02)],
        hex: HexBytes::new(vec![0x01, 0x00]),
    }
}

/// Backend tiruan dengan data tetap.
struct Mock;

impl RpcBackend for Mock {
    fn chain_info(&self) -> Result<ChainInfo, RpcError> {
        Ok(chain_info())
    }

    fn block_hash(&self, height: u64) -> Result<Hex32, RpcError> {
        match height {
            1 => Ok(h(0xb1)),
            2 => Err(RpcError::Internal {
                message: "disk".to_owned(),
            }),
            _ => Err(RpcError::NotFound),
        }
    }

    fn block(&self, block_id: Hex32) -> Result<BlockInfo, RpcError> {
        if block_id == h(0xb1) {
            Ok(block_info())
        } else {
            Err(RpcError::NotFound)
        }
    }

    fn block_template(&self, params: &GetBlockTemplateParams) -> Result<BlockTemplate, RpcError> {
        // `hex` memuat jenis dan payload penerima, supaya test bisa memastikan parameter sampai ke backend.
        let mut hex = vec![params.payout_kind.kind().to_u8()];
        hex.extend_from_slice(params.payout_payload.bytes());
        Ok(BlockTemplate {
            height: 2,
            prev_block_id: h(0xb1),
            target: CompactHex::new(0x207f_ffff),
            min_timestamp: 1_793_145_601,
            subsidy: Quanta::new(Amount::from_quanta(1_910_127_426_536_917)),
            fees: Quanta::new(Amount::ZERO),
            hex: HexBytes::new(hex),
        })
    }

    fn submit_block(&self, block: &[u8]) -> Result<SubmitBlockResult, RpcError> {
        if block == [0x01] {
            Ok(SubmitBlockResult {
                status: SubmitStatus::Accepted,
                block_id: Some(h(0xb2)),
                reason: None,
            })
        } else {
            Ok(SubmitBlockResult {
                status: SubmitStatus::Invalid,
                block_id: None,
                reason: Some("V1".to_owned()),
            })
        }
    }

    fn send_raw_transaction(&self, tx: &[u8]) -> Result<SendRawTransactionResult, RpcError> {
        match tx {
            [0xaa] => Ok(SendRawTransactionResult { txid: h(0xaa) }),
            [] => Err(RpcError::InvalidEncoding),
            _ => Err(RpcError::InvalidTransaction {
                reason: "T6".to_owned(),
            }),
        }
    }

    fn mempool(&self) -> Result<MempoolInfo, RpcError> {
        Ok(MempoolInfo {
            txids: vec![h(0x0a), h(0x0b)],
        })
    }

    fn scan_utxos(&self, payloads: &[Hex32]) -> Result<Vec<UtxoInfo>, RpcError> {
        Ok(payloads
            .iter()
            .map(|payload| UtxoInfo {
                txid: h(0x11),
                index: 0,
                amount: Quanta::new(Amount::from_quanta(5)),
                kind: KindNumber::new(OutputKind::SingleKey),
                payload: *payload,
                height: 1,
                is_coinbase: false,
                confirmations: 1,
                spendable_at: 2,
            })
            .collect())
    }
}

async fn start() -> (RpcServer, RpcClient) {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let server = RpcServer::start(addr, Arc::new(Mock))
        .await
        .expect("server");
    let url = format!("http://{}", server.local_addr());
    let client = RpcClient::new(&url).expect("klien");
    (server, client)
}

#[tokio::test]
async fn round_trip_all_methods() {
    let (server, client) = start().await;
    assert_ne!(server.local_addr().port(), 0);

    assert_eq!(client.get_chain_info().await.expect("chain"), chain_info());
    assert_eq!(client.get_block_hash(1).await.expect("hash"), h(0xb1));
    assert_eq!(client.get_block(h(0xb1)).await.expect("blok"), block_info());

    let template = client
        .get_block_template(KindNumber::new(OutputKind::ConditionHash), h(0x77))
        .await
        .expect("template");
    let mut expected = vec![0x01_u8];
    expected.extend_from_slice(&[0x77; 32]);
    assert_eq!(template.hex.as_slice(), expected.as_slice());
    assert_eq!(template.height, 2);

    assert_eq!(
        client.submit_block(&[0x01]).await.expect("submit"),
        SubmitBlockResult {
            status: SubmitStatus::Accepted,
            block_id: Some(h(0xb2)),
            reason: None,
        }
    );
    let invalid = client.submit_block(&[0x02]).await.expect("submit");
    assert_eq!(invalid.status, SubmitStatus::Invalid);
    assert_eq!(invalid.reason.as_deref(), Some("V1"));

    assert_eq!(
        client.send_raw_transaction(&[0xaa]).await.expect("tx"),
        SendRawTransactionResult { txid: h(0xaa) }
    );
    assert_eq!(
        client.get_mempool().await.expect("mempool").txids,
        vec![h(0x0a), h(0x0b)]
    );

    let utxos = client.scan_utxos(&[h(0x21), h(0x22)]).await.expect("scan");
    let payloads: Vec<Hex32> = utxos.iter().map(|u| u.payload).collect();
    assert_eq!(payloads, vec![h(0x21), h(0x22)]);
    assert!(client.scan_utxos(&[]).await.is_err());

    assert!(
        client
            .raw_request("get_chain_info", json!([]))
            .await
            .is_ok()
    );
    assert!(
        client
            .raw_request("get_chain_info", json!({}))
            .await
            .is_ok()
    );
    server.stop().await;
}

#[tokio::test]
async fn backend_errors_use_spec_codes() {
    let (server, client) = start().await;

    let err = client.get_block_hash(9).await.expect_err("tidak ada");
    assert_eq!(error_code(&err), Some(NOT_FOUND));
    let err = client.get_block(h(0x99)).await.expect_err("tidak ada");
    assert_eq!(error_code(&err), Some(NOT_FOUND));
    let err = client.get_block_hash(2).await.expect_err("internal");
    assert_eq!(error_code(&err), Some(INTERNAL_ERROR));

    let err = client.send_raw_transaction(&[0xbb]).await.expect_err("T6");
    assert_eq!(error_code(&err), Some(INVALID_TRANSACTION));
    assert_eq!(reject_reason(&err).as_deref(), Some("T6"));
    let err = client
        .send_raw_transaction(&[])
        .await
        .expect_err("encoding");
    assert_eq!(error_code(&err), Some(INVALID_ENCODING));
    assert_eq!(reject_reason(&err), None);
    server.stop().await;
}

#[tokio::test]
async fn invalid_params_and_unknown_method_are_rejected() {
    let (server, client) = start().await;
    let upper = "AB".repeat(32);
    let payload = "77".repeat(32);
    let too_many = vec!["11".repeat(32); 65];
    let cases = [
        ("get_block_hash", json!({"height": 1, "extra": 0})),
        ("get_block_hash", json!([1])),
        ("get_block_hash", json!({"height": "1"})),
        ("get_block_hash", json!({"height": -1})),
        ("get_block", json!({"block_id": upper})),
        (
            "get_block_template",
            json!({"payout_kind": 2, "payout_payload": payload}),
        ),
        ("submit_block", json!({"hex": "0G"})),
        ("submit_block", json!({"hex": "010"})),
        ("send_raw_transaction", json!({})),
        ("scan_utxos", json!({"payloads": []})),
        ("scan_utxos", json!({ "payloads": too_many })),
        ("get_chain_info", json!({"x": 1})),
        ("get_mempool", json!([1])),
    ];
    for (method, params) in cases {
        let shown = params.to_string();
        let err = client.raw_request(method, params).await.expect_err(method);
        assert_eq!(error_code(&err), Some(INVALID_PARAMS), "{method} {shown}");
    }

    let err = client
        .raw_request("get_balance", json!({}))
        .await
        .expect_err("metode tidak ada");
    assert_eq!(error_code(&err), Some(METHOD_NOT_FOUND));
    server.stop().await;
}

#[tokio::test]
async fn request_size_limit_is_one_mib() {
    let (server, client) = start().await;

    // 400_000 byte (800_000 karakter hex) masih di bawah 1 MiB: sampai ke backend dan dijawab `invalid`.
    let below = vec![0x02_u8; 400_000];
    let result = client.submit_block(&below).await.expect("di bawah batas");
    assert_eq!(result.status, SubmitStatus::Invalid);

    // 600_000 byte (1_200_000 karakter hex) melewati 1 MiB dan ditolak server.
    let url = format!("http://{}", server.local_addr());
    let big_client = HttpClientBuilder::default()
        .max_request_size(8_388_608)
        .build(url)
        .expect("klien besar");
    let mut params = Map::new();
    params.insert("hex".to_owned(), Value::String("00".repeat(600_000)));
    let response: Result<Value, _> = big_client.request("submit_block", params).await;
    assert!(response.is_err());
    server.stop().await;
}
