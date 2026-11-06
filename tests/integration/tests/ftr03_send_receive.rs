//! FTR-03 (TSK-F03): kirim dan terima ZVRG lewat CLI di regtest, dari ujung ke ujung.
//!
//! Node (MOD-11, `RedbStore` di memori), server RPC (MOD-08), miner (MOD-13), dan perintah wallet CLI (MOD-12, DEC-23)
//! berjalan di satu proses lewat RPC HTTP sungguhan. Selesai bila (Dokumen fitur FTR-03): wallet CLI membuat alamat,
//! menerima coinbase yang sudah matang (100 blok), mengirim ke wallet lain, dan transaksi terkonfirmasi. Node
//! memeriksa setiap transaksi dengan aturan T1 sampai T6 (MOD-04). Selama transaksi menunggu di mempool, saldo kedua
//! wallet memperhitungkannya (`scan_mempool`, TSK-M36). Jam node disuntikkan, maju 120 detik per blok, sehingga
//! target tetap `pow_limit` dan hasilnya deterministik.

#![forbid(unsafe_code)]

#[cfg(test)]
mod ftr03 {
    use std::net::SocketAddr;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    use tokio::runtime::Runtime;
    use zevrigo_cli::{
        Balance, Cli, CliError, Protection, Sent, next_address, wallet_create, wallet_restore,
    };
    use zevrigo_consensus::{
        ChainParams, REGTEST_GENESIS_TIMESTAMP, ScheduleEntry, SupplySchedule,
    };
    use zevrigo_crypto::SecretKey;
    use zevrigo_mempool::DEFAULT_MAX_BYTES;
    use zevrigo_miner::Miner;
    use zevrigo_node::{Clock, Node};
    use zevrigo_primitives::{Amount, Block, Decode, Network, OutputKind};
    use zevrigo_rpc::{Hex32, KindNumber, RpcClient, RpcServer};
    use zevrigo_storage::RedbStore;
    use zevrigo_wallet_core::{KdfParams, SendError, decode_address};

    /// Dikirim wallet A ke wallet B: 0,005 ZVRG (butuh dua coinbase matang A, sehingga ada dua input dan kembalian).
    const A_TO_B: u128 = 5_000_000_000_000_000;
    /// Dikirim balik wallet B ke wallet A: 0,001 ZVRG (satu input bukan coinbase, dengan kembalian).
    const B_TO_A: u128 = 1_000_000_000_000_000;
    /// Tarif biaya bawaan CLI: 1 quanta per byte.
    const FEE_RATE: u128 = 1;

    /// Folder sementara khusus test ini, dihapus saat selesai.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("zevrigo-q75-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Kunci x-only dari kunci rahasia `n`, untuk penerima hadiah yang bukan milik kedua wallet.
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

    /// Payload x-only dari alamat regtest.
    fn payload(address: &str) -> [u8; 32] {
        let (kind, payload) = decode_address(Network::Regtest, address).unwrap();
        assert_eq!(kind, OutputKind::SingleKey);
        payload
    }

    fn miner(url: &str, payout: [u8; 32]) -> Miner {
        Miner::connect(
            url,
            KindNumber::new(OutputKind::SingleKey),
            Hex32::new(payout),
            1_000_000,
        )
        .unwrap()
    }

    /// Jumlah subsidy jadwal pasokan untuk tinggi `from..=to`.
    fn subsidies(schedule: &[ScheduleEntry], from: usize, to: usize) -> Amount {
        Amount::checked_sum(schedule[from..=to].iter().map(|step| step.subsidy)).unwrap()
    }

    fn quanta(value: u128) -> Amount {
        Amount::from_quanta(value)
    }

    /// Password kedua wallet uji (DEC-27).
    const PASSWORD: &str = "password uji ftr03";

