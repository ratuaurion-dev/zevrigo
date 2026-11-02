use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use homeostasis_pow::{Target, regtest_pow_hash, verify_regtest_pow};
use homeostasis_primitives::{
    Amount, Block, BlockHeight, Decode, Encode, Hash32, Header, Output, OutputKind, Transaction,
};
use homeostasis_rpc::{
    BlockInfo, BlockTemplate, ChainInfo, CompactHex, GetBlockTemplateParams, Hex32, HexBytes,
    KindNumber, MempoolInfo, MempoolScan, Quanta, RpcBackend, RpcError, RpcServer,
    SendRawTransactionResult, SubmitBlockResult, SubmitStatus, UtxoInfo,
};
use tokio::runtime::Runtime;

use super::{Miner, MinerError, Round};

/// Target compact `pow_limit` regtest.
const LIMIT: u32 = 0x207f_ffff;

/// `pow_limit` regtest sebagai 256-bit big-endian: `0x7fffff` lalu 29 byte nol.
fn limit_bytes() -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    for (dst, src) in bytes.iter_mut().zip([0x7f, 0xff, 0xff]) {
        *dst = src;
    }
    bytes
}

/// Perilaku node tiruan.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Menerima blok yang PoW-nya lolos dan menyambung ke ujung.
    Honest,
    /// Template dengan `full_target` nol: tidak ada nonce yang memenuhi.
    Impossible,
    /// Template berisi byte yang bukan blok.
    Garbage,
    /// Selalu menolak blok sebagai tidak sah.
    Reject,
}

struct State {
    tip_id: Hash32,
    blocks: Vec<Block>,
}

/// Node tiruan: hanya `get_block_template` dan `submit_block` yang berisi.
struct MockNode {
    mode: Mode,
    state: Mutex<State>,
}

impl MockNode {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            state: Mutex::new(State {
                tip_id: Hash32::from_bytes([0xb0; 32]),
                blocks: Vec::new(),
            }),
        }
    }
}

impl RpcBackend for MockNode {
    fn chain_info(&self) -> Result<ChainInfo, RpcError> {
        Err(RpcError::NotFound)
    }

    fn block_hash(&self, _height: u64) -> Result<Hex32, RpcError> {
        Err(RpcError::NotFound)
    }

    fn block(&self, _block_id: Hex32) -> Result<BlockInfo, RpcError> {
        Err(RpcError::NotFound)
    }

    fn block_template(&self, params: &GetBlockTemplateParams) -> Result<BlockTemplate, RpcError> {
        let state = self.state.lock().unwrap();
        let height = u64::try_from(state.blocks.len()).unwrap() + 1;
        let payout = Output::new(
            Amount::from_quanta(50),
            params.payout_kind.kind(),
            *params.payout_payload.bytes(),
        )
        .unwrap();
        let coinbase =
            Transaction::coinbase(BlockHeight::new(height), Vec::new(), vec![payout]).unwrap();
        let header = Header::new(
            BlockHeight::new(height),
            state.tip_id,
            Hash32::ZERO,
            1_800_000_000 + height,
            LIMIT,
            0,
        );
        let block = Block::new(header, vec![coinbase]).unwrap();
        let hex = match self.mode {
            Mode::Garbage => vec![0x00, 0x01],
            Mode::Honest | Mode::Impossible | Mode::Reject => block.to_bytes(),
        };
        let full_target = match self.mode {
            Mode::Impossible => [0_u8; 32],
            Mode::Honest | Mode::Garbage | Mode::Reject => limit_bytes(),
        };
        Ok(BlockTemplate {
            height,
            prev_block_id: Hex32::from(state.tip_id),
            target: CompactHex::new(LIMIT),
            full_target: Hex32::new(full_target),
            min_timestamp: 1_800_000_000,
            subsidy: Quanta::new(Amount::from_quanta(50)),
            fees: Quanta::new(Amount::ZERO),
            hex: HexBytes::new(hex),
        })
    }

    fn submit_block(&self, bytes: &[u8]) -> Result<SubmitBlockResult, RpcError> {
        let invalid = |rule: &str| SubmitBlockResult {
            status: SubmitStatus::Invalid,
            block_id: None,
            reason: Some(rule.to_owned()),
        };
        if self.mode == Mode::Reject {
            return Ok(invalid("V6"));
        }
        let Ok(block) = Block::from_bytes(bytes) else {
            return Ok(invalid("V1"));
        };
        let header = *block.header();
        if verify_regtest_pow(&header, &Target::from_be_bytes(limit_bytes())).is_err() {
            return Ok(invalid("V3"));
        }
        let mut state = self.state.lock().unwrap();
        if header.prev_block_id() != state.tip_id {
            return Ok(SubmitBlockResult {
                status: SubmitStatus::Orphan,
                block_id: None,
                reason: None,
            });
        }
        let id = regtest_pow_hash(&header);
        state.tip_id = id;
        state.blocks.push(block);
        Ok(SubmitBlockResult {
            status: SubmitStatus::Accepted,
            block_id: Some(Hex32::from(id)),
            reason: None,
        })
    }

