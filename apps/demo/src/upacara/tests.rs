use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::runtime::Runtime;
use zevrigo_crypto::{PublicKey, SecretKey, Signature, sighash_v1, txid, verify};
use zevrigo_primitives::params::GENESIS_ALLOCATION;
use zevrigo_primitives::{
    Amount, Decode, Encode, Hash32, Network, Output, OutputKind, Transaction, TxBody, Witness,
};
use zevrigo_rpc::{
    BlockInfo, BlockTemplate, ChainInfo, CompactHex, GetBlockTemplateParams, Hex32, KindNumber,
    MempoolInfo, MempoolScan, NetworkName, Quanta, RpcBackend, RpcClient, RpcError, RpcServer,
    SendRawTransactionResult, SubmitBlockResult, UtxoInfo,
};

use super::{
    GENESIS_PAYLOAD, GenesisUtxo, MAX_POCKETS, Pocket, REGTEST_NETWORK_ID, UpacaraError,
    build_ceremony, format_report, genesis_from_scan, pocket_address, run,
};

const ZVRG: u128 = 1_000_000_000_000_000_000;

fn key(n: u8) -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    if let Some(last) = bytes.last_mut() {
        *last = n;
    }
    *SecretKey::from_bytes(&bytes)
        .unwrap()
        .public_key()
        .as_bytes()
}

fn zvrg(n: u128) -> Amount {
    Amount::from_quanta(n * ZVRG)
}

/// Delapan kantong contoh demo (Rancangan Ekosistem Demo v0.1 Bagian 4.2), jumlah tepat G.
fn pockets() -> Vec<Pocket> {
    [
        1_000_000, 1_000_000, 1_500_000, 1_500_000, 1_000_000, 1_500_000, 2_000_000, 500_000,
    ]
    .iter()
    .enumerate()
    .map(|(i, amount)| Pocket {
        payload: key(u8::try_from(i).unwrap() + 2),
        amount: zvrg(*amount),
    })
    .collect()
}

fn genesis() -> GenesisUtxo {
    GenesisUtxo {
        txid: Hash32::from_bytes([0x6a; 32]),
        amount: GENESIS_ALLOCATION,
        spendable_at: 100,
    }
}

fn network_id() -> Hash32 {
    Hash32::from_bytes(REGTEST_NETWORK_ID)
}

#[test]
fn constants_match_specification() {
    assert_eq!(GENESIS_PAYLOAD, key(1));
    assert_eq!(
        String::from(Hex32::new(GENESIS_PAYLOAD)),
        "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
    );
    assert_eq!(
        String::from(Hex32::new(REGTEST_NETWORK_ID)),
        "8b9eb3187250bbdfe1577ebcb668c30003213a9ee6a4424427b08e38da364dab"
    );
    assert_eq!(GENESIS_ALLOCATION, zvrg(10_000_000));
    assert_eq!(
        pockets().iter().map(|p| p.amount.as_quanta()).sum::<u128>(),
        GENESIS_ALLOCATION.as_quanta()
    );
}

#[test]
fn ceremony_spends_genesis_to_pockets_with_valid_signature() {
    let pockets = pockets();
    let ceremony = build_ceremony(&genesis(), &pockets, &network_id(), 1).unwrap();
    let TxBody::Regular(body) = ceremony.tx.body() else {
        unreachable!("transaksi upacara bukan coinbase");
    };
    assert_eq!(body.inputs().len(), 1);
    assert_eq!(body.inputs()[0].prev_txid(), genesis().txid);
    assert_eq!(body.inputs()[0].prev_index(), 0);
    assert_eq!(body.outputs().len(), 8);
    // Biaya = 1 quanta per byte dan diambil dari kantong terakhir; kantong lain utuh.
    assert_eq!(ceremony.size, ceremony.tx.to_bytes().len());
    assert_eq!(
        ceremony.fee.as_quanta(),
        u128::try_from(ceremony.size).unwrap()
    );
    for (i, (output, pocket)) in body.outputs().iter().zip(&pockets).enumerate() {
        assert_eq!(output.payload(), &pocket.payload);
        assert_eq!(output.kind(), OutputKind::SingleKey);
        assert_eq!(output.amount(), ceremony.received[i]);
        if i < 7 {
            assert_eq!(output.amount(), pocket.amount);
        }
    }
    let sum: u128 = body.outputs().iter().map(|o| o.amount().as_quanta()).sum();
    assert_eq!(
        sum + ceremony.fee.as_quanta(),
        GENESIS_ALLOCATION.as_quanta()
    );
    // Tanda tangan BIP340 kunci G atas sighash v1 regtest; txid dan encoding kanonis konsisten.
    let spent = Output::new(GENESIS_ALLOCATION, OutputKind::SingleKey, GENESIS_PAYLOAD).unwrap();
    let msg = sighash_v1(&network_id(), &ceremony.tx, 0, std::slice::from_ref(&spent)).unwrap();
    let [Witness::SingleKey(sig)] = ceremony.tx.witnesses() else {
        unreachable!("upacara memakai satu witness kunci tunggal");
    };
    let public = PublicKey::from_bytes(&GENESIS_PAYLOAD).unwrap();
    assert!(verify(&public, msg.as_bytes(), &Signature::from_bytes(sig)));
    assert_eq!(ceremony.txid, txid(&ceremony.tx));
    assert_eq!(
        Transaction::from_bytes(&ceremony.tx.to_bytes()).unwrap(),
        ceremony.tx
    );
    // Deterministik: aux_rand nol.
    assert_eq!(
        build_ceremony(&genesis(), &pockets, &network_id(), 1).unwrap(),
        ceremony
    );
    // Tarif lebih tinggi: biaya naik, kantong terakhir turun sebesar itu.
    let higher = build_ceremony(&genesis(), &pockets, &network_id(), 3).unwrap();
    assert_eq!(
        higher.fee.as_quanta(),
        3 * u128::try_from(higher.size).unwrap()
    );
    assert_eq!(
        higher.received[7].as_quanta() + higher.fee.as_quanta(),
        pockets[7].amount.as_quanta()
    );
}

