use std::path::PathBuf;

use homeostasis_primitives::Network;
use homeostasis_wallet_core::{ExtendedPubKey, KdfParams, Mnemonic, seal, unseal};

use super::{
    NextIndex, WalletFile, WalletFileError, create_new, load, load_network, network_name, save,
    wallet_path,
};

/// Frasa W1 Spesifikasi Wallet v0.1 Bagian 4 (entropi 32 byte nol).
const W1: &str = concat!(
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ",
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
);

const W1_JSON: &str = concat!(
    "{\n",
    "  \"version\": 1,\n",
    "  \"network\": \"regtest\",\n",
    "  \"mnemonic\": \"abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ",
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art\",\n",
    "  \"next_index\": {\n",
    "    \"receive\": 0,\n",
    "    \"change\": 0\n",
    "  }\n",
    "}\n"
);

/// Kunci akun publik W4 (frasa W1, tanpa passphrase).
const W4: &str = concat!(
    "03b9385a6fbee0caa6e79407f83b0053e2d864b6c7a8f32ce61c3f495b5dd5d027",
    "12ed17c3e72223beb38ea98ec1317581f02458f02b0ecf4abe02c6490eca170a"
);

/// File versi 2 untuk frasa W1 terkunci dengan vektor V1 MOD-09 (password `kata sandi uji`, salt 11 × 16, nonce
/// 22 × 24, Argon2id 64 KiB, 2 putaran, 1 jalur).
const W2_JSON: &str = concat!(
    "{\n",
    "  \"version\": 2,\n",
    "  \"network\": \"regtest\",\n",
    "  \"account\": \"",
    "03b9385a6fbee0caa6e79407f83b0053e2d864b6c7a8f32ce61c3f495b5dd5d027",
    "12ed17c3e72223beb38ea98ec1317581f02458f02b0ecf4abe02c6490eca170a\",\n",
    "  \"encrypted\": {\n",
    "    \"kdf\": \"argon2id\",\n",
    "    \"memory_kib\": 64,\n",
    "    \"iterations\": 2,\n",
    "    \"parallelism\": 1,\n",
    "    \"salt\": \"11111111111111111111111111111111\",\n",
    "    \"cipher\": \"xchacha20poly1305\",\n",
    "    \"nonce\": \"222222222222222222222222222222222222222222222222\",\n",
    "    \"ciphertext\": \"",
    "efa16af8f8ddf1cdbc1b3a18c7804a97debde5cdc1513996958a965a02489a2c1d2be9eb10cbfd5c87bbd5bf3d35c4f3",
    "38f86961d38dfe3216cad2e5fd78335ee80aa37d46e519966f00a162c6d23113d59d97ecf1e237d1ad5014a3ba3440c7",
    "eb7a84c7df4efe96f6317a3296a6ef03ab5b4ecbc231547bfb8441f4e0b86c549288d57333153863d7e23bf40eb008d5",
    "6f13cdf175276d6b693f1e2bc8dde36b04b3f982afc0b9f83f20ff9c97983e5b434bb99a4aaab7052022e714ca23bf74",
    "cad2e6e4408800050ad267",
    "\"\n",
    "  },\n",
    "  \"next_index\": {\n",
    "    \"receive\": 0,\n",
    "    \"change\": 0\n",
    "  }\n",
    "}\n",
);

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

fn w1() -> WalletFile {
    WalletFile::new(Network::Regtest, &Mnemonic::parse(W1).unwrap())
}

fn w2() -> WalletFile {
    let account = ExtendedPubKey::from_account_hex(W4).unwrap();
    let kdf = KdfParams {
        memory_kib: 64,
        iterations: 2,
        parallelism: 1,
    };
    let sealed = seal(
        W1.as_bytes(),
        b"kata sandi uji",
        kdf,
        [0x11; 16],
        [0x22; 24],
    )
    .unwrap();
    WalletFile::new_encrypted(Network::Regtest, &account, &sealed, NextIndex::default())
}

#[test]
fn json_format_is_exact_and_round_trips() {
    let wallet = w1();
    assert_eq!(wallet.to_json().unwrap(), W1_JSON);
    assert_eq!(WalletFile::from_json(W1_JSON).unwrap(), wallet);
    assert_eq!(wallet.mnemonic().unwrap().phrase(), W1);
    assert_eq!(
        wallet_path(Network::Regtest, &PathBuf::from("/data")),
        PathBuf::from("/data/regtest/wallet.json")
    );
    assert_eq!(
        wallet_path(Network::Testnet, &PathBuf::from("/data")),
        PathBuf::from("/data/testnet/wallet.json")
    );
    assert!(matches!(wallet.network(), Ok(Network::Regtest)));
    let text = format!("{wallet:?}");
    assert!(!text.contains("abandon"), "{text}");
}

#[test]
fn from_json_rejects_bad_content() {
    let bad = [
        W1_JSON.replace("\"version\": 1", "\"version\": 2"),
        W1_JSON.replace("regtest", "mainnet"),
        W1_JSON.replace("regtest", "Testnet"),
        W1_JSON.replace(" art\"", " abandon\""),
        W1_JSON.replace("abandon abandon abandon", "Abandon  abandon abandon"),
        W1_JSON.replace("  }\n", "  },\n  \"extra\": 1\n"),
        W1_JSON.replace("\"change\": 0", "\"change\": -1"),
        "{".to_owned(),
    ];
    for text in bad {
        assert!(
            matches!(
                WalletFile::from_json(&text),
                Err(WalletFileError::Format(_))
            ),
            "{text}"
        );
    }
}

