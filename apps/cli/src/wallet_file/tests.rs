use std::path::PathBuf;

use homeostasis_wallet_core::Mnemonic;

use super::{NextIndex, WalletFile, WalletFileError, create_new, load, save, wallet_path};

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
    WalletFile::new(&Mnemonic::parse(W1).unwrap())
}

#[test]
fn json_format_is_exact_and_round_trips() {
    let wallet = w1();
    assert_eq!(wallet.to_json().unwrap(), W1_JSON);
    assert_eq!(WalletFile::from_json(W1_JSON).unwrap(), wallet);
    assert_eq!(wallet.mnemonic().unwrap().phrase(), W1);
    assert_eq!(
        wallet_path(&PathBuf::from("/data")),
        PathBuf::from("/data/regtest/wallet.json")
    );
    let text = format!("{wallet:?}");
    assert!(!text.contains("abandon"), "{text}");
}

#[test]
fn from_json_rejects_bad_content() {
    let bad = [
        W1_JSON.replace("\"version\": 1", "\"version\": 2"),
        W1_JSON.replace("regtest", "testnet"),
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
    let path = wallet_path(&dir.0);
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
