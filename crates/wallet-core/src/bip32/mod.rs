//! Turunan kunci BIP32 (Spesifikasi Wallet v0.1 Bagian 2): kunci induk, CKDpriv, CKDpub, jalur
//! `m/44'/coin'/0'/branch/index`, dan kunci akun publik untuk watch-only (MOD-10).
//!
//! HMAC-SHA512 memakai `hmac` dan `sha2` (DEC-10 bagian 4). Penjumlahan skalar dan titik secp256k1 memakai MOD-02.

use core::fmt;

use hmac::{Hmac, Mac};
use homeostasis_crypto::{
    COMPRESSED_PUBLIC_KEY_LEN, is_valid_public_key, public_key_compressed, public_tweak_add,
    secret_tweak_add,
};
use homeostasis_primitives::Network;
use sha2::Sha512;
use zeroize::Zeroize;

use crate::mnemonic::Seed;

type HmacSha512 = Hmac<Sha512>;

/// Bit penanda indeks hardened BIP32 (`2^31`).
pub const HARDENED: u32 = 0x8000_0000;

/// Tujuan jalur `44'` (hanya struktur jalur; HSIS tidak punya skrip).
pub const PURPOSE: u32 = 44;

/// Nomor `coin` untuk regtest dan testnet (konvensi SLIP-44 "testnet semua koin").
pub const TESTNET_COIN_TYPE: u32 = 1;

// TODO(param): nomor SLIP-44 mainnet untuk jalur turunan wallet (belum didaftarkan).
const MAINNET_COIN_TYPE: Option<u32> = None;

/// Nomor akun v0.1 (`0`).
pub const ACCOUNT: u32 = 0;

/// Cabang alamat penerimaan (`0`).
pub const RECEIVE_BRANCH: u32 = 0;

/// Cabang alamat kembalian (`1`).
pub const CHANGE_BRANCH: u32 = 1;

/// Panjang kunci akun publik dalam hex huruf kecil (33 byte kunci ‖ 32 byte chain code = `130` karakter).
pub const ACCOUNT_KEY_HEX_LEN: usize = 130;

const MASTER_HMAC_KEY: &[u8] = b"Bitcoin seed";
const MIN_SEED_LEN: usize = 16;
const MAX_SEED_LEN: usize = 64;
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Galat turunan kunci BIP32.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Bip32Error {
    /// Panjang seed di luar 16 sampai 64 byte.
    #[error("panjang seed {len} byte tidak sah (harus 16 sampai 64)")]
    InvalidSeedLength {
        /// Panjang seed yang diberikan.
        len: usize,
    },
    /// Seed menghasilkan kunci induk tidak sah (peluang sekitar 2^-127).
    #[error("seed menghasilkan kunci induk tidak sah")]
    InvalidMasterKey,
    /// Turunan indeks ini menghasilkan kunci tidak sah (peluang sekitar 2^-127). Sesuai BIP32, lewati indeks ini.
    #[error("turunan indeks {index} menghasilkan kunci tidak sah; lewati indeks ini")]
    InvalidChild {
        /// Indeks anak yang diminta.
        index: u32,
    },
    /// Indeks hardened tidak bisa diturunkan dari kunci publik.
    #[error("indeks hardened {index} tidak bisa diturunkan dari kunci publik")]
    HardenedFromPublic {
        /// Indeks anak yang diminta.
        index: u32,
    },
    /// Kunci publik bukan titik sah pada kurva `secp256k1`.
    #[error("kunci publik tidak sah")]
    InvalidPublicKey,
    /// Teks kunci akun publik bukan 130 karakter hex huruf kecil.
    #[error("kunci akun publik harus 130 karakter hex huruf kecil")]
    InvalidAccountKey,
    /// Nomor SLIP-44 mainnet belum ditetapkan.
    #[error("nomor SLIP-44 mainnet belum ditetapkan")]
    MainnetCoinTypeUnset,
}