    /// Kunci file wallet dengan [`PASSWORD`], Argon2id ringan (8 KiB, 1 putaran) agar uji cepat; `fill` membedakan
    /// salt dan nonce.
    fn lock(fill: u8) -> Protection<'static> {
        Protection {
            password: PASSWORD,
            kdf: KdfParams {
                memory_kib: 8,
                iterations: 1,
                parallelism: 1,
            },
            salt: [fill; 16],
            nonce: [fill; 24],
        }
    }

    fn send(cli: &Cli, datadir: &Path, to: &str, amount: u128, aux: u8) -> Result<Sent, CliError> {
        cli.send(
            datadir,
            "",
            PASSWORD,
            to,
            quanta(amount),
            FEE_RATE,
            &[aux; 32],
        )
    }

    /// Txid semua transaksi di blok setinggi `height`, dan jumlah output coinbase-nya.
    fn block_at(rt: &Runtime, client: &RpcClient, height: u64) -> (Vec<Hex32>, Amount) {
        let id = rt.block_on(client.get_block_hash(height)).unwrap();
        let info = rt.block_on(client.get_block(id)).unwrap();
        let block = Block::from_bytes(info.hex.as_slice()).unwrap();
        let coinbase = Amount::checked_sum(
            block.txs()[0]
                .body()
                .outputs()
                .iter()
                .map(|output| output.amount()),
        )
        .unwrap();
        (info.txids, coinbase)
    }

    #[test]
    fn two_cli_wallets_send_and_receive_over_rpc() {
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
        let cli = Cli::connect(&url, Network::Regtest).unwrap();
        let schedule: Vec<_> = SupplySchedule::new().take(104).collect();

        // Dua wallet baru (entropi tetap agar deterministik; biner memakai /dev/urandom).
        let dir_a = TempDir::new("a");
        let dir_b = TempDir::new("b");
        let (mnemonic_a, _) =
            wallet_create(Network::Regtest, &dir_a.0, &[0x11; 32], "", &lock(1)).unwrap();
        wallet_create(Network::Regtest, &dir_b.0, &[0x22; 32], "", &lock(2)).unwrap();
        let address_a = next_address(Network::Regtest, &dir_a.0, "").unwrap();
        assert!(address_a.starts_with("rzvrg1q"), "{address_a}");

        // 99 blok ke alamat A: belum ada coinbase yang matang, jadi belum bisa mengirim.
        let miner_a = miner(&url, payload(&address_a));
        assert_eq!(miner_a.run(99, |_| {}).unwrap(), 99);
        assert_eq!(
            cli.balance(&dir_a.0, "").unwrap(),
            Balance {
                tip_height: 99,
                total: subsidies(&schedule, 1, 99),
                spendable: Amount::ZERO,
                immature: subsidies(&schedule, 1, 99),
                pending_spent: Amount::ZERO,
                pending_received: Amount::ZERO,
                after_pending: subsidies(&schedule, 1, 99),
                utxos: 99,
            }
        );
        let address_b = next_address(Network::Regtest, &dir_b.0, "").unwrap();
        assert!(matches!(
            send(&cli, &dir_a.0, &address_b, A_TO_B, 1),
            Err(CliError::Send(SendError::InsufficientFunds { .. }))
        ));

        // Dua blok lagi: di tinggi 101, coinbase tinggi 1 dan 2 boleh dibelanjakan di blok 102.
        assert_eq!(miner_a.run(2, |_| {}).unwrap(), 2);
        let mature = subsidies(&schedule, 1, 2);
        let balance_a = cli.balance(&dir_a.0, "").unwrap();
        assert_eq!(balance_a.tip_height, 101);
        assert_eq!(balance_a.total, subsidies(&schedule, 1, 101));
        assert_eq!(balance_a.spendable, mature);
        assert_eq!(balance_a.utxos, 101);

        // A mengirim ke B: dua input coinbase, kembalian ke cabang 1 indeks 0.
        let sent_ab = send(&cli, &dir_a.0, &address_b, A_TO_B, 1).unwrap();
        assert_eq!(sent_ab.inputs, 2);
        assert_eq!(sent_ab.amount, quanta(A_TO_B));
        assert!(sent_ab.fee.as_quanta() >= u128::try_from(sent_ab.size).unwrap() * FEE_RATE);
        let change_a = sent_ab.change.unwrap();
        assert_eq!(
            mature,
            quanta(A_TO_B)
                .checked_add(sent_ab.fee)
                .and_then(|sum| sum.checked_add(change_a))
                .unwrap()
        );
        assert_eq!(
            rt.block_on(client.get_mempool()).unwrap().txids,
            vec![sent_ab.txid]
        );

        // Selama transaksi menunggu di mempool: dua coinbase A sedang dibelanjakan, kembalian A dan kiriman ke B
        // tercatat masuk tertunda, dan koin yang sedang dibelanjakan tidak dipilih lagi.
        let pending_a = cli.balance(&dir_a.0, "").unwrap();
        assert_eq!(pending_a.total, balance_a.total);
        assert_eq!(pending_a.spendable, Amount::ZERO);
        assert_eq!(pending_a.pending_spent, mature);
        assert_eq!(pending_a.pending_received, change_a);
        assert_eq!(
            pending_a.after_pending,
            balance_a
                .total
                .checked_sub(mature)
                .and_then(|rest| rest.checked_add(change_a))
                .unwrap()
        );
        let pending_b = cli.balance(&dir_b.0, "").unwrap();
        assert_eq!(pending_b.total, Amount::ZERO);
        assert_eq!(pending_b.pending_received, quanta(A_TO_B));
        assert_eq!(pending_b.after_pending, quanta(A_TO_B));
        assert!(matches!(
            send(&cli, &dir_a.0, &address_b, A_TO_B, 3),
            Err(CliError::Send(SendError::InsufficientFunds { .. }))
        ));

        // Blok 102 ditambang ke kunci lain, agar saldo kedua wallet hanya berubah karena transaksi.
        let miner_other = miner(&url, key(9));
        assert_eq!(miner_other.run(1, |_| {}).unwrap(), 1);
        let (txids, coinbase) = block_at(&rt, &client, 102);
        assert_eq!(txids.len(), 2);
        assert_eq!(txids[1], sent_ab.txid);
        // Biaya yang dihitung CLI sama dengan biaya yang diklaim coinbase (DEC-17).
        assert_eq!(
            coinbase,
            schedule[102].subsidy.checked_add(sent_ab.fee).unwrap()
        );
        assert!(rt.block_on(client.get_mempool()).unwrap().txids.is_empty());
        assert_eq!(
            cli.balance(&dir_b.0, "").unwrap(),
            Balance {
                tip_height: 102,
                total: quanta(A_TO_B),
                spendable: quanta(A_TO_B),
                immature: Amount::ZERO,
                pending_spent: Amount::ZERO,
                pending_received: Amount::ZERO,
                after_pending: quanta(A_TO_B),
                utxos: 1,
            }
        );

        // B mengirim sebagian balik ke alamat A yang baru: satu input bukan coinbase, dengan kembalian.
        let address_a2 = next_address(Network::Regtest, &dir_a.0, "").unwrap();
        assert_ne!(address_a2, address_a);
        let sent_ba = send(&cli, &dir_b.0, &address_a2, B_TO_A, 2).unwrap();
        assert_eq!(sent_ba.inputs, 1);
        let change_b = sent_ba.change.unwrap();
        assert_eq!(
            quanta(A_TO_B),
            quanta(B_TO_A)
                .checked_add(sent_ba.fee)
                .and_then(|sum| sum.checked_add(change_b))
                .unwrap()
        );
        assert_eq!(miner_other.run(1, |_| {}).unwrap(), 1);
        let (txids, coinbase) = block_at(&rt, &client, 103);
        assert_eq!(txids.get(1), Some(&sent_ba.txid));
        assert_eq!(
            coinbase,
            schedule[103].subsidy.checked_add(sent_ba.fee).unwrap()
        );

        // Saldo akhir di tinggi 103 (blok 104 boleh membelanjakan coinbase tinggi 3 dan 4).
        assert_eq!(
            cli.balance(&dir_b.0, "").unwrap(),
            Balance {
                tip_height: 103,
                total: change_b,
                spendable: change_b,
                immature: Amount::ZERO,
                pending_spent: Amount::ZERO,
                pending_received: Amount::ZERO,
                after_pending: change_b,
                utxos: 1,
            }
        );
        let spendable_a = subsidies(&schedule, 3, 4)
            .checked_add(change_a)
            .and_then(|sum| sum.checked_add(quanta(B_TO_A)))
            .unwrap();
        let immature_a = subsidies(&schedule, 5, 101);
        let total_a = spendable_a.checked_add(immature_a).unwrap();
        let expected_a = Balance {
            tip_height: 103,
            total: total_a,
            spendable: spendable_a,
            immature: immature_a,
            pending_spent: Amount::ZERO,
            pending_received: Amount::ZERO,
            after_pending: total_a,
            utxos: 101,
        };
        assert_eq!(cli.balance(&dir_a.0, "").unwrap(), expected_a);

        // Wallet yang dipulihkan dari frasa A melihat saldo yang sama, termasuk kembalian di cabang 1.
        let dir_restored = TempDir::new("restored");
        wallet_restore(
            Network::Regtest,
            &dir_restored.0,
            mnemonic_a.phrase(),
            "",
            &lock(3),
        )
        .unwrap();
        assert_eq!(cli.balance(&dir_restored.0, "").unwrap(), expected_a);

        // Pasokan tetap sama dengan jadwal: transaksi hanya memindahkan koin, biaya masuk ke penambang.
        let info = rt.block_on(client.get_chain_info()).unwrap();
        assert_eq!(info.height, 103);
        assert_eq!(info.mined_total.amount(), schedule[103].mined_total_after);
        assert_eq!(info.mempool_count, 0);

        drop(cli);
        drop(client);
        drop(miner_a);
        drop(miner_other);
        rt.block_on(server.stop());
    }
}
