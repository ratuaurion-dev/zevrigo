use std::sync::Arc;

use homeostasis_consensus::{BlockError, ChainParams, TxError, check_block_context_free};
use homeostasis_crypto::block_id;
use homeostasis_mempool::{DEFAULT_MAX_BYTES, MempoolError};
use homeostasis_network::Ack;
use homeostasis_primitives::{
    Block, BlockHeight, Decode, Encode, Hash32, Header, OutPoint, OutputKind,
};
use homeostasis_rpc::{GetBlockTemplateParams, Hex32, KindNumber, RpcBackend, SubmitStatus};
use homeostasis_storage::MemoryStore;
use tokio::sync::mpsc;

use super::{Announcement, Relay, block_ack, local_status, tx_ack};
use crate::chain::{Node, TxOutcome};

/// Cap waktu genesis regtest.
const T0: u64 = 1_793_145_600;

fn node() -> Arc<Node<MemoryStore>> {
    let node = Node::open(
        MemoryStore::default(),
        ChainParams::regtest().unwrap(),
        DEFAULT_MAX_BYTES,
        Box::new(|| T0 + 120),
    )
    .unwrap();
    Arc::new(node)
}

/// Blok tinggi 1 dari template node, dengan nonce yang lolos PoW regtest.
fn mined(node: &Node<MemoryStore>) -> Block {
    let payout = GetBlockTemplateParams {
        payout_kind: KindNumber::new(OutputKind::SingleKey),
        payout_payload: Hex32::new([7; 32]),
    };
    let template = node.block_template(&payout).unwrap();
    let draft = Block::from_bytes(template.hex.as_slice()).unwrap();
    let params = ChainParams::regtest().unwrap();
    let h = *draft.header();
    (0_u64..)
        .map(|nonce| {
            let header = Header::new(
                h.height(),
                h.prev_block_id(),
                h.merkle_root(),
                h.timestamp(),
                h.target(),
                nonce,
            );
            Block::new(header, draft.txs().to_vec()).unwrap()
        })
        .find(|candidate| {
            !matches!(
                check_block_context_free(candidate, &params),
                Err(BlockError::HighHash)
            )
        })
        .unwrap()
}

#[test]
fn acks_follow_spec_table() {
    assert_eq!(block_ack(SubmitStatus::Accepted), Ack::Accepted);
    assert_eq!(block_ack(SubmitStatus::Duplicate), Ack::Accepted);
    assert_eq!(block_ack(SubmitStatus::Orphan), Ack::UnknownParent);
    assert_eq!(block_ack(SubmitStatus::Deferred), Ack::Deferred);
    assert_eq!(block_ack(SubmitStatus::Invalid), Ack::Invalid);

    let outpoint = OutPoint::new(Hash32::from_bytes([1; 32]), 0);
    let invalid = |error: TxError| TxOutcome::Rejected(MempoolError::Invalid(error));
    let cases = [
        (TxOutcome::Added, Ack::Accepted),
        (TxOutcome::Duplicate, Ack::Accepted),
        (
            TxOutcome::Rejected(MempoolError::Full {
                size: 1,
                used: 2,
                limit: 2,
            }),
            Ack::Rejected,
        ),
        (
            TxOutcome::Rejected(MempoolError::TooLarge { size: 200_000 }),
            Ack::Rejected,
        ),
        (
            TxOutcome::Rejected(MempoolError::Conflict {
                outpoint,
                existing: Hash32::ZERO,
            }),
            Ack::Rejected,
        ),
        (invalid(TxError::MissingUtxo { outpoint }), Ack::Rejected),
        (
            invalid(TxError::ImmatureCoinbase { outpoint }),
            Ack::Rejected,
        ),
        (
            invalid(TxError::ConditionLocked { input_index: 0 }),
            Ack::Rejected,
        ),
        (invalid(TxError::DuplicateInput { outpoint }), Ack::Invalid),
        (
            invalid(TxError::InvalidSignature { input_index: 0 }),
            Ack::Invalid,
        ),
        (invalid(TxError::OutputsExceedInputs), Ack::Invalid),
        (invalid(TxError::NotRegular), Ack::Invalid),
    ];
    for (outcome, ack) in cases {
        assert_eq!(tx_ack(&outcome), ack, "{outcome:?}");
    }
}

#[test]
fn relay_announces_new_tips_only() {
    let node = node();
    let (sender, mut receiver) = mpsc::unbounded_channel();
    let relay = Relay::new(Arc::clone(&node), sender);
    let genesis = ChainParams::regtest().unwrap().network_id();
    let status = local_status(&node).unwrap();
    assert_eq!(status.network_id, genesis);
    assert_eq!(status.tip_height, BlockHeight::new(0));
    assert_eq!(status.tip_id, genesis);
    assert_eq!(relay.chain_info().unwrap(), node.chain_info().unwrap());

    let block = mined(&node);
    let result = relay.submit_block(&block.to_bytes()).unwrap();
    assert_eq!(result.status, SubmitStatus::Accepted);
    assert_eq!(
        receiver.try_recv().unwrap(),
        Announcement::Block(Box::new(block.clone()))
    );
    // Duplikat dan byte rusak tidak disiarkan.
    let again = relay.submit_block(&block.to_bytes()).unwrap();
    assert_eq!(again.status, SubmitStatus::Duplicate);
    assert_eq!(
        relay.submit_block(&[0]).unwrap().status,
        SubmitStatus::Invalid
    );
    assert!(receiver.try_recv().is_err());

    let status = local_status(&node).unwrap();
    assert_eq!(status.tip_height, BlockHeight::new(1));
    assert_eq!(status.tip_id, block_id(block.header()));
    assert_eq!(status.chainwork, node.tip_summary().unwrap().chainwork);

    // Transaksi rusak ditolak tanpa siaran; mempool tetap kosong.
    assert!(relay.send_raw_transaction(&[0]).is_err());
    assert!(receiver.try_recv().is_err());
    assert!(relay.mempool().unwrap().txids.is_empty());
}
