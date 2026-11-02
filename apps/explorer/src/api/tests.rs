use homeostasis_consensus::{SupplySchedule, regtest_network_id};
use homeostasis_crypto::txid;
use homeostasis_primitives::params::GENESIS_ALLOCATION;
use homeostasis_primitives::{Hash32, Network, OutputKind};
use homeostasis_wallet_core::encode_address;
use serde_json::Value;

use super::{NodeView, SyncStatus, respond};
use crate::index::Indexer;
use crate::test_support::{Chain, Fault, genesis_output, out, payload, spend};

const FEE: u128 = 1_000;

fn addr(payload: &[u8; 32]) -> String {
    encode_address(Network::Regtest, OutputKind::SingleKey, payload)
}

/// Genesis + 3 blok ke pemilik 2; blok 3 membelanjakan alokasi genesis ke pemilik 3 (alokasi − 1_500) dan pemilik 2
/// (500) dengan biaya [`FEE`].
fn indexed() -> (Chain, Indexer, Hash32) {
    let (genesis_txid, allocation, _) = genesis_output();
    let tx = spend(
        &[(genesis_txid, 0)],
        vec![out(allocation - 1_500, 3), out(500, 2)],
    );
    let tx_id = txid(&tx);
    let mut chain = Chain::new();
    chain.push(2, Vec::new(), 0);
    chain.push(2, Vec::new(), 0);
    chain.push(2, vec![tx], FEE);
    let mut index = Indexer::open_in_memory(regtest_network_id().unwrap()).unwrap();
    index.sync_step(&chain.source()).unwrap();
    (chain, index, tx_id)
}

fn call(
    index: &Indexer,
    sync: &SyncStatus,
    method: &str,
    path: &str,
    query: Option<&str>,
) -> (u16, Value) {
    let response = respond(index, sync, Network::Regtest, method, path, query);
    (
        response.status,
        serde_json::from_str(&response.body).unwrap(),
    )
}

fn get_query(index: &Indexer, path: &str, query: Option<&str>) -> Value {
    let (status, body) = call(index, &SyncStatus::default(), "GET", path, query);
    assert_eq!(status, 200, "{path}: {body}");
    body
}

fn get(index: &Indexer, path: &str) -> Value {
    get_query(index, path, None)
}

