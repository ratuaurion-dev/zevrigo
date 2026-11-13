// Uji perintah wallet terhadap node tiruan di atas `RpcServer` (MOD-08). Skenario kirim memakai koin, kunci, dan
// network_id yang sama dengan vektor T1 TSK-M16, sehingga byte transaksi yang diterima node wajib sama persis
// dengan T1. Alamat W5, W6, W9 dan frasa W1, W8: Spesifikasi Wallet v0.1 Bagian 4.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tokio::runtime::Runtime;
use zeroize::Zeroizing;
use zevrigo_primitives::{Amount, Network, OutputKind};
use zevrigo_rpc::{
    BlockInfo, BlockTemplate, ChainInfo, CompactHex, GetBlockTemplateParams, Hex32, KindNumber,
    MempoolInfo, MempoolOutput, MempoolScan, MempoolSpend, NetworkName, Quanta, RpcBackend,
    RpcError, RpcServer, SendRawTransactionResult, SubmitBlockResult, UtxoInfo,
};
use zevrigo_wallet_core::{AddressError, KdfParams, Mnemonic, SendError, decode_address};

use super::{
    Balance, Cli, CliError, PasswordPrompt, Protection, Sent, check_new_password, format_balance,
    format_history, format_info, format_sent, next_address, os_random, run, show_phrase,
    wallet_create, wallet_encrypt, wallet_restore,
};
use crate::args::{Action, Config};
use crate::history::{History, HistoryEntry};
use crate::wallet_file::{NextIndex, WalletFile, WalletFileError, create_new, load, wallet_path};

const ZVRG: u128 = 1_000_000_000_000_000_000;
const W1: &str = concat!(
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ",
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
);
const W5: &str = "rzvrg1qtfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939mseq59qlxg";
const W6: &str = "rzvrg1q3uesn8pvcdeeufxr98v794e07l8ghd85fyd3wclknj5xj04rfqrqyeht59";
const W8_PHRASE: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";
const W9: &str = "rzvrg1q2x064qcn4zk84y9zjtpjt8e8ksjjja7peenap4lcse2d0vmw2xyqm2rpq4";
const W5_XONLY: &str = "5a6f28253f35e2ff1ab0b086b8c87f731159661157e5be2ca98188369625dc32";
const W6_XONLY: &str = "8f33099c2cc3739e24c329d9e2d72ff7ce8bb4f4491b1763f69ca8693ea34806";
const W7_XONLY: &str = "c71cdd713224479fff24b56db524c2385acf09725605e7440de0c34272e2abf5";
/// Alamat regtest kunci contoh BIP340 #0 (Spesifikasi Format Bagian 8).
const RECIPIENT: &str = "rzvrg1qlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmusf2k5zq";
const TESTNET_RECIPIENT: &str = "tzvrg1qlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmusgxrxjr";
/// Alamat W5 di testnet (vektor W10 Spesifikasi Wallet v0.1 Bagian 4).
const W10: &str = "tzvrg1qtfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939mseq4f4dkt";
const T1_TX: &str = concat!(
    "010002a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a100b2b2b2b2b2b2b2b2b2b2b2b2",
    "b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b201028080e0e2de8692a25300f9308a019258c31049344f85f89d5229",
    "b531c845836f99b08601f113bce036f9e4fd9ff6f4acdbe01b00730ad008ce8bd2f391b08b9c4838af732341e9704cba",
    "0dc03d83f977dfcf7d960064257c38bb427b5c650d6f334c5fda2ff216894ac89e2d1a94e8b49912bcae594cdd96ea3b",
    "ad773da077d80b65df62eab8f37376a275e1b7d415379131cadb1800a12f89879bdfc101915b914572c2be38bed60b15",
    "41fd6c8078a67df06dd7fcc4e6cbd56b8917931dddbf0a493b6c60262197af6df8f8444618de163657e73e82",
);

/// Password wallet uji.
const PASSWORD: &str = "kata sandi uji";

/// Argon2id ringan untuk uji (8 KiB, 1 putaran); CLI memakai `KdfParams::DEFAULT`.
const LIGHT_KDF: KdfParams = KdfParams {
    memory_kib: 8,
    iterations: 1,
    parallelism: 1,
};

/// Kunci file wallet uji dengan [`PASSWORD`]; `fill` membedakan salt dan nonce.
fn lock(fill: u8) -> Protection<'static> {
    Protection {
        password: PASSWORD,
        kdf: LIGHT_KDF,
        salt: [fill; 16],
        nonce: [fill; 24],
    }
}

/// Kembalian T1: 8 ZVRG − 6 ZVRG − biaya 284.
const T1_CHANGE: u128 = 2 * ZVRG - 284;
const FIFTEEN_ZVRG: u128 = 15 * ZVRG;
const EIGHT_ZVRG: u128 = 8 * ZVRG;
const SEVEN_ZVRG: u128 = 7 * ZVRG;
const SIX_ZVRG: u128 = 6 * ZVRG;
const FIVE_ZVRG: u128 = 5 * ZVRG;
const EIGHTEEN_ZVRG: u128 = 18 * ZVRG;
const FOUR_ZVRG: u128 = 4 * ZVRG;
const THREE_ZVRG: u128 = 3 * ZVRG;
const TWO_ZVRG: u128 = 2 * ZVRG;
const HUNDRED_ZVRG: u128 = 100 * ZVRG;

