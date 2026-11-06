//! FTR-04 (TSK-F04): jaringan multi-node dan uji reorg dalam, dari ujung ke ujung.
//!
//! Lima node (MOD-11, `MemoryStore`) berjalan di satu proses, masing-masing dengan layanan P2P sungguhan (MOD-06:
//! TCP, Noise, Yamux di 127.0.0.1) dan backend `Relay` untuk menambang. Selesai bila (Dokumen fitur FTR-04): 4 node
//! atau lebih tetap sinkron; uji reorg dalam (tanpa batas kedalaman) lolos; node yang sempat offline mengejar chain
//! dengan kerja terbanyak.
//!
//! Jaringan dibelah dengan menghentikan sesi P2P node penghubung (task tokio dibatalkan, koneksinya tertutup) lalu
//! menjalankan sesi baru dengan daftar `--connect` lain. Isi node (chain, cabang, mempool) tetap, seperti daemon yang
//! dibuka ulang dari penyimpanannya. Jam semua node sama dan tetap, jauh di belakang jadwal, sehingga target tetap
//! `pow_limit` dan setiap blok berkerja sama.

#![forbid(unsafe_code)]

#[cfg(test)]
mod ftr04 {
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    use tokio::runtime::Runtime;
    use tokio::sync::mpsc;
    use tokio::task::JoinHandle;
    use zevrigo_consensus::{
        BlockError, ChainParams, REGTEST_GENESIS_TIMESTAMP, check_block_context_free,
        regtest_genesis,
    };
    use zevrigo_crypto::{SecretKey, sighash_v1, sign, txid};
    use zevrigo_mempool::DEFAULT_MAX_BYTES;
    use zevrigo_network::{Keypair, Multiaddr, NetEvent, Network, NetworkConfig};
    use zevrigo_node::{Node, P2pService, Relay, local_status};
    use zevrigo_primitives::params::GENESIS_ALLOCATION;
    use zevrigo_primitives::{
        Amount, Block, Decode, Encode, Header, Input, Output, OutputKind, Transaction, Witness,
    };
    use zevrigo_rpc::{GetBlockTemplateParams, Hex32, KindNumber, RpcBackend, SubmitStatus};
    use zevrigo_storage::MemoryStore;

    /// Jam semua node: 1_000 jadwal blok sesudah genesis, sehingga chain uji selalu di belakang jadwal.
    const NOW: u64 = REGTEST_GENESIS_TIMESTAMP + 120 * 1_000;
    /// Biaya transaksi yang membelanjakan alokasi genesis.
    const FEE: u128 = 1_000;
    /// Batas waktu menunggu node sinkron.
    const WAIT: Duration = Duration::from_secs(60);

    type TestNode = Arc<Node<MemoryStore>>;

    /// Sesi P2P satu node: task layanan P2P dan backend RPC yang mengumumkan blok dan transaksi ke sesi itu.
    struct Session {
        task: JoinHandle<()>,
        relay: Relay<MemoryStore>,
    }

    fn secret(n: u8) -> SecretKey {
        let mut bytes = [0_u8; 32];
        if let Some(last) = bytes.last_mut() {
            *last = n;
        }
        SecretKey::from_bytes(&bytes).unwrap()
    }

    fn key(n: u8) -> [u8; 32] {
        *secret(n).public_key().as_bytes()
    }

    fn open_node() -> TestNode {
        let node = Node::open(
            MemoryStore::default(),
            ChainParams::regtest().unwrap(),
            DEFAULT_MAX_BYTES,
            Box::new(|| NOW),
        )
        .unwrap();
        Arc::new(node)
    }

    /// Menjalankan sesi P2P untuk `node`. Tanpa `connect`, sesi mendengar di port bebas 127.0.0.1 dan alamat lengkapnya
    /// (`/ip4/127.0.0.1/tcp/<port>/p2p/<PeerId>`) dikembalikan; dengan `connect`, sesi hanya mendial.
    fn start(rt: &Runtime, node: &TestNode, connect: &[Multiaddr]) -> (Session, Option<Multiaddr>) {
        let mut config = NetworkConfig::new(node.params().network_id());
        if connect.is_empty() {
            config.listen = vec!["/ip4/127.0.0.1/tcp/0".parse().unwrap()];
        }
        config.connect = connect.to_vec();
        config.redial_interval = Duration::from_secs(1);
        let status = local_status(node.as_ref()).unwrap();
        let (net, address) = rt.block_on(async {
            let mut net = Network::new(Keypair::generate_ed25519(), config, status).unwrap();
            let mut address = None;
            if connect.is_empty() {
                let event = net.next_event().await;
                let NetEvent::Listening { address: listen } = event else {
                    unreachable!("event pertama sesi yang mendengar wajib Listening")
                };
                let full = format!("{listen}/p2p/{}", net.local_peer_id());
                address = Some(full.parse::<Multiaddr>().unwrap());
            }
            (net, address)
        });
        let (sender, receiver) = mpsc::unbounded_channel();
        let relay = Relay::new(Arc::clone(node), sender);
        let task = rt.spawn(P2pService::new(Arc::clone(node), net).run(receiver));
        (Session { task, relay }, address)
    }

