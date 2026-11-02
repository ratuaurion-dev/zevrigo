// Uji perintah wallet terhadap node tiruan di atas `RpcServer` (MOD-08). Skenario kirim memakai koin, kunci, dan
// network_id yang sama dengan vektor T1 TSK-M16, sehingga byte transaksi yang diterima node wajib sama persis
// dengan T1. Alamat W5, W6, W9 dan frasa W1, W8: Spesifikasi Wallet v0.1 Bagian 4.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use homeostasis_primitives::{Amount, Network, OutputKind};
use homeostasis_rpc::{
    BlockInfo, BlockTemplate, ChainInfo, CompactHex, GetBlockTemplateParams, Hex32, KindNumber,
    MempoolInfo, MempoolOutput, MempoolScan, MempoolSpend, NetworkName, Quanta, RpcBackend,
    RpcError, RpcServer, SendRawTransactionResult, SubmitBlockResult, UtxoInfo,
};
use homeostasis_wallet_core::{AddressError, KdfParams, Mnemonic, SendError};
use tokio::runtime::Runtime;
use zeroize::Zeroizing;

use super::{
    Balance, Cli, CliError, PasswordPrompt, Protection, Sent, check_new_password, format_balance,
    format_info, format_sent, next_address, run, show_phrase, wallet_create, wallet_encrypt,
    wallet_restore,
};
use crate::args::{Action, Config};
use crate::wallet_file::{NextIndex, WalletFile, WalletFileError, create_new, load, wallet_path};

