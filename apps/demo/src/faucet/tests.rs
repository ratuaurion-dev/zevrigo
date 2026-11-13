use std::collections::BTreeMap;
use std::io::{Read as _, Write as _};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use axum::Router;
use axum::http::{HeaderMap, HeaderValue};
use serde_json::{Value, json};
use tokio::runtime::Runtime;
use zevrigo_crypto::{PublicKey, Signature, sighash_v1, txid, verify};
use zevrigo_primitives::{
    Amount, Decode, Hash32, Network, OutPoint, Output, OutputKind, Transaction, TxBody, Witness,
};
use zevrigo_rpc::{
    BlockInfo, BlockTemplate, ChainInfo, CompactHex, GetBlockTemplateParams, Hex32, KindNumber,
    MempoolInfo, MempoolOutput, MempoolScan, MempoolSpend, NetworkName, Quanta, RpcBackend,
    RpcClient, RpcError, RpcServer, SendRawTransactionResult, SubmitBlockResult, UtxoInfo,
};
use zevrigo_wallet_core::{decode_address, encode_address};

use super::{
    CLIENT_IP_HEADER, FAUCET_PATH, Faucet, FaucetError, FaucetStatus, KEY_FILE, Payout,
    RandomSource, RouterOptions, SECONDS_PER_DAY, STATE_FILE, Settings, faucet_client_ip, router,
};
use crate::limit::Limits;
use crate::upacara::REGTEST_NETWORK_ID;

const ZVRG: u128 = 1_000_000_000_000_000_000;

/// Satu waktu tetap (detik Unix) di tengah hari UTC.
const NOW: u64 = 1_800_000_000;

const ORIGIN: &str = "https://zevrigo.ratuaurion.store";

fn zvrg(n: u128) -> Amount {
    Amount::from_quanta(n * ZVRG)
}

fn clock() -> u64 {
    NOW
}

fn settings(daily_limit: u32) -> Settings {
    Settings {
        amount: zvrg(100),
        fee_rate: 1,
        daily_limit,
    }
}

/// Sumber acak tiruan: `[seed + 1; 32]`, `[seed + 2; 32]`, ...
fn random(seed: u8) -> RandomSource {
    let mut next = seed;
    Box::new(move || {
        next = next.wrapping_add(1);
        Ok([next; 32])
    })
}

fn recipient(byte: u8) -> String {
    encode_address(Network::Regtest, OutputKind::SingleKey, &[byte; 32])
}

fn payload_of(address: &str) -> [u8; 32] {
    decode_address(Network::Regtest, address).unwrap().1
}

fn visitor(n: u8) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(198, 51, 100, n))
}

fn inputs(tx: &Transaction) -> Vec<OutPoint> {
    match tx.body() {
        TxBody::Regular(body) => body.inputs().iter().map(OutPoint::from_input).collect(),
        TxBody::Coinbase(_) => Vec::new(),
    }
}

fn reject(reason: &str) -> RpcError {
    RpcError::InvalidTransaction {
        reason: reason.to_owned(),
    }
}

/// Chain tiruan: set UTXO terkonfirmasi dan mempool berantai (input boleh output mempool).
#[derive(Default)]
struct Chain {
    tip: u64,
    confirmed: BTreeMap<OutPoint, (Output, u64)>,
    mempool: Vec<Transaction>,
}

impl Chain {
    fn mempool_output(&self, outpoint: &OutPoint) -> Option<Output> {
        let tx = self.mempool.iter().find(|tx| txid(tx) == outpoint.txid())?;
        tx.body()
            .outputs()
            .get(usize::try_from(outpoint.index()).ok()?)
            .cloned()
    }

    fn spent_in_mempool(&self, outpoint: &OutPoint) -> bool {
        self.mempool.iter().any(|tx| inputs(tx).contains(outpoint))
    }

