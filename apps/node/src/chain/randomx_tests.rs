//! Uji PoW RandomX di node (TSK-M06 bagian B): kunci diambil dari chain blok itu sendiri (chain aktif, blok cabang,
//! indeks header, dan presync), lalu layanan RandomX sungguhan.

use std::sync::Arc;

use homeostasis_consensus::{
    ChainParams, RandomxHasher, ScheduleEntry, SupplySchedule, check_block_pow, randomx_hasher,
    randomx_key_height,
};
use homeostasis_crypto::{Domain, SecretKey, block_id, block_merkle_root, hash};
use homeostasis_mempool::DEFAULT_MAX_BYTES;
use homeostasis_primitives::params::GENESIS_ALLOCATION;
use homeostasis_primitives::{
    Amount, Block, BlockHeight, Encode, Hash32, Header, Network, Output, OutputKind, Transaction,
};
use homeostasis_rpc::{
    GetBlockTemplateParams, Hex32, KindNumber, RpcBackend, SubmitBlockResult, SubmitStatus,
};
use homeostasis_storage::MemoryStore;

use super::{Clock, HeadersOutcome, Node, NodeError, NodeLimits, Presync, PresyncOutcome, Randomx};

/// Cap waktu genesis regtest (genesis yang sama dipakai parameter uji testnet).
const T0: u64 = 1_793_145_600;
/// Target genesis regtest dan `pow_limit`.
const LIMIT: u32 = 0x207f_ffff;
/// Jam node: semua blok uji bercap waktu `T0 + 120 × h` (h ≤ 2_200) lolos V8.
const NOW: u64 = T0 + 120 * 3_000;
/// Tinggi titik cabang B.
const FORK: usize = 2_040;
/// Tinggi ujung chain A.
const A_TIP: usize = 2_120;

/// Penghitung uji: BLAKE3 dari kunci ‖ header, sehingga kunci yang salah memberi hash lain.
struct Keyed;

impl RandomxHasher for Keyed {
    fn randomx_hash(&self, key: &Hash32, header: &Header) -> Option<Hash32> {
        let mut bytes = key.as_bytes().to_vec();
        bytes.extend_from_slice(&header.to_array());
        Some(hash(Domain::RegtestPow, &bytes))
    }
}

/// Parameter testnet uji: aturan testnet (PoW RandomX) di atas genesis dan target regtest.
fn testnet() -> ChainParams {
    let regtest = ChainParams::regtest().unwrap();
    ChainParams::new(
        Network::Testnet,
        regtest.pow_limit(),
        regtest.genesis_target(),
        regtest.genesis_timestamp(),
        regtest.network_id(),
    )
}

fn open_testnet(randomx: Randomx, limits: NodeLimits) -> Node<MemoryStore> {
    let clock: Clock = Box::new(|| NOW);
    Node::open_with_pow(
        MemoryStore::default(),
        testnet(),
        DEFAULT_MAX_BYTES,
        clock,
        limits,
        Some(randomx),
    )
    .unwrap()
}

fn payload(owner: u8) -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    if let Some(last) = bytes.last_mut() {
        *last = owner;
    }
    *SecretKey::from_bytes(&bytes)
        .unwrap()
        .public_key()
        .as_bytes()
}

/// Tinggi kunci RandomX untuk tinggi `height`, sebagai indeks.
fn key_index(height: usize) -> usize {
    let key = randomx_key_height(BlockHeight::new(u64::try_from(height).unwrap()));
    usize::try_from(key.get()).unwrap()
}

/// Penambang uji: blok berisi coinbase saja ke `owner`, cap waktu sesuai jadwal (target tetap `pow_limit`).
struct Miner<'a> {
    hasher: &'a dyn RandomxHasher,
    params: ChainParams,
    schedule: Vec<ScheduleEntry>,
}

impl<'a> Miner<'a> {
    fn new(hasher: &'a dyn RandomxHasher) -> Self {
        Self {
            hasher,
            params: testnet(),
            schedule: SupplySchedule::new().take(2_200).collect(),
        }
    }

    fn draft(&self, prev: Hash32, height: usize, owner: u8, nonce: u64) -> Block {
        let at = BlockHeight::new(u64::try_from(height).unwrap());
        let reward = self.schedule[height].subsidy;
        let output = Output::new(reward, OutputKind::SingleKey, payload(owner)).unwrap();
        let coinbase = Transaction::coinbase(at, Vec::new(), vec![output]).unwrap();
        let timestamp = T0 + 120 * at.get();
        let header = Header::new(at, prev, Hash32::ZERO, timestamp, LIMIT, nonce);
        let draft = Block::new(header, vec![coinbase.clone()]).unwrap();
        let merkle = block_merkle_root(&draft).unwrap();
        Block::new(
            Header::new(at, prev, merkle, timestamp, LIMIT, nonce),
            vec![coinbase],
        )
        .unwrap()
    }

