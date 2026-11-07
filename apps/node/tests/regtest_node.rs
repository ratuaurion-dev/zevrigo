//! Uji integrasi daemon `zevrigo-node` lewat RPC dan P2P sungguhan (TSK-M18 bagian B, TSK-F01, TSK-M28).
//!
//! Biner dijalankan di folder data sementara dengan port RPC dan P2P bebas. Uji pertama menambang satu blok lewat
//! `get_block_template` dan `submit_block`, memeriksa `scan_utxos`, lalu menghentikan node dengan SIGINT dan
//! membukanya ulang (PeerId tetap). Uji kedua menjalankan dua node: node kedua mengejar 100 blok lewat P2P, lalu
//! transaksi dan blok baru disiarkan ke dua arah.

#![forbid(unsafe_code)]
// Uji ini memakai sinyal Unix (`kill -INT`). Di platform lain crate test ini kosong.
#![cfg(unix)]

#[cfg(test)]
mod regtest_node {
    use std::io::{BufRead, BufReader, Read};
    use std::path::Path;
    use std::process::{Child, ChildStdout, Command, Stdio};
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    use tokio::runtime::Runtime;
    use zevrigo_consensus::{BlockError, ChainParams, check_block_context_free, regtest_genesis};
    use zevrigo_crypto::{SecretKey, sighash_v1, sign, txid};
    use zevrigo_primitives::{
        Amount, Block, Decode, Encode, Header, Input, Output, OutputKind, Transaction, Witness,
    };
    use zevrigo_rpc::{Hex32, KindNumber, RpcClient, SubmitStatus};

    const BIN: &str = env!("CARGO_BIN_EXE_zevrigo-node");
    /// Subsidy tinggi 1 (Spesifikasi Format Bagian 5.5).
    const SUBSIDY_1: u128 = 4_361_021_521_773_785;

    /// Node yang sedang berjalan. Stdout tetap dipegang agar pipe tidak tertutup selama node hidup.
    struct Running {
        child: Child,
        _stdout: BufReader<ChildStdout>,
        url: String,
        /// Alamat P2P lengkap dari baris `p2p: ...`.
        p2p: String,
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

    /// Membaca satu baris stdout yang wajib diawali `prefix`, lalu mengembalikan sisanya.
    fn read_prefixed(stdout: &mut BufReader<ChildStdout>, prefix: &str) -> String {
        let mut line = String::new();
        stdout.read_line(&mut line).unwrap();
        line.trim()
            .strip_prefix(prefix)
            .expect("baris stdout tidak diawali awalan yang diharapkan")
            .to_owned()
    }

    fn start(datadir: &Path, extra: &[&str]) -> Running {
        let mut child = Command::new(BIN)
            .args([
                "--regtest",
                "--rpc-port",
                "0",
                "--p2p-port",
                "0",
                "--datadir",
            ])
            .arg(datadir)
            .args(extra)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdout = BufReader::new(child.stdout.take().unwrap());
        let url = read_prefixed(&mut stdout, "rpc: ");
        let p2p = read_prefixed(&mut stdout, "p2p: ");
        assert!(p2p.starts_with("/ip4/127.0.0.1/tcp/"), "{p2p}");
        Running {
            child,
            _stdout: stdout,
            url,
            p2p,
        }
    }

    /// PeerId di akhir alamat P2P.
    fn peer_id(p2p: &str) -> &str {
        p2p.rsplit("/p2p/").next().unwrap()
    }

    /// Mengirim SIGINT, menunggu node keluar dengan kode 0, lalu mengembalikan log stderr.
    fn stop(mut running: Running) -> String {
        let pid = running.child.id().to_string();
        let sent = Command::new("kill")
            .args(["-INT", pid.as_str()])
            .status()
            .unwrap();
        assert!(sent.success());
        let deadline = Instant::now() + Duration::from_secs(30);
        let exit = loop {
            if let Some(exit) = running.child.try_wait().unwrap() {
                break exit;
            }
            if Instant::now() > deadline {
                let _ = running.child.kill();
            }
            assert!(
                Instant::now() <= deadline,
                "node tidak berhenti dalam 30 detik"
            );
            sleep(Duration::from_millis(50));
        };
        let mut log = String::new();
        running
            .child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut log)
            .unwrap();
        assert!(exit.success(), "status {exit:?}, log:\n{log}");
        log
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