    /// Satu blok baru berisi semua transaksi mempool.
    fn confirm(&mut self) {
        self.tip += 1;
        let txs = std::mem::take(&mut self.mempool);
        for tx in &txs {
            for outpoint in inputs(tx) {
                self.confirmed.remove(&outpoint);
            }
            let id = txid(tx);
            for (index, output) in (0_u32..).zip(tx.body().outputs()) {
                self.confirmed
                    .insert(OutPoint::new(id, index), (output.clone(), self.tip));
            }
        }
    }
}

/// Node tiruan yang memeriksa input, tanda tangan BIP340, dan jumlah seperti node sungguhan (tanpa aturan lain).
struct Node {
    network: Network,
    genesis: [u8; 32],
    chain: Mutex<Chain>,
}

impl Node {
    fn with(network: Network, genesis: [u8; 32]) -> Arc<Self> {
        Arc::new(Self {
            network,
            genesis,
            chain: Mutex::new(Chain {
                tip: 10,
                ..Chain::default()
            }),
        })
    }

    fn regtest() -> Arc<Self> {
        Self::with(Network::Regtest, REGTEST_NETWORK_ID)
    }

    fn fund(&self, payload: [u8; 32], amount: u128, tag: u8) {
        let mut chain = self.chain.lock().unwrap();
        let tip = chain.tip;
        chain.confirmed.insert(
            OutPoint::new(Hash32::from_bytes([tag; 32]), 0),
            (
                Output::new(zvrg(amount), OutputKind::SingleKey, payload).unwrap(),
                tip,
            ),
        );
    }
}

impl RpcBackend for Node {
    fn chain_info(&self) -> Result<ChainInfo, RpcError> {
        let chain = self.chain.lock().unwrap();
        Ok(ChainInfo {
            network: NetworkName::new(self.network),
            height: chain.tip,
            best_block_id: Hex32::new([0xbb; 32]),
            chainwork: Hex32::new([0; 32]),
            next_target: CompactHex::new(0x207f_ffff),
            median_time_past: 1_793_145_600,
            mined_total: Quanta::new(Amount::ZERO),
            total_supply: Quanta::new(Amount::ZERO),
            mempool_count: u64::try_from(chain.mempool.len()).unwrap(),
        })
    }