    fn passes(&self, block: &Block, key: &Hash32) -> bool {
        check_block_pow(block, &self.params, key, Some(self.hasher)).is_ok()
    }

    /// Blok dengan nonce pertama yang lolos V3 dengan kunci `key`.
    fn mine(&self, prev: Hash32, height: usize, owner: u8, key: &Hash32) -> Block {
        (0_u64..)
            .map(|nonce| self.draft(prev, height, owner, nonce))
            .find(|block| self.passes(block, key))
            .unwrap()
    }

    /// Blok yang lolos V3 dengan kunci `wrong`, tetapi tidak dengan kunci `right`.
    fn mine_wrong(
        &self,
        prev: Hash32,
        height: usize,
        owner: u8,
        wrong: &Hash32,
        right: &Hash32,
    ) -> Block {
        (0_u64..)
            .map(|nonce| self.draft(prev, height, owner, nonce))
            .find(|block| self.passes(block, wrong) && !self.passes(block, right))
            .unwrap()
    }

    /// Menambang di atas `ids` (ID blok per tinggi, mulai genesis) sampai tinggi `tip`; `ids` ikut bertambah. Kunci
    /// setiap blok: ID di tinggi kuncinya pada `ids`.
    fn grow(&self, ids: &mut Vec<Hash32>, tip: usize, owner: u8) -> Vec<Block> {
        let mut blocks = Vec::new();
        while ids.len() <= tip {
            let height = ids.len();
            let key = ids[key_index(height)];
            let block = self.mine(*ids.last().unwrap(), height, owner, &key);
            ids.push(block_id(block.header()));
            blocks.push(block);
        }
        blocks
    }
}

fn submit(node: &Node<MemoryStore>, block: &Block) -> SubmitBlockResult {
    node.submit_block(&block.to_bytes()).unwrap()
}

fn accepted(node: &Node<MemoryStore>, blocks: &[Block]) {
    for block in blocks {
        assert_eq!(
            submit(node, block).status,
            SubmitStatus::Accepted,
            "tinggi {}",
            block.header().height()
        );
    }
}

fn tip(node: &Node<MemoryStore>) -> Hex32 {
    node.chain_info().unwrap().best_block_id
}

/// Kunci RandomX di template blok berikutnya dari `node`.
fn template_key(node: &Node<MemoryStore>) -> Hex32 {
    let params = GetBlockTemplateParams {
        payout_kind: KindNumber::new(OutputKind::SingleKey),
        payout_payload: Hex32::new(payload(7)),
    };
    node.block_template(&params).unwrap().pow_key
}

fn headers_of(blocks: &[Block]) -> Vec<Header> {
    blocks.iter().map(|block| *block.header()).collect()
}

