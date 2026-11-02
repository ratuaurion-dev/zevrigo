//! Enkripsi rahasia wallet dengan password (DEC-27, Spesifikasi Wallet v0.1 Bagian 7).
//!
//! Kunci 32 byte diturunkan dari password lewat Argon2id versi 0x13 (bawaan 64 MiB, 3 putaran, 1 jalur), lalu
//! rahasia dienkripsi XChaCha20-Poly1305 dengan nonce 24 byte. Parameter KDF dan salt ikut diautentikasi sebagai data
//! tambahan (AAD), sehingga isi yang diubah ditolak sama seperti password yang salah.
//!
//! Salt dan nonce diberikan pemanggil dari sumber acak sistem operasi; modul ini tidak membaca sumber acak. Kunci
//! turunan, memori kerja Argon2id, dan hasil dekripsi dihapus dari memori sesudah dipakai (`zeroize`).

use argon2::{Algorithm, Argon2, Block, Params, Version};
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use thiserror::Error;
use zeroize::{Zeroize, Zeroizing};

/// Panjang salt Argon2id dalam byte.
pub const SALT_LEN: usize = 16;

/// Panjang nonce XChaCha20-Poly1305 dalam byte.
pub const NONCE_LEN: usize = 24;

/// Panjang kunci turunan dalam byte.
pub const KEY_LEN: usize = 32;

/// Panjang tag autentikasi Poly1305 dalam byte, di akhir ciphertext.
pub const TAG_LEN: usize = 16;

/// Awalan data tambahan (AAD) enkripsi.
const AAD_DOMAIN: &[u8] = b"homeostasis wallet vault v1";

/// Parameter Argon2id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KdfParams {
    /// Memori dalam KiB.
    pub memory_kib: u32,
    /// Jumlah putaran.
    pub iterations: u32,
    /// Jumlah jalur.
    pub parallelism: u32,
}

impl KdfParams {
    /// Bawaan untuk wallet baru: 64 MiB, 3 putaran, 1 jalur.
    pub const DEFAULT: Self = Self {
        memory_kib: 65_536,
        iterations: 3,
        parallelism: 1,
    };

    /// Memori paling besar yang diterima dari file wallet (1 GiB), agar file buatan tidak menghabiskan memori.
    pub const MAX_MEMORY_KIB: u32 = 1_048_576;

    /// Jumlah putaran paling banyak yang diterima.
    pub const MAX_ITERATIONS: u32 = 64;

    /// Jumlah jalur paling banyak yang diterima.
    pub const MAX_PARALLELISM: u32 = 16;

    /// Parameter `argon2` dengan keluaran [`KEY_LEN`] byte, setelah batas di atas diperiksa. Memori paling sedikit
    /// 8 KiB per jalur (syarat Argon2).
    fn argon2_params(self) -> Result<Params, VaultError> {
        let in_range = (1..=Self::MAX_PARALLELISM).contains(&self.parallelism)
            && (1..=Self::MAX_ITERATIONS).contains(&self.iterations)
            && self.memory_kib <= Self::MAX_MEMORY_KIB
            && self
                .parallelism
                .checked_mul(8)
                .is_some_and(|minimum| self.memory_kib >= minimum);
        if !in_range {
            return Err(VaultError::InvalidParams);
        }
        Params::new(
            self.memory_kib,
            self.iterations,
            self.parallelism,
            Some(KEY_LEN),
        )
        .map_err(|_| VaultError::InvalidParams)
    }
}

impl Default for KdfParams {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Rahasia terenkripsi beserta parameter untuk membukanya. Tidak ada yang rahasia di sini.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sealed {
    /// Parameter Argon2id.
    pub kdf: KdfParams,
    /// Salt Argon2id.
    pub salt: [u8; SALT_LEN],
    /// Nonce XChaCha20-Poly1305.
    pub nonce: [u8; NONCE_LEN],
    /// Ciphertext diikuti tag [`TAG_LEN`] byte.
    pub ciphertext: Vec<u8>,
}

/// Galat enkripsi rahasia wallet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum VaultError {
    /// Parameter Argon2id di luar batas [`KdfParams`].
    #[error("parameter Argon2id di luar batas")]
    InvalidParams,
    /// Argon2id gagal.
    #[error("Argon2id gagal")]
    Kdf,
    /// Enkripsi gagal.
    #[error("enkripsi gagal")]
    Encrypt,
    /// Password salah, atau isi terenkripsi atau parameternya diubah.
    #[error("password salah atau isi terenkripsi rusak")]
    Decrypt,
}

