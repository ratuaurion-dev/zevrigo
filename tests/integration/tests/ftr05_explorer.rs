//! FTR-05 (TSK-F05): explorer minimal dari ujung ke ujung.
//!
//! Node A dan B (MOD-11, `MemoryStore`) masing-masing dengan server RPC sungguhan (MOD-08) dan miner sungguhan (MOD-13)
//! di satu proses. Explorer (MOD-14) memakai indeks SQLite di file sementara dan membaca node A lewat RPC
//! (`RpcSource`), lewat `Shared` yang sama dengan program `zevrigo-explorer`: `sync_once` untuk satu langkah
//! sinkronisasi dan `respond` untuk API (fungsi yang dipanggil handler axum). Selesai bila (Dokumen fitur FTR-05, diubah
//! DEC-28): API menampilkan blok, ringkasan transaksi, dan statistik pasokan tanpa alamat, jumlah per penerima, maupun
//! alokasi genesis; indeks (dibaca langsung lewat `Shared::read_index`) tetap mencatat alamat dan alokasi genesis dengan
//! benar; indexer tahan reorg.
//!
//! Reorg dibuat tanpa P2P: B menambang cabang yang lebih panjang, lalu blok-bloknya dikirim ke A lewat RPC
//! `submit_block`. Jam kedua node sama dan tetap, jauh di belakang jadwal, sehingga target tetap `pow_limit` dan setiap
//! blok berkerja sama.

#![forbid(unsafe_code)]

#[cfg(test)]
mod ftr05 {
    use std::net::SocketAddr;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use serde_json::Value;
    use tokio::runtime::Runtime;
    use zevrigo_consensus::{
        ChainParams, REGTEST_GENESIS_TIMESTAMP, SupplySchedule, regtest_genesis, regtest_network_id,
    };
    use zevrigo_crypto::{SecretKey, sighash_v1, sign, txid};
    use zevrigo_explorer::{
        AddressSummary, AllocationInfo, DB_FILE, Indexer, RpcSource, Shared, TxDetail,
    };
    use zevrigo_mempool::DEFAULT_MAX_BYTES;
    use zevrigo_miner::Miner;
    use zevrigo_node::Node;
    use zevrigo_primitives::params::GENESIS_ALLOCATION;
    use zevrigo_primitives::{
        Amount, Encode, Hash32, Input, Network, Output, OutputKind, Transaction, Witness,
    };
    use zevrigo_rpc::{Hex32, KindNumber, RpcBackend, RpcClient, RpcServer, SubmitStatus};
    use zevrigo_storage::MemoryStore;
    use zevrigo_wallet_core::encode_address;

    /// Jam kedua node: 1_000 jadwal blok sesudah genesis, sehingga chain uji selalu di belakang jadwal.
    const NOW: u64 = REGTEST_GENESIS_TIMESTAMP + 120 * 1_000;
    /// Biaya transaksi yang membelanjakan alokasi genesis.
    const FEE: u128 = 1_000;

    type TestNode = Arc<Node<MemoryStore>>;

    /// Folder sementara khusus test ini, dihapus saat selesai.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("zevrigo-q84-explorer-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Explorer seperti program `zevrigo-explorer`, tanpa server HTTP dan tanpa jeda 2 detik.
    struct Explorer {
        shared: Shared,
        source: RpcSource,
    }

    impl Explorer {
        /// Membuka indeks `path` (dibuat jika belum ada) dan sumber RPC ke node di `url`.
        fn open(rt: &Runtime, path: &Path, url: &str) -> Self {
            let indexer = Indexer::open(path, regtest_network_id().unwrap()).unwrap();
            let client = rt.block_on(async { RpcClient::new(url) }).unwrap();
            Self {
                shared: Shared::new(indexer, Network::Regtest),
                source: RpcSource::new(client, rt.handle().clone()),
            }
        }

        /// Satu langkah sinkronisasi; wajib berhasil tanpa galat.
        fn sync(&self) -> Value {
            self.shared.sync_once(&self.source);
            let status = self.get("/api/v1/status");
            assert_eq!(status["last_error"], Value::Null, "{status}");
            assert_eq!(status["index_error"], Value::Null, "{status}");
            status
        }

        /// `GET` satu jalur API (boleh dengan query string).
        fn call(&self, target: &str) -> (u16, Value) {
            let (path, query) = match target.split_once('?') {
                Some((path, query)) => (path, Some(query)),
                None => (target, None),
            };
            let response = self.shared.respond("GET", path, query);
            (
                response.status,
                serde_json::from_str(&response.body).unwrap(),
            )
        }