fn decode_hex(hex: &str) -> Vec<u8> {
    let (pairs, rem) = hex.as_bytes().as_chunks::<2>();
    assert!(rem.is_empty(), "panjang string hex harus genap");
    let mut out = Vec::with_capacity(pairs.len());
    for &[hi, lo] in pairs {
        let hi_val = char::from(hi).to_digit(16).expect("hex nibble valid");
        let lo_val = char::from(lo).to_digit(16).expect("hex nibble valid");
        let byte = u8::try_from(hi_val.wrapping_shl(4) | lo_val).expect("byte muat u8");
        out.push(byte);
    }
    out
}

fn hex32(hex: &str) -> Hex32 {
    Hex32::try_from(hex).unwrap()
}

/// Folder sementara khusus satu test, dihapus saat selesai.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("zevrigo-q74-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn coin(
    fill: u8,
    index: u32,
    quanta: u128,
    payload: &str,
    coinbase: bool,
    spendable_at: u64,
) -> UtxoInfo {
    UtxoInfo {
        txid: Hex32::new([fill; 32]),
        index,
        amount: Quanta::new(Amount::from_quanta(quanta)),
        kind: KindNumber::new(OutputKind::SingleKey),
        payload: hex32(payload),
        height: 1,
        is_coinbase: coinbase,
        confirmations: 1,
        spendable_at,
    }
}

/// Koin T1: 5 ZVRG di 0/0 (coinbase matang), 3 ZVRG di 0/1, 7 ZVRG di 1/0 (coinbase belum matang).
fn t1_utxos() -> Vec<UtxoInfo> {
    vec![
        coin(0xa1, 0, FIVE_ZVRG, W5_XONLY, true, 101),
        coin(0xb2, 1, THREE_ZVRG, W6_XONLY, false, 50),
        coin(0xc3, 0, SEVEN_ZVRG, W7_XONLY, true, 200),
    ]
}

/// Node tiruan: tinggi ujung 150, `network_id` 5a × 32, UTXO dan transaksi tertunda tetap, mencatat transaksi yang
/// dikirim.
struct MockNode {
    network: Network,
    utxos: Vec<UtxoInfo>,
    pending: MempoolScan,
    reject: Option<String>,
    sent: Mutex<Vec<Vec<u8>>>,
}

impl RpcBackend for MockNode {
    fn chain_info(&self) -> Result<ChainInfo, RpcError> {
        Ok(ChainInfo {
            network: NetworkName::new(self.network),
            height: 150,
            best_block_id: Hex32::new([0x0b; 32]),
            chainwork: Hex32::new([0; 32]),
            next_target: CompactHex::new(0x207f_ffff),
            median_time_past: 1_800_000_000,
            mined_total: Quanta::new(Amount::from_quanta(FIVE_ZVRG)),
            total_supply: Quanta::new(Amount::from_quanta(SIX_ZVRG)),
            mempool_count: 0,
        })
    }

    fn block_hash(&self, height: u64) -> Result<Hex32, RpcError> {
        if height == 0 {
            Ok(Hex32::new([0x5a; 32]))
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
        if let Some(reason) = &self.reject {
            return Err(RpcError::InvalidTransaction {
                reason: reason.clone(),
            });
        }
        self.sent.lock().unwrap().push(tx.to_vec());
        Ok(SendRawTransactionResult {
            txid: Hex32::new([0xee; 32]),
        })
    }

    fn mempool(&self) -> Result<MempoolInfo, RpcError> {
        Ok(MempoolInfo { txids: Vec::new() })
    }

    fn scan_utxos(&self, payloads: &[Hex32]) -> Result<Vec<UtxoInfo>, RpcError> {
        let mut out: Vec<UtxoInfo> = self
            .utxos
            .iter()
            .filter(|utxo| payloads.contains(&utxo.payload))
            .cloned()
            .collect();
        out.sort_by_key(|utxo| (utxo.txid, utxo.index));
        Ok(out)
    }

    fn scan_mempool(&self, payloads: &[Hex32]) -> Result<MempoolScan, RpcError> {
        let owned = |txid: Hex32, index: u32| {
            self.utxos.iter().any(|utxo| {
                utxo.txid == txid && utxo.index == index && payloads.contains(&utxo.payload)
            })
        };
        Ok(MempoolScan {
            spent: self
                .pending
                .spent
                .iter()
                .filter(|spend| owned(spend.txid, spend.index))
                .cloned()
                .collect(),
            outputs: self
                .pending
                .outputs
                .iter()
                .filter(|output| payloads.contains(&output.payload))
                .cloned()
                .collect(),
        })
    }
}

/// Node tiruan di runtime terpisah, dengan klien CLI yang terhubung ke sana.
struct Harness {
    server_rt: Runtime,
    server: Option<RpcServer>,
    node: Arc<MockNode>,
    url: String,
}

impl Harness {
    fn start(network: Network, utxos: Vec<UtxoInfo>, reject: Option<&str>) -> Self {
        Self::launch(network, utxos, MempoolScan::default(), reject)
    }

