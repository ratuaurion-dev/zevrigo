//! Penjumlahan skalar dan titik secp256k1 untuk turunan kunci BIP32 (TSK-M15, Spesifikasi Wallet v0.1 Bagian 2).
//!
//! Fungsi di modul ini dipakai MOD-09 (wallet-core), sehingga MOD-09 tidak memakai `k256` langsung. Kunci publik
//! ditulis dalam encoding SEC1 terkompresi 33 byte (`02` atau `03`, lalu koordinat x big-endian). Kunci rahasia dan
//! tweak adalah skalar 32 byte big-endian. Modul ini bukan aturan konsensus.

use k256::elliptic_curve::PrimeField;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::{FieldBytes, NonZeroScalar, ProjectivePoint, PublicKey as CurvePoint, Scalar};

/// Panjang kunci publik SEC1 terkompresi (`33` byte).
pub const COMPRESSED_PUBLIC_KEY_LEN: usize = 33;

/// Galat penjumlahan skalar dan titik secp256k1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TweakError {
    /// Kunci rahasia bernilai nol atau lebih besar/sama dengan orde kurva `secp256k1`.
    #[error("kunci rahasia tidak sah (nol atau >= orde kurva secp256k1)")]
    InvalidSecretKey,
    /// Kunci publik bukan encoding SEC1 terkompresi dari titik pada kurva `secp256k1`.
    #[error(
        "kunci publik terkompresi tidak sah (awalan bukan 02/03 atau x tidak berada pada kurva)"
    )]
    InvalidPublicKey,
    /// Tweak lebih besar/sama dengan orde kurva `secp256k1`.
    #[error("tweak tidak sah (>= orde kurva secp256k1)")]
    InvalidTweak,
    /// Hasil penjumlahan adalah skalar nol atau titik tak hingga.
    #[error("hasil penjumlahan tidak sah (skalar nol atau titik tak hingga)")]
    InvalidResult,
}

fn parse_scalar(bytes: &[u8; 32]) -> Option<Scalar> {
    Option::from(Scalar::from_repr(FieldBytes::from(*bytes)))
}

fn parse_secret(bytes: &[u8; 32]) -> Option<Scalar> {
    parse_scalar(bytes).filter(|scalar| !bool::from(scalar.is_zero()))
}

/// Mengurai kunci publik SEC1 terkompresi. Hanya awalan `02`/`03` yang diterima: `sec1` 0.7 juga menerima awalan
/// `05` (bentuk "compact", hanya x) dengan panjang 33 byte yang sama, yang bukan encoding terkompresi.
fn parse_point(point: &[u8; COMPRESSED_PUBLIC_KEY_LEN]) -> Option<CurvePoint> {
    match point.first() {
        Some(0x02 | 0x03) => CurvePoint::from_sec1_bytes(point).ok(),
        _ => None,
    }
}

fn encode_point(point: &CurvePoint) -> Result<[u8; COMPRESSED_PUBLIC_KEY_LEN], TweakError> {
    let encoded = point.to_encoded_point(true);
    <[u8; COMPRESSED_PUBLIC_KEY_LEN]>::try_from(encoded.as_bytes())
        .map_err(|_| TweakError::InvalidResult)
}

/// Menghitung kunci publik SEC1 terkompresi `k·G` dari kunci rahasia `k`.
///
/// # Errors
///
/// [`TweakError::InvalidSecretKey`] bila `secret` bernilai nol atau lebih besar/sama dengan orde kurva.
pub fn public_key_compressed(
    secret: &[u8; 32],
) -> Result<[u8; COMPRESSED_PUBLIC_KEY_LEN], TweakError> {
    let scalar: NonZeroScalar = Option::from(NonZeroScalar::from_repr(FieldBytes::from(*secret)))
        .ok_or(TweakError::InvalidSecretKey)?;
    encode_point(&CurvePoint::from_secret_scalar(&scalar))
}

/// Menghitung `(secret + tweak) mod n` untuk CKDpriv BIP32.
///
/// # Errors
///
/// [`TweakError::InvalidSecretKey`] bila `secret` tidak sah, [`TweakError::InvalidTweak`] bila `tweak` lebih
/// besar/sama dengan orde kurva, dan [`TweakError::InvalidResult`] bila hasilnya nol.
pub fn secret_tweak_add(secret: &[u8; 32], tweak: &[u8; 32]) -> Result<[u8; 32], TweakError> {
    let key = parse_secret(secret).ok_or(TweakError::InvalidSecretKey)?;
    let shift = parse_scalar(tweak).ok_or(TweakError::InvalidTweak)?;
    let sum = key.add(&shift);
    if bool::from(sum.is_zero()) {
        return Err(TweakError::InvalidResult);
    }
    Ok(sum.to_bytes().into())
}

/// Menghitung titik `point + tweak·G` untuk CKDpub BIP32, dalam encoding SEC1 terkompresi.
///
/// `tweak` nol diperbolehkan (hasilnya titik yang sama), sesuai BIP32.
///
/// # Errors
///
/// [`TweakError::InvalidPublicKey`] bila `point` tidak sah, [`TweakError::InvalidTweak`] bila `tweak` lebih
/// besar/sama dengan orde kurva, dan [`TweakError::InvalidResult`] bila hasilnya titik tak hingga.
pub fn public_tweak_add(
    point: &[u8; COMPRESSED_PUBLIC_KEY_LEN],
    tweak: &[u8; 32],
) -> Result<[u8; COMPRESSED_PUBLIC_KEY_LEN], TweakError> {
    let base = parse_point(point).ok_or(TweakError::InvalidPublicKey)?;
    let shift = parse_scalar(tweak).ok_or(TweakError::InvalidTweak)?;
    let shift_point: ProjectivePoint = core::ops::Mul::mul(ProjectivePoint::GENERATOR, &shift);
    let sum: ProjectivePoint = core::ops::Add::add(shift_point, base.to_projective());
    let result = CurvePoint::from_affine(sum.to_affine()).map_err(|_| TweakError::InvalidResult)?;
    encode_point(&result)
}

/// Memeriksa apakah 33 byte adalah encoding SEC1 terkompresi (awalan `02`/`03`) dari titik sah pada kurva `secp256k1`.
#[must_use]
pub fn is_valid_public_key(point: &[u8; COMPRESSED_PUBLIC_KEY_LEN]) -> bool {
    parse_point(point).is_some()
}

#[cfg(test)]
mod tests;