        /// `GET` yang wajib `200`.
        fn get(&self, target: &str) -> Value {
            let (status, body) = self.call(target);
            assert_eq!(status, 200, "{target}: {body}");
            body
        }

        /// `GET` yang wajib `404 not_found`: jalur yang tidak disajikan API (DEC-28).
        fn hidden(&self, target: &str) {
            let (status, body) = self.call(target);
            assert_eq!(
                (status, body["error"].as_str()),
                (404, Some("not_found")),
                "{target}"
            );
        }

        /// Alokasi genesis dibaca langsung dari indeks (tidak lewat API).
        fn allocation(&self) -> AllocationInfo {
            self.shared
                .read_index(|index| index.allocation().unwrap().unwrap())
                .unwrap()
        }

        /// Ringkasan alamat kunci tunggal `payload` (halaman 0) dibaca langsung dari indeks.
        fn account(&self, payload: &[u8; 32]) -> AddressSummary {
            self.shared
                .read_index(|index| index.address(0, payload, 0, 25).unwrap())
                .unwrap()
        }

        /// Rincian transaksi dibaca langsung dari indeks.
        fn tx(&self, id: &Hash32) -> TxDetail {
            self.shared
                .read_index(|index| index.tx(id).unwrap().unwrap())
                .unwrap()
        }
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

    fn address(payload: &[u8; 32]) -> String {
        encode_address(Network::Regtest, OutputKind::SingleKey, payload)
    }

    fn hex(id: Hex32) -> String {
        id.to_hash32().to_string()
    }

    fn quanta(value: u128) -> String {
        value.to_string()
    }

