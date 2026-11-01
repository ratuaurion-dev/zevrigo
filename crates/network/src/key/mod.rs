//! Kunci identitas rekan (Spesifikasi P2P Bagian 1): Ed25519 libp2p, terpisah dari kunci wallet.
//!
//! 32 byte rahasia mentah di `<datadir>/<jaringan>/p2p_key` (izin `0600`), dibuat dari sumber acak sistem operasi saat
//! node pertama kali berjalan dan tidak pernah ditimpa. File baru ditulis ke `p2p_key.tmp`, lalu diganti nama.
//! `PeerId` diturunkan dari kunci ini.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use libp2p::identity::{Keypair, ed25519};
use thiserror::Error;

/// Nama file kunci rekan di folder jaringan.
pub const P2P_KEY_FILE: &str = "p2p_key";

/// Panjang rahasia Ed25519 (32 byte).
pub const SECRET_LEN: usize = 32;

/// Akhiran file sementara saat membuat kunci.
const TEMP_SUFFIX: &str = ".tmp";

/// Galat kunci rekan.
#[derive(Debug, Error)]
pub enum KeyError {
    /// Galat baca atau tulis file.
    #[error("galat file kunci rekan {path}: {reason}")]
    Io {
        /// Path file.
        path: String,
        /// Pesan galat sistem operasi.
        reason: String,
    },
    /// Isi file bukan tepat 32 byte.
    #[error("file kunci rekan {path} berisi {len} byte (wajib 32)")]
    BadLength {
        /// Path file.
        path: String,
        /// Panjang isi file.
        len: usize,
    },
    /// Rahasia tidak diterima libp2p.
    #[error("rahasia Ed25519 tidak sah")]
    Invalid,
}

fn io_error(path: &Path, error: &std::io::Error) -> KeyError {
    KeyError::Io {
        path: path.display().to_string(),
        reason: error.to_string(),
    }
}

/// Keypair libp2p dari 32 byte rahasia Ed25519.
///
/// # Errors
///
/// [`KeyError::Invalid`] jika libp2p menolak rahasia itu.
pub fn keypair_from_secret(secret: [u8; SECRET_LEN]) -> Result<Keypair, KeyError> {
    Keypair::ed25519_from_bytes(secret).map_err(|_| KeyError::Invalid)
}

/// 32 byte rahasia baru dari sumber acak sistem operasi (lewat libp2p).
#[must_use]
pub fn generate_secret() -> [u8; SECRET_LEN] {
    let pair = ed25519::Keypair::generate().to_bytes();
    let mut secret = [0_u8; SECRET_LEN];
    for (out, byte) in secret.iter_mut().zip(pair.iter()) {
        *out = *byte;
    }
    secret
}

/// Membaca kunci rekan dari `path`, atau membuatnya jika file belum ada.
///
/// # Errors
///
/// [`KeyError::BadLength`] jika file ada tetapi bukan 32 byte, [`KeyError::Io`] untuk galat file lain.
pub fn load_or_create(path: &Path) -> Result<Keypair, KeyError> {
    match fs::read(path) {
        Ok(bytes) => {
            let secret = <[u8; SECRET_LEN]>::try_from(bytes.as_slice()).map_err(|_| {
                KeyError::BadLength {
                    path: path.display().to_string(),
                    len: bytes.len(),
                }
            })?;
            keypair_from_secret(secret)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let secret = generate_secret();
            write_key(path, &secret)?;
            keypair_from_secret(secret)
        }
        Err(error) => Err(io_error(path, &error)),
    }
}

/// Menulis kunci baru: `<path>.tmp` dengan izin `0600`, lalu ganti nama menjadi `path`.
fn write_key(path: &Path, secret: &[u8; SECRET_LEN]) -> Result<(), KeyError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| io_error(parent, &error))?;
    }
    let mut temp = path.as_os_str().to_owned();
    temp.push(TEMP_SUFFIX);
    let temp = PathBuf::from(temp);
    match fs::remove_file(&temp) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(io_error(&temp, &error)),
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temp)
        .map_err(|error| io_error(&temp, &error))?;
    file.write_all(secret)
        .map_err(|error| io_error(&temp, &error))?;
    file.sync_all().map_err(|error| io_error(&temp, &error))?;
    fs::rename(&temp, path).map_err(|error| io_error(path, &error))
}

#[cfg(test)]
mod tests;