    fn launch(
        network: Network,
        utxos: Vec<UtxoInfo>,
        pending: MempoolScan,
        reject: Option<&str>,
    ) -> Self {
        let server_rt = Runtime::new().unwrap();
        let node = Arc::new(MockNode {
            network,
            utxos,
            pending,
            reject: reject.map(str::to_owned),
            sent: Mutex::new(Vec::new()),
        });
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let server = server_rt
            .block_on(RpcServer::start(addr, Arc::clone(&node)))
            .unwrap();
        let url = format!("http://{}", server.local_addr());
        Self {
            server_rt,
            server: Some(server),
            node,
            url,
        }
    }

    fn cli(&self) -> Cli {
        Cli::connect(&self.url, Network::Regtest).unwrap()
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
fn create_restore_and_address_without_node() {
    let dir = TempDir::new("create");
    let (mnemonic, path) =
        wallet_create(Network::Regtest, &dir.0, &[0_u8; 32], "", &lock(1)).unwrap();
    assert_eq!(mnemonic.phrase(), W1);
    assert_eq!(path, wallet_path(Network::Regtest, &dir.0));
    assert!(matches!(
        wallet_create(Network::Regtest, &dir.0, &[1_u8; 32], "", &lock(1)),
        Err(CliError::File(WalletFileError::Exists { .. }))
    ));
    // File versi 2: frasa tidak tersimpan terbuka, hanya bisa dibuka dengan password.
    let stored = load(&path).unwrap();
    assert!(stored.is_encrypted());
    assert_eq!(stored.mnemonic, None);
    assert!(!std::fs::read_to_string(&path).unwrap().contains("abandon"));
    assert_eq!(
        show_phrase(Network::Regtest, &dir.0, PASSWORD)
            .unwrap()
            .as_str(),
        W1
    );

    assert_eq!(next_address(Network::Regtest, &dir.0, "").unwrap(), W5);
    assert_eq!(next_address(Network::Regtest, &dir.0, "").unwrap(), W6);
    assert_eq!(
        load(&path).unwrap().next_index,
        NextIndex {
            receive: 2,
            change: 0
        }
    );
    let bad_passphrase = TempDir::new("bad-passphrase");
    assert!(matches!(
        wallet_create(
            Network::Regtest,
            &bad_passphrase.0,
            &[0_u8; 32],
            "\u{e9}",
            &lock(1)
        ),
        Err(CliError::Mnemonic(_))
    ));
    assert!(!wallet_path(Network::Regtest, &bad_passphrase.0).exists());

    // Passphrase dipakai saat wallet dibuat; alamat sesudahnya dari kunci akun publik di file.
    let trezor = TempDir::new("trezor");
    wallet_create(Network::Regtest, &trezor.0, &[0_u8; 32], "TREZOR", &lock(2)).unwrap();
    assert_eq!(next_address(Network::Regtest, &trezor.0, "").unwrap(), W9);

    let restored = TempDir::new("restore");
    let messy = W8_PHRASE.to_ascii_uppercase().replace(' ', "  ");
    let path = wallet_restore(Network::Regtest, &restored.0, &messy, "", &lock(3)).unwrap();
    assert!(load(&path).unwrap().is_encrypted());
    assert_eq!(
        show_phrase(Network::Regtest, &restored.0, PASSWORD)
            .unwrap()
            .as_str(),
        W8_PHRASE
    );
    assert!(matches!(
        wallet_restore(Network::Regtest, &restored.0, W8_PHRASE, "", &lock(3)),
        Err(CliError::File(WalletFileError::Exists { .. }))
    ));
    let empty = TempDir::new("bad-phrase");
    assert!(matches!(
        wallet_restore(Network::Regtest, &empty.0, "legal winner", "", &lock(4)),
        Err(CliError::Mnemonic(_))
    ));
    assert!(!wallet_path(Network::Regtest, &empty.0).exists());
    assert!(matches!(
        next_address(Network::Regtest, &empty.0, ""),
        Err(CliError::File(WalletFileError::Missing { .. }))
    ));
}

#[test]
fn balance_and_send_match_vector_t1() {
    let dir = TempDir::new("send");
    wallet_create(Network::Regtest, &dir.0, &[0_u8; 32], "", &lock(1)).unwrap();
    let harness = Harness::start(Network::Regtest, t1_utxos(), None);
    let cli = harness.cli();

    let balance = cli.balance(&dir.0, "").unwrap();
    assert_eq!(
        balance,
        Balance {
            tip_height: 150,
            total: Amount::from_quanta(FIFTEEN_ZVRG),
            spendable: Amount::from_quanta(EIGHT_ZVRG),
            immature: Amount::from_quanta(SEVEN_ZVRG),
            pending_spent: Amount::ZERO,
            pending_received: Amount::ZERO,
            after_pending: Amount::from_quanta(FIFTEEN_ZVRG),
            utxos: 3,
        }
    );
    assert_eq!(
        format_balance(&balance),
        concat!(
            "tinggi ujung: 150\nsaldo: 15 ZVRG\nbisa dipakai: 8 ZVRG\nbelum matang: 7 ZVRG\n",
            "keluar tertunda: 0 ZVRG\nmasuk tertunda: 0 ZVRG\nsaldo setelah tertunda masuk blok: 15 ZVRG\n",
            "utxo: 3\n"
        )
    );
    // Pemindaian menemukan 0/0, 0/1, dan 1/0, sehingga indeks berikutnya ikut naik.
    let path = wallet_path(Network::Regtest, &dir.0);
    assert_eq!(
        load(&path).unwrap().next_index,
        NextIndex {
            receive: 2,
            change: 1
        }
    );

    let sent = cli
        .send(
            &dir.0,
            "",
            PASSWORD,
            RECIPIENT,
            Amount::from_quanta(SIX_ZVRG),
            1,
            &[0_u8; 32],
        )
        .unwrap();
    assert_eq!(
        sent,
        Sent {
            txid: Hex32::new([0xee; 32]),
            amount: Amount::from_quanta(SIX_ZVRG),
            fee: Amount::from_quanta(284),
            change: Some(Amount::from_quanta(T1_CHANGE)),
            inputs: 2,
            size: 284,
        }
    );
    assert_eq!(
        harness.node.sent.lock().unwrap().as_slice(),
        &[decode_hex(T1_TX)]
    );
    assert_eq!(
        format_sent(&sent),
        format!(
            "txid: {}\ndikirim: 6 ZVRG\nbiaya: 0.000000000000000284 ZVRG\nkembalian: 1.999999999999999716 ZVRG\ninput: 2\nukuran: 284 byte\n",
            "ee".repeat(32)
        )
    );
    assert_eq!(
        load(&path).unwrap().next_index,
        NextIndex {
            receive: 2,
            change: 2
        }
    );

    let info = cli.info().unwrap();
    assert_eq!(
        format_info(&info),
        format!(
            "jaringan: regtest\ntinggi: 150\nujung: {}\nhasil tambang: 5 ZVRG\ntotal pasokan: 6 ZVRG\nmempool: 0 transaksi\n",
            "0b".repeat(32)
        )
    );
}

#[test]
fn send_errors_leave_wallet_unchanged() {
    let dir = TempDir::new("errors");
    wallet_create(Network::Regtest, &dir.0, &[0_u8; 32], "", &lock(1)).unwrap();
    let harness = Harness::start(Network::Regtest, t1_utxos(), None);
    let cli = harness.cli();
    let send = |to: &str, quanta: u128| {
        cli.send(
            &dir.0,
            "",
            PASSWORD,
            to,
            Amount::from_quanta(quanta),
            1,
            &[0_u8; 32],
        )
    };
    assert!(matches!(
        send(TESTNET_RECIPIENT, SIX_ZVRG),
        Err(CliError::Address(AddressError::WrongNetwork))
    ));
    assert!(matches!(
        send(RECIPIENT, HUNDRED_ZVRG),
        Err(CliError::Send(SendError::InsufficientFunds { .. }))
    ));
    assert!(harness.node.sent.lock().unwrap().is_empty());
    // Pemindaian tetap menyimpan indeks terpakai, tetapi indeks kembalian tidak naik tanpa transaksi.
    assert_eq!(
        load(&wallet_path(Network::Regtest, &dir.0))
            .unwrap()
            .next_index,
        NextIndex {
            receive: 2,
            change: 1
        }
    );

    let rejecting = Harness::start(Network::Regtest, t1_utxos(), Some("T6"));
    let error = rejecting
        .cli()
        .send(
            &dir.0,
            "",
            PASSWORD,
            RECIPIENT,
            Amount::from_quanta(SIX_ZVRG),
            1,
            &[0_u8; 32],
        )
        .unwrap_err();
    assert!(matches!(&error, CliError::Rejected { reason } if reason == "T6"));
    assert_eq!(error.to_string(), "node menolak transaksi: aturan T6");
    assert_eq!(
        load(&wallet_path(Network::Regtest, &dir.0))
            .unwrap()
            .next_index
            .change,
        1
    );

    let testnet = Harness::start(Network::Testnet, t1_utxos(), None);
    assert!(matches!(
        testnet.cli().balance(&dir.0, ""),
        Err(CliError::WrongNetwork { expected, actual }) if expected == "regtest" && actual == "testnet"
    ));
}

#[test]
fn run_dispatches_actions() {
    let dir = TempDir::new("run");
    let config = |action: Action| Config {
        network: Network::Regtest,
        rpc_url: "http://127.0.0.1:9".to_owned(),
        datadir: Some(dir.0.clone()),
        passphrase: String::new(),
        action,
    };
    let given = |text: &'static str| {
        move |_prompt: PasswordPrompt| -> Result<Zeroizing<String>, CliError> {
            Ok(Zeroizing::new(text.to_owned()))
        }
    };
    let never = || -> Result<[u8; 32], CliError> { unreachable!("tidak butuh angka acak") };
    let no_password = |_prompt: PasswordPrompt| -> Result<Zeroizing<String>, CliError> {
        unreachable!("tidak butuh password")
    };

    // wallet-create: password baru ditanya dua kali; entropi, salt, dan nonce dari tiga panggilan acak.
    let mut draws = 0_u32;
    let mut asked: Vec<PasswordPrompt> = Vec::new();
    let output = run(
        &config(Action::WalletCreate),
        || {
            draws = draws.saturating_add(1);
            Ok([0_u8; 32])
        },
        |prompt| {
            asked.push(prompt);
            Ok(Zeroizing::new(PASSWORD.to_owned()))
        },
    )
    .unwrap();
    assert_eq!(draws, 3);
    assert_eq!(asked, vec![PasswordPrompt::New, PasswordPrompt::Confirm]);
    assert!(output.starts_with("wallet dibuat: "), "{output}");
    assert!(
        output.contains("\nfile wallet dikunci password (Argon2id + XChaCha20-Poly1305)\n"),
        "{output}"
    );
    assert!(output.ends_with(&format!(":\n{W1}\n")), "{output}");
    let stored = load(&wallet_path(Network::Regtest, &dir.0)).unwrap();
    let kdf = stored
        .encrypted
        .as_ref()
        .map(|phrase| (phrase.memory_kib, phrase.iterations, phrase.parallelism));
    assert_eq!(kdf, Some((65_536, 3, 1)));

    // address tanpa password; wallet-show-phrase dengan password; wallet-encrypt ditolak untuk wallet versi 2.
    assert_eq!(
        run(&config(Action::Address), never, no_password).unwrap(),
        format!("alamat: {W5}\npayload: {W5_XONLY}\n")
    );
    assert_eq!(
        run(&config(Action::WalletShowPhrase), never, given(PASSWORD)).unwrap(),
        format!("frasa pemulihan (24 kata):\n{W1}\n")
    );
    assert!(matches!(
        run(
            &config(Action::WalletShowPhrase),
            never,
            given("password salah")
        ),
        Err(CliError::WrongPassword)
    ));
    assert!(matches!(
        run(&config(Action::WalletEncrypt), never, no_password),
        Err(CliError::AlreadyEncrypted)
    ));

    // Password baru tidak sama dengan ulangannya: wallet tidak dibuat.
    let other = TempDir::new("run-other");
    let other_config = Config {
        datadir: Some(other.0.clone()),
        ..config(Action::WalletCreate)
    };
    let mut answers = ["satu dua tiga", "satu dua empat"].into_iter();
    assert!(matches!(
        run(
            &other_config,
            || Ok([0_u8; 32]),
            |_prompt| Ok(Zeroizing::new(answers.next().unwrap().to_owned()))
        ),
        Err(CliError::PasswordMismatch)
    ));
    assert!(!wallet_path(Network::Regtest, &other.0).exists());

    let missing = Config {
        datadir: None,
        ..config(Action::Balance)
    };
    assert!(matches!(
        run(&missing, never, no_password),
        Err(CliError::MissingDatadir)
    ));

    // Port yang baru saja ditutup: koneksi ditolak, jadi galat RPC.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let closed = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let offline = Config {
        rpc_url: closed,
        ..config(Action::Info)
    };
    assert!(matches!(
        run(&offline, never, no_password),
        Err(CliError::Rpc { .. })
    ));
}

#[test]
fn encrypted_wallet_rules() {
    assert!(matches!(
        check_new_password("1234567", "1234567"),
        Err(CliError::PasswordTooShort)
    ));
    assert!(matches!(
        check_new_password("kata sandi", "kata sandI"),
        Err(CliError::PasswordMismatch)
    ));
    assert!(check_new_password("12345678", "12345678").is_ok());
    assert_eq!(PasswordPrompt::Unlock.text(), "password wallet: ");
    assert_eq!(
        PasswordPrompt::New.text(),
        "password baru (minimal 8 karakter): "
    );

    // Wallet lama versi 1 (frasa terbuka) dikunci dengan wallet-encrypt; indeks berikutnya tetap.
    let dir = TempDir::new("encrypt");
    let path = wallet_path(Network::Regtest, &dir.0);
    let mut old = WalletFile::new(Network::Regtest, &Mnemonic::parse(W1).unwrap());
    old.next_index = NextIndex {
        receive: 4,
        change: 2,
    };
    create_new(&path, &old).unwrap();
    assert_eq!(
        show_phrase(Network::Regtest, &dir.0, "").unwrap().as_str(),
        W1
    );
    assert_eq!(
        wallet_encrypt(Network::Regtest, &dir.0, "", &lock(5)).unwrap(),
        path
    );
    let stored = load(&path).unwrap();
    assert!(stored.is_encrypted());
    assert_eq!(
        stored.next_index,
        NextIndex {
            receive: 4,
            change: 2
        }
    );
    assert!(!std::fs::read_to_string(&path).unwrap().contains("abandon"));
    assert!(matches!(
        wallet_encrypt(Network::Regtest, &dir.0, "", &lock(5)),
        Err(CliError::AlreadyEncrypted)
    ));
    assert!(matches!(
        show_phrase(Network::Regtest, &dir.0, "kata sandi lain"),
        Err(CliError::WrongPassword)
    ));
    assert_eq!(
        show_phrase(Network::Regtest, &dir.0, PASSWORD)
            .unwrap()
            .as_str(),
        W1
    );

    // Mengirim: password salah dan passphrase lain ditolak sebelum transaksi disusun.
    let harness = Harness::start(Network::Regtest, t1_utxos(), None);
    let cli = harness.cli();
    let send = |passphrase: &str, password: &str| {
        cli.send(
            &dir.0,
            passphrase,
            password,
            RECIPIENT,
            Amount::from_quanta(SIX_ZVRG),
            1,
            &[0_u8; 32],
        )
    };
    assert!(matches!(
        send("", "kata sandi lain"),
        Err(CliError::WrongPassword)
    ));
    assert!(matches!(
        send("TREZOR", PASSWORD),
        Err(CliError::PassphraseMismatch)
    ));
    assert!(harness.node.sent.lock().unwrap().is_empty());
    assert!(send("", PASSWORD).is_ok());
    assert_eq!(harness.node.sent.lock().unwrap().len(), 1);
}

fn pending_output(
    fill: u8,
    index: u32,
    quanta: u128,
    kind: OutputKind,
    payload: &str,
) -> MempoolOutput {
    MempoolOutput {
        txid: Hex32::new([fill; 32]),
        index,
        amount: Quanta::new(Amount::from_quanta(quanta)),
        kind: KindNumber::new(kind),
        payload: hex32(payload),
    }
}

#[test]
fn balance_and_send_follow_pending_mempool() {
    let dir = TempDir::new("pending");
    wallet_create(Network::Regtest, &dir.0, &[0_u8; 32], "", &lock(1)).unwrap();
    // Transaksi dd membelanjakan 0/1 (3 ZVRG) dengan kembalian 2 ZVRG ke 1/0; transaksi cc mengirim 4 ZVRG ke 0/0.
    // Output jenis 1 dengan payload wallet dan output ke payload lain tidak dihitung.
    let pending = MempoolScan {
        spent: vec![MempoolSpend {
            txid: Hex32::new([0xb2; 32]),
            index: 1,
            spent_by: Hex32::new([0xdd; 32]),
        }],
        outputs: vec![
            pending_output(0xcc, 0, FOUR_ZVRG, OutputKind::SingleKey, W5_XONLY),
            pending_output(0xcc, 1, HUNDRED_ZVRG, OutputKind::ConditionHash, W5_XONLY),
            pending_output(0xdd, 0, ZVRG, OutputKind::SingleKey, &"ee".repeat(32)),
            pending_output(0xdd, 1, TWO_ZVRG, OutputKind::SingleKey, W7_XONLY),
        ],
    };
    let harness = Harness::launch(Network::Regtest, t1_utxos(), pending, None);
    let cli = harness.cli();

    let balance = cli.balance(&dir.0, "").unwrap();
    assert_eq!(
        balance,
        Balance {
            tip_height: 150,
            total: Amount::from_quanta(FIFTEEN_ZVRG),
            spendable: Amount::from_quanta(FIVE_ZVRG),
            immature: Amount::from_quanta(SEVEN_ZVRG),
            pending_spent: Amount::from_quanta(THREE_ZVRG),
            pending_received: Amount::from_quanta(SIX_ZVRG),
            after_pending: Amount::from_quanta(EIGHTEEN_ZVRG),
            utxos: 3,
        }
    );
    assert_eq!(
        format_balance(&balance),
        concat!(
            "tinggi ujung: 150\nsaldo: 15 ZVRG\nbisa dipakai: 5 ZVRG\nbelum matang: 7 ZVRG\n",
            "keluar tertunda: 3 ZVRG\nmasuk tertunda: 6 ZVRG\nsaldo setelah tertunda masuk blok: 18 ZVRG\n",
            "utxo: 3\n"
        )
    );

    // 0/1 sedang dibelanjakan, jadi hanya 0/0 (5 ZVRG) yang boleh dipilih: 6 ZVRG tidak cukup, 4 ZVRG memakai 0/0.
    let send = |quanta: u128| {
        cli.send(
            &dir.0,
            "",
            PASSWORD,
            RECIPIENT,
            Amount::from_quanta(quanta),
            1,
            &[0_u8; 32],
        )
    };
    assert!(matches!(
        send(SIX_ZVRG),
        Err(CliError::Send(SendError::InsufficientFunds { .. }))
    ));
    assert!(harness.node.sent.lock().unwrap().is_empty());
    let sent = send(FOUR_ZVRG).unwrap();
    assert_eq!(sent.inputs, 1);
    assert_eq!(sent.amount, Amount::from_quanta(FOUR_ZVRG));
    assert!(sent.change.is_some());
    let txs = harness.node.sent.lock().unwrap();
    assert_eq!(txs.len(), 1);
    let bytes = txs.first().unwrap();
    assert!(bytes.windows(32).any(|window| window == [0xa1_u8; 32]));
    assert!(!bytes.windows(32).any(|window| window == [0xb2_u8; 32]));
}

#[test]
fn testnet_wallet_uses_own_folder_addresses_and_node() {
    let dir = TempDir::new("testnet");
    let (mnemonic, path) =
        wallet_create(Network::Testnet, &dir.0, &[0_u8; 32], "", &lock(1)).unwrap();
    assert_eq!(mnemonic.phrase(), W1);
    assert_eq!(path, dir.0.join("testnet").join("wallet.json"));
    assert_eq!(path, wallet_path(Network::Testnet, &dir.0));
    assert_eq!(load(&path).unwrap().network, "testnet");
    assert!(!wallet_path(Network::Regtest, &dir.0).exists());
    assert!(matches!(
        next_address(Network::Regtest, &dir.0, ""),
        Err(CliError::File(WalletFileError::Missing { .. }))
    ));

    // Kunci akun sama dengan regtest (coin 1); alamat memakai HRP tzvrg.
    assert_eq!(next_address(Network::Testnet, &dir.0, "").unwrap(), W10);
    let (_kind, payload) = decode_address(Network::Testnet, W10).unwrap();
    assert_eq!(&payload, hex32(W5_XONLY).bytes());
    assert_eq!(
        show_phrase(Network::Testnet, &dir.0, PASSWORD)
            .unwrap()
            .as_str(),
        W1
    );

    // Saldo dan kirim lewat node testnet: kunci dan network_id node sama dengan T1, sehingga byte transaksi sama.
    let harness = Harness::start(Network::Testnet, t1_utxos(), None);
    let cli = Cli::connect(&harness.url, Network::Testnet).unwrap();
    let balance = cli.balance(&dir.0, "").unwrap();
    assert_eq!(balance.total, Amount::from_quanta(FIFTEEN_ZVRG));
    assert_eq!(balance.spendable, Amount::from_quanta(EIGHT_ZVRG));
    let send = |to: &str| {
        cli.send(
            &dir.0,
            "",
            PASSWORD,
            to,
            Amount::from_quanta(SIX_ZVRG),
            1,
            &[0_u8; 32],
        )
    };
    assert!(matches!(
        send(RECIPIENT),
        Err(CliError::Address(AddressError::WrongNetwork))
    ));
    let sent = send(TESTNET_RECIPIENT).unwrap();
    assert_eq!(sent.fee, Amount::from_quanta(284));
    assert_eq!(
        harness.node.sent.lock().unwrap().as_slice(),
        &[decode_hex(T1_TX)]
    );

    // CLI testnet menolak node regtest.
    let regtest = Harness::start(Network::Regtest, t1_utxos(), None);
    let error = Cli::connect(&regtest.url, Network::Testnet)
        .unwrap()
        .balance(&dir.0, "")
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "node berjalan di jaringan regtest, bukan testnet (pilih --regtest atau --testnet)"
    );