    /// Menghentikan sesi P2P: task dibatalkan, swarm dan semua koneksinya ditutup.
    fn stop(rt: &Runtime, session: Session) {
        session.task.abort();
        assert!(
            rt.block_on(session.task)
                .is_err_and(|error| error.is_cancelled())
        );
    }

    /// Mencari nonce agar PoW regtest lolos (sekitar dua percobaan di `pow_limit`).
    fn solve(block: &Block) -> Block {
        let params = ChainParams::regtest().unwrap();
        let h = block.header();
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
                Block::new(header, block.txs().to_vec()).unwrap()
            })
            .find(|candidate| {
                !matches!(
                    check_block_context_free(candidate, &params),
                    Err(BlockError::HighHash)
                )
            })
            .unwrap()
    }

    /// Menambang `count` blok lewat backend sesi (seperti miner lewat RPC); setiap blok wajib `accepted`. Mengembalikan
    /// ID blok terakhir.
    fn mine(session: &Session, owner: u8, count: u64) -> Hex32 {
        let payout = GetBlockTemplateParams {
            payout_kind: KindNumber::new(OutputKind::SingleKey),
            payout_payload: Hex32::new(key(owner)),
        };
        let mut last = None;
        for _ in 0..count {
            let template = session.relay.block_template(&payout).unwrap();
            let block = solve(&Block::from_bytes(template.hex.as_slice()).unwrap());
            let result = session.relay.submit_block(&block.to_bytes()).unwrap();
            assert_eq!(result.status, SubmitStatus::Accepted, "{result:?}");
            last = result.block_id;
        }
        last.unwrap()
    }

    /// Tinggi dan ID ujung chain aktif.
    fn tip(node: &TestNode) -> (u64, Hex32) {
        let info = node.chain_info().unwrap();
        (info.height, info.best_block_id)
    }

    /// Menunggu sampai semua `nodes` berujung di `height` dengan ID `id`, paling lama [`WAIT`].
    fn wait_tip(what: &str, nodes: &[&TestNode], height: u64, id: Hex32) {
        let deadline = Instant::now() + WAIT;
        while !nodes.iter().all(|node| tip(node) == (height, id)) {
            let tips: Vec<(u64, Hex32)> = nodes.iter().copied().map(tip).collect();
            assert!(
                Instant::now() < deadline,
                "{what}: ujung {tips:?}, seharusnya {height} {id:?}"
            );
            sleep(Duration::from_millis(50));
        }
    }

    fn mempool(node: &TestNode) -> Vec<Hex32> {
        node.mempool().unwrap().txids
    }

    fn utxos_of(node: &TestNode, owner: u8) -> usize {
        node.scan_utxos(&[Hex32::new(key(owner))]).unwrap().len()
    }

    /// Membelanjakan alokasi genesis (kunci rahasia 1) ke kunci 3 dengan biaya [`FEE`].
    fn spend_genesis() -> Transaction {
        let params = ChainParams::regtest().unwrap();
        let genesis = regtest_genesis().unwrap();
        let coinbase = &genesis.txs()[0];
        let spent = coinbase.body().outputs()[0].clone();
        let inputs = vec![Input::new(txid(coinbase), 0)];
        let amount = GENESIS_ALLOCATION.as_quanta() - FEE;
        let outputs =
            vec![Output::new(Amount::from_quanta(amount), OutputKind::SingleKey, key(3)).unwrap()];
        let unsigned = Transaction::regular(
            inputs.clone(),
            outputs.clone(),
            vec![Witness::single_key([0; 64])],
        )
        .unwrap();
        let msg = sighash_v1(
            &params.network_id(),
            &unsigned,
            0,
            std::slice::from_ref(&spent),
        )
        .unwrap();
        let sig = sign(&secret(1), msg.as_bytes(), &[0; 32]).unwrap();
        Transaction::regular(inputs, outputs, vec![Witness::single_key(*sig.as_bytes())]).unwrap()
    }

    #[test]
    fn five_nodes_partition_reorg_and_catch_up() {
        let rt = Runtime::new().unwrap();
        let [a, b, c, d, e] = [(); 5].map(|()| open_node());

        // 1. Empat node tersambung: A dan C mendengar; B dan D masing-masing menyambung ke A dan C, sehingga A dan C
        //    hanya terhubung lewat B dan D. A menambang 98 blok, lalu C menambang blok 99; semua node sinkron.
        let (sa, addr_a) = start(&rt, &a, &[]);
        let (sc, addr_c) = start(&rt, &c, &[]);
        let addr_a = addr_a.unwrap();
        let addr_c = addr_c.unwrap();
        let (sb, _) = start(&rt, &b, &[addr_a.clone(), addr_c.clone()]);
        let (sd, _) = start(&rt, &d, &[addr_c.clone(), addr_a.clone()]);
        let tip_98 = mine(&sa, 2, 98);
        wait_tip("98 blok dari A", &[&a, &b, &c, &d], 98, tip_98);
        let tip_99 = mine(&sc, 4, 1);
        wait_tip("blok 99 dari C", &[&a, &b, &c, &d], 99, tip_99);
        let fork_id = tip_99;

        // 2. Jaringan dibelah menjadi {A, B} dan {C, D}. Transaksi yang membelanjakan alokasi genesis dikirim ke A dan
        //    masuk blok 100 cabang A. A menambang 10 blok (tinggi 100 sampai 109), C menambang 15 (100 sampai 114).
        stop(&rt, sb);
        stop(&rt, sd);
        let (sb, _) = start(&rt, &b, std::slice::from_ref(&addr_a));
        let (sd, _) = start(&rt, &d, std::slice::from_ref(&addr_c));
        let tx = spend_genesis();
        let tx_id = Hex32::from(txid(&tx));
        let sent = sa.relay.send_raw_transaction(&tx.to_bytes()).unwrap();
        assert_eq!(sent.txid, tx_id);
        assert_eq!(mempool(&a), vec![tx_id]);
        let tip_a = mine(&sa, 2, 10);
        let tip_c = mine(&sc, 4, 15);
        wait_tip("cabang A di {A, B}", &[&a, &b], 109, tip_a);
        wait_tip("cabang C di {C, D}", &[&c, &d], 114, tip_c);
        let a_100 = a.block_hash(100).unwrap();
        assert_ne!(a_100, c.block_hash(100).unwrap());
        assert_eq!(a.block_hash(99).unwrap(), fork_id);
        assert_eq!(c.block_hash(99).unwrap(), fork_id);
        assert!(mempool(&a).is_empty());
        assert_eq!(utxos_of(&a, 3), 1);
        assert_eq!(utxos_of(&b, 3), 1);
        assert_eq!(utxos_of(&c, 3), 0);
        assert!(mempool(&c).is_empty());
        assert!(mempool(&d).is_empty());

        // 3. B tersambung lagi ke A dan C: B dan A reorg sedalam 10 blok ke cabang C yang berkerja lebih banyak.
        //    Transaksi dari blok yang dilepas kembali ke mempool A dan B.
        stop(&rt, sb);
        let (sb, _) = start(&rt, &b, &[addr_a.clone(), addr_c.clone()]);
        wait_tip("semua node di cabang C", &[&a, &b, &c, &d], 114, tip_c);
        assert_eq!(a.block_hash(100).unwrap(), c.block_hash(100).unwrap());
        assert_ne!(a.block_hash(100).unwrap(), a_100);
        assert_eq!(utxos_of(&a, 3), 0);
        assert_eq!(utxos_of(&b, 3), 0);
        assert_eq!(utxos_of(&a, 2), 98);
        assert_eq!(utxos_of(&d, 4), 16);
        assert_eq!(mempool(&a), vec![tx_id]);
        assert_eq!(mempool(&b), vec![tx_id]);
        assert!(mempool(&c).is_empty());
        assert!(mempool(&d).is_empty());

        // A menambang blok 115 berisi transaksi itu; blok sampai ke semua node lewat B.
        let tip_115 = mine(&sa, 2, 1);
        wait_tip("blok 115 dari A", &[&a, &b, &c, &d], 115, tip_115);
        for node in [&a, &b, &c, &d] {
            assert_eq!(utxos_of(node, 3), 1);
            assert!(mempool(node).is_empty());
        }

        // 4. D offline (sesi P2P berhenti) selama C menambang 5 blok, lalu tersambung lagi ke C dan mengejar.
        stop(&rt, sd);
        let tip_120 = mine(&sc, 4, 5);
        wait_tip("blok 116 sampai 120 tanpa D", &[&a, &b, &c], 120, tip_120);
        assert_eq!(tip(&d), (115, tip_115));
        let (sd, _) = start(&rt, &d, std::slice::from_ref(&addr_c));
        wait_tip("D mengejar", &[&d], 120, tip_120);

        // 5. Node baru E (hanya genesis) tersambung ke A dan mengejar 120 blok lewat header lalu GetBlocks.
        let (se, _) = start(&rt, &e, &[addr_a]);
        wait_tip("E mengejar dari genesis", &[&e], 120, tip_120);
        for height in [1, 50, 99, 100, 115, 120] {
            assert_eq!(e.block_hash(height).unwrap(), c.block_hash(height).unwrap());
        }
        assert_eq!(utxos_of(&e, 3), 1);
        assert_eq!(e.chain_info().unwrap(), c.chain_info().unwrap());

        for session in [sa, sb, sc, sd, se] {
            stop(&rt, session);
        }
    }
}