const HSIS: u128 = 1_000_000_000_000_000_000;
const W1: &str = concat!(
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ",
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
);
const W5: &str = "rhsis1qtfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939msequa3ft3";
const W6: &str = "rhsis1q3uesn8pvcdeeufxr98v794e07l8ghd85fyd3wclknj5xj04rfqrqvpxaeu";
const W8_PHRASE: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";
const W9: &str = "rhsis1q2x064qcn4zk84y9zjtpjt8e8ksjjja7peenap4lcse2d0vmw2xyqnjjhdv";
const W5_XONLY: &str = "5a6f28253f35e2ff1ab0b086b8c87f731159661157e5be2ca98188369625dc32";
const W6_XONLY: &str = "8f33099c2cc3739e24c329d9e2d72ff7ce8bb4f4491b1763f69ca8693ea34806";
const W7_XONLY: &str = "c71cdd713224479fff24b56db524c2385acf09725605e7440de0c34272e2abf5";
/// Alamat regtest kunci contoh BIP340 #0 (Spesifikasi Format Bagian 8).
const RECIPIENT: &str = "rhsis1qlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmuspj8z0e";
const TESTNET_RECIPIENT: &str = "thsis1qlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmusq7jsl6";
const T1_TX: &str = concat!(
    "010002a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a100b2b2b2b2b2b2b2b2b2b2b2b2",
    "b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b201028080e0e2de8692a25300f9308a019258c31049344f85f89d5229",
    "b531c845836f99b08601f113bce036f9e4fd9ff6f4acdbe01b00730ad008ce8bd2f391b08b9c4838af732341e9704cba",
    "0dc03d83f977dfcf7d9600f98278c858ef3dfca1d0bfea415e604a45681c23ff65f0040a110202af619e8ffccbe7b7b4",
    "34efd3f7d66cff1c5ba294d04803b2bd298b1ba5aae19dc09ba56f00d208bcda0d1e9ae93e1e68f9f298d75c4d86a649",
    "928be7d63a48765340a35ecd11acf3b31110d7b8eee5b491e1220be4d8a084e53632fd23f4f7fbd9d57157dd",
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

/// Kembalian T1: 8 HSIS − 6 HSIS − biaya 284.
const T1_CHANGE: u128 = 2 * HSIS - 284;
const FIFTEEN_HSIS: u128 = 15 * HSIS;
const EIGHT_HSIS: u128 = 8 * HSIS;
const SEVEN_HSIS: u128 = 7 * HSIS;
const SIX_HSIS: u128 = 6 * HSIS;
const FIVE_HSIS: u128 = 5 * HSIS;
const EIGHTEEN_HSIS: u128 = 18 * HSIS;
const FOUR_HSIS: u128 = 4 * HSIS;
const THREE_HSIS: u128 = 3 * HSIS;
const TWO_HSIS: u128 = 2 * HSIS;
const HUNDRED_HSIS: u128 = 100 * HSIS;

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
        let path =
            std::env::temp_dir().join(format!("homeostasis-q74-{name}-{}", std::process::id()));
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

/// Koin T1: 5 HSIS di 0/0 (coinbase matang), 3 HSIS di 0/1, 7 HSIS di 1/0 (coinbase belum matang).
fn t1_utxos() -> Vec<UtxoInfo> {
    vec![
        coin(0xa1, 0, FIVE_HSIS, W5_XONLY, true, 101),
        coin(0xb2, 1, THREE_HSIS, W6_XONLY, false, 50),
        coin(0xc3, 0, SEVEN_HSIS, W7_XONLY, true, 200),
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
            mined_total: Quanta::new(Amount::from_quanta(FIVE_HSIS)),
            total_supply: Quanta::new(Amount::from_quanta(SIX_HSIS)),
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
        Cli::connect(&self.url).unwrap()
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
    let (mnemonic, path) = wallet_create(&dir.0, &[0_u8; 32], "", &lock(1)).unwrap();
    assert_eq!(mnemonic.phrase(), W1);
    assert_eq!(path, wallet_path(&dir.0));
    assert!(matches!(
        wallet_create(&dir.0, &[1_u8; 32], "", &lock(1)),
        Err(CliError::File(WalletFileError::Exists { .. }))
    ));
    // File versi 2: frasa tidak tersimpan terbuka, hanya bisa dibuka dengan password.
    let stored = load(&path).unwrap();
    assert!(stored.is_encrypted());
    assert_eq!(stored.mnemonic, None);
    assert!(!std::fs::read_to_string(&path).unwrap().contains("abandon"));
    assert_eq!(show_phrase(&dir.0, PASSWORD).unwrap().as_str(), W1);

    assert_eq!(next_address(&dir.0, "").unwrap(), W5);
    assert_eq!(next_address(&dir.0, "").unwrap(), W6);
    assert_eq!(
        load(&path).unwrap().next_index,
        NextIndex {
            receive: 2,
            change: 0
        }
    );
    let bad_passphrase = TempDir::new("bad-passphrase");
    assert!(matches!(
        wallet_create(&bad_passphrase.0, &[0_u8; 32], "\u{e9}", &lock(1)),
        Err(CliError::Mnemonic(_))
    ));
    assert!(!wallet_path(&bad_passphrase.0).exists());

    // Passphrase dipakai saat wallet dibuat; alamat sesudahnya dari kunci akun publik di file.
    let trezor = TempDir::new("trezor");
    wallet_create(&trezor.0, &[0_u8; 32], "TREZOR", &lock(2)).unwrap();
    assert_eq!(next_address(&trezor.0, "").unwrap(), W9);

    let restored = TempDir::new("restore");
    let messy = W8_PHRASE.to_ascii_uppercase().replace(' ', "  ");
    let path = wallet_restore(&restored.0, &messy, "", &lock(3)).unwrap();
    assert!(load(&path).unwrap().is_encrypted());
    assert_eq!(
        show_phrase(&restored.0, PASSWORD).unwrap().as_str(),
        W8_PHRASE
    );
    assert!(matches!(
        wallet_restore(&restored.0, W8_PHRASE, "", &lock(3)),
        Err(CliError::File(WalletFileError::Exists { .. }))
    ));
    let empty = TempDir::new("bad-phrase");
    assert!(matches!(
        wallet_restore(&empty.0, "legal winner", "", &lock(4)),
        Err(CliError::Mnemonic(_))
    ));
    assert!(!wallet_path(&empty.0).exists());
    assert!(matches!(
        next_address(&empty.0, ""),
        Err(CliError::File(WalletFileError::Missing { .. }))
    ));
}

#[test]
fn balance_and_send_match_vector_t1() {
    let dir = TempDir::new("send");
    wallet_create(&dir.0, &[0_u8; 32], "", &lock(1)).unwrap();
    let harness = Harness::start(Network::Regtest, t1_utxos(), None);
    let cli = harness.cli();

    let balance = cli.balance(&dir.0, "").unwrap();
    assert_eq!(
        balance,
        Balance {
            tip_height: 150,
            total: Amount::from_quanta(FIFTEEN_HSIS),
            spendable: Amount::from_quanta(EIGHT_HSIS),
            immature: Amount::from_quanta(SEVEN_HSIS),
            pending_spent: Amount::ZERO,
            pending_received: Amount::ZERO,
            after_pending: Amount::from_quanta(FIFTEEN_HSIS),
            utxos: 3,
        }
    );
    assert_eq!(
        format_balance(&balance),
        concat!(
            "tinggi ujung: 150\nsaldo: 15 HSIS\nbisa dipakai: 8 HSIS\nbelum matang: 7 HSIS\n",
            "keluar tertunda: 0 HSIS\nmasuk tertunda: 0 HSIS\nsaldo setelah tertunda masuk blok: 15 HSIS\n",
            "utxo: 3\n"
        )
    );
    // Pemindaian menemukan 0/0, 0/1, dan 1/0, sehingga indeks berikutnya ikut naik.
    let path = wallet_path(&dir.0);
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
            Amount::from_quanta(SIX_HSIS),
            1,
            &[0_u8; 32],
        )
        .unwrap();
    assert_eq!(
        sent,
        Sent {
            txid: Hex32::new([0xee; 32]),
            amount: Amount::from_quanta(SIX_HSIS),
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
            "txid: {}\ndikirim: 6 HSIS\nbiaya: 0.000000000000000284 HSIS\nkembalian: 1.999999999999999716 HSIS\ninput: 2\nukuran: 284 byte\n",
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
            "jaringan: regtest\ntinggi: 150\nujung: {}\nhasil tambang: 5 HSIS\ntotal pasokan: 6 HSIS\nmempool: 0 transaksi\n",
            "0b".repeat(32)
        )
    );
}

#[test]
fn send_errors_leave_wallet_unchanged() {
    let dir = TempDir::new("errors");
    wallet_create(&dir.0, &[0_u8; 32], "", &lock(1)).unwrap();
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
        send(TESTNET_RECIPIENT, SIX_HSIS),
        Err(CliError::Address(AddressError::WrongNetwork))
    ));
    assert!(matches!(
        send(RECIPIENT, HUNDRED_HSIS),
        Err(CliError::Send(SendError::InsufficientFunds { .. }))
    ));
    assert!(harness.node.sent.lock().unwrap().is_empty());
    // Pemindaian tetap menyimpan indeks terpakai, tetapi indeks kembalian tidak naik tanpa transaksi.
    assert_eq!(
        load(&wallet_path(&dir.0)).unwrap().next_index,
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
            Amount::from_quanta(SIX_HSIS),
            1,
            &[0_u8; 32],
        )
        .unwrap_err();
    assert!(matches!(&error, CliError::Rejected { reason } if reason == "T6"));
    assert_eq!(error.to_string(), "node menolak transaksi: aturan T6");
    assert_eq!(load(&wallet_path(&dir.0)).unwrap().next_index.change, 1);

    let testnet = Harness::start(Network::Testnet, t1_utxos(), None);
    assert!(matches!(
        testnet.cli().balance(&dir.0, ""),
        Err(CliError::WrongNetwork { network }) if network == "testnet"
    ));
}