    fn local() -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], 0))
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

    fn miner(url: &str, owner: u8) -> Miner {
        Miner::connect(
            url,
            KindNumber::new(OutputKind::SingleKey),
            Hex32::new(key(owner)),
            1_000_000,
        )
        .unwrap()
    }

    /// Tinggi dan ID ujung chain aktif.
    fn tip(node: &TestNode) -> (u64, Hex32) {
        let info = node.chain_info().unwrap();
        (info.height, info.best_block_id)
    }

    fn block_bytes(node: &TestNode, height: u64) -> Vec<u8> {
        node.block(node.block_hash(height).unwrap())
            .unwrap()
            .hex
            .into_vec()
    }

    fn heights(body: &Value) -> Vec<u64> {
        body["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|block| block["height"].as_u64().unwrap())
            .collect()
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
    fn explorer_follows_node_through_reorg_and_genesis_spend() {
        let rt = Runtime::new().unwrap();
        let dir = TempDir::new();
        let db = dir.0.join(DB_FILE);
        let a = open_node();
        let b = open_node();
        let server_a = rt
            .block_on(RpcServer::start(local(), Arc::clone(&a)))
            .unwrap();
        let server_b = rt
            .block_on(RpcServer::start(local(), Arc::clone(&b)))
            .unwrap();
        let url_a = format!("http://{}", server_a.local_addr());
        let url_b = format!("http://{}", server_b.local_addr());
        let client_a = rt.block_on(async { RpcClient::new(&url_a) }).unwrap();
        let miner_a = miner(&url_a, 2);
        let miner_b = miner(&url_b, 4);
        let explorer = Explorer::open(&rt, &db, &url_a);

        let genesis = regtest_genesis().unwrap();
        let coinbase = &genesis.txs()[0];
        let genesis_id = txid(coinbase);
        let owner_payload = *coinbase.body().outputs()[0].payload();
        let owner = address(&owner_payload);
        let after_fee = Amount::from_quanta(GENESIS_ALLOCATION.as_quanta() - FEE);
        let tx = spend_genesis();
        let tx_id = txid(&tx);
        let tx_hex = tx_id.to_string();
        let receiver = address(&key(3));

        // 1. Sebelum sinkron pertama hanya /status yang menjawab; jalur lain 503 not_ready.
        let status = explorer.get("/api/v1/status");
        assert_eq!(status["network"], "regtest");
        assert_eq!(status["indexed_height"], Value::Null);
        assert_eq!(status["node_height"], Value::Null);
        let (code, body) = explorer.call("/api/v1/blocks");
        assert_eq!(code, 503);
        assert_eq!(body["error"], "not_ready");

        // 2. A menambang 99 blok (miner sungguhan lewat RPC); B menerima blok yang sama. Explorer mengindeks 0..99 dan
        //    indeks mencatat alokasi genesis utuh; API tidak menyajikan alokasi maupun alamat.
        assert_eq!(miner_a.run(99, |_| {}).unwrap(), 99);
        for height in 1..=99 {
            let result = b.submit_block(&block_bytes(&a, height)).unwrap();
            assert_eq!(result.status, SubmitStatus::Accepted, "{result:?}");
        }
        assert_eq!(tip(&b), tip(&a));
        let status = explorer.sync();
        let info = a.chain_info().unwrap();
        assert_eq!(status["indexed_height"], 99);
        assert_eq!(status["indexed_block_id"], hex(info.best_block_id));
        assert_eq!(status["node_height"], 99);
        assert_eq!(status["node_best_block_id"], hex(info.best_block_id));
        assert_eq!(status["chainwork"], hex(info.chainwork));
        let allocation = explorer.allocation();
        assert_eq!(
            allocation.genesis_block_id.to_string(),
            regtest_network_id().unwrap().to_string()
        );
        assert_eq!(allocation.allocated, GENESIS_ALLOCATION);
        assert_eq!(allocation.unspent, GENESIS_ALLOCATION);
        assert_eq!(allocation.outputs.len(), 1);
        assert_eq!(allocation.outputs[0].txid, genesis_id);
        assert_eq!(allocation.outputs[0].payload, owner_payload);
        assert!(allocation.outputs[0].spent_by.is_none());
        assert_eq!(explorer.account(&owner_payload).balance, GENESIS_ALLOCATION);
        explorer.hidden("/api/v1/allocation");
        explorer.hidden(&format!("/api/v1/address/{owner}"));

        // 3. Transaksi yang membelanjakan alokasi genesis masuk mempool A; explorer melaporkannya sebagai in_mempool.
        let sent = rt
            .block_on(client_a.send_raw_transaction(&tx.to_bytes()))
            .unwrap();
        assert_eq!(hex(sent.txid), tx_hex);
        let status = explorer.sync();
        assert_eq!(status["mempool_count"], 1);
        let mempool = explorer.get("/api/v1/mempool");
        assert_eq!(mempool["count"], 1);
        assert!(mempool.get("txids").is_none(), "{mempool}");
        assert_eq!(
            explorer.shared.read_index(|index| index.mempool().unwrap()),
            Some(vec![tx_id])
        );
        for target in [
            format!("/api/v1/tx/{tx_hex}"),
            format!("/api/v1/search/{tx_hex}"),
        ] {
            let (code, body) = explorer.call(&target);
            assert_eq!(code, 404, "{target}");
            assert_eq!(body["error"], "not_found");
            assert_eq!(body["in_mempool"], true);
        }

        // 4. A menambang blok 100 (berisi transaksi itu) sampai 102. API menampilkan ringkasan transaksi, blok, dan
        //    pasokan yang sama dengan node; indeks mencatat input, output, alamat, dan alokasi yang sudah dibelanjakan.
        assert_eq!(miner_a.run(3, |_| {}).unwrap(), 3);
        let a_100 = hex(a.block_hash(100).unwrap());
        let status = explorer.sync();
        assert_eq!(status["indexed_height"], 102);
        assert_eq!(status["mempool_count"], 0);
        let detail = explorer.get(&format!("/api/v1/tx/{tx_hex}"));
        assert_eq!(detail["block_id"], a_100.as_str());
        assert_eq!(detail["height"], 100);
        assert_eq!(detail["position"], 1);
        assert_eq!(detail["is_coinbase"], false);
        assert_eq!(detail["fee"], quanta(FEE).as_str());
        assert_eq!(detail["confirmations"], 3);
        assert_eq!(detail["input_count"], 1);
        assert_eq!(detail["output_count"], 1);
        for key in ["inputs", "outputs", "address"] {
            assert!(!detail.to_string().contains(key), "{key}: {detail}");
        }
        let full = explorer.tx(&tx_id);
        assert_eq!(full.inputs[0].txid, genesis_id);
        assert_eq!(full.inputs[0].index, 0);
        assert_eq!(full.inputs[0].amount, GENESIS_ALLOCATION);
        assert_eq!(full.inputs[0].payload, owner_payload);
        assert_eq!(full.outputs[0].amount, after_fee);
        assert_eq!(full.outputs[0].payload, key(3));
        assert!(full.outputs[0].spent_by.is_none());
        let block = explorer.get("/api/v1/block/100");
        assert_eq!(block["block_id"], a_100.as_str());
        assert_eq!(block["tx_count"], 2);
        assert_eq!(block["fees"], quanta(FEE).as_str());
        assert_eq!(block["txids"][1], tx_hex.as_str());
        assert_eq!(block["confirmations"], 3);
        let allocation = explorer.allocation();
        assert_eq!(allocation.unspent.as_quanta(), 0);
        let spender = allocation.outputs[0].spent_by.as_ref().unwrap();
        assert_eq!(
            (spender.txid, spender.input, spender.height),
            (tx_id, 0, 100)
        );
        let creator = explorer.account(&owner_payload);
        assert_eq!(creator.balance.as_quanta(), 0);
        assert_eq!(creator.received, GENESIS_ALLOCATION);
        assert_eq!(creator.sent, GENESIS_ALLOCATION);
        assert_eq!(creator.tx_count, 2);
        let account = explorer.account(&key(3));
        assert_eq!(account.balance, after_fee);
        assert_eq!(account.utxo_count, 1);
        assert_eq!(account.tx_count, 1);
        assert_eq!((account.txs[0].txid, account.txs[0].height), (tx_id, 100));
        explorer.hidden(&format!("/api/v1/address/{receiver}"));
        let info = a.chain_info().unwrap();
        let supply = explorer.get("/api/v1/supply");
        assert_eq!(supply["height"], 102);
        assert_eq!(supply["matches_schedule"], true);
        assert_eq!(
            supply["mined_total"],
            quanta(info.mined_total.amount().as_quanta()).as_str()
        );
        assert!(supply.get("total_supply").is_none(), "{supply}");
        assert!(supply.get("genesis_allocation").is_none(), "{supply}");
        let indexed = explorer
            .shared
            .read_index(|index| index.supply().unwrap().unwrap())
            .unwrap();
        assert_eq!(indexed.total_supply, info.total_supply.amount());
        assert_eq!(indexed.genesis_allocation, GENESIS_ALLOCATION);
        assert_eq!(supply["fees_total"], quanta(FEE).as_str());
        let found = explorer.get("/api/v1/search/100");
        assert_eq!(found["kind"], "block");
        assert_eq!(found["key"], a_100.as_str());
        assert_eq!(
            explorer.get(&format!("/api/v1/search/{tx_hex}"))["kind"],
            "tx"
        );
        let (code, body) = explorer.call(&format!("/api/v1/search/{owner}"));
        assert_eq!((code, body["error"].as_str()), (400, Some("bad_request")));

        // 5. B (tanpa transaksi itu) menambang 5 blok di atas blok 99, lalu bloknya dikirim ke A lewat RPC: A reorg
        //    3 blok ke cabang B dan transaksi kembali ke mempool. Explorer mundur dan mengikuti chain baru.
        assert_eq!(miner_b.run(5, |_| {}).unwrap(), 5);
        let b_tip = tip(&b);
        assert_eq!(b_tip.0, 104);
        for height in 100..=104 {
            let result = rt
                .block_on(client_a.submit_block(&block_bytes(&b, height)))
                .unwrap();
            assert_eq!(result.status, SubmitStatus::Accepted, "{result:?}");
        }
        assert_eq!(tip(&a), b_tip);
        assert_eq!(a.mempool().unwrap().txids, vec![sent.txid]);
        let status = explorer.sync();
        assert_eq!(status["indexed_height"], 104);
        assert_eq!(status["indexed_block_id"], hex(b_tip.1));
        assert_eq!(status["mempool_count"], 1);
        let b_100 = hex(b.block_hash(100).unwrap());
        assert_ne!(b_100, a_100);
        assert_eq!(
            explorer.get("/api/v1/block/100")["block_id"],
            b_100.as_str()
        );
        assert_eq!(explorer.call(&format!("/api/v1/block/{a_100}")).0, 404);
        let (code, body) = explorer.call(&format!("/api/v1/tx/{tx_hex}"));
        assert_eq!(code, 404);
        assert_eq!(body["in_mempool"], true);
        let allocation = explorer.allocation();
        assert_eq!(allocation.unspent, GENESIS_ALLOCATION);
        assert!(allocation.outputs[0].spent_by.is_none());
        assert_eq!(explorer.account(&owner_payload).balance, GENESIS_ALLOCATION);
        let account = explorer.account(&key(3));
        assert_eq!(
            (
                account.balance.as_quanta(),
                account.utxo_count,
                account.tx_count
            ),
            (0, 0, 0)
        );
        let rewards_b = explorer.account(&key(4));
        assert_eq!((rewards_b.utxo_count, rewards_b.tx_count), (5, 5));
        assert_eq!(explorer.account(&key(2)).utxo_count, 99);
        let info = a.chain_info().unwrap();
        let supply = explorer.get("/api/v1/supply");
        assert_eq!(supply["height"], 104);
        assert_eq!(supply["matches_schedule"], true);
        assert_eq!(
            supply["mined_total"],
            quanta(info.mined_total.amount().as_quanta()).as_str()
        );
        assert_eq!(supply["fees_total"], "0");

        // 6. Explorer dibuka ulang dari file indeks: isi indeks tetap tanpa sinkronisasi.
        drop(explorer);
        let explorer = Explorer::open(&rt, &db, &url_a);
        let status = explorer.get("/api/v1/status");
        assert_eq!(status["indexed_height"], 104);
        assert_eq!(status["indexed_block_id"], hex(b_tip.1));
        assert_eq!(status["node_height"], Value::Null);
        assert_eq!(
            explorer.get("/api/v1/block/100")["block_id"],
            b_100.as_str()
        );

        // 7. A menambang blok 105 berisi transaksi dari mempool. Setiap blok di indeks sama dengan chain aktif node,
        //    dan pasokan sama dengan jadwal MOD-04.
        assert_eq!(miner_a.run(1, |_| {}).unwrap(), 1);
        let status = explorer.sync();
        assert_eq!(status["indexed_height"], 105);
        assert_eq!(status["mempool_count"], 0);
        let detail = explorer.get(&format!("/api/v1/tx/{tx_hex}"));
        assert_eq!(detail["height"], 105);
        assert_eq!(detail["confirmations"], 1);
        assert_eq!(detail["block_id"], hex(a.block_hash(105).unwrap()).as_str());
        let allocation = explorer.allocation();
        assert_eq!(allocation.unspent.as_quanta(), 0);
        assert_eq!(
            allocation.outputs[0].spent_by.as_ref().map(|by| by.height),
            Some(105)
        );
        for height in 0..=105_u64 {
            let block = explorer.get(&format!("/api/v1/block/{height}"));
            assert_eq!(block["height"], height);
            assert_eq!(
                block["block_id"],
                hex(a.block_hash(height).unwrap()).as_str(),
                "tinggi {height}"
            );
        }
        let newest: Vec<u64> = (86..=105).rev().collect();
        assert_eq!(heights(&explorer.get("/api/v1/blocks")), newest);
        let older: Vec<u64> = (66..=85).rev().collect();
        assert_eq!(heights(&explorer.get("/api/v1/blocks?before=86")), older);
        let info = a.chain_info().unwrap();
        let scheduled = SupplySchedule::new().nth(105).unwrap().mined_total_after;
        assert_eq!(info.mined_total.amount(), scheduled);
        let supply = explorer.get("/api/v1/supply");
        assert_eq!(supply["height"], 105);
        assert_eq!(supply["matches_schedule"], true);
        assert_eq!(
            supply["scheduled_mined_total"],
            quanta(scheduled.as_quanta()).as_str()
        );
        assert_eq!(
            supply["mined_total"],
            quanta(info.mined_total.amount().as_quanta()).as_str()
        );
        assert!(supply.get("total_supply").is_none(), "{supply}");
        assert_eq!(supply["fees_total"], quanta(FEE).as_str());

        // 8. Node A berhenti: langkah sinkronisasi gagal dengan last_error, indeks terakhir tetap dilayani.
        drop(miner_a);
        drop(miner_b);
        drop(client_a);
        rt.block_on(server_a.stop());
        explorer.shared.sync_once(&explorer.source);
        let status = explorer.get("/api/v1/status");
        assert!(status["last_error"].is_string(), "{status}");
        assert_eq!(status["index_error"], Value::Null);
        assert_eq!(status["indexed_height"], 105);
        assert_eq!(explorer.get("/api/v1/block/105")["height"], 105);

        drop(explorer);
        rt.block_on(server_b.stop());
    }
}