/// Menghitung HMAC-SHA512 dengan kunci `key` atas gabungan semua potongan `data`, lalu membaginya menjadi dua bagian
/// 32 byte (`IL`, `IR`).
fn hmac_halves(key: &[u8], data: &[&[u8]]) -> Option<([u8; 32], [u8; 32])> {
    let mut mac = HmacSha512::new_from_slice(key).ok()?;
    for part in data {
        mac.update(part);
    }
    let output = mac.finalize().into_bytes();
    // Lewat `Deref` ke slice: metode bawaan `GenericArray` (generic-array 0.14) ditandai usang.
    let bytes: &[u8] = &output;
    let (left, right) = bytes.split_first_chunk::<32>()?;
    let right: [u8; 32] = right.try_into().ok()?;
    Some((*left, right))
}

/// Kunci rahasia BIP32 beserta chain code.
///
/// [`fmt::Debug`] tidak pernah mencetak kunci rahasia maupun chain code.
#[derive(Clone, PartialEq, Eq)]
pub struct ExtendedPrivKey {
    secret: [u8; 32],
    chain_code: [u8; 32],
}

impl ExtendedPrivKey {
    /// Kunci induk BIP32: `HMAC-SHA512(kunci = "Bitcoin seed", seed)`.
    ///
    /// # Errors
    ///
    /// [`Bip32Error::InvalidSeedLength`] bila seed di luar 16 sampai 64 byte, dan [`Bip32Error::InvalidMasterKey`]
    /// bila kunci yang dihasilkan nol atau lebih besar/sama dengan orde kurva.
    pub fn master(seed: &[u8]) -> Result<Self, Bip32Error> {
        let len = seed.len();
        if !(MIN_SEED_LEN..=MAX_SEED_LEN).contains(&len) {
            return Err(Bip32Error::InvalidSeedLength { len });
        }
        let (secret, chain_code) =
            hmac_halves(MASTER_HMAC_KEY, &[seed]).ok_or(Bip32Error::InvalidMasterKey)?;
        public_key_compressed(&secret).map_err(|_| Bip32Error::InvalidMasterKey)?;
        Ok(Self { secret, chain_code })
    }

    /// CKDpriv BIP32: menurunkan kunci anak `index` (hardened jika `index >= 2^31`).
    ///
    /// # Errors
    ///
    /// [`Bip32Error::InvalidChild`] bila hasilnya kunci tidak sah; pemanggil melewati indeks itu.
    pub fn derive_child(&self, index: u32) -> Result<Self, Bip32Error> {
        let invalid = Bip32Error::InvalidChild { index };
        let index_bytes = index.to_be_bytes();
        let halves = if index >= HARDENED {
            hmac_halves(
                &self.chain_code,
                &[
                    [0_u8].as_slice(),
                    self.secret.as_slice(),
                    index_bytes.as_slice(),
                ],
            )
        } else {
            let public = public_key_compressed(&self.secret).map_err(|_| invalid)?;
            hmac_halves(
                &self.chain_code,
                &[public.as_slice(), index_bytes.as_slice()],
            )
        };
        let (tweak, chain_code) = halves.ok_or(invalid)?;
        let secret = secret_tweak_add(&self.secret, &tweak).map_err(|_| invalid)?;
        Ok(Self { secret, chain_code })
    }

    /// Menurunkan kunci di sepanjang `path` secara berurutan.
    ///
    /// # Errors
    ///
    /// [`Bip32Error::InvalidChild`] dari langkah pertama yang gagal.
    pub fn derive_path(&self, path: &[u32]) -> Result<Self, Bip32Error> {
        let mut key = self.clone();
        for &index in path {
            key = key.derive_child(index)?;
        }
        Ok(key)
    }

    /// Kunci publik BIP32 yang berpadanan (tanpa kunci rahasia).
    ///
    /// # Errors
    ///
    /// [`Bip32Error::InvalidPublicKey`] bila kunci rahasia tidak sah (tidak terjadi pada kunci hasil turunan).
    pub fn to_public(&self) -> Result<ExtendedPubKey, Bip32Error> {
        let key = public_key_compressed(&self.secret).map_err(|_| Bip32Error::InvalidPublicKey)?;
        Ok(ExtendedPubKey {
            key,
            chain_code: self.chain_code,
        })
    }