#[test]
fn ceremony_rejects_bad_pockets() {
    let base = pockets();
    assert_eq!(
        build_ceremony(&genesis(), &[], &network_id(), 1),
        Err(UpacaraError::PocketCount(0))
    );
    let many: Vec<Pocket> = (0..=MAX_POCKETS)
        .map(|i| Pocket {
            payload: key(u8::try_from(i).unwrap() + 2),
            amount: Amount::from_quanta(1),
        })
        .collect();
    assert_eq!(
        build_ceremony(&genesis(), &many, &network_id(), 1),
        Err(UpacaraError::PocketCount(17))
    );
    let mut duplicate = base.clone();
    duplicate[1].payload = duplicate[0].payload;
    assert!(matches!(
        build_ceremony(&genesis(), &duplicate, &network_id(), 1),
        Err(UpacaraError::DuplicatePocket(address)) if address == pocket_address(&base[0])
    ));
    let mut short = base.clone();
    short[0].amount = zvrg(999_999);
    assert!(matches!(
        build_ceremony(&genesis(), &short, &network_id(), 1),
        Err(UpacaraError::Total { .. })
    ));
    // Kantong terakhir terlalu kecil untuk biaya.
    let mut tiny = base.clone();
    tiny[6].amount =
        Amount::from_quanta(tiny[6].amount.as_quanta() + tiny[7].amount.as_quanta() - 10);
    tiny[7].amount = Amount::from_quanta(10);
    assert!(matches!(
        build_ceremony(&genesis(), &tiny, &network_id(), 1),
        Err(UpacaraError::FeeTooLarge { .. })
    ));
    // Output G yang bukan alokasi genesis.
    let mut other = genesis();
    other.amount = zvrg(1);
    assert!(matches!(
        build_ceremony(&other, &base, &network_id(), 1),
        Err(UpacaraError::Total { .. })
    ));
}

fn utxo(height: u64, index: u32, amount: Amount, coinbase: bool) -> UtxoInfo {
    UtxoInfo {
        txid: Hex32::new([0x6a; 32]),
        index,
        amount: Quanta::new(amount),
        kind: KindNumber::new(OutputKind::SingleKey),
        payload: Hex32::new(GENESIS_PAYLOAD),
        height,
        is_coinbase: coinbase,
        confirmations: 121,
        spendable_at: height + if coinbase { 100 } else { 1 },
    }
}

#[test]
fn genesis_scan_accepts_only_the_genesis_output() {
    assert_eq!(genesis_from_scan(&[]), Err(UpacaraError::GenesisMissing));
    let found = genesis_from_scan(&[
        utxo(5, 0, zvrg(1), false),
        utxo(0, 0, GENESIS_ALLOCATION, true),
    ])
    .unwrap();
    assert_eq!(found, genesis());
    for bad in [
        utxo(0, 1, GENESIS_ALLOCATION, true),
        utxo(0, 0, GENESIS_ALLOCATION, false),
        utxo(0, 0, zvrg(1), true),
    ] {
        assert_eq!(
            genesis_from_scan(&[bad]),
            Err(UpacaraError::GenesisMismatch)
        );
    }
    assert_eq!(
        genesis_from_scan(&[utxo(3, 0, zvrg(1), false)]),
        Err(UpacaraError::GenesisMismatch)
    );
}

/// Node tiruan: tinggi dan isi set UTXO bisa diatur; transaksi yang dikirim dicatat.
struct Node {
    network: Network,
    height: u64,
    genesis_id: [u8; 32],
    utxos: Vec<UtxoInfo>,
    sent: Mutex<Vec<Vec<u8>>>,
}

impl Node {
    fn regtest(height: u64) -> Self {
        Self {
            network: Network::Regtest,
            height,
            genesis_id: REGTEST_NETWORK_ID,
            utxos: vec![utxo(0, 0, GENESIS_ALLOCATION, true)],
            sent: Mutex::new(Vec::new()),
        }
    }
}

