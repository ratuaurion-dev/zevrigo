use zevrigo_primitives::amount::Amount;
use zevrigo_primitives::block::{Block, Header};
use zevrigo_primitives::encoding::{Decode, Encode};
use zevrigo_primitives::hash::Hash32;
use zevrigo_primitives::height::BlockHeight;
use zevrigo_primitives::params::Network;
use zevrigo_primitives::tx::{Output, OutputKind, Transaction};
use zevrigo_primitives::utxo::{OutPoint, UtxoEntry};

use super::{
    BlockInfo, ChainInfo, GetBlockHashParams, GetBlockParams, GetBlockTemplateParams,
    ScanUtxosParams, SubmitBlockParams, SubmitBlockResult, SubmitStatus, UtxoInfo,
};
use crate::value::{CompactHex, Hex32, NetworkName, Quanta};

/// Vektor `get_chain_info` (dihitung Quick).
const CHAIN_INFO_JSON: &str = "{\"network\":\"regtest\",\"height\":1,\"best_block_id\":\"abababababababababababababababababababababababababababababababab\",\"chainwork\":\"0000000000000000000000000000000000000000000000000000000000000004\",\"next_target\":\"207fffff\",\"median_time_past\":1793145600,\"mined_total\":\"1910127426536917\",\"total_supply\":\"4380000001910127426536917\",\"mempool_count\":0}";

/// Vektor satu elemen `scan_utxos` (dihitung Quick).
const UTXO_JSON: &str = "{\"txid\":\"1111111111111111111111111111111111111111111111111111111111111111\",\"index\":3,\"amount\":\"5\",\"kind\":1,\"payload\":\"2222222222222222222222222222222222222222222222222222222222222222\",\"height\":5,\"is_coinbase\":true,\"confirmations\":6,\"spendable_at\":105}";

fn hex_of(byte: u8) -> String {
    format!("{byte:02x}").repeat(32)
}

#[test]
fn params_reject_unknown_fields_and_wrong_types() {
    let ok: GetBlockHashParams = serde_json::from_str("{\"height\":7}").expect("json");
    assert_eq!(ok, GetBlockHashParams { height: 7 });
    let bad_inputs = [
        "{\"height\":7,\"extra\":1}",
        "{\"height\":-1}",
        "{\"height\":1.5}",
        "{\"height\":\"7\"}",
        "{}",
        "{\"height\":18446744073709551616}",
    ];
    for bad in bad_inputs {
        assert!(
            serde_json::from_str::<GetBlockHashParams>(bad).is_err(),
            "{bad}"
        );
    }

    let id = hex_of(0xab);
    let p: GetBlockParams =
        serde_json::from_str(&format!("{{\"block_id\":\"{id}\"}}")).expect("json");
    assert_eq!(p.block_id, Hex32::new([0xab; 32]));
    let upper = id.to_uppercase();
    assert!(
        serde_json::from_str::<GetBlockParams>(&format!("{{\"block_id\":\"{upper}\"}}")).is_err()
    );

    let t: GetBlockTemplateParams = serde_json::from_str(&format!(
        "{{\"payout_kind\":1,\"payout_payload\":\"{id}\"}}"
    ))
    .expect("json");
    assert_eq!(t.payout_kind.kind(), OutputKind::ConditionHash);
    assert_eq!(t.payout_payload, Hex32::new([0xab; 32]));
    assert!(
        serde_json::from_str::<GetBlockTemplateParams>(&format!(
            "{{\"payout_kind\":2,\"payout_payload\":\"{id}\"}}"
        ))
        .is_err()
    );

    let s: SubmitBlockParams = serde_json::from_str("{\"hex\":\"0100\"}").expect("json");
    assert_eq!(s.hex.as_slice(), &[0x01_u8, 0x00]);
    assert!(serde_json::from_str::<SubmitBlockParams>("{\"hex\":\"0G\"}").is_err());
}

#[test]
fn scan_utxos_params_accept_1_to_64_payloads() {
    let one = format!("\"{}\"", hex_of(0x11));
    for count in [1_usize, 64] {
        let json = format!("{{\"payloads\":[{}]}}", vec![one.clone(); count].join(","));
        let p: ScanUtxosParams = serde_json::from_str(&json).expect("json");
        assert_eq!(p.payloads.as_slice().len(), count);
        assert_eq!(p.payloads.as_slice().first(), Some(&Hex32::new([0x11; 32])));
    }
    for count in [0_usize, 65] {
        let json = format!("{{\"payloads\":[{}]}}", vec![one.clone(); count].join(","));
        assert!(
            serde_json::from_str::<ScanUtxosParams>(&json).is_err(),
            "{count}"
        );
    }
    assert!(serde_json::from_str::<ScanUtxosParams>("{\"payloads\":\"11\"}").is_err());
}