    // File wallet regtest yang disalin ke folder testnet ditolak.
    let copied = TempDir::new("testnet-copy");
    let regtest_path = wallet_create(Network::Regtest, &copied.0, &[0_u8; 32], "", &lock(2))
        .unwrap()
        .1;
    let testnet_path = wallet_path(Network::Testnet, &copied.0);
    std::fs::create_dir_all(testnet_path.parent().unwrap()).unwrap();
    std::fs::copy(&regtest_path, &testnet_path).unwrap();
    assert!(matches!(
        next_address(Network::Testnet, &copied.0, ""),
        Err(CliError::File(WalletFileError::WrongNetwork {
            expected: "testnet",
            ..
        }))
    ));

    // run memakai jaringan dari konfigurasi.
    let config = Config {
        network: Network::Testnet,
        rpc_url: "http://127.0.0.1:9".to_owned(),
        datadir: Some(dir.0.clone()),
        passphrase: String::new(),
        action: Action::Address,
    };
    let never = || -> Result<[u8; 32], CliError> { unreachable!("tidak butuh angka acak") };
    let no_password = |_prompt: PasswordPrompt| -> Result<Zeroizing<String>, CliError> {
        unreachable!("tidak butuh password")
    };
    let output = run(&config, never, no_password).unwrap();
    assert!(output.starts_with("alamat: tzvrg1"), "{output}");
}

