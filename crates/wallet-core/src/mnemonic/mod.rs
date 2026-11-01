//! Frasa pemulihan BIP39 dan seed (DEC-22, Spesifikasi Wallet v0.1 Bagian 1).
//!
//! Daftar kata Inggris resmi BIP39 disimpan di `english.txt` (2_048 kata urut, sha256 `2f5eed53…24dbda`). Entropi
//! wallet baru diberikan pemanggil (CLI v0.1: `/dev/urandom`), sehingga modul ini tetap deterministik.

use core::fmt;

use sha2::{Digest, Sha256, Sha512};

const ENGLISH: &str = include_str!("english.txt");

/// Jumlah kata di daftar kata BIP39 (`2_048`).
pub const WORDLIST_LEN: usize = 2_048;

/// Jumlah iterasi PBKDF2-HMAC-SHA512 untuk seed BIP39 (`2_048`).
pub const SEED_ITERATIONS: u32 = 2_048;

/// Panjang seed BIP39 (`64` byte).
pub const SEED_LEN: usize = 64;

/// Panjang entropi wallet baru (`32` byte, 24 kata).
pub const NEW_WALLET_ENTROPY_LEN: usize = 32;

const SALT_PREFIX: &str = "mnemonic";
const BITS_PER_WORD: u32 = 11;
const WORD_MASK: u32 = 0x07ff;

/// Galat frasa pemulihan BIP39.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum MnemonicError {
    /// Panjang entropi bukan 16, 20, 24, 28, atau 32 byte.
    #[error("panjang entropi {len} byte tidak sah (harus 16, 20, 24, 28, atau 32)")]
    InvalidEntropyLength {
        /// Panjang entropi yang diberikan.
        len: usize,
    },
    /// Jumlah kata bukan 12, 15, 18, 21, atau 24.
    #[error("jumlah kata {count} tidak sah (harus 12, 15, 18, 21, atau 24)")]
    InvalidWordCount {
        /// Jumlah kata setelah normalisasi.
        count: usize,
    },
    /// Sebuah kata tidak ada di daftar kata BIP39.
    #[error("kata ke-{position} tidak ada di daftar kata BIP39")]
    UnknownWord {
        /// Urutan kata, mulai dari 1.
        position: usize,
    },
    /// Checksum BIP39 tidak cocok.
    #[error("checksum frasa BIP39 tidak cocok")]
    BadChecksum,
    /// Passphrase berisi karakter di luar ASCII cetak (`0x20` sampai `0x7e`).
    #[error("passphrase hanya boleh berisi ASCII cetak (0x20 sampai 0x7e)")]
    InvalidPassphrase,
    /// Daftar kata bawaan rusak. Tidak terjadi pada build yang sah; dijaga oleh test daftar kata.
    #[error("daftar kata bawaan tidak sah")]
    Wordlist,
}

fn words() -> Vec<&'static str> {
    ENGLISH.lines().collect()
}

fn low_mask(bits: u32) -> u32 {
    1_u32.wrapping_shl(bits).wrapping_sub(1)
}

fn checksum_bits(entropy_len: usize) -> Option<u32> {
    match entropy_len {
        16 => Some(4),
        20 => Some(5),
        24 => Some(6),
        28 => Some(7),
        32 => Some(8),
        _ => None,
    }
}

fn entropy_len_for(word_count: usize) -> Option<usize> {
    match word_count {
        12 => Some(16),
        15 => Some(20),
        18 => Some(24),
        21 => Some(28),
        24 => Some(32),
        _ => None,
    }
}

/// Frasa pemulihan BIP39 yang sah: entropi beserta kalimat kata yang dinormalkan (huruf kecil, satu spasi).
///
/// [`fmt::Debug`] tidak pernah mencetak kata maupun entropi.
#[derive(Clone, PartialEq, Eq)]
pub struct Mnemonic {
    entropy: Vec<u8>,
    phrase: String,
}

impl Mnemonic {
    /// Menyusun frasa dari entropi 16, 20, 24, 28, atau 32 byte (12 sampai 24 kata).
    ///
    /// # Errors
    ///
    /// [`MnemonicError::InvalidEntropyLength`] untuk panjang lain.
    pub fn from_entropy(entropy: &[u8]) -> Result<Self, MnemonicError> {
        let len = entropy.len();
        let cs_bits = checksum_bits(len).ok_or(MnemonicError::InvalidEntropyLength { len })?;
        let digest = Sha256::digest(entropy);
        let first = digest.first().copied().ok_or(MnemonicError::Wordlist)?;
        let checksum = u32::from(first).wrapping_shr(8_u32.wrapping_sub(cs_bits));

        let list = words();
        let mut chosen: Vec<&'static str> = Vec::with_capacity(24);
        let mut acc: u32 = 0;
        let mut acc_len: u32 = 0;
        let chunks = entropy
            .iter()
            .map(|&byte| (u32::from(byte), 8_u32))
            .chain(core::iter::once((checksum, cs_bits)));
        for (value, count) in chunks {
            // acc_len < 11 sebelum ditambah dan count <= 8, jadi acc tidak pernah lebih dari 18 bit.
            acc = acc.wrapping_shl(count) | value;
            acc_len = acc_len.wrapping_add(count);
            if acc_len >= BITS_PER_WORD {
                acc_len = acc_len.wrapping_sub(BITS_PER_WORD);
                let index = acc.wrapping_shr(acc_len) & WORD_MASK;
                let word = usize::try_from(index)
                    .ok()
                    .and_then(|i| list.get(i).copied())
                    .ok_or(MnemonicError::Wordlist)?;
                chosen.push(word);
                acc &= low_mask(acc_len);
            }
        }
        Ok(Self {
            entropy: entropy.to_vec(),
            phrase: chosen.join(" "),
        })
    }

