//! File wallet CLI (Spesifikasi Wallet v0.1 Bagian 7): `<datadir>/<jaringan>/wallet.json`, jaringan regtest atau
//! testnet (TSK-M06 bagian C3b).
//!
//! Versi 2 (DEC-27, dibuat `wallet-create`, `wallet-restore`, `wallet-encrypt`): `version`, `network`, `account`
//! (kunci akun publik untuk alamat dan saldo tanpa password), `encrypted` (frasa terenkripsi Argon2id +
//! XChaCha20-Poly1305 beserta parameternya), dan `next_index`. Versi 1 (lama): `mnemonic` tanpa enkripsi menggantikan
//! `account` dan `encrypted`; tetap bisa dibaca dan diubah ke versi 2 dengan `wallet-encrypt`.
//!
//! Passphrase dan password tidak disimpan. File baru dibuat dengan izin `0600` dan tidak pernah menimpa file yang
//! sudah ada; pembaruan ditulis ke file sementara lalu diganti nama.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use homeostasis_primitives::Network;
use homeostasis_rpc::value::{decode_hex, encode_hex};
use homeostasis_wallet_core::{
    ExtendedPubKey, KdfParams, Mnemonic, NONCE_LEN, SALT_LEN, Sealed, vault::TAG_LEN,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Versi format file wallet terenkripsi (`2`), dipakai untuk semua wallet baru.
pub const WALLET_VERSION: u32 = 2;

/// Versi format file wallet lama tanpa enkripsi (`1`).
pub const WALLET_VERSION_PLAIN: u32 = 1;

/// Nama KDF di file wallet versi 2.
pub const KDF_NAME: &str = "argon2id";

/// Nama cipher di file wallet versi 2.
pub const CIPHER_NAME: &str = "xchacha20poly1305";

/// Nama jaringan di file wallet, sekaligus nama folder jaringan di folder data (sama dengan node).
#[must_use]
pub const fn network_name(network: Network) -> &'static str {
    match network {
        Network::Mainnet => "mainnet",
        Network::Testnet => "testnet",
        Network::Regtest => "regtest",
    }
}

/// Jaringan yang didukung file wallet: regtest dan testnet.
fn parse_network(name: &str) -> Option<Network> {
    match name {
        "regtest" => Some(Network::Regtest),
        "testnet" => Some(Network::Testnet),
        _ => None,
    }
}

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
    /// Frasa dibutuhkan tanpa password, tetapi wallet terenkripsi.
    #[error("wallet terenkripsi; frasa hanya bisa dibuka dengan password")]
    Encrypted,
    /// File wallet milik jaringan lain (misalnya disalin dari folder jaringan lain).
    #[error("wallet di {path} untuk jaringan {found}, bukan {expected}")]
    WrongNetwork {
        /// Path file wallet.
        path: String,
        /// Jaringan yang dipakai CLI.
        expected: &'static str,
        /// Jaringan di file wallet.
        found: String,
    },
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

/// Frasa terenkripsi di file wallet versi 2. Tidak ada yang rahasia di sini.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncryptedPhrase {
    /// Selalu [`KDF_NAME`].
    pub kdf: String,
    /// Memori Argon2id dalam KiB.
    pub memory_kib: u32,
    /// Putaran Argon2id.
    pub iterations: u32,
    /// Jalur Argon2id.
    pub parallelism: u32,
    /// Salt 16 byte, hex huruf kecil.
    pub salt: String,
    /// Selalu [`CIPHER_NAME`].
    pub cipher: String,
    /// Nonce 24 byte, hex huruf kecil.
    pub nonce: String,
    /// Ciphertext frasa diikuti tag 16 byte, hex huruf kecil.
    pub ciphertext: String,
}

fn format_error(message: impl Into<String>) -> WalletFileError {
    WalletFileError::Format(message.into())
}

/// Hex huruf kecil tepat `N` byte.
fn fixed_hex<const N: usize>(text: &str, field: &str) -> Result<[u8; N], WalletFileError> {
    decode_hex(text)
        .ok()
        .and_then(|bytes| <[u8; N]>::try_from(bytes).ok())
        .ok_or_else(|| format_error(format!("{field}: wajib hex huruf kecil {N} byte")))
}