#[test]
fn format_history_lists_pending_then_blocks() {
    let history = History {
        tip_height: 103,
        entries: vec![
            HistoryEntry {
                txid: Hex32::new([0x11; 32]),
                height: None,
                timestamp: None,
                coinbase: false,
                received: Amount::from_quanta(ZVRG),
                spent: Amount::ZERO,
                fee: None,
            },
            HistoryEntry {
                txid: Hex32::new([0x22; 32]),
                height: Some(102),
                timestamp: Some(1_800_000_000),
                coinbase: false,
                received: Amount::from_quanta(2 * ZVRG),
                spent: Amount::from_quanta(3 * ZVRG),
                fee: Some(Amount::from_quanta(284)),
            },
            HistoryEntry {
                txid: Hex32::new([0x33; 32]),
                height: Some(1),
                timestamp: Some(1_799_999_000),
                coinbase: true,
                received: Amount::from_quanta(5 * ZVRG),
                spent: Amount::ZERO,
                fee: None,
            },
        ],
    };
    let text = format_history(&history);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 5, "{text}");
    assert_eq!(lines[0], "tinggi ujung: 103");
    assert_eq!(lines[1], "transaksi: 3");
    assert!(
        lines[2].starts_with(&format!("tertunda {} masuk ", "11".repeat(32))),
        "{text}"
    );
    assert!(
        lines[3].starts_with(&format!("blok 102 {} masuk ", "22".repeat(32))),
        "{text}"
    );
    assert!(lines[3].contains(" biaya "), "{text}");
    assert!(lines[4].ends_with(" coinbase"), "{text}");
    assert!(text.ends_with('\n'));
}

