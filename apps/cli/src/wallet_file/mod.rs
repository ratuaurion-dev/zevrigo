//! File wallet CLI regtest (Spesifikasi Wallet v0.1 Bagian 7): `<datadir>/regtest/wallet.json`.
//!
//! Isi: `version` (1), `network` (`regtest`), `mnemonic`, dan `next_index` cabang penerimaan dan kembalian.
//! Passphrase tidak disimpan. File baru dibuat dengan izin `0600` dan tidak pernah menimpa file yang sudah ada;
//! pembaruan ditulis ke file sementara lalu diganti nama. Tanpa enkripsi, karena koin regtest tidak bernilai.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use homeostasis_wallet_core::Mnemonic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Versi format file wallet (`1`).
pub const WALLET_VERSION: u32 = 1;

/// Nama jaringan di file wallet v0.1 (`regtest`).
pub const WALLET_NETWORK: &str = "regtest";

/// Nama folder jaringan di folder data (sama dengan node).
pub const NETWORK_DIR: &str = "regtest";

/// Nama file wallet.
pub const WALLET_FILE: &str = "wallet.json";

/// Akhiran file sementara saat memperbarui wallet.
const TEMP_SUFFIX: &str = ".tmp";

/// Galat file wallet.
#[derive(Debug, Error)]
pub enum WalletFileError {
    /// File wallet belum ada.
    #[error("wallet belum ada di {path} (buat dengan wallet-create atau wallet-restore)")]
    Missing {
        /// Path file wallet.
        path: String,
    },
    /// File wallet sudah ada dan tidak ditimpa.
    #[error("wallet sudah ada di {path}; tidak ditimpa")]
    Exists {
        /// Path file wallet.
        path: String,
    },
    /// Galat baca atau tulis.
    #[error("berkas wallet {path}: {source}")]
    Io {
        /// Path yang gagal dibaca atau ditulis.
        path: String,
        /// Galat dari sistem operasi.
        source: std::io::Error,
    },
    /// Isi file wallet tidak sah.
    #[error("isi wallet tidak sah: {0}")]
    Format(String),
}

fn io_error(path: &Path, source: std::io::Error) -> WalletFileError {
    WalletFileError::Io {
        path: path.display().to_string(),
        source,
    }
}

/// Indeks berikutnya yang belum pernah dipakai di cabang penerimaan (`0`) dan kembalian (`1`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NextIndex {
    /// Cabang penerimaan.
    pub receive: u32,
    /// Cabang kembalian.
    pub change: u32,
}

/// Isi file wallet. [`core::fmt::Debug`] tidak mencetak frasa.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalletFile {
    /// Versi format, selalu [`WALLET_VERSION`].
    pub version: u32,
    /// Jaringan, selalu [`WALLET_NETWORK`].
    pub network: String,
    /// Frasa pemulihan yang dinormalkan (huruf kecil, satu spasi).
    pub mnemonic: String,
    /// Indeks berikutnya per cabang.
    pub next_index: NextIndex,
}

impl core::fmt::Debug for WalletFile {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WalletFile")
            .field("version", &self.version)
            .field("network", &self.network)
            .field("mnemonic", &"<rahasia>")
            .field("next_index", &self.next_index)
            .finish()
    }
}

impl WalletFile {
    /// Wallet baru untuk frasa ini, dengan indeks berikutnya 0 di kedua cabang.
    #[must_use]
    pub fn new(mnemonic: &Mnemonic) -> Self {
        Self {
            version: WALLET_VERSION,
            network: WALLET_NETWORK.to_owned(),
            mnemonic: mnemonic.phrase().to_owned(),
            next_index: NextIndex::default(),
        }
    }

    /// Encoding JSON berindentasi, diakhiri baris baru.
    ///
    /// # Errors
    ///
    /// [`WalletFileError::Format`] jika serialisasi gagal.
    pub fn to_json(&self) -> Result<String, WalletFileError> {
        let mut text = serde_json::to_string_pretty(self)
            .map_err(|error| WalletFileError::Format(error.to_string()))?;
        text.push('\n');
        Ok(text)
    }