    /// 32 byte kunci rahasia (skalar big-endian), dipakai langsung untuk tanda tangan BIP340.
    #[must_use]
    pub const fn secret_bytes(&self) -> &[u8; 32] {
        &self.secret
    }

    /// 32 byte chain code.
    #[must_use]
    pub const fn chain_code(&self) -> &[u8; 32] {
        &self.chain_code
    }
}

impl fmt::Debug for ExtendedPrivKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExtendedPrivKey(<rahasia>)")
    }
}

/// Kunci rahasia dan chain code dihapus dari memori saat kunci dibuang (DEC-27).
impl Drop for ExtendedPrivKey {
    fn drop(&mut self) {
        self.secret.zeroize();
        self.chain_code.zeroize();
    }
}

/// Kunci publik BIP32 (SEC1 terkompresi 33 byte) beserta chain code, untuk watch-only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtendedPubKey {
    key: [u8; COMPRESSED_PUBLIC_KEY_LEN],
    chain_code: [u8; 32],
}

impl ExtendedPubKey {
    /// Membangun kunci publik BIP32 dari kunci SEC1 terkompresi dan chain code.
    ///
    /// # Errors
    ///
    /// [`Bip32Error::InvalidPublicKey`] bila `key` bukan titik sah pada kurva.
    pub fn from_parts(
        key: [u8; COMPRESSED_PUBLIC_KEY_LEN],
        chain_code: [u8; 32],
    ) -> Result<Self, Bip32Error> {
        if !is_valid_public_key(&key) {
            return Err(Bip32Error::InvalidPublicKey);
        }
        Ok(Self { key, chain_code })
    }

    /// CKDpub BIP32: menurunkan kunci publik anak non-hardened `index`.
    ///
    /// # Errors
    ///
    /// [`Bip32Error::HardenedFromPublic`] bila `index >= 2^31`, dan [`Bip32Error::InvalidChild`] bila hasilnya
    /// tidak sah.
    pub fn derive_child(&self, index: u32) -> Result<Self, Bip32Error> {
        if index >= HARDENED {
            return Err(Bip32Error::HardenedFromPublic { index });
        }
        let invalid = Bip32Error::InvalidChild { index };
        let index_bytes = index.to_be_bytes();
        let (tweak, chain_code) = hmac_halves(
            &self.chain_code,
            &[self.key.as_slice(), index_bytes.as_slice()],
        )
        .ok_or(invalid)?;
        let key = public_tweak_add(&self.key, &tweak).map_err(|_| invalid)?;
        Ok(Self { key, chain_code })
    }

    /// Menurunkan kunci publik di sepanjang `path` non-hardened secara berurutan.
    ///
    /// # Errors
    ///
    /// Galat dari langkah pertama yang gagal.
    pub fn derive_path(&self, path: &[u32]) -> Result<Self, Bip32Error> {
        let mut key = *self;
        for &index in path {
            key = key.derive_child(index)?;
        }
        Ok(key)
    }

    /// 33 byte kunci publik SEC1 terkompresi.
    #[must_use]
    pub const fn public_key(&self) -> &[u8; COMPRESSED_PUBLIC_KEY_LEN] {
        &self.key
    }

    /// 32 byte chain code.
    #[must_use]
    pub const fn chain_code(&self) -> &[u8; 32] {
        &self.chain_code
    }

    /// Kunci x-only 32 byte untuk payload output jenis 0 (byte awalan `02`/`03` dibuang).
    #[must_use]
    pub const fn x_only(&self) -> [u8; 32] {
        let [_prefix, x @ ..] = self.key;
        x
    }