impl EncryptedPhrase {
    /// Bentuk file dari hasil [`homeostasis_wallet_core::seal`].
    #[must_use]
    pub fn from_sealed(sealed: &Sealed) -> Self {
        Self {
            kdf: KDF_NAME.to_owned(),
            memory_kib: sealed.kdf.memory_kib,
            iterations: sealed.kdf.iterations,
            parallelism: sealed.kdf.parallelism,
            salt: encode_hex(&sealed.salt),
            cipher: CIPHER_NAME.to_owned(),
            nonce: encode_hex(&sealed.nonce),
            ciphertext: encode_hex(&sealed.ciphertext),
        }
    }

    /// Nilai untuk [`homeostasis_wallet_core::unseal`].
    ///
    /// # Errors
    ///
    /// [`WalletFileError::Format`] jika nama KDF atau cipher lain, atau hex salt, nonce, atau ciphertext tidak sah.
    pub fn to_sealed(&self) -> Result<Sealed, WalletFileError> {
        if self.kdf != KDF_NAME {
            return Err(format_error(format!("kdf {} tidak didukung", self.kdf)));
        }
        if self.cipher != CIPHER_NAME {
            return Err(format_error(format!(
                "cipher {} tidak didukung",
                self.cipher
            )));
        }
        let ciphertext = decode_hex(&self.ciphertext)
            .ok()
            .filter(|bytes| bytes.len() > TAG_LEN)
            .ok_or_else(|| format_error("ciphertext: wajib hex huruf kecil lebih dari 16 byte"))?;
        Ok(Sealed {
            kdf: KdfParams {
                memory_kib: self.memory_kib,
                iterations: self.iterations,
                parallelism: self.parallelism,
            },
            salt: fixed_hex::<SALT_LEN>(&self.salt, "salt")?,
            nonce: fixed_hex::<NONCE_LEN>(&self.nonce, "nonce")?,
            ciphertext,
        })
    }
}

/// Isi file wallet. [`core::fmt::Debug`] tidak mencetak frasa.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalletFile {
    /// Versi format: [`WALLET_VERSION`] atau [`WALLET_VERSION_PLAIN`].
    pub version: u32,
    /// Jaringan: `regtest` atau `testnet` ([`network_name`]).
    pub network: String,
    /// Versi 1: frasa pemulihan yang dinormalkan (huruf kecil, satu spasi).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mnemonic: Option<String>,
    /// Versi 2: kunci akun publik `m/44'/1'/0'` (hex 130 karakter).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// Versi 2: frasa terenkripsi.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encrypted: Option<EncryptedPhrase>,
    /// Indeks berikutnya per cabang.
    pub next_index: NextIndex,
}

impl core::fmt::Debug for WalletFile {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WalletFile")
            .field("version", &self.version)
            .field("network", &self.network)
            .field("mnemonic", &self.mnemonic.as_ref().map(|_| "<rahasia>"))
            .field("account", &self.account)
            .field("encrypted", &self.encrypted)
            .field("next_index", &self.next_index)
            .finish()
    }
}

impl WalletFile {
    /// Wallet versi 1 tanpa enkripsi untuk frasa ini, dengan indeks berikutnya 0 di kedua cabang. Hanya untuk file
    /// lama dan test; perintah CLI selalu membuat versi 2.
    #[must_use]
    pub fn new(network: Network, mnemonic: &Mnemonic) -> Self {
        Self {
            version: WALLET_VERSION_PLAIN,
            network: network_name(network).to_owned(),
            mnemonic: Some(mnemonic.phrase().to_owned()),
            account: None,
            encrypted: None,
            next_index: NextIndex::default(),
        }
    }

    /// Wallet versi 2: kunci akun publik dan frasa terenkripsi.
    #[must_use]
    pub fn new_encrypted(
        network: Network,
        account: &ExtendedPubKey,
        sealed: &Sealed,
        next_index: NextIndex,
    ) -> Self {
        Self {
            version: WALLET_VERSION,
            network: network_name(network).to_owned(),
            mnemonic: None,
            account: Some(account.to_account_hex()),
            encrypted: Some(EncryptedPhrase::from_sealed(sealed)),
            next_index,
        }
    }

    /// Jaringan file wallet.
    ///
    /// # Errors
    ///
    /// [`WalletFileError::Format`] jika jaringannya bukan regtest atau testnet.
    pub fn network(&self) -> Result<Network, WalletFileError> {
        parse_network(&self.network).ok_or_else(|| {
            WalletFileError::Format(format!("jaringan {} tidak didukung", self.network))
        })
    }

