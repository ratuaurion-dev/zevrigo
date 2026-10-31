//! Uji integrasi daemon `homeostasis-node` lewat RPC sungguhan (TSK-M18 bagian B, TSK-F01 node regtest tunggal).
//!
//! Biner dijalankan di folder data sementara dengan port RPC bebas, satu blok ditambang lewat `get_block_template`
//! dan `submit_block`, hadiahnya terlihat di `scan_utxos`, lalu node dihentikan dengan SIGINT dan dibuka ulang.

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

    use homeostasis_consensus::{BlockError, ChainParams, check_block_context_free};
    use homeostasis_crypto::SecretKey;
    use homeostasis_primitives::{Block, Decode, Encode, Header, OutputKind};
    use homeostasis_rpc::{Hex32, KindNumber, RpcClient, SubmitStatus};
    use tokio::runtime::Runtime;

    const BIN: &str = env!("CARGO_BIN_EXE_homeostasis-node");
    /// Subsidy tinggi 1 (Spesifikasi Format Bagian 5.5).
    const SUBSIDY_1: u128 = 1_910_127_426_536_917;

    /// Node yang sedang berjalan. Stdout tetap dipegang agar pipe tidak tertutup selama node hidup.
    struct Running {
        child: Child,
        _stdout: BufReader<ChildStdout>,
        url: String,
    }

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

    fn start(datadir: &Path) -> Running {
        let mut child = Command::new(BIN)
            .args(["--regtest", "--rpc-port", "0", "--datadir"])
            .arg(datadir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdout = BufReader::new(child.stdout.take().unwrap());
        let mut line = String::new();
        stdout.read_line(&mut line).unwrap();
        let url = line
            .trim()
            .strip_prefix("rpc: ")
            .expect("baris pertama stdout wajib `rpc: <url>`")
            .to_owned();
        Running {
            child,
            _stdout: stdout,
            url,
        }
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
        let datadir = std::env::temp_dir().join(format!("homeostasis-q68-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&datadir);
        let rt = Runtime::new().unwrap();
        let payout = Hex32::new(key(2));

        let node = start(&datadir);
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

        let reopened = start(&datadir);
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
}