    fn block_hash(&self, height: u64) -> Result<Hex32, RpcError> {
        if height == 0 {
            Ok(Hex32::new(self.genesis))
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

    fn send_raw_transaction(&self, bytes: &[u8]) -> Result<SendRawTransactionResult, RpcError> {
        let tx = Transaction::from_bytes(bytes).map_err(|_| RpcError::InvalidEncoding)?;
        let mut chain = self.chain.lock().unwrap();
        let mut spent = Vec::new();
        for outpoint in inputs(&tx) {
            if chain.spent_in_mempool(&outpoint) {
                return Err(reject("konflik"));
            }
            let output = chain
                .confirmed
                .get(&outpoint)
                .map(|(output, _)| output.clone())
                .or_else(|| chain.mempool_output(&outpoint))
                .ok_or_else(|| reject("input tidak ada"))?;
            spent.push(output);
        }
        let network_id = Hash32::from_bytes(self.genesis);
        for (index, (witness, output)) in (0_u32..).zip(tx.witnesses().iter().zip(&spent)) {
            let Witness::SingleKey(signature) = witness else {
                return Err(reject("witness"));
            };
            let message = sighash_v1(&network_id, &tx, index, &spent).unwrap();
            let key = PublicKey::from_bytes(output.payload()).unwrap();
            if !verify(&key, message.as_bytes(), &Signature::from_bytes(signature)) {
                return Err(reject("T6"));
            }
        }
        let total_in: u128 = spent.iter().map(|o| o.amount().as_quanta()).sum();
        let total_out: u128 = tx
            .body()
            .outputs()
            .iter()
            .map(|o| o.amount().as_quanta())
            .sum();
        if total_out > total_in {
            return Err(reject("T5"));
        }
        let id = txid(&tx);
        chain.mempool.push(tx);
        Ok(SendRawTransactionResult {
            txid: Hex32::from(id),
        })
    }

    fn mempool(&self) -> Result<MempoolInfo, RpcError> {
        let chain = self.chain.lock().unwrap();
        Ok(MempoolInfo {
            txids: chain
                .mempool
                .iter()
                .map(|tx| Hex32::from(txid(tx)))
                .collect(),
        })
    }

    fn scan_utxos(&self, payloads: &[Hex32]) -> Result<Vec<UtxoInfo>, RpcError> {
        let chain = self.chain.lock().unwrap();
        Ok(chain
            .confirmed
            .iter()
            .filter(|(_, (output, _))| payloads.contains(&Hex32::new(*output.payload())))
            .map(|(outpoint, (output, height))| UtxoInfo {
                txid: Hex32::from(outpoint.txid()),
                index: outpoint.index(),
                amount: Quanta::new(output.amount()),
                kind: KindNumber::new(output.kind()),
                payload: Hex32::new(*output.payload()),
                height: *height,
                is_coinbase: false,
                confirmations: chain.tip - height + 1,
                spendable_at: height + 1,
            })
            .collect())
    }

    fn scan_mempool(&self, payloads: &[Hex32]) -> Result<MempoolScan, RpcError> {
        let chain = self.chain.lock().unwrap();
        let mut scan = MempoolScan::default();
        for tx in &chain.mempool {
            let id = Hex32::from(txid(tx));
            for outpoint in inputs(tx) {
                if let Some((output, _)) = chain.confirmed.get(&outpoint)
                    && payloads.contains(&Hex32::new(*output.payload()))
                {
                    scan.spent.push(MempoolSpend {
                        txid: Hex32::from(outpoint.txid()),
                        index: outpoint.index(),
                        spent_by: id,
                    });
                }
            }
            for (index, output) in (0_u32..).zip(tx.body().outputs()) {
                if payloads.contains(&Hex32::new(*output.payload())) {
                    scan.outputs.push(MempoolOutput {
                        txid: id,
                        index,
                        amount: Quanta::new(output.amount()),
                        kind: KindNumber::new(output.kind()),
                        payload: Hex32::new(*output.payload()),
                    });
                }
            }
        }
        Ok(scan)
    }
}

/// Runtime, node tiruan di belakang server JSON-RPC sungguhan (MOD-08), dan folder data sementara.
struct Fixture {
    rt: Runtime,
    node: Arc<Node>,
    server: RpcServer,
    url: String,
    dir: PathBuf,
}

impl Fixture {
    fn new(name: &str, node: Arc<Node>) -> Self {
        let rt = Runtime::new().unwrap();
        let server = rt
            .block_on(RpcServer::start(
                SocketAddr::from(([127, 0, 0, 1], 0)),
                Arc::clone(&node),
            ))
            .unwrap();
        let url = format!("http://{}", server.local_addr());
        let dir =
            std::env::temp_dir().join(format!("zevrigo-faucet-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Self {
            rt,
            node,
            server,
            url,
            dir,
        }
    }

    fn client(&self, url: &str) -> RpcClient {
        self.rt.block_on(async { RpcClient::new(url) }).unwrap()
    }

    fn open(&self, daily_limit: u32) -> Result<Faucet, FaucetError> {
        Faucet::open(
            self.client(&self.url),
            &self.dir,
            settings(daily_limit),
            random(9),
            NOW,
        )
    }

    fn request(
        &self,
        faucet: &mut Faucet,
        address: &str,
        who: u8,
        now: u64,
    ) -> Result<Payout, FaucetError> {
        self.rt
            .block_on(faucet.request(address, Some(visitor(who)), now))
    }

    fn status(&self, faucet: &mut Faucet, now: u64) -> FaucetStatus {
        self.rt.block_on(faucet.status(now)).unwrap()
    }

    fn read(&self, file: &str) -> String {
        std::fs::read_to_string(self.dir.join(file)).unwrap()
    }

    fn finish(self) {
        let Self {
            rt, server, dir, ..
        } = self;
        rt.block_on(server.stop());
        let _ = std::fs::remove_dir_all(dir);
    }
}

#[test]
fn key_file_is_created_once_and_reused() {
    let fx = Fixture::new("kunci", Node::regtest());
    let first = fx.open(3).unwrap();
    let address = first.address().to_owned();
    assert!(address.starts_with("rzvrg1q"), "{address}");
    let phrase = fx.read(KEY_FILE);
    assert_eq!(phrase.trim().split(' ').count(), 24);
    assert!(phrase.ends_with('\n'));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(fx.dir.join(KEY_FILE))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    // Sumber acak lain: frasa lama tetap dipakai, alamat sama.
    let again = Faucet::open(fx.client(&fx.url), &fx.dir, settings(3), random(77), NOW).unwrap();
    assert_eq!(again.address(), address);
    assert_eq!(again.settings(), &settings(3));
    // Frasa rusak: gagal start, file tidak ditimpa.
    std::fs::write(fx.dir.join(KEY_FILE), "bukan frasa\n").unwrap();
    assert!(matches!(fx.open(3), Err(FaucetError::Key(_))));
    assert_eq!(fx.read(KEY_FILE), "bukan frasa\n");
    fx.finish();
}

#[test]
fn payouts_chain_through_mempool_and_respect_limits() {
    let fx = Fixture::new("kirim", Node::regtest());
    let mut faucet = fx.open(3).unwrap();
    let own = faucet.address().to_owned();
    fx.node.fund(payload_of(&own), 1_000, 0x42);
    let status = fx.status(&mut faucet, NOW);
    assert!(status.synced && status.ready);
    assert_eq!(status.balance, zvrg(1_000));
    assert_eq!((status.daily_limit, status.remaining_today), (3, 3));
    assert_eq!(
        status.to_json()["amount"],
        json!(zvrg(100).as_quanta().to_string())
    );

    let (a, b, c, d) = (
        recipient(0x11),
        recipient(0x22),
        recipient(0x33),
        recipient(0x44),
    );
    let first = fx.request(&mut faucet, &a, 1, NOW).unwrap();
    assert_eq!(
        (first.address.as_str(), first.amount),
        (a.as_str(), zvrg(100))
    );
    assert!(first.fee > Amount::ZERO && first.fee < Amount::from_quanta(1_000));
    assert_eq!(
        fx.request(&mut faucet, &b, 1, NOW),
        Err(FaucetError::VisitorLimit)
    );
    assert_eq!(
        fx.request(&mut faucet, &format!(" {a} "), 2, NOW),
        Err(FaucetError::AddressLimit)
    );
    let second = fx.request(&mut faucet, &b, 2, NOW).unwrap();
    let third = fx.request(&mut faucet, &c, 3, NOW).unwrap();
    assert_eq!(
        fx.request(&mut faucet, &d, 4, NOW),
        Err(FaucetError::DailyLimit)
    );
    assert_eq!(
        fx.request(&mut faucet, "bukan-alamat", 5, NOW),
        Err(FaucetError::BadAddress)
    );
    assert_eq!(
        fx.request(&mut faucet, &own, 5, NOW),
        Err(FaucetError::OwnAddress)
    );
    let testnet = encode_address(Network::Testnet, OutputKind::SingleKey, &[0x55; 32]);
    assert_eq!(
        fx.request(&mut faucet, &testnet, 5, NOW),
        Err(FaucetError::BadAddress)
    );

    // Tiga transaksi berantai di mempool: kiriman kedua dan ketiga memakai kembalian yang belum masuk blok.
    {
        let chain = fx.node.chain.lock().unwrap();
        let ids: Vec<Hash32> = chain.mempool.iter().map(txid).collect();
        assert_eq!(ids, vec![first.txid, second.txid, third.txid]);
        assert_eq!(
            inputs(&chain.mempool[0]),
            vec![OutPoint::new(Hash32::from_bytes([0x42; 32]), 0)]
        );
        assert_eq!(
            inputs(&chain.mempool[1]),
            vec![OutPoint::new(first.txid, 1)]
        );
        assert_eq!(
            inputs(&chain.mempool[2]),
            vec![OutPoint::new(second.txid, 1)]
        );
        for (tx, to) in chain.mempool.iter().zip([&a, &b, &c]) {
            let outputs = tx.body().outputs();
            assert_eq!(outputs.len(), 2);
            assert_eq!(outputs[0].payload(), &payload_of(to));
            assert_eq!(outputs[0].amount(), zvrg(100));
            assert_eq!(outputs[1].payload(), &payload_of(&own));
        }
    }
    let fees = first.fee.as_quanta() + second.fee.as_quanta() + third.fee.as_quanta();
    let status = fx.status(&mut faucet, NOW);
    assert_eq!(status.balance, Amount::from_quanta(700 * ZVRG - fees));
    assert_eq!(status.remaining_today, 0);
    assert!(status.synced && !status.ready);

    // Batas tersimpan: hari, jumlah, dan alamat penerima saja.
    let saved: Value = serde_json::from_str(&fx.read(STATE_FILE)).unwrap();
    assert_eq!(saved["versi"], json!(1));
    assert_eq!(saved["hari"], json!(NOW / SECONDS_PER_DAY));
    assert_eq!(saved["jumlah"], json!(3));
    let mut expected = vec![a, b, c];
    expected.sort();
    assert_eq!(saved["alamat"], json!(expected));
    assert!(!fx.read(STATE_FILE).contains("198.51.100"));
    fx.finish();
}

#[test]
fn limits_survive_restart_and_reset_next_day() {
    let fx = Fixture::new("restart", Node::regtest());
    let mut faucet = fx.open(3).unwrap();
    fx.node.fund(payload_of(faucet.address()), 1_000, 0x42);
    let (a, b, c) = (recipient(0x11), recipient(0x22), recipient(0x33));
    fx.request(&mut faucet, &a, 1, NOW).unwrap();
    fx.request(&mut faucet, &b, 2, NOW).unwrap();

    // Restart dengan batas lebih besar. Mempool masih memuat kiriman lama: faucet menunggu blok.
    let mut faucet = fx.open(10).unwrap();
    assert_eq!(fx.request(&mut faucet, &c, 3, NOW), Err(FaucetError::Busy));
    let status = fx.status(&mut faucet, NOW);
    assert!(!status.synced && !status.ready);
    assert_eq!(status.remaining_today, 8);
    // Kembalian kiriman lama belum dihitung: bisa saja sudah dibelanjakan kiriman berikutnya.
    assert_eq!(status.balance, Amount::ZERO);
    fx.node.chain.lock().unwrap().confirm();

    // Batas alamat tersimpan di file; pengunjung hanya di memori sehingga mulai dari nol.
    assert_eq!(
        fx.request(&mut faucet, &a, 5, NOW),
        Err(FaucetError::AddressLimit)
    );
    fx.request(&mut faucet, &c, 1, NOW).unwrap();
    assert_eq!(fx.status(&mut faucet, NOW).remaining_today, 7);

    // Hari UTC berikutnya: semua batas kosong lagi.
    let tomorrow = NOW + SECONDS_PER_DAY;
    fx.request(&mut faucet, &a, 1, tomorrow).unwrap();
    let status = fx.status(&mut faucet, tomorrow);
    assert_eq!(status.remaining_today, 9);
    let saved: Value = serde_json::from_str(&fx.read(STATE_FILE)).unwrap();
    assert_eq!(saved["hari"], json!(tomorrow / SECONDS_PER_DAY));
    assert_eq!(saved["alamat"], json!([a]));

    // File batas rusak: faucet tidak start (tidak diam-diam mengosongkan batas).
    std::fs::write(fx.dir.join(STATE_FILE), "{}\n").unwrap();
    assert!(matches!(fx.open(3), Err(FaucetError::State(_))));
    fx.finish();
}

#[test]
fn empty_faucet_and_wrong_nodes_are_refused_without_using_limits() {
    let fx = Fixture::new("kosong", Node::regtest());
    let mut faucet = fx.open(3).unwrap();
    let a = recipient(0x11);
    assert_eq!(fx.request(&mut faucet, &a, 1, NOW), Err(FaucetError::Empty));
    let status = fx.status(&mut faucet, NOW);
    assert_eq!(status.balance, Amount::ZERO);
    assert!(status.synced && !status.ready);
    // Penolakan tadi tidak memakai batas pengunjung maupun alamat.
    fx.node.fund(payload_of(faucet.address()), 150, 0x42);
    fx.request(&mut faucet, &a, 1, NOW).unwrap();
    // Sisa 50 ZVRG dikurangi biaya: kiriman berikutnya tidak cukup.
    assert_eq!(
        fx.request(&mut faucet, &recipient(0x22), 2, NOW),
        Err(FaucetError::Empty)
    );
    // Node tidak menjawab.
    let mut offline = Faucet::open(
        fx.client("http://127.0.0.1:1"),
        &fx.dir,
        settings(3),
        random(9),
        NOW,
    )
    .unwrap();
    assert!(matches!(
        fx.rt
            .block_on(offline.request(&recipient(0x33), Some(visitor(3)), NOW)),
        Err(FaucetError::NodeUnreachable(_))
    ));
    fx.finish();

    for (name, node) in [
        ("testnet", Node::with(Network::Testnet, REGTEST_NETWORK_ID)),
        ("genesis", Node::with(Network::Regtest, [0x99; 32])),
    ] {
        let fx = Fixture::new(name, node);
        let mut faucet = fx.open(3).unwrap();
        fx.node.fund(payload_of(faucet.address()), 1_000, 0x42);
        assert!(matches!(
            fx.request(&mut faucet, &a, 1, NOW),
            Err(FaucetError::WrongNetwork(_))
        ));
        assert!(matches!(
            fx.rt.block_on(faucet.status(NOW)),
            Err(FaucetError::WrongNetwork(_))
        ));
        assert!(fx.node.chain.lock().unwrap().mempool.is_empty(), "{name}");
        fx.finish();
    }
}

/// Server HTTP faucet di port bebas; mengembalikan alamatnya.
async fn serve(app: Router) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });
    addr
}