    /// Kunci akun publik untuk watch-only: hex huruf kecil 130 karakter (33 byte kunci ‖ 32 byte chain code).
    #[must_use]
    pub fn to_account_hex(&self) -> String {
        let mut out = String::with_capacity(ACCOUNT_KEY_HEX_LEN);
        for &byte in self.key.iter().chain(self.chain_code.iter()) {
            for nibble in [byte.wrapping_shr(4), byte & 0x0f] {
                if let Some(&digit) = HEX_DIGITS.get(usize::from(nibble)) {
                    out.push(char::from(digit));
                }
            }
        }
        out
    }

    /// Mengurai kunci akun publik hex huruf kecil 130 karakter.
    ///
    /// # Errors
    ///
    /// [`Bip32Error::InvalidAccountKey`] bila panjang atau karakter tidak sah, dan [`Bip32Error::InvalidPublicKey`]
    /// bila 33 byte pertama bukan titik sah pada kurva.
    pub fn from_account_hex(text: &str) -> Result<Self, Bip32Error> {
        if text.len() != ACCOUNT_KEY_HEX_LEN {
            return Err(Bip32Error::InvalidAccountKey);
        }
        let (pairs, rest) = text.as_bytes().as_chunks::<2>();
        if !rest.is_empty() {
            return Err(Bip32Error::InvalidAccountKey);
        }
        let mut bytes = Vec::with_capacity(pairs.len());
        for &[high, low] in pairs {
            let high = lower_hex_value(high).ok_or(Bip32Error::InvalidAccountKey)?;
            let low = lower_hex_value(low).ok_or(Bip32Error::InvalidAccountKey)?;
            bytes.push(high.wrapping_shl(4) | low);
        }
        let (key, chain_code) = bytes
            .split_first_chunk::<COMPRESSED_PUBLIC_KEY_LEN>()
            .ok_or(Bip32Error::InvalidAccountKey)?;
        let chain_code: [u8; 32] = chain_code
            .try_into()
            .map_err(|_| Bip32Error::InvalidAccountKey)?;
        Self::from_parts(*key, chain_code)
    }
}

fn lower_hex_value(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => digit.checked_sub(b'0'),
        b'a'..=b'f' => digit.checked_sub(b'a')?.checked_add(10),
        _ => None,
    }
}

/// Nomor `coin` jalur turunan untuk jaringan ini: `1` untuk regtest dan testnet.
///
/// # Errors
///
/// [`Bip32Error::MainnetCoinTypeUnset`] untuk mainnet, sampai nomor SLIP-44 HSIS ditetapkan.
pub fn coin_type(network: Network) -> Result<u32, Bip32Error> {
    match network {
        Network::Testnet | Network::Regtest => Ok(TESTNET_COIN_TYPE),
        Network::Mainnet => MAINNET_COIN_TYPE.ok_or(Bip32Error::MainnetCoinTypeUnset),
    }
}

/// Jalur akun `m/44'/coin'/0'` untuk jaringan ini.
///
/// # Errors
///
/// [`Bip32Error::MainnetCoinTypeUnset`] untuk mainnet.
pub fn account_path(network: Network) -> Result<[u32; 3], Bip32Error> {
    let coin = coin_type(network)?;
    Ok([hardened(PURPOSE), hardened(coin), hardened(ACCOUNT)])
}

/// Indeks hardened `index'` (`index | 2^31`).
#[must_use]
pub const fn hardened(index: u32) -> u32 {
    index | HARDENED
}

/// Kunci akun `m/44'/coin'/0'` dari seed BIP39.
///
/// Alamat diturunkan dari kunci ini dengan jalur `branch/index` (non-hardened), baik dari kunci rahasia maupun dari
/// kunci publiknya (watch-only).
///
/// # Errors
///
/// [`Bip32Error::MainnetCoinTypeUnset`] untuk mainnet, atau galat turunan BIP32.
pub fn account_key(seed: &Seed, network: Network) -> Result<ExtendedPrivKey, Bip32Error> {
    ExtendedPrivKey::master(seed.as_bytes())?.derive_path(&account_path(network)?)
}

#[cfg(test)]
mod tests;