#[test]
fn run_dispatches_actions() {
    let dir = TempDir::new("run");
    let config = |action: Action| Config {
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
    let stored = load(&wallet_path(&dir.0)).unwrap();
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
    assert!(!wallet_path(&other.0).exists());

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
    let path = wallet_path(&dir.0);
    let mut old = WalletFile::new(&Mnemonic::parse(W1).unwrap());
    old.next_index = NextIndex {
        receive: 4,
        change: 2,
    };
    create_new(&path, &old).unwrap();
    assert_eq!(show_phrase(&dir.0, "").unwrap().as_str(), W1);
    assert_eq!(wallet_encrypt(&dir.0, "", &lock(5)).unwrap(), path);
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
        wallet_encrypt(&dir.0, "", &lock(5)),
        Err(CliError::AlreadyEncrypted)
    ));
    assert!(matches!(
        show_phrase(&dir.0, "kata sandi lain"),
        Err(CliError::WrongPassword)
    ));
    assert_eq!(show_phrase(&dir.0, PASSWORD).unwrap().as_str(), W1);

    // Mengirim: password salah dan passphrase lain ditolak sebelum transaksi disusun.
    let harness = Harness::start(Network::Regtest, t1_utxos(), None);
    let cli = harness.cli();
    let send = |passphrase: &str, password: &str| {
        cli.send(
            &dir.0,
            passphrase,
            password,
            RECIPIENT,
            Amount::from_quanta(SIX_HSIS),
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
    wallet_create(&dir.0, &[0_u8; 32], "", &lock(1)).unwrap();
    // Transaksi dd membelanjakan 0/1 (3 HSIS) dengan kembalian 2 HSIS ke 1/0; transaksi cc mengirim 4 HSIS ke 0/0.
    // Output jenis 1 dengan payload wallet dan output ke payload lain tidak dihitung.
    let pending = MempoolScan {
        spent: vec![MempoolSpend {
            txid: Hex32::new([0xb2; 32]),
            index: 1,
            spent_by: Hex32::new([0xdd; 32]),
        }],
        outputs: vec![
            pending_output(0xcc, 0, FOUR_HSIS, OutputKind::SingleKey, W5_XONLY),
            pending_output(0xcc, 1, HUNDRED_HSIS, OutputKind::ConditionHash, W5_XONLY),
            pending_output(0xdd, 0, HSIS, OutputKind::SingleKey, &"ee".repeat(32)),
            pending_output(0xdd, 1, TWO_HSIS, OutputKind::SingleKey, W7_XONLY),
        ],
    };
    let harness = Harness::launch(Network::Regtest, t1_utxos(), pending, None);
    let cli = harness.cli();

    let balance = cli.balance(&dir.0, "").unwrap();
    assert_eq!(
        balance,
        Balance {
            tip_height: 150,
            total: Amount::from_quanta(FIFTEEN_HSIS),
            spendable: Amount::from_quanta(FIVE_HSIS),
            immature: Amount::from_quanta(SEVEN_HSIS),
            pending_spent: Amount::from_quanta(THREE_HSIS),
            pending_received: Amount::from_quanta(SIX_HSIS),
            after_pending: Amount::from_quanta(EIGHTEEN_HSIS),
            utxos: 3,
        }
    );
    assert_eq!(
        format_balance(&balance),
        concat!(
            "tinggi ujung: 150\nsaldo: 15 HSIS\nbisa dipakai: 5 HSIS\nbelum matang: 7 HSIS\n",
            "keluar tertunda: 3 HSIS\nmasuk tertunda: 6 HSIS\nsaldo setelah tertunda masuk blok: 18 HSIS\n",
            "utxo: 3\n"
        )
    );

    // 0/1 sedang dibelanjakan, jadi hanya 0/0 (5 HSIS) yang boleh dipilih: 6 HSIS tidak cukup, 4 HSIS memakai 0/0.
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
        send(SIX_HSIS),
        Err(CliError::Send(SendError::InsufficientFunds { .. }))
    ));
    assert!(harness.node.sent.lock().unwrap().is_empty());
    let sent = send(FOUR_HSIS).unwrap();
    assert_eq!(sent.inputs, 1);
    assert_eq!(sent.amount, Amount::from_quanta(FOUR_HSIS));
    assert!(sent.change.is_some());
    let txs = harness.node.sent.lock().unwrap();
    assert_eq!(txs.len(), 1);
    let bytes = txs.first().unwrap();
    assert!(bytes.windows(32).any(|window| window == [0xa1_u8; 32]));
    assert!(!bytes.windows(32).any(|window| window == [0xb2_u8; 32]));
}