/// Satu permintaan HTTP/1.1 mentah; mengembalikan status, kepala jawaban (huruf kecil), dan badan teks.
fn raw(
    addr: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    payload: &[u8],
) -> (u16, String, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    let mut head = format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        payload.len()
    );
    for (name, value) in headers {
        head.push_str(name);
        head.push_str(": ");
        head.push_str(value);
        head.push_str("\r\n");
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes()).unwrap();
    stream.write_all(payload).unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    let (head, rest) = text.split_once("\r\n\r\n").unwrap();
    let status = head.split(' ').nth(1).unwrap().parse().unwrap();
    (status, head.to_ascii_lowercase(), rest.to_owned())
}

/// `POST` alamat dari pengunjung `who` (kepala Cloudflare), badan jawaban sebagai JSON.
fn post(addr: SocketAddr, address: &str, who: Option<&str>) -> (u16, Value) {
    let body = json!({ "address": address }).to_string();
    let headers: Vec<(&str, &str)> = who.map(|ip| (CLIENT_IP_HEADER, ip)).into_iter().collect();
    let (status, _, text) = raw(addr, "POST", FAUCET_PATH, &headers, body.as_bytes());
    (status, serde_json::from_str(&text).unwrap())
}

#[test]
fn http_serves_status_payouts_and_cors() {
    let fx = Fixture::new("http", Node::regtest());
    let faucet = fx.open(5).unwrap();
    let own = faucet.address().to_owned();
    fx.node.fund(payload_of(&own), 1_000, 0x42);
    let trusted = Arc::new(RwLock::new(vec![IpAddr::V4(Ipv4Addr::LOCALHOST)]));
    let app = router(
        faucet,
        RouterOptions {
            limits: None,
            trusted,
            origins: vec![ORIGIN.to_owned()],
            clock,
        },
    );
    let addr = fx.rt.block_on(serve(app));

    let (status, head, text) = raw(addr, "GET", FAUCET_PATH, &[("Origin", ORIGIN)], b"");
    assert_eq!(status, 200, "{text}");
    let body: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(body["address"], json!(own));
    assert_eq!(body["network"], json!("regtest"));
    assert_eq!(body["ready"], json!(true));
    assert!(
        head.contains(&format!("access-control-allow-origin: {ORIGIN}")),
        "{head}"
    );
    assert!(
        head.contains("vary: origin") && head.contains("cache-control: no-store"),
        "{head}"
    );

    // Pengunjung dari kepala Cloudflare (koneksi dari proxy tepercaya 127.0.0.1).
    let (a, b, c, d) = (
        recipient(0x11),
        recipient(0x22),
        recipient(0x33),
        recipient(0x44),
    );
    let (status, body) = post(addr, &a, Some("198.51.100.1"));
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["address"], json!(a));
    assert_eq!(body["txid"].as_str().unwrap().len(), 64);
    let (status, body) = post(addr, &b, Some("198.51.100.1"));
    assert_eq!(
        (status, body["error"].clone()),
        (429, json!("visitor_limit"))
    );
    assert!(
        body["message"]
            .as_str()
            .is_some_and(|text| !text.is_empty())
    );
    assert_eq!(post(addr, &b, Some("198.51.100.2")).0, 200);
    let (status, body) = post(addr, &a, Some("198.51.100.3"));
    assert_eq!(
        (status, body["error"].clone()),
        (429, json!("address_limit"))
    );
    // Tanpa kepala: alamat koneksi (127.0.0.1) menjadi pengunjung.
    assert_eq!(post(addr, &c, None).0, 200);
    assert_eq!(post(addr, &d, None).1["error"], json!("visitor_limit"));
    let (status, body) = post(addr, "x", Some("198.51.100.9"));
    assert_eq!((status, body["error"].clone()), (400, json!("bad_address")));

    for payload in [
        b"{".as_slice(),
        br#"{"address": 5}"#.as_slice(),
        b"[]".as_slice(),
    ] {
        let (status, _, text) = raw(addr, "POST", FAUCET_PATH, &[], payload);
        assert_eq!(status, 400, "{text}");
        assert!(text.contains("bad_request"), "{text}");
    }
    let (status, _, _) = raw(addr, "POST", FAUCET_PATH, &[], &[b' '; 5_000]);
    assert_eq!(status, 413);
    let (status, head, _) = raw(addr, "PUT", FAUCET_PATH, &[], b"");
    assert_eq!(status, 405);
    assert!(head.contains("allow: get, post, options"), "{head}");
    let (status, _, text) = raw(addr, "GET", "/lain", &[], b"");
    assert_eq!(status, 404);
    assert!(text.contains("not_found"), "{text}");

    // CORS: hanya origin di daftar izin.
    let preflight = [
        ("Origin", ORIGIN),
        ("Access-Control-Request-Method", "POST"),
    ];
    let (status, head, _) = raw(addr, "OPTIONS", FAUCET_PATH, &preflight, b"");
    assert_eq!(status, 204);
    assert!(
        head.contains(&format!("access-control-allow-origin: {ORIGIN}")),
        "{head}"
    );
    assert!(
        head.contains("access-control-allow-methods: get, post"),
        "{head}"
    );
    let other = [
        ("Origin", "https://jahat.example"),
        ("Access-Control-Request-Method", "POST"),
    ];
    let (status, head, _) = raw(addr, "OPTIONS", FAUCET_PATH, &other, b"");
    assert_eq!(status, 204);
    assert!(!head.contains("access-control-allow"), "{head}");
    let (_, head, _) = raw(
        addr,
        "GET",
        FAUCET_PATH,
        &[("Origin", "https://jahat.example")],
        b"",
    );
    assert!(!head.contains("access-control-allow-origin"), "{head}");

    // Tiga kiriman masuk node.
    assert_eq!(fx.node.chain.lock().unwrap().mempool.len(), 3);
    fx.finish();
}

