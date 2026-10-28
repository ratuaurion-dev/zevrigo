//! Tanda tangan dan verifikasi Schnorr BIP340 (x-only) di atas kurva `secp256k1`.

use core::fmt;

use k256::schnorr::{SigningKey, VerifyingKey};

/// Galat pada operasi kunci dan tanda tangan Schnorr BIP340.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SchnorrError {
    /// Koordinat x-only 32 byte tidak mewakili titik sah pada kurva `secp256k1`.
    #[error("kunci publik x-only BIP340 tidak berada pada kurva secp256k1")]
    InvalidPublicKey,
    /// Skalar kunci rahasia bernilai nol atau lebih besar/sama dengan orde kurva `secp256k1`.
    #[error("kunci rahasia BIP340 tidak sah (nol atau >= orde kurva secp256k1)")]
    InvalidSecretKey,
    /// Pembuatan tanda tangan Schnorr BIP340 gagal.
    #[error("pembuatan tanda tangan Schnorr BIP340 gagal")]
    SigningFailed,
}

/// Kunci publik x-only Schnorr BIP340 berukuran tetap 32 byte pada kurva `secp256k1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    /// Memvalidasi dan membangun kunci publik x-only dari 32 byte big-endian.
    ///
    /// Memastikan bahwa koordinat `x` berada pada kurva `secp256k1` (`lift_x` sesuai BIP340).
    ///
    /// # Errors
    ///
    /// Mengembalikan [`SchnorrError::InvalidPublicKey`] bila koordinat `x` melebihi ukuran
    /// medan prima atau tidak memiliki akar kuadrat `y` pada kurva `secp256k1`.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, SchnorrError> {
        VerifyingKey::from_bytes(bytes).map_err(|_| SchnorrError::InvalidPublicKey)?;
        Ok(Self(*bytes))
    }

    /// Mengembalikan referensi ke 32 byte koordinat x-only kunci publik.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Kunci rahasia Schnorr BIP340 pada kurva `secp256k1`.
///
/// Tipe ini sengaja tidak mengimplementasikan `Display`, dan [`fmt::Debug`] tidak pernah
/// mencetak byte rahasia. Saat nilai keluar dari cakupan (`Drop`), memori skalar rahasia
/// dibersihkan secara otomatis oleh implementasi `Drop` (`ZeroizeOnDrop`) pada
/// [`k256::schnorr::SigningKey`].
#[derive(Clone)]
pub struct SecretKey {
    inner: SigningKey,
}

impl SecretKey {
    /// Memvalidasi dan membangun kunci rahasia dari 32 byte skalar big-endian.
    ///
    /// # Errors
    ///
    /// Mengembalikan [`SchnorrError::InvalidSecretKey`] bila skalar bernilai `0` atau
    /// lebih besar/sama dengan orde kurva `secp256k1`.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, SchnorrError> {
        let inner = SigningKey::from_bytes(bytes).map_err(|_| SchnorrError::InvalidSecretKey)?;
        Ok(Self { inner })
    }

    /// Menurunkan kunci publik x-only BIP340 yang berpadanan dengan kunci rahasia ini.
    #[must_use]
    pub fn public_key(&self) -> PublicKey {
        let bytes: [u8; 32] = self.inner.verifying_key().to_bytes().into();
        PublicKey(bytes)
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretKey(<rahasia>)")
    }
}

/// Tanda tangan Schnorr BIP340 berukuran tetap 64 byte (`r || s`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Signature([u8; 64]);

impl Signature {
    /// Membungkus 64 byte kandidat tanda tangan Schnorr BIP340.
    ///
    /// Validasi rentang komponen (`r < p`, `s < n`) dan persamaan kurva dilakukan saat [`verify`].
    #[must_use]
    pub const fn from_bytes(bytes: &[u8; 64]) -> Self {
        Self(*bytes)
    }

    /// Mengembalikan referensi ke 64 byte tanda tangan (`r || s`).
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

/// Menandatangani pesan 32 byte memakai skema Schnorr BIP340 secara deterministik
/// terhadap tupel `(sk, msg, aux_rand)`.
///
/// Crate ini tidak pernah menghasilkan angka acak sendiri; `aux_rand` wajib disediakan
/// oleh pemanggil.
///
/// # Errors
///
/// Mengembalikan [`SchnorrError::SigningFailed`] apabila pembentukan tanda tangan BIP340 gagal.
pub fn sign(
    sk: &SecretKey,
    msg: &[u8; 32],
    aux_rand: &[u8; 32],
) -> Result<Signature, SchnorrError> {
    let sig = sk
        .inner
        .sign_raw(msg, aux_rand)
        .map_err(|_| SchnorrError::SigningFailed)?;
    Ok(Signature(sig.to_bytes()))
}

/// Memverifikasi tanda tangan Schnorr BIP340 atas pesan 32 byte terhadap kunci publik x-only.
#[must_use]
pub fn verify(pk: &PublicKey, msg: &[u8; 32], sig: &Signature) -> bool {
    let Ok(vk) = VerifyingKey::from_bytes(pk.as_bytes()) else {
        return false;
    };
    let Ok(parsed_sig) = k256::schnorr::Signature::try_from(sig.as_bytes().as_slice()) else {
        return false;
    };
    vk.verify_raw(msg, &parsed_sig).is_ok()
}

#[cfg(test)]
mod tests;