impl RpcBackend for Node {
    fn chain_info(&self) -> Result<ChainInfo, RpcError> {
        Ok(ChainInfo {
            network: NetworkName::new(self.network),
            height: self.height,
            best_block_id: Hex32::new([0xbb; 32]),
            chainwork: Hex32::new([0; 32]),
            next_target: CompactHex::new(0x207f_ffff),
            median_time_past: 1_793_145_600,
            mined_total: Quanta::new(Amount::ZERO),
            total_supply: Quanta::new(GENESIS_ALLOCATION),
            mempool_count: 0,
        })
    }

    fn block_hash(&self, height: u64) -> Result<Hex32, RpcError> {
        if height == 0 {
            Ok(Hex32::new(self.genesis_id))
        } else {
            Err(RpcError::NotFound)
        }
    }

    fn block(&self, _block_id: Hex32) -> Result<BlockInfo, RpcError> {
        Err(RpcError::NotFound)
    }

    fn block_template(&self, _params: &GetBlockTemplateParams) -> Result<BlockTemplate, RpcError> {
        Err(RpcError::NotFound)
    }

    fn submit_block(&self, _block: &[u8]) -> Result<SubmitBlockResult, RpcError> {
        Err(RpcError::NotFound)
    }

    fn send_raw_transaction(&self, tx: &[u8]) -> Result<SendRawTransactionResult, RpcError> {
        let decoded = Transaction::from_bytes(tx).map_err(|_| RpcError::InvalidEncoding)?;
        self.sent.lock().unwrap().push(tx.to_vec());
        Ok(SendRawTransactionResult {
            txid: Hex32::from(txid(&decoded)),
        })
    }

    fn mempool(&self) -> Result<MempoolInfo, RpcError> {
        Ok(MempoolInfo { txids: Vec::new() })
    }

    fn scan_utxos(&self, payloads: &[Hex32]) -> Result<Vec<UtxoInfo>, RpcError> {
        Ok(self
            .utxos
            .iter()
            .filter(|u| payloads.contains(&u.payload))
            .cloned()
            .collect())
    }

    fn scan_mempool(&self, _payloads: &[Hex32]) -> Result<MempoolScan, RpcError> {
        Ok(MempoolScan::default())
    }
}

/// Menjalankan upacara terhadap `node`; mengembalikan hasil dan transaksi yang diterima node.
fn against(node: Node, send: bool) -> (Result<super::Report, UpacaraError>, Vec<Vec<u8>>) {
    let rt = Runtime::new().unwrap();
    let node = Arc::new(node);
    let server = rt
        .block_on(RpcServer::start(
            SocketAddr::from(([127, 0, 0, 1], 0)),
            Arc::clone(&node),
        ))
        .unwrap();
    let url = format!("http://{}", server.local_addr());
    let client = rt.block_on(async { RpcClient::new(&url) }).unwrap();
    let result = rt.block_on(run(&client, &pockets(), 1, send));
    rt.block_on(server.stop());
    let sent = node.sent.lock().unwrap().clone();
    (result, sent)
}

#[test]
fn run_checks_node_then_plans_or_sends() {
    // Tanpa --kirim: rencana saja, node tidak menerima apa pun. Tinggi 99 sudah cukup (blok berikutnya = 100).
    let (planned, sent) = against(Node::regtest(99), false);
    let planned = planned.unwrap();
    assert!(!planned.sent && sent.is_empty());
    assert_eq!(planned.tip_height, 99);
    let text = format_report(&planned);
    assert!(text.contains("rencana saja"), "{text}");
    assert!(text.contains(&pocket_address(&pockets()[0])), "{text}");
    assert!(text.contains("(dikurangi biaya)"), "{text}");
    assert!(
        text.contains(&String::from(Hex32::from(planned.ceremony.txid))),
        "{text}"
    );
    // Dengan --kirim: tepat satu transaksi, byte sama dengan rencana.
    let (done, sent) = against(Node::regtest(130), true);
    let done = done.unwrap();
    assert!(done.sent);
    assert_eq!(sent, vec![done.ceremony.tx.to_bytes()]);
    assert_eq!(done.ceremony, planned.ceremony);
    assert!(format_report(&done).contains("status: dikirim"));
}

#[test]
fn run_refuses_wrong_node_states() {
    assert_eq!(
        against(Node::regtest(98), true).0,
        Err(UpacaraError::NotMature {
            height: 98,
            spendable_at: 100
        })
    );
    let mut spent = Node::regtest(150);
    spent.utxos.clear();
    assert_eq!(against(spent, true).0, Err(UpacaraError::GenesisMissing));
    let mut testnet = Node::regtest(150);
    testnet.network = Network::Testnet;
    assert_eq!(
        against(testnet, true).0,
        Err(UpacaraError::WrongNetwork("testnet".to_owned()))
    );
    let mut other = Node::regtest(150);
    other.genesis_id = [0x11; 32];
    assert!(matches!(
        against(other, true).0,
        Err(UpacaraError::WrongGenesis(_))
    ));
    // Tidak ada yang terkirim dalam kasus galat.
    let mut none = Node::regtest(98);
    none.utxos = vec![utxo(0, 0, GENESIS_ALLOCATION, true)];
    assert!(against(none, true).1.is_empty());
}