    #[test]
    fn daemon_mines_over_rpc_and_restarts() {
        let datadir = std::env::temp_dir().join(format!("zevrigo-q68-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&datadir);
        let rt = Runtime::new().unwrap();
        let payout = Hex32::new(key(2));

        let node = start(&datadir, &[]);
        let first_p2p = node.p2p.clone();
        let client = rt.block_on(async { RpcClient::new(&node.url) }).unwrap();
        let genesis = rt.block_on(client.get_chain_info()).unwrap();
        assert_eq!(genesis.height, 0);
        assert_eq!(
            Some(genesis.best_block_id),
            rt.block_on(client.get_block_hash(0)).ok()
        );

        let template = rt
            .block_on(client.get_block_template(KindNumber::new(OutputKind::SingleKey), payout))
            .unwrap();
        assert_eq!(template.height, 1);
        assert_eq!(template.prev_block_id, genesis.best_block_id);
        let block = solve(&Block::from_bytes(template.hex.as_slice()).unwrap());
        let result = rt.block_on(client.submit_block(&block.to_bytes())).unwrap();
        assert_eq!(result.status, SubmitStatus::Accepted, "{result:?}");
        let again = rt.block_on(client.submit_block(&block.to_bytes())).unwrap();
        assert_eq!(again.status, SubmitStatus::Duplicate);

        let tip = rt.block_on(client.get_chain_info()).unwrap();
        assert_eq!(tip.height, 1);
        assert_eq!(Some(tip.best_block_id), result.block_id);
        assert_eq!(tip.mined_total.amount().as_quanta(), SUBSIDY_1);
        let utxos = rt.block_on(client.scan_utxos(&[payout])).unwrap();
        assert_eq!(utxos.len(), 1);
        assert_eq!(utxos[0].amount.amount().as_quanta(), SUBSIDY_1);
        assert!(utxos[0].is_coinbase);
        assert_eq!(utxos[0].confirmations, 1);
        assert_eq!(utxos[0].spendable_at, 101);
        assert!(rt.block_on(client.get_mempool()).unwrap().txids.is_empty());
        drop(client);

        let log = stop(node);
        assert!(log.contains("server RPC berjalan"), "{log}");
        assert!(log.contains("node berhenti"), "{log}");
        assert!(datadir.join("regtest").join("chain.redb").is_file());

        let reopened = start(&datadir, &[]);
        assert_eq!(peer_id(&reopened.p2p), peer_id(&first_p2p));
        assert!(datadir.join("regtest").join("p2p_key").is_file());
        let client = rt
            .block_on(async { RpcClient::new(&reopened.url) })
            .unwrap();
        assert_eq!(rt.block_on(client.get_chain_info()).unwrap(), tip);
        assert_eq!(rt.block_on(client.scan_utxos(&[payout])).unwrap(), utxos);
        drop(client);
        let log = stop(reopened);
        assert!(log.contains("node berhenti"), "{log}");
        std::fs::remove_dir_all(&datadir).unwrap();
    }

    /// Menambang satu blok dari template node lewat RPC; wajib `accepted`. Mengembalikan ID bloknya.
    fn mine(rt: &Runtime, client: &RpcClient, payout: Hex32) -> Hex32 {
        let template = rt
            .block_on(client.get_block_template(KindNumber::new(OutputKind::SingleKey), payout))
            .unwrap();
        let block = solve(&Block::from_bytes(template.hex.as_slice()).unwrap());
        let result = rt.block_on(client.submit_block(&block.to_bytes())).unwrap();
        assert_eq!(result.status, SubmitStatus::Accepted, "{result:?}");
        result.block_id.unwrap()
    }

    /// Menunggu sampai `ready` bernilai benar, paling lama 60 detik.
    fn wait_until(what: &str, mut ready: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(60);
        while !ready() {
            assert!(
                Instant::now() < deadline,
                "menunggu {what} lebih dari 60 detik"
            );
            sleep(Duration::from_millis(100));
        }
    }

    /// Transaksi yang membelanjakan alokasi genesis regtest (kunci 1) ke kunci 3, biaya 1_000 quanta.
    fn spend_genesis() -> Transaction {
        let params = ChainParams::regtest().unwrap();
        let genesis = regtest_genesis().unwrap();
        let coinbase = genesis.txs().first().unwrap();
        let spent = coinbase.body().outputs().first().unwrap().clone();
        let inputs = vec![Input::new(txid(coinbase), 0)];
        let amount = Amount::from_quanta(spent.amount().as_quanta() - 1_000);
        let outputs = vec![Output::new(amount, OutputKind::SingleKey, key(3)).unwrap()];
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
    fn two_daemons_sync_and_relay_over_p2p() {
        let base = std::env::temp_dir().join(format!("zevrigo-q79-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let rt = Runtime::new().unwrap();
        let payout = Hex32::new(key(2));

        // Node A menambang 100 blok sendirian, lalu node B tersambung dan mengejar lewat header dan GetBlocks.
        let a = start(&base.join("a"), &[]);
        let client_a = rt.block_on(async { RpcClient::new(&a.url) }).unwrap();
        let mut tip_100 = Hex32::new([0; 32]);
        for _ in 0..100 {
            tip_100 = mine(&rt, &client_a, payout);
        }
        let b = start(&base.join("b"), &["--connect", a.p2p.as_str()]);
        let client_b = rt.block_on(async { RpcClient::new(&b.url) }).unwrap();
        wait_until("node B mengejar 100 blok", || {
            rt.block_on(client_b.get_chain_info())
                .is_ok_and(|info| info.height == 100 && info.best_block_id == tip_100)
        });

        // Transaksi lewat RPC node B disiarkan ke mempool node A.
        let tx = spend_genesis();
        let sent = rt
            .block_on(client_b.send_raw_transaction(&tx.to_bytes()))
            .unwrap();
        assert_eq!(sent.txid, Hex32::from(txid(&tx)));
        wait_until("transaksi sampai ke node A", || {
            rt.block_on(client_a.get_mempool())
                .is_ok_and(|mempool| mempool.txids == [sent.txid])
        });

        // Blok A berisi transaksi itu disiarkan ke B; blok B disiarkan ke A.
        let tip_101 = mine(&rt, &client_a, payout);
        wait_until("blok 101 dari A sampai ke B", || {
            rt.block_on(client_b.get_chain_info())
                .is_ok_and(|info| info.height == 101 && info.best_block_id == tip_101)
        });
        assert!(
            rt.block_on(client_b.get_mempool())
                .unwrap()
                .txids
                .is_empty()
        );
        assert_eq!(
            rt.block_on(client_b.scan_utxos(&[Hex32::new(key(3))]))
                .unwrap()
                .len(),
            1
        );
        let tip_102 = mine(&rt, &client_b, payout);
        wait_until("blok 102 dari B sampai ke A", || {
            rt.block_on(client_a.get_chain_info())
                .is_ok_and(|info| info.height == 102 && info.best_block_id == tip_102)
        });

        drop(client_a);
        drop(client_b);
        let log_b = stop(b);
        let log_a = stop(a);
        assert!(log_b.contains("rekan terhubung"), "{log_b}");
        assert!(log_a.contains("rekan terhubung"), "{log_a}");
        std::fs::remove_dir_all(&base).unwrap();
    }
}