    /// Mengurai JSON dan memeriksa versi, jaringan, dan frasa.
    ///
    /// # Errors
    ///
    /// [`WalletFileError::Format`] jika JSON rusak, ada field asing, versi bukan 1, jaringan bukan regtest, atau
    /// frasa tidak sah atau tidak dalam bentuk ternormalkan.
    pub fn from_json(text: &str) -> Result<Self, WalletFileError> {
        let file: Self = serde_json::from_str(text)
            .map_err(|error| WalletFileError::Format(error.to_string()))?;
        if file.version != WALLET_VERSION {
            return Err(WalletFileError::Format(format!(
                "versi {} tidak didukung",
                file.version
            )));
        }
        if file.network != WALLET_NETWORK {
            return Err(WalletFileError::Format(format!(
                "jaringan {} tidak didukung",
                file.network
            )));
        }
        let mnemonic = Mnemonic::parse(&file.mnemonic)
            .map_err(|error| WalletFileError::Format(format!("frasa: {error}")))?;
        if mnemonic.phrase() != file.mnemonic {
            return Err(WalletFileError::Format(
                "frasa tidak dalam bentuk ternormalkan".to_owned(),
            ));
        }
        Ok(file)
    }

    /// Frasa pemulihan yang sudah diperiksa.
    ///
    /// # Errors
    ///
    /// [`WalletFileError::Format`] jika frasa tidak sah.
    pub fn mnemonic(&self) -> Result<Mnemonic, WalletFileError> {
        Mnemonic::parse(&self.mnemonic)
            .map_err(|error| WalletFileError::Format(format!("frasa: {error}")))
    }
}

/// Path file wallet: `<datadir>/regtest/wallet.json`.
#[must_use]
pub fn wallet_path(datadir: &Path) -> PathBuf {
    datadir.join(NETWORK_DIR).join(WALLET_FILE)
}

/// Opsi buka file baru: hanya tulis, gagal jika sudah ada, izin `0600` di Unix.
fn new_file_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

/// Menulis `text` ke file baru `path` lalu memaksa isinya ke disk.
fn write_new(path: &Path, text: &str) -> Result<(), WalletFileError> {
    let mut file = new_file_options().open(path).map_err(|error| {
        if error.kind() == ErrorKind::AlreadyExists {
            WalletFileError::Exists {
                path: path.display().to_string(),
            }
        } else {
            io_error(path, error)
        }
    })?;
    file.write_all(text.as_bytes())
        .map_err(|error| io_error(path, error))?;
    file.sync_all().map_err(|error| io_error(path, error))
}

/// Membuat file wallet baru beserta foldernya. Tidak pernah menimpa file yang sudah ada.
///
/// # Errors
///
/// [`WalletFileError::Exists`] jika file sudah ada, [`WalletFileError::Io`] untuk galat lain.
pub fn create_new(path: &Path, wallet: &WalletFile) -> Result<(), WalletFileError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
    }
    write_new(path, &wallet.to_json()?)
}

/// Membaca file wallet.
///
/// # Errors
///
/// [`WalletFileError::Missing`] jika file belum ada, [`WalletFileError::Io`], atau [`WalletFileError::Format`].
pub fn load(path: &Path) -> Result<WalletFile, WalletFileError> {
    let text = fs::read_to_string(path).map_err(|error| {
        if error.kind() == ErrorKind::NotFound {
            WalletFileError::Missing {
                path: path.display().to_string(),
            }
        } else {
            io_error(path, error)
        }
    })?;
    WalletFile::from_json(&text)
}

/// Memperbarui file wallet yang sudah ada: tulis ke `<path>.tmp` (izin `0600`), lalu ganti nama menjadi `path`.
///
/// # Errors
///
/// [`WalletFileError::Missing`] jika file wallet belum ada, [`WalletFileError::Io`] untuk galat lain.
pub fn save(path: &Path, wallet: &WalletFile) -> Result<(), WalletFileError> {
    if !path.is_file() {
        return Err(WalletFileError::Missing {
            path: path.display().to_string(),
        });
    }
    let mut temp = path.as_os_str().to_owned();
    temp.push(TEMP_SUFFIX);
    let temp = PathBuf::from(temp);
    match fs::remove_file(&temp) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(io_error(&temp, error)),
    }
    write_new(&temp, &wallet.to_json()?)?;
    fs::rename(&temp, path).map_err(|error| io_error(path, error))
}

#[cfg(test)]
mod tests;