#[test]
fn os_random_differs_between_calls() {
    let first = os_random().unwrap();
    let second = os_random().unwrap();
    assert_ne!(first, second);
    assert_ne!(first, [0_u8; 32]);
}

#[test]
fn send_from_spends_only_that_address_and_returns_change_to_it() {
    use zevrigo_primitives::{Decode, Hash32, Transaction, TxBody};
    use zevrigo_wallet_core::encode_address;

    let dir = TempDir::new("send-from");
    wallet_create(Network::Regtest, &dir.0, &[0_u8; 32], "", &lock(1)).unwrap();
    let harness = Harness::start(Network::Regtest, t1_utxos(), None);
    let cli = harness.cli();
    let send = |from: &str, quanta: u128| {
        cli.send_from(
            &dir.0,
            "",
            PASSWORD,
            Some(from),
            RECIPIENT,
            Amount::from_quanta(quanta),
            1,
            &[0_u8; 32],
        )
    };

    // `send` biasa memilih 5 ZVRG di W5 dulu; dengan --dari W6 hanya 3 ZVRG di W6 yang dipakai.
    let sent = send(W6, TWO_ZVRG).unwrap();
    assert_eq!(sent.inputs, 1);
    assert_eq!(sent.amount, Amount::from_quanta(TWO_ZVRG));
    let change = sent.change.unwrap();
    assert_eq!(change.as_quanta() + sent.fee.as_quanta(), ZVRG);
    let bytes = harness.node.sent.lock().unwrap()[0].clone();
    let tx = Transaction::from_bytes(&bytes).unwrap();
    let TxBody::Regular(body) = tx.body() else {
        unreachable!("transaksi kirim bukan coinbase");
    };
    assert_eq!(body.inputs().len(), 1);
    assert_eq!(body.inputs()[0].prev_txid(), Hash32::from_bytes([0xb2; 32]));
    assert_eq!(body.inputs()[0].prev_index(), 1);
    assert_eq!(body.outputs().len(), 2);
    assert_eq!(body.outputs()[0].amount(), Amount::from_quanta(TWO_ZVRG));
    assert_eq!(body.outputs()[1].payload(), hex32(W6_XONLY).bytes());
    assert_eq!(body.outputs()[1].amount(), change);
    // Kembalian ke W6 sendiri: indeks kembalian di file wallet tidak naik.
    let path = wallet_path(Network::Regtest, &dir.0);
    assert_eq!(
        load(&path).unwrap().next_index,
        NextIndex {
            receive: 2,
            change: 1
        }
    );

    // Saldo yang dilaporkan hanya saldo alamat --dari; alamat lain wallet tidak dipakai.
    assert!(matches!(
        send(W6, FOUR_ZVRG),
        Err(CliError::Send(SendError::InsufficientFunds { available, .. }))
            if available == Amount::from_quanta(THREE_ZVRG)
    ));
    let w7 = encode_address(
        Network::Regtest,
        OutputKind::SingleKey,
        hex32(W7_XONLY).bytes(),
    );
    assert!(matches!(
        send(&w7, ZVRG),
        Err(CliError::Send(SendError::InsufficientFunds { available, .. }))
            if available == Amount::ZERO
    ));
    // Alamat di luar wallet, jaringan lain, atau bukan alamat.
    let error = send(RECIPIENT, ZVRG).unwrap_err();
    assert!(matches!(&error, CliError::SourceNotOwned { address } if address == RECIPIENT));
    assert_eq!(
        error.to_string(),
        format!("alamat --dari {RECIPIENT} bukan alamat wallet ini")
    );
    assert!(matches!(
        send(TESTNET_RECIPIENT, ZVRG),
        Err(CliError::SourceAddress(AddressError::WrongNetwork))
    ));
    assert!(matches!(
        send("bukan-alamat", ZVRG),
        Err(CliError::SourceAddress(_))
    ));
    assert_eq!(harness.node.sent.lock().unwrap().len(), 1);
    assert_eq!(load(&path).unwrap().next_index.change, 1);
}