    /// Apakah frasa terenkripsi (versi 2).
    #[must_use]
    pub const fn is_encrypted(&self) -> bool {
        self.encrypted.is_some()
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

    /// Mengurai JSON dan memeriksa versi, jaringan, serta isi sesuai versi.
    ///
    /// # Errors
    ///
    /// [`WalletFileError::Format`] jika JSON rusak, ada field asing, versi bukan 1 atau 2, jaringan bukan regtest atau
    /// testnet, field versi lain ikut ada, frasa versi 1 tidak sah atau tidak ternormalkan, atau kunci akun dan frasa
    /// terenkripsi versi 2 tidak sah.
    pub fn from_json(text: &str) -> Result<Self, WalletFileError> {
        let file: Self = serde_json::from_str(text)
            .map_err(|error| WalletFileError::Format(error.to_string()))?;
        file.network()?;
        match file.version {
            WALLET_VERSION_PLAIN => {
                if file.account.is_some() || file.encrypted.is_some() {
                    return Err(format_error(
                        "versi 1 tidak boleh memuat account atau encrypted",
                    ));
                }
                let text = file
                    .mnemonic
                    .as_deref()
                    .ok_or_else(|| format_error("versi 1 wajib memuat mnemonic"))?;
                let mnemonic = Mnemonic::parse(text)
                    .map_err(|error| WalletFileError::Format(format!("frasa: {error}")))?;
                if mnemonic.phrase() != text {
                    return Err(WalletFileError::Format(
                        "frasa tidak dalam bentuk ternormalkan".to_owned(),
                    ));
                }
            }
            WALLET_VERSION => {
                if file.mnemonic.is_some() {
                    return Err(format_error("versi 2 tidak boleh memuat mnemonic"));
                }
                if file.account()?.is_none() || file.sealed()?.is_none() {
                    return Err(format_error("versi 2 wajib memuat account dan encrypted"));
                }
            }
            other => {
                return Err(WalletFileError::Format(format!(
                    "versi {other} tidak didukung"
                )));
            }
        }
        Ok(file)
    }

    /// Frasa pemulihan versi 1 yang sudah diperiksa.
    ///
    /// # Errors
    ///
    /// [`WalletFileError::Encrypted`] untuk versi 2, [`WalletFileError::Format`] jika frasa tidak ada atau tidak sah.
    pub fn mnemonic(&self) -> Result<Mnemonic, WalletFileError> {
        if self.is_encrypted() {
            return Err(WalletFileError::Encrypted);
        }
        let text = self
            .mnemonic
            .as_deref()
            .ok_or_else(|| format_error("frasa tidak ada"))?;
        Mnemonic::parse(text).map_err(|error| WalletFileError::Format(format!("frasa: {error}")))
    }

    /// Kunci akun publik versi 2 (`None` untuk versi 1).
    ///
    /// # Errors
    ///
    /// [`WalletFileError::Format`] jika kunci akun tidak sah.
    pub fn account(&self) -> Result<Option<ExtendedPubKey>, WalletFileError> {
        self.account
            .as_deref()
            .map(|text| {
                ExtendedPubKey::from_account_hex(text)
                    .map_err(|error| WalletFileError::Format(format!("account: {error}")))
            })
            .transpose()
    }

    /// Frasa terenkripsi versi 2 (`None` untuk versi 1).
    ///
    /// # Errors
    ///
    /// [`WalletFileError::Format`] jika isinya tidak sah.
    pub fn sealed(&self) -> Result<Option<Sealed>, WalletFileError> {
        self.encrypted
            .as_ref()
            .map(EncryptedPhrase::to_sealed)
            .transpose()
    }
}

/// Path file wallet: `<datadir>/<jaringan>/wallet.json`.
#[must_use]
pub fn wallet_path(network: Network, datadir: &Path) -> PathBuf {
    datadir.join(network_name(network)).join(WALLET_FILE)
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

/// Membaca file wallet jaringan `network`, dan menolaknya jika file itu milik jaringan lain.
///
/// # Errors
///
/// Sama dengan [`load`], ditambah [`WalletFileError::WrongNetwork`].
pub fn load_network(network: Network, path: &Path) -> Result<WalletFile, WalletFileError> {
    let file = load(path)?;
    if file.network()? != network {
        return Err(WalletFileError::WrongNetwork {
            path: path.display().to_string(),
            expected: network_name(network),
            found: file.network,
        });
    }
    Ok(file)
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