    /// Mengurai frasa pemulihan 12, 15, 18, 21, atau 24 kata dan memeriksa checksum BIP39.
    ///
    /// Masukan dinormalkan dulu: spasi (termasuk tab dan baris baru) di awal dan akhir dibuang, spasi berurutan
    /// dianggap satu, dan huruf besar ASCII menjadi huruf kecil.
    ///
    /// # Errors
    ///
    /// [`MnemonicError::InvalidWordCount`], [`MnemonicError::UnknownWord`], atau [`MnemonicError::BadChecksum`].
    pub fn parse(phrase: &str) -> Result<Self, MnemonicError> {
        let normalized: Vec<String> = phrase
            .split_whitespace()
            .map(str::to_ascii_lowercase)
            .collect();
        let count = normalized.len();
        let entropy_len =
            entropy_len_for(count).ok_or(MnemonicError::InvalidWordCount { count })?;

        let list = words();
        let mut entropy: Vec<u8> = Vec::with_capacity(entropy_len);
        let mut acc: u32 = 0;
        let mut acc_len: u32 = 0;
        for (offset, word) in normalized.iter().enumerate() {
            let index = list
                .binary_search_by(|probe| probe.as_bytes().cmp(word.as_bytes()))
                .map_err(|_| MnemonicError::UnknownWord {
                    position: offset.saturating_add(1),
                })?;
            let index = u32::try_from(index).map_err(|_| MnemonicError::Wordlist)?;
            // acc_len < 8 sebelum ditambah 11 bit, jadi acc tidak pernah lebih dari 18 bit.
            acc = acc.wrapping_shl(BITS_PER_WORD) | index;
            acc_len = acc_len.wrapping_add(BITS_PER_WORD);
            while acc_len >= 8 {
                acc_len = acc_len.wrapping_sub(8);
                let [byte, ..] = acc.wrapping_shr(acc_len).to_le_bytes();
                if entropy.len() < entropy_len {
                    entropy.push(byte);
                }
                acc &= low_mask(acc_len);
            }
        }

        // Bit sisa (dan byte terakhir untuk 24 kata) adalah checksum: susun ulang frasa dari entropi lalu bandingkan.
        let mnemonic = Self::from_entropy(&entropy)?;
        if mnemonic.phrase != normalized.join(" ") {
            return Err(MnemonicError::BadChecksum);
        }
        Ok(mnemonic)
    }

    /// Kalimat frasa yang dinormalkan: kata huruf kecil dipisah satu spasi.
    #[must_use]
    pub fn phrase(&self) -> &str {
        &self.phrase
    }

    /// Entropi frasa (16 sampai 32 byte).
    #[must_use]
    pub fn entropy(&self) -> &[u8] {
        &self.entropy
    }

    /// Jumlah kata frasa (12, 15, 18, 21, atau 24).
    #[must_use]
    pub fn word_count(&self) -> usize {
        self.phrase.split(' ').count()
    }

    /// Menghitung seed BIP39: `PBKDF2-HMAC-SHA512(frasa, "mnemonic" ‖ passphrase, 2_048 iterasi, 64 byte)`.
    ///
    /// Passphrase boleh kosong. v0.1 hanya menerima ASCII cetak (`0x20` sampai `0x7e`), yang tidak diubah oleh
    /// normalisasi NFKD BIP39.
    ///
    /// # Errors
    ///
    /// [`MnemonicError::InvalidPassphrase`] bila passphrase berisi karakter lain.
    pub fn to_seed(&self, passphrase: &str) -> Result<Seed, MnemonicError> {
        if !passphrase.bytes().all(|byte| (0x20..=0x7e).contains(&byte)) {
            return Err(MnemonicError::InvalidPassphrase);
        }
        let salt = format!("{SALT_PREFIX}{passphrase}");
        let bytes = pbkdf2::pbkdf2_hmac_array::<Sha512, SEED_LEN>(
            self.phrase.as_bytes(),
            salt.as_bytes(),
            SEED_ITERATIONS,
        );
        Ok(Seed(bytes))
    }
}

impl fmt::Debug for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mnemonic(<rahasia>, {} kata)", self.word_count())
    }
}

/// Seed BIP39 64 byte, masukan kunci induk BIP32.
///
/// [`fmt::Debug`] tidak pernah mencetak byte seed.
#[derive(Clone, PartialEq, Eq)]
pub struct Seed([u8; SEED_LEN]);

impl Seed {
    /// Mengembalikan referensi ke 64 byte seed.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; SEED_LEN] {
        &self.0
    }
}

impl fmt::Debug for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Seed(<rahasia>)")
    }
}

#[cfg(test)]
mod tests;