    fn send_raw_transaction(&self, _tx: &[u8]) -> Result<SendRawTransactionResult, RpcError> {
        Err(RpcError::InvalidEncoding)
    }

    fn mempool(&self) -> Result<MempoolInfo, RpcError> {
        Ok(MempoolInfo { txids: Vec::new() })
    }

    fn scan_utxos(&self, _payloads: &[Hex32]) -> Result<Vec<UtxoInfo>, RpcError> {
        Ok(Vec::new())
    }

    fn scan_mempool(&self, _payloads: &[Hex32]) -> Result<MempoolScan, RpcError> {
        Ok(MempoolScan::default())
    }
}

/// Node tiruan yang berjalan di runtime terpisah, dengan miner yang terhubung ke sana.
struct Harness {
    server_rt: Runtime,
    server: Option<RpcServer>,
    node: Arc<MockNode>,
    miner: Miner,
}

impl Harness {
    fn start(mode: Mode, attempts: u64) -> Self {
        let server_rt = Runtime::new().unwrap();
        let node = Arc::new(MockNode::new(mode));
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let server = server_rt
            .block_on(RpcServer::start(addr, Arc::clone(&node)))
            .unwrap();
        let url = format!("http://{}", server.local_addr());
        let miner = Miner::connect(
            &url,
            KindNumber::new(OutputKind::SingleKey),
            Hex32::new([0x22; 32]),
            attempts,
        )
        .unwrap();
        Self {
            server_rt,
            server: Some(server),
            node,
            miner,
        }
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        if let Some(server) = self.server.take() {
            self.server_rt.block_on(server.stop());
        }
    }
}

#[test]
fn mines_chain_of_blocks_over_rpc() {
    let harness = Harness::start(Mode::Honest, 1_000_000);
    let mut rounds = Vec::new();
    let count = harness
        .miner
        .run(3, |round| rounds.push(round.clone()))
        .unwrap();
    assert_eq!(count, 3);
    assert_eq!(rounds.len(), 3);

    let state = harness.node.state.lock().unwrap();
    assert_eq!(state.blocks.len(), 3);
    let limit = Target::from_be_bytes(limit_bytes());
    let mut prev = Hash32::from_bytes([0xb0; 32]);
    for ((block, round), height) in state.blocks.iter().zip(&rounds).zip(1_u64..) {
        let header = block.header();
        assert_eq!(header.height().get(), height);
        assert_eq!(header.prev_block_id(), prev);
        assert!(verify_regtest_pow(header, &limit).is_ok());
        let payout = block.txs()[0].body().outputs()[0].clone();
        assert_eq!(payout.payload(), &[0x22; 32]);
        assert_eq!(payout.amount(), Amount::from_quanta(50));
        prev = regtest_pow_hash(header);
        assert_eq!(
            round,
            &Round::Accepted {
                height,
                block_id: Some(Hex32::from(prev)),
                nonce: header.nonce(),
            }
        );
    }
    assert_eq!(state.tip_id, prev);
}

#[test]
fn unreachable_target_exhausts_round() {
    let harness = Harness::start(Mode::Impossible, 16);
    assert_eq!(harness.miner.round().unwrap(), Round::Exhausted);
    assert!(harness.node.state.lock().unwrap().blocks.is_empty());
}

#[test]
fn bad_template_and_rejection_are_errors() {
    let garbage = Harness::start(Mode::Garbage, 1_000_000);
    assert!(matches!(
        garbage.miner.round(),
        Err(MinerError::BadTemplate { .. })
    ));

    let reject = Harness::start(Mode::Reject, 1_000_000);
    let error = reject.miner.round().unwrap_err();
    assert!(matches!(&error, MinerError::Rejected { reason } if reason == "V6"));
    assert_eq!(error.to_string(), "node menolak blok: aturan V6");
    assert!(matches!(
        reject.miner.run(1, |_| {}),
        Err(MinerError::Rejected { .. })
    ));
}

#[test]
fn unreachable_node_is_rpc_error() {
    // Port bebas yang langsung ditutup lagi, sehingga koneksi ditolak.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let miner = Miner::connect(
        &format!("http://{addr}"),
        KindNumber::new(OutputKind::SingleKey),
        Hex32::new([0x22; 32]),
        16,
    )
    .unwrap();
    assert!(matches!(miner.round(), Err(MinerError::Rpc { .. })));
    assert!(matches!(
        Miner::connect(
            "bukan url",
            KindNumber::new(OutputKind::SingleKey),
            Hex32::new([0; 32]),
            1
        ),
        Err(MinerError::Rpc { .. })
    ));
}