fn heights(body: &Value) -> Vec<u64> {
    body["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|block| block["height"].as_u64().unwrap())
        .collect()
}

#[test]
fn api_serves_indexed_chain() {
    let (chain, index, tx_id) = indexed();
    let (genesis_txid, allocation, genesis_payload) = genesis_output();
    let schedule: Vec<_> = SupplySchedule::new().take(4).collect();
    let hex = |id: Hash32| id.to_string();
    let quanta = |value: u128| value.to_string();

    let status = get(&index, "/api/v1/status");
    assert_eq!(status["network"], "regtest");
    assert_eq!(status["indexed_height"], 3);
    assert_eq!(status["indexed_block_id"], hex(chain.id(3)));
    assert_eq!(status["node_height"], Value::Null);
    assert_eq!(status["index_error"], Value::Null);
    assert_eq!(status["last_error"], Value::Null);
    assert_eq!(status["mempool_count"], 0);

    let blocks = get(&index, "/api/v1/blocks");
    assert_eq!(heights(&blocks), vec![3, 2, 1, 0]);
    assert_eq!(blocks["blocks"][0]["confirmations"], 1);
    assert_eq!(blocks["blocks"][3]["confirmations"], 4);
    assert_eq!(blocks["blocks"][0]["tx_count"], 2);
    assert_eq!(
        heights(&get_query(&index, "/api/v1/blocks", Some("before=2"))),
        vec![1, 0]
    );

    let tip = get(&index, "/api/v1/block/3");
    assert_eq!(tip, get(&index, &format!("/api/v1/block/{}", chain.id(3))));
    assert_eq!(tip["block_id"], hex(chain.id(3)));
    assert_eq!(tip["prev_block_id"], hex(chain.id(2)));
    assert_eq!(tip["target"], "207fffff");
    assert_eq!(tip["nonce"], "0");
    assert_eq!(tip["tx_count"], 2);
    assert_eq!(tip["fees"], quanta(FEE));
    assert_eq!(tip["subsidy"], quanta(schedule[3].subsidy.as_quanta()));
    assert_eq!(tip["confirmations"], 1);
    assert_eq!(tip["txids"][1], hex(tx_id));
    assert_eq!(tip["next_block_id"], Value::Null);
    let genesis = get(&index, "/api/v1/block/0");
    assert_eq!(genesis["subsidy"], "0");
    assert_eq!(genesis["next_block_id"], hex(chain.id(1)));

    let tx = get(&index, &format!("/api/v1/tx/{tx_id}"));
    assert_eq!(tx["height"], 3);
    assert_eq!(tx["position"], 1);
    assert_eq!(tx["is_coinbase"], false);
    assert_eq!(tx["fee"], quanta(FEE));
    assert_eq!(tx["confirmations"], 1);
    assert_eq!(tx["inputs"][0]["txid"], hex(genesis_txid));
    assert_eq!(tx["inputs"][0]["index"], 0);
    assert_eq!(tx["inputs"][0]["amount"], quanta(allocation));
    assert_eq!(tx["inputs"][0]["address"], addr(&genesis_payload));
    assert!(tx["inputs"][0].get("spent_by").is_none());
    assert_eq!(tx["outputs"][0]["amount"], quanta(allocation - 1_500));
    assert_eq!(tx["outputs"][0]["kind"], 0);
    assert_eq!(tx["outputs"][0]["address"], addr(&payload(3)));
    assert_eq!(tx["outputs"][0]["spent_by"], Value::Null);
    assert_eq!(tx["outputs"][1]["index"], 1);

    let receiver = get(&index, &format!("/api/v1/address/{}", addr(&payload(3))));
    assert_eq!(receiver["balance"], quanta(allocation - 1_500));
    assert_eq!(receiver["payload"], hex(Hash32::from_bytes(payload(3))));
    assert_eq!(receiver["kind"], 0);
    assert_eq!(
        (
            receiver["tx_count"].as_u64(),
            receiver["utxo_count"].as_u64()
        ),
        (Some(1), Some(1))
    );
    assert_eq!(
        (receiver["page"].as_u64(), receiver["page_size"].as_u64()),
        (Some(0), Some(25))
    );
    assert_eq!(receiver["txs"][0]["txid"], hex(tx_id));
    assert_eq!(receiver["txs"][0]["confirmations"], 1);
    let creator = get(
        &index,
        &format!("/api/v1/address/{}", addr(&genesis_payload)),
    );
    assert_eq!(creator["balance"], "0");
    assert_eq!(creator["sent"], quanta(allocation));
    assert_eq!(creator["tx_count"], 2);
    let later = get_query(
        &index,
        &format!("/api/v1/address/{}", addr(&payload(2))),
        Some("page=1"),
    );
    assert_eq!(later["tx_count"], 4);
    assert_eq!(later["page"], 1);
    assert!(later["txs"].as_array().unwrap().is_empty());

    let supply = get(&index, "/api/v1/supply");
    let mined = schedule[3].mined_total_after;
    assert_eq!(supply["height"], 3);
    assert_eq!(supply["mined_total"], quanta(mined.as_quanta()));
    assert_eq!(supply["scheduled_mined_total"], quanta(mined.as_quanta()));
    assert_eq!(supply["matches_schedule"], true);
    assert_eq!(
        supply["genesis_allocation"],
        quanta(GENESIS_ALLOCATION.as_quanta())
    );
    assert_eq!(
        supply["total_supply"],
        quanta(GENESIS_ALLOCATION.checked_add(mined).unwrap().as_quanta())
    );
    assert_eq!(supply["fees_total"], quanta(FEE));
    assert_eq!(supply["tip_target"], "207fffff");
    assert_eq!(supply["average_block_seconds"], 120);

    let allocation_page = get(&index, "/api/v1/allocation");
    assert_eq!(allocation_page["genesis_block_id"], hex(chain.id(0)));
    assert_eq!(
        allocation_page["allocated"],
        quanta(GENESIS_ALLOCATION.as_quanta())
    );
    assert_eq!(allocation_page["unspent"], "0");
    assert_eq!(allocation_page["outputs"][0]["txid"], hex(genesis_txid));
    assert_eq!(
        allocation_page["outputs"][0]["spent_by"]["txid"],
        hex(tx_id)
    );
    assert_eq!(allocation_page["outputs"][0]["spent_by"]["height"], 3);
    assert_eq!(
        allocation_page["addresses"][0]["address"],
        addr(&genesis_payload)
    );
    assert_eq!(allocation_page["addresses"][0]["balance"], "0");

    let mempool = get(&index, "/api/v1/mempool");
    assert_eq!(mempool["count"], 0);
    assert!(mempool["txids"].as_array().unwrap().is_empty());

    let search = |q: &str| get(&index, &format!("/api/v1/search/{q}"));
    assert_eq!(search("3")["kind"], "block");
    assert_eq!(search("3")["key"], hex(chain.id(3)));
    assert_eq!(search(&hex(chain.id(1)))["kind"], "block");
    assert_eq!(search(&hex(tx_id))["kind"], "tx");
    let address = search(&addr(&payload(3)));
    assert_eq!(address["kind"], "address");
    assert_eq!(address["key"], addr(&payload(3)));
}

#[test]
fn api_rejects_bad_requests_and_reports_status() {
    let none = SyncStatus::default();
    let empty = Indexer::open_in_memory(regtest_network_id().unwrap()).unwrap();
    let (status, body) = call(&empty, &none, "GET", "/api/v1/supply", None);
    assert_eq!((status, body["error"].as_str()), (503, Some("not_ready")));
    let (status, body) = call(&empty, &none, "GET", "/api/v1/status", None);
    assert_eq!((status, body["indexed_height"].clone()), (200, Value::Null));

    let (chain, mut index, _) = indexed();
    let tip = chain.id(3).to_string();
    assert_ne!(tip, tip.to_uppercase());
    let cases: Vec<(&str, String, Option<&str>, u16, &str)> = vec![
        (
            "POST",
            "/api/v1/status".to_owned(),
            None,
            405,
            "method_not_allowed",
        ),
        ("GET", "/".to_owned(), None, 404, "not_found"),
        ("GET", "/api/v1/".to_owned(), None, 404, "not_found"),
        ("GET", "/api/v1/status/".to_owned(), None, 404, "not_found"),
        ("GET", "/api/v1/unknown".to_owned(), None, 404, "not_found"),
        (
            "GET",
            "/api/v1/block/01".to_owned(),
            None,
            400,
            "bad_request",
        ),
        (
            "GET",
            "/api/v1/block/-1".to_owned(),
            None,
            400,
            "bad_request",
        ),
        (
            "GET",
            format!("/api/v1/block/{}", tip.to_uppercase()),
            None,
            400,
            "bad_request",
        ),
        (
            "GET",
            "/api/v1/block/999".to_owned(),
            None,
            404,
            "not_found",
        ),
        (
            "GET",
            "/api/v1/blocks".to_owned(),
            Some("before=x"),
            400,
            "bad_request",
        ),
        (
            "GET",
            "/api/v1/tx/1234".to_owned(),
            None,
            400,
            "bad_request",
        ),
        (
            "GET",
            "/api/v1/address/bukan-alamat".to_owned(),
            None,
            400,
            "bad_request",
        ),
        (
            "GET",
            format!("/api/v1/address/{}", addr(&payload(3))),
            Some("page=01"),
            400,
            "bad_request",
        ),
        (
            "GET",
            "/api/v1/allocation".to_owned(),
            Some("page=x"),
            400,
            "bad_request",
        ),
        (
            "GET",
            "/api/v1/search/xyz".to_owned(),
            None,
            400,
            "bad_request",
        ),
        (
            "GET",
            "/api/v1/search/999".to_owned(),
            None,
            404,
            "not_found",
        ),
    ];
    for (method, path, query, expected, name) in &cases {
        let (status, body) = call(&index, &none, method, path, *query);
        assert_eq!(
            (status, body["error"].as_str()),
            (*expected, Some(*name)),
            "{method} {path}"
        );
    }

    // Txid yang belum terindeks: 404, dengan tanda apakah ada di mempool.
    let pending = Hash32::from_bytes([0x42; 32]);
    let (status, body) = call(&index, &none, "GET", &format!("/api/v1/tx/{pending}"), None);
    assert_eq!((status, body["in_mempool"].as_bool()), (404, Some(false)));
    let mut source = chain.source();
    source.mempool = vec![pending];
    index.sync_step(&source).unwrap();
    let (status, body) = call(&index, &none, "GET", &format!("/api/v1/tx/{pending}"), None);
    assert_eq!((status, body["in_mempool"].as_bool()), (404, Some(true)));
    let (status, body) = call(
        &index,
        &none,
        "GET",
        &format!("/api/v1/search/{pending}"),
        None,
    );
    assert_eq!((status, body["in_mempool"].as_bool()), (404, Some(true)));
    let (_, mempool) = call(&index, &none, "GET", "/api/v1/mempool", None);
    assert_eq!(mempool["count"], 1);
    assert_eq!(mempool["txids"][0], pending.to_string());

    // Ujung node dan galat terakhir dari server, galat yang menghentikan indexer dari indeks.
    let sync = SyncStatus {
        node: Some(NodeView {
            height: 9,
            best_block_id: Hash32::from_bytes([1; 32]),
            chainwork: [2; 32],
        }),
        last_error: Some("node tidak bisa dihubungi".to_owned()),
    };
    let (_, status) = call(&index, &sync, "GET", "/api/v1/status", None);
    assert_eq!(status["node_height"], 9);
    assert_eq!(
        status["node_best_block_id"],
        Hash32::from_bytes([1; 32]).to_string()
    );
    assert_eq!(status["chainwork"], Hash32::from_bytes([2; 32]).to_string());
    assert_eq!(status["last_error"], "node tidak bisa dihubungi");
    assert_eq!(status["mempool_count"], 1);
    let mut bad = Chain::new();
    bad.push(2, Vec::new(), 0);
    bad.push(2, Vec::new(), 0);
    let mut halted = Indexer::open_in_memory(regtest_network_id().unwrap()).unwrap();
    let mut wrong = bad.source();
    wrong.fault = Fault::WrongTxids(2);
    assert!(halted.sync_step(&wrong).is_err());
    let (_, status) = call(&halted, &none, "GET", "/api/v1/status", None);
    assert_eq!(status["indexed_height"], 1);
    assert!(status["index_error"].as_str().unwrap().contains("I2"));
}