#[test]
fn chain_info_json_vector() {
    let mut chainwork = [0_u8; 32];
    if let Some(last) = chainwork.last_mut() {
        *last = 0x04;
    }
    let info = ChainInfo {
        network: NetworkName::new(Network::Regtest),
        height: 1,
        best_block_id: Hex32::new([0xab; 32]),
        chainwork: Hex32::new(chainwork),
        next_target: CompactHex::new(0x207f_ffff),
        median_time_past: 1_793_145_600,
        mined_total: Quanta::new(Amount::from_quanta(1_910_127_426_536_917)),
        total_supply: Quanta::new(Amount::from_quanta(4_380_000_001_910_127_426_536_917)),
        mempool_count: 0,
    };
    let json = serde_json::to_string(&info).expect("json");
    assert_eq!(json, CHAIN_INFO_JSON);
    assert_eq!(
        serde_json::from_str::<ChainInfo>(&json).expect("json"),
        info
    );
}

#[test]
fn block_info_from_block_matches_header() {
    let out =
        Output::new(Amount::from_quanta(30), OutputKind::SingleKey, [0x02; 32]).expect("output");
    let cb = Transaction::coinbase(BlockHeight::new(1), Vec::new(), vec![out]).expect("coinbase");
    let header = Header::new(
        BlockHeight::new(1),
        Hash32::from_bytes([0xb0; 32]),
        Hash32::from_bytes([0xcd; 32]),
        1_800_000_000,
        0x1d00_ffff,
        u64::MAX,
    );
    let block = Block::new(header, vec![cb]).expect("blok");
    let txid = Hash32::from_bytes([0x02; 32]);
    let info = BlockInfo::from_block(&block, Hash32::from_bytes([0xb1; 32]), &[txid]);

    assert_eq!(info.block_id, Hex32::new([0xb1; 32]));
    assert_eq!(info.height, 1);
    assert_eq!(info.prev_block_id, Hex32::new([0xb0; 32]));
    assert_eq!(info.merkle_root, Hex32::new([0xcd; 32]));
    assert_eq!(info.timestamp, 1_800_000_000);
    assert_eq!(info.target.get(), 0x1d00_ffff);
    assert_eq!(info.nonce.get(), u64::MAX);
    assert_eq!(info.txids, vec![Hex32::from(txid)]);
    let bytes = block.to_bytes();
    assert_eq!(info.size, u64::try_from(bytes.len()).expect("ukuran"));
    assert_eq!(info.hex.as_slice(), bytes.as_slice());
    assert_eq!(Block::from_bytes(info.hex.as_slice()), Ok(block));

    let json = serde_json::to_string(&info).expect("json");
    assert!(json.contains("\"nonce\":\"18446744073709551615\""));
    assert!(json.contains("\"target\":\"1d00ffff\""));
    assert_eq!(
        serde_json::from_str::<BlockInfo>(&json).expect("json"),
        info
    );
}

#[test]
fn utxo_info_confirmations_and_spendable_at() {
    let out = Output::new(
        Amount::from_quanta(5),
        OutputKind::ConditionHash,
        [0x22; 32],
    )
    .expect("output");
    let op = OutPoint::new(Hash32::from_bytes([0x11; 32]), 3);

    let coinbase = UtxoEntry::new(out.clone(), BlockHeight::new(5), true);
    let info = UtxoInfo::from_entry(&op, &coinbase, 10).expect("info");
    assert_eq!(info.confirmations, 6);
    assert_eq!(info.spendable_at, 105);
    assert_eq!(serde_json::to_string(&info).expect("json"), UTXO_JSON);
    assert_eq!(
        serde_json::from_str::<UtxoInfo>(UTXO_JSON).expect("json"),
        info
    );

    let regular = UtxoEntry::new(out.clone(), BlockHeight::new(5), false);
    assert_eq!(
        UtxoInfo::from_entry(&op, &regular, 5).map(|i| (i.confirmations, i.spendable_at)),
        Some((1, 6))
    );
    assert_eq!(UtxoInfo::from_entry(&op, &regular, 4), None);
    let top = UtxoEntry::new(out, BlockHeight::new(u64::MAX), false);
    assert_eq!(UtxoInfo::from_entry(&op, &top, u64::MAX), None);
}

#[test]
fn submit_block_result_omits_absent_fields() {
    let invalid = SubmitBlockResult {
        status: SubmitStatus::Invalid,
        block_id: None,
        reason: Some("V1".to_owned()),
    };
    assert_eq!(
        serde_json::to_string(&invalid).expect("json"),
        "{\"status\":\"invalid\",\"reason\":\"V1\"}"
    );

    let accepted = SubmitBlockResult {
        status: SubmitStatus::Accepted,
        block_id: Some(Hex32::new([0xb1; 32])),
        reason: None,
    };
    let json = serde_json::to_string(&accepted).expect("json");
    assert_eq!(
        json,
        format!(
            "{{\"status\":\"accepted\",\"block_id\":\"{}\"}}",
            hex_of(0xb1)
        )
    );
    assert_eq!(
        serde_json::from_str::<SubmitBlockResult>(&json).expect("json"),
        accepted
    );

    let names = [
        (SubmitStatus::Duplicate, "duplicate"),
        (SubmitStatus::Orphan, "orphan"),
        (SubmitStatus::Deferred, "deferred"),
    ];
    for (status, name) in names {
        assert_eq!(
            serde_json::to_string(&status).expect("json"),
            format!("\"{name}\"")
        );
    }
}
