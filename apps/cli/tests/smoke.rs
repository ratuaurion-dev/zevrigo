//! Smoke test CLI untuk biner `zevrigo-cli` (MOD-12).

#![forbid(unsafe_code)]

use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_zevrigo-cli");

#[test]
fn version_flag_prints_package_version() {
    let out = Command::new(BIN).arg("--version").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        stdout,
        format!("zevrigo-cli {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn help_and_no_args_print_usage() {
    for args in [&[][..], &["--help"][..]] {
        let out = Command::new(BIN).args(args).output().unwrap();
        assert_eq!(out.status.code(), Some(0));
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert!(stdout.contains("zevrigo-cli"));
        assert!(stdout.contains("MOD-12"));
        assert!(stdout.contains("wallet-create"));
        assert!(stdout.contains("send --to <alamat> --amount <ZVRG>"));
        assert!(!stdout.contains("kerangka R0"));
    }
}

#[test]
fn unknown_arg_exits_with_code_two() {
    let out = Command::new(BIN).arg("--tidak-dikenal").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(!out.stderr.is_empty());
}

/// Folder sementara khusus satu test, dihapus saat selesai.
struct TempDir(std::path::PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn encrypted_wallet_with_password_from_environment() {
    let dir = TempDir(std::env::temp_dir().join(format!("zevrigo-q89-cli-{}", std::process::id())));
    let _ = std::fs::remove_dir_all(&dir.0);
    let cli = |args: &[&str], password: Option<&str>| {
        let mut command = Command::new(BIN);
        command.arg("--datadir").arg(&dir.0).args(args);
        command.env_remove("ZEVRIGO_WALLET_PASSWORD");
        if let Some(password) = password {
            command.env("ZEVRIGO_WALLET_PASSWORD", password);
        }
        command.output().unwrap()
    };

    // wallet-create dengan parameter Argon2id bawaan (64 MiB): frasa dicetak, file hanya berisi frasa terenkripsi.
    let created = cli(&["wallet-create"], Some("kata sandi uji smoke"));
    assert_eq!(created.status.code(), Some(0), "{created:?}");
    let stdout = String::from_utf8(created.stdout).unwrap();
    assert!(stdout.contains("dikunci password"), "{stdout}");
    let phrase = stdout.lines().next_back().unwrap().to_owned();
    assert_eq!(phrase.split(' ').count(), 24, "{stdout}");
    let file = std::fs::read_to_string(dir.0.join("regtest").join("wallet.json")).unwrap();
    assert!(file.contains("\"memory_kib\": 65536"), "{file}");
    assert!(!file.contains(&phrase));

    let shown = cli(&["wallet-show-phrase"], Some("kata sandi uji smoke"));
    assert_eq!(shown.status.code(), Some(0), "{shown:?}");
    assert_eq!(
        String::from_utf8(shown.stdout).unwrap(),
        format!("frasa pemulihan (24 kata):\n{phrase}\n")
    );
    let wrong = cli(&["wallet-show-phrase"], Some("password lain sekali"));
    assert_eq!(wrong.status.code(), Some(1));
    assert!(
        String::from_utf8(wrong.stderr)
            .unwrap()
            .contains("password salah")
    );

    // address tidak butuh password.
    let address = cli(&["address"], None);
    assert_eq!(address.status.code(), Some(0), "{address:?}");
    assert!(
        String::from_utf8(address.stdout)
            .unwrap()
            .starts_with("alamat: rzvrg1")
    );

    // Password baru terlalu pendek: wallet tidak dibuat.
    let other =
        TempDir(std::env::temp_dir().join(format!("zevrigo-q89-cli-short-{}", std::process::id())));
    let short = Command::new(BIN)
        .arg("--datadir")
        .arg(&other.0)
        .arg("wallet-create")
        .env("ZEVRIGO_WALLET_PASSWORD", "pendek")
        .output()
        .unwrap();
    assert_eq!(short.status.code(), Some(1));
    assert!(
        String::from_utf8(short.stderr)
            .unwrap()
            .contains("minimal 8 karakter")
    );
    assert!(!other.0.join("regtest").join("wallet.json").exists());
}