#[test]
fn http_rate_limit_and_client_ip() {
    let fx = Fixture::new("laju", Node::regtest());
    let app = router(
        fx.open(5).unwrap(),
        RouterOptions {
            limits: Limits::from_rate(1),
            trusted: Arc::new(RwLock::new(Vec::new())),
            origins: Vec::new(),
            clock,
        },
    );
    let addr = fx.rt.block_on(serve(app));
    let codes: Vec<u16> = (0..12)
        .map(|_| raw(addr, "GET", FAUCET_PATH, &[], b"").0)
        .collect();
    assert_eq!(codes.first(), Some(&200));
    assert!(codes.contains(&429), "{codes:?}");
    let (head, text) = loop {
        let (status, head, text) = raw(addr, "GET", FAUCET_PATH, &[], b"");
        if status == 429 {
            break (head, text);
        }
    };
    assert!(head.contains("retry-after: 1"), "{head}");
    assert!(text.contains("rate_limited"), "{text}");
    fx.finish();

    let proxy = IpAddr::V4(Ipv4Addr::new(172, 18, 0, 5));
    let other = IpAddr::V4(Ipv4Addr::new(172, 18, 0, 9));
    let mut headers = HeaderMap::new();
    headers.insert(CLIENT_IP_HEADER, HeaderValue::from_static("198.51.100.7"));
    assert_eq!(
        faucet_client_ip(Some(proxy), &headers, &[proxy]),
        Some(visitor(7))
    );
    let mapped: IpAddr = "::ffff:172.18.0.5".parse().unwrap();
    assert_eq!(
        faucet_client_ip(Some(mapped), &headers, &[proxy]),
        Some(visitor(7))
    );
    // Koneksi bukan dari proxy tepercaya: kepala diabaikan.
    assert_eq!(
        faucet_client_ip(Some(other), &headers, &[proxy]),
        Some(other)
    );
    assert_eq!(faucet_client_ip(Some(proxy), &headers, &[]), Some(proxy));
    assert_eq!(faucet_client_ip(None, &headers, &[proxy]), None);
    headers.insert(CLIENT_IP_HEADER, HeaderValue::from_static("bukan-ip"));
    assert_eq!(
        faucet_client_ip(Some(proxy), &headers, &[proxy]),
        Some(proxy)
    );

    // Kode dan status HTTP galat; rincian galat node tidak sampai ke pengunjung.
    for (error, code, status) in [
        (FaucetError::BadAddress, "bad_address", 400),
        (FaucetError::VisitorLimit, "visitor_limit", 429),
        (FaucetError::Empty, "empty", 503),
        (FaucetError::Busy, "busy", 503),
        (
            FaucetError::Rejected("T6 rahasia".to_owned()),
            "rejected",
            503,
        ),
        (FaucetError::Internal("rahasia".to_owned()), "internal", 500),
    ] {
        assert_eq!((error.code(), error.http_status()), (code, status));
        assert!(!error.public_message().contains("rahasia"));
    }
}