/// Data tambahan: awalan, parameter Argon2id (masing-masing `u32` big-endian), dan salt.
fn associated_data(kdf: KdfParams, salt: &[u8; SALT_LEN]) -> Vec<u8> {
    let mut aad = AAD_DOMAIN.to_vec();
    aad.extend_from_slice(&kdf.memory_kib.to_be_bytes());
    aad.extend_from_slice(&kdf.iterations.to_be_bytes());
    aad.extend_from_slice(&kdf.parallelism.to_be_bytes());
    aad.extend_from_slice(salt);
    aad
}

/// Kunci [`KEY_LEN`] byte dari `password` dan `salt` (Argon2id versi 0x13, tanpa kunci rahasia dan data tambahan).
fn derive_key(
    password: &[u8],
    salt: &[u8; SALT_LEN],
    kdf: KdfParams,
) -> Result<Zeroizing<[u8; KEY_LEN]>, VaultError> {
    let params = kdf.argon2_params()?;
    let mut memory = vec![Block::new(); params.block_count()];
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0_u8; KEY_LEN]);
    let result = argon.hash_password_into_with_memory(
        password,
        salt,
        key.as_mut_slice(),
        memory.as_mut_slice(),
    );
    memory.iter_mut().for_each(Zeroize::zeroize);
    result.map_err(|_| VaultError::Kdf)?;
    Ok(key)
}

fn cipher(key: &Zeroizing<[u8; KEY_LEN]>) -> Result<XChaCha20Poly1305, VaultError> {
    XChaCha20Poly1305::new_from_slice(key.as_slice()).map_err(|_| VaultError::Encrypt)
}

/// Mengenkripsi `plaintext` dengan kunci dari `password`.
///
/// # Errors
///
/// [`VaultError::InvalidParams`] jika `kdf` di luar batas, [`VaultError::Kdf`] atau [`VaultError::Encrypt`] jika
/// pustaka kriptografi gagal.
pub fn seal(
    plaintext: &[u8],
    password: &[u8],
    kdf: KdfParams,
    salt: [u8; SALT_LEN],
    nonce: [u8; NONCE_LEN],
) -> Result<Sealed, VaultError> {
    let key = derive_key(password, &salt, kdf)?;
    let aad = associated_data(kdf, &salt);
    let ciphertext = cipher(&key)?
        .encrypt(
            &XNonce::from(nonce),
            Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| VaultError::Encrypt)?;
    Ok(Sealed {
        kdf,
        salt,
        nonce,
        ciphertext,
    })
}

/// Membuka `sealed` dengan `password`. Hasilnya dihapus dari memori saat dibuang.
///
/// # Errors
///
/// [`VaultError::Decrypt`] jika password salah atau isi diubah, [`VaultError::InvalidParams`] jika parameter di luar
/// batas, [`VaultError::Kdf`] atau [`VaultError::Encrypt`] jika pustaka kriptografi gagal.
pub fn unseal(sealed: &Sealed, password: &[u8]) -> Result<Zeroizing<Vec<u8>>, VaultError> {
    if sealed.ciphertext.len() < TAG_LEN {
        return Err(VaultError::Decrypt);
    }
    let key = derive_key(password, &sealed.salt, sealed.kdf)?;
    let aad = associated_data(sealed.kdf, &sealed.salt);
    cipher(&key)?
        .decrypt(
            &XNonce::from(sealed.nonce),
            Payload {
                msg: &sealed.ciphertext,
                aad: &aad,
            },
        )
        .map(Zeroizing::new)
        .map_err(|_| VaultError::Decrypt)
}

#[cfg(test)]
mod tests;
