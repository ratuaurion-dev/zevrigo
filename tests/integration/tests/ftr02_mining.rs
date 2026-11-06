//! FTR-02 (TSK-F02): menambang di regtest dari ujung ke ujung.
//!
//! Node (MOD-11, `RedbStore` di memori) dan server RPC (MOD-08) berjalan di satu proses; miner (MOD-13) menambang
//! lewat RPC sungguhan. Selesai bila (Dokumen fitur FTR-02): node menerima setiap blok, dan coinbase sama persis
//! dengan `block_subsidy` (jadwal pasokan MOD-04) ditambah biaya transaksi. Jam node disuntikkan, maju 120 detik per
//! blok, sehingga target tetap `pow_limit` dan hasilnya deterministik.

#![forbid(unsafe_code)]

#[cfg(test)]
mod ftr02 {
    use std::net::SocketAddr;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    use tokio::runtime::Runtime;
    use zevrigo_consensus::{
        ChainParams, REGTEST_GENESIS_TIMESTAMP, SupplySchedule, regtest_genesis,
    };
    use zevrigo_crypto::{SecretKey, sighash_v1, sign, txid};
    use zevrigo_mempool::DEFAULT_MAX_BYTES;
    use zevrigo_miner::Miner;
    use zevrigo_node::{Clock, Node};
    use zevrigo_primitives::params::GENESIS_ALLOCATION;
    use zevrigo_primitives::{
        Amount, Block, Decode, Encode, Input, Output, OutputKind, Transaction, Witness,
    };
    use zevrigo_rpc::{Hex32, KindNumber, RpcClient, RpcServer};
    use zevrigo_storage::RedbStore;

    /// Biaya transaksi yang dimuat blok 100.
    const FEE: u128 = 1_000;
    /// Target compact `pow_limit` regtest.
    const LIMIT: u32 = 0x207f_ffff;

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

    /// Chainwork 256-bit big-endian bernilai `n`.
    fn work(n: u8) -> Hex32 {
        let mut bytes = [0_u8; 32];
        if let Some(last) = bytes.last_mut() {
            *last = n;
        }
        Hex32::new(bytes)
    }

    /// Membelanjakan alokasi genesis (kunci rahasia 1) ke kunci 3 dengan biaya [`FEE`].
    fn spend_genesis(params: &ChainParams) -> Transaction {
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
    fn miner_and_node_follow_supply_schedule() {
        let params = ChainParams::regtest().unwrap();
        // Jam dipanggil dua kali per blok (template, lalu V8 saat submit): maju 60 detik per panggilan.
        let ticks = Arc::new(AtomicU64::new(REGTEST_GENESIS_TIMESTAMP + 120));
        let clock_ticks = Arc::clone(&ticks);
        let clock: Clock = Box::new(move || clock_ticks.fetch_add(60, Ordering::SeqCst));
        let store = RedbStore::in_memory().unwrap();
        let node = Arc::new(Node::open(store, params, DEFAULT_MAX_BYTES, clock).unwrap());

        let rt = Runtime::new().unwrap();
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let server = rt
            .block_on(RpcServer::start(addr, Arc::clone(&node)))
            .unwrap();
        let url = format!("http://{}", server.local_addr());
        let client = rt.block_on(async { RpcClient::new(&url) }).unwrap();
        let payout = Hex32::new(key(2));
        let miner = Miner::connect(
            &url,
            KindNumber::new(OutputKind::SingleKey),
            payout,
            1_000_000,
        )
        .unwrap();

        // 99 blok tanpa transaksi; alokasi genesis matang di blok 100.
        assert_eq!(miner.run(99, |_| {}).unwrap(), 99);
        let tx = spend_genesis(&params);
        let sent = rt
            .block_on(client.send_raw_transaction(&tx.to_bytes()))
            .unwrap();
        assert_eq!(sent.txid, Hex32::from(txid(&tx)));
        assert_eq!(
            rt.block_on(client.get_mempool()).unwrap().txids,
            vec![sent.txid]
        );
        assert_eq!(miner.run(1, |_| {}).unwrap(), 1);

        let schedule: Vec<_> = SupplySchedule::new().take(101).collect();
        let mined_total = schedule[100].mined_total_after;
        let info = rt.block_on(client.get_chain_info()).unwrap();
        assert_eq!(info.height, 100);
        assert_eq!(info.mined_total.amount(), mined_total);
        assert_eq!(
            info.total_supply.amount(),
            GENESIS_ALLOCATION.checked_add(mined_total).unwrap()
        );
        assert_eq!(info.next_target.get(), LIMIT);
        assert_eq!(info.chainwork, work(202));
        assert_eq!(info.mempool_count, 0);

        for height in 1..=100_u64 {
            let id = rt.block_on(client.get_block_hash(height)).unwrap();
            let block = rt.block_on(client.get_block(id)).unwrap();
            assert_eq!(block.height, height);
            let decoded = Block::from_bytes(block.hex.as_slice()).unwrap();
            let header = decoded.header();
            assert_eq!(header.timestamp(), REGTEST_GENESIS_TIMESTAMP + 120 * height);
            assert_eq!(header.target(), LIMIT);
            let fee = if height == 100 { FEE } else { 0 };
            let expected = schedule[usize::try_from(height).unwrap()]
                .subsidy
                .checked_add(Amount::from_quanta(fee))
                .unwrap();
            let outputs = decoded.txs()[0].body().outputs();
            assert_eq!(outputs.len(), 1, "tinggi {height}");
            assert_eq!(outputs[0].amount(), expected, "tinggi {height}");
            assert_eq!(outputs[0].payload(), &key(2));
            let tx_count = if height == 100 { 2 } else { 1 };
            assert_eq!(decoded.txs().len(), tx_count, "tinggi {height}");
        }

        let rewards = rt.block_on(client.scan_utxos(&[payout])).unwrap();
        assert_eq!(rewards.len(), 100);
        assert!(rewards.iter().all(|utxo| utxo.is_coinbase));
        let total = Amount::checked_sum(rewards.iter().map(|utxo| utxo.amount.amount())).unwrap();
        assert_eq!(
            total,
            mined_total.checked_add(Amount::from_quanta(FEE)).unwrap()
        );
        let received = rt
            .block_on(client.scan_utxos(&[Hex32::new(key(3))]))
            .unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].txid, sent.txid);
        assert_eq!(
            received[0].amount.amount().as_quanta(),
            GENESIS_ALLOCATION.as_quanta() - FEE
        );
        assert!(!received[0].is_coinbase);
        assert_eq!(received[0].height, 100);
        assert!(
            rt.block_on(client.scan_utxos(&[Hex32::new(key(1))]))
                .unwrap()
                .is_empty()
        );

        drop(client);
        drop(miner);
        rt.block_on(server.stop());
    }
}
