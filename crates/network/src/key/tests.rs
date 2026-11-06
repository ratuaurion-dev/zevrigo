use std::path::PathBuf;

use super::{KeyError, P2P_KEY_FILE, SECRET_LEN, keypair_from_secret, load_or_create};

/// Folder sementara khusus satu test, dihapus saat selesai.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("zevrigo-q78-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// RFC 8032 Bagian 7.1, TEST 1: rahasia `9d61b19d…7f60`, kunci publik `d75a9801…511a`.
const RFC8032_SECRET: [u8; SECRET_LEN] = [
    0x9d, 0x61, 0xb1, 0x9d, 0xef, 0xfd, 0x5a, 0x60, 0xba, 0x84, 0x4a, 0xf4, 0x92, 0xec, 0x2c, 0xc4,
    0x44, 0x49, 0xc5, 0x69, 0x7b, 0x32, 0x69, 0x19, 0x70, 0x3b, 0xac, 0x03, 0x1c, 0xae, 0x7f, 0x60,
];

#[test]
fn rfc8032_secret_gives_known_peer_id() {
    // PeerId = base58(multihash identity(protobuf {Type: Ed25519, Data: kunci publik})), dihitung Quick terpisah.
    let keypair = keypair_from_secret(RFC8032_SECRET).unwrap();
    assert_eq!(
        keypair.public().to_peer_id().to_string(),
        "12D3KooWQK1wnefoLrcVHbbnf5tLzbopUd3K3bFAoJpA7YJgL5pV"
    );
}

#[test]
fn key_file_is_created_once_and_reloaded() {
    let dir = TempDir::new("key");
    let path = dir.0.join("regtest").join(P2P_KEY_FILE);
    let first = load_or_create(&path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(bytes.len(), SECRET_LEN);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    assert!(!dir.0.join("regtest").join("p2p_key.tmp").exists());
    let second = load_or_create(&path).unwrap();
    assert_eq!(first.public().to_peer_id(), second.public().to_peer_id());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}

#[test]
fn key_file_with_wrong_length_is_rejected() {
    let dir = TempDir::new("bad-key");
    std::fs::create_dir_all(&dir.0).unwrap();
    let path = dir.0.join(P2P_KEY_FILE);
    std::fs::write(&path, [1_u8; 31]).unwrap();
    assert!(matches!(
        load_or_create(&path),
        Err(KeyError::BadLength { len: 31, .. })
    ));
    assert_eq!(std::fs::read(&path).unwrap(), vec![1_u8; 31]);
}