#[test]
fn create_load_save_and_never_overwrite() {
    let dir = TempDir::new("file");
    let path = wallet_path(Network::Regtest, &dir.0);
    assert!(matches!(load(&path), Err(WalletFileError::Missing { .. })));
    assert!(matches!(
        save(&path, &w1()),
        Err(WalletFileError::Missing { .. })
    ));

    create_new(&path, &w1()).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), W1_JSON);
    assert_eq!(load(&path).unwrap(), w1());
    assert!(matches!(
        create_new(&path, &w1()),
        Err(WalletFileError::Exists { .. })
    ));

    let mut updated = w1();
    updated.next_index = NextIndex {
        receive: 3,
        change: 7,
    };
    save(&path, &updated).unwrap();
    assert_eq!(load(&path).unwrap(), updated);
    let mut temp = path.as_os_str().to_owned();
    temp.push(".tmp");
    assert!(!PathBuf::from(temp).exists());
    let leftovers: Vec<_> = std::fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(leftovers, vec![std::ffi::OsString::from("wallet.json")]);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}

#[test]
fn encrypted_json_format_is_exact_and_round_trips() {
    let wallet = w2();
    assert_eq!(wallet.to_json().unwrap(), W2_JSON);
    assert_eq!(WalletFile::from_json(W2_JSON).unwrap(), wallet);
    assert!(wallet.is_encrypted());
    assert!(!w1().is_encrypted());
    assert!(!W2_JSON.contains("abandon"));
    assert!(matches!(wallet.mnemonic(), Err(WalletFileError::Encrypted)));
    assert_eq!(wallet.account().unwrap().unwrap().to_account_hex(), W4);
    assert_eq!(w1().account().unwrap(), None);
    assert_eq!(w1().sealed().unwrap(), None);
    let sealed = wallet.sealed().unwrap().unwrap();
    assert_eq!(
        unseal(&sealed, b"kata sandi uji").unwrap().as_slice(),
        W1.as_bytes()
    );
    let text = format!("{:?}", w1());
    assert!(!text.contains("abandon"), "{text}");
}

#[test]
fn from_json_rejects_bad_encrypted_content() {
    let salt = "\"salt\": \"11111111111111111111111111111111\"";
    let bad = [
        W2_JSON.replace("argon2id", "scrypt"),
        W2_JSON.replace("xchacha20poly1305", "aes256gcm"),
        W2_JSON.replace(salt, "\"salt\": \"111111111111111111111111111111\""),
        W2_JSON.replace(salt, "\"salt\": \"1111111111111111111111111111111G\""),
        W2_JSON.replace("222222222222222222222222", "2222222222222222"),
        W2_JSON.replace("ad267\"", "ad26\""),
        W2_JSON.replace("\"version\": 2", "\"version\": 1"),
        W2_JSON.replace("\"version\": 2", "\"version\": 3"),
        W2_JSON.replace("03b9385a", "07b9385a"),
        W2_JSON.replace(
            "\"encrypted\": {",
            "\"mnemonic\": \"abandon\",\n  \"encrypted\": {",
        ),
        W2_JSON.replace(
            "\"memory_kib\": 64",
            "\"memory_kib\": 64,\n    \"extra\": 1",
        ),
    ];
    for text in bad {
        assert!(
            matches!(
                WalletFile::from_json(&text),
                Err(WalletFileError::Format(_))
            ),
            "{text}"
        );
    }
    // File tanpa kunci akun atau tanpa frasa terenkripsi.
    let mut missing_account = w2();
    missing_account.account = None;
    let mut missing_phrase = w2();
    missing_phrase.encrypted = None;
    for wallet in [missing_account, missing_phrase] {
        let text = wallet.to_json().unwrap();
        assert!(
            matches!(
                WalletFile::from_json(&text),
                Err(WalletFileError::Format(_))
            ),
            "{text}"
        );
    }
}

#[test]
fn testnet_wallet_file_and_network_check() {
    let testnet_json = W1_JSON.replace("regtest", "testnet");
    let testnet = WalletFile::from_json(&testnet_json).unwrap();
    assert!(matches!(testnet.network(), Ok(Network::Testnet)));
    let mnemonic = Mnemonic::parse(W1).unwrap();
    assert_eq!(WalletFile::new(Network::Testnet, &mnemonic), testnet);
    assert_eq!(network_name(Network::Testnet), "testnet");
    assert_eq!(network_name(Network::Regtest), "regtest");

    let dir = TempDir::new("network");
    let path = wallet_path(Network::Testnet, &dir.0);
    create_new(&path, &testnet).unwrap();
    assert_eq!(load_network(Network::Testnet, &path).unwrap(), testnet);
    let copied = wallet_path(Network::Regtest, &dir.0);
    create_new(&copied, &testnet).unwrap();
    let error = load_network(Network::Regtest, &copied).unwrap_err();
    assert!(matches!(
        &error,
        WalletFileError::WrongNetwork { expected: "regtest", found, .. } if found == "testnet"
    ));
    assert!(
        error
            .to_string()
            .ends_with("untuk jaringan testnet, bukan regtest"),
        "{error}"
    );
}