#[test]
fn randomx_key_comes_from_the_blocks_own_branch() {
    let hasher: Randomx = Arc::new(Keyed);
    let miner = Miner::new(hasher.as_ref());
    let genesis = testnet().network_id();
    let mut ids_a = vec![genesis];
    let a = miner.grow(&mut ids_a, A_TIP, 2);
    let mut ids_b = ids_a[..=FORK].to_vec();
    let b = miner.grow(&mut ids_b, A_TIP + 2, 4);
    assert_eq!(b.len(), A_TIP + 2 - FORK);
    // Sejak tinggi 2_113 kuncinya blok di tinggi 2_048, yang berbeda di chain A dan cabang B.
    assert_eq!(key_index(2_112), 0);
    assert_eq!(key_index(2_113), 2_048);
    assert_ne!(ids_a[2_048], ids_b[2_048]);

    // Blok A, header B lewat indeks header, lalu blok B sebagai cabang sampai reorg.
    let node = open_testnet(Arc::clone(&hasher), NodeLimits::DEFAULT);
    accepted(&node, &a);
    assert_eq!(tip(&node), Hex32::from(ids_a[A_TIP]));
    // Template tinggi 2_121: kunci blok A di tinggi 2_048.
    assert_eq!(template_key(&node), Hex32::from(ids_a[2_048]));
    let headers_b = headers_of(&b);
    assert_eq!(
        node.accept_headers(&headers_b).unwrap(),
        HeadersOutcome::Accepted {
            added: b.len(),
            last: ids_b.last().copied()
        }
    );
    // Kunci chain lain ditolak V3: header di atas indeks header, blok di atas blok cabang dan di atas chain aktif.
    let wrong_b = miner.mine_wrong(ids_b[2_112], 2_113, 5, &ids_a[2_048], &ids_b[2_048]);
    assert_eq!(
        node.accept_headers(&[*wrong_b.header()]).unwrap(),
        HeadersOutcome::Invalid { rule: "V3" }
    );
    accepted(&node, &b[..A_TIP - FORK]);
    assert_eq!(tip(&node), Hex32::from(ids_a[A_TIP]));
    assert_eq!(submit(&node, &wrong_b).reason.as_deref(), Some("V3"));
    let stale = miner.mine_wrong(ids_a[A_TIP], A_TIP + 1, 6, &genesis, &ids_a[2_048]);
    assert_eq!(submit(&node, &stale).reason.as_deref(), Some("V3"));
    accepted(&node, &b[A_TIP - FORK..]);
    assert_eq!(tip(&node), Hex32::from(ids_b[A_TIP + 2]));
    // Sesudah reorg, template mengikuti chain B.
    assert_eq!(template_key(&node), Hex32::from(ids_b[2_048]));

    // Presync: tahap hitung mencatat ID header B di tinggi 2_048 sebagai kunci header B sesudah 2_112.
    let limited = open_testnet(
        Arc::clone(&hasher),
        NodeLimits {
            max_headers: 3,
            ..NodeLimits::DEFAULT
        },
    );
    accepted(&limited, &a);
    assert_eq!(
        limited.accept_headers(&headers_b).unwrap(),
        HeadersOutcome::Full { added: 3 }
    );
    let mut presync = Presync::default();
    assert_eq!(
        limited.presync_headers(&mut presync, &headers_b).unwrap(),
        PresyncOutcome::Redownload { fork: ids_a[FORK] }
    );
    assert_eq!(
        limited.presync_headers(&mut presync, &headers_b).unwrap(),
        PresyncOutcome::Done {
            added: A_TIP + 1 - FORK - 3
        }
    );
    assert_eq!(
        limited.missing_blocks(200).unwrap(),
        ids_b[FORK + 1..=A_TIP + 1].to_vec()
    );
    accepted(&limited, &b[..=A_TIP - FORK]);
    assert_eq!(tip(&limited), Hex32::from(ids_b[A_TIP + 1]));
}

#[test]
fn testnet_node_checks_real_randomx() {
    let hasher = randomx_hasher().unwrap();
    let miner = Miner::new(hasher.as_ref());
    let genesis = testnet().network_id();
    let mut ids = vec![genesis];
    let blocks = miner.grow(&mut ids, 2, 2);
    let node = open_testnet(Arc::clone(&hasher), NodeLimits::DEFAULT);
    accepted(&node, &blocks);
    assert_eq!(tip(&node), Hex32::from(ids[2]));
    let other = Hash32::from_bytes([0x77; 32]);
    let wrong = miner.mine_wrong(ids[2], 3, 3, &other, &genesis);
    assert_eq!(submit(&node, &wrong).reason.as_deref(), Some("V3"));

    // Node::open memulai layanan RandomX sendiri untuk testnet.
    let own = Node::open(
        MemoryStore::default(),
        testnet(),
        DEFAULT_MAX_BYTES,
        Box::new(|| NOW),
    )
    .unwrap();
    accepted(&own, &blocks[..1]);
}

#[test]
fn opens_testnet_genesis_and_rejects_unknown_genesis() {
    let params = ChainParams::testnet().unwrap();
    let node = Node::open(
        MemoryStore::default(),
        params,
        DEFAULT_MAX_BYTES,
        Box::new(|| NOW),
    )
    .unwrap();
    let info = node.chain_info().unwrap();
    assert_eq!(info.network.network(), Network::Testnet);
    assert_eq!(info.height, 0);
    assert_eq!(info.best_block_id, Hex32::from(params.network_id()));
    // Blok 1 di atas genesis bercap waktu 0 memakai target genesis (ASERT tepat jadwal); kuncinya genesis.
    assert_eq!(info.next_target.get(), 0x1e8b_cf64);
    assert_eq!(template_key(&node), Hex32::from(params.network_id()));
    assert_eq!(info.median_time_past, 0);
    assert_eq!(info.mined_total.amount(), Amount::ZERO);
    assert_eq!(info.total_supply.amount(), GENESIS_ALLOCATION);

    let unknown = ChainParams::new(
        Network::Testnet,
        params.pow_limit(),
        params.genesis_target(),
        params.genesis_timestamp(),
        Hash32::from_bytes([0x55; 32]),
    );
    let clock: Clock = Box::new(|| NOW);
    let result = Node::open_with_pow(
        MemoryStore::default(),
        unknown,
        DEFAULT_MAX_BYTES,
        clock,
        NodeLimits::DEFAULT,
        None,
    );
    assert!(matches!(result.err(), Some(NodeError::Genesis { .. })));
}
