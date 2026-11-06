//! Encoding nilai JSON RPC sesuai Spesifikasi RPC v0.1 Bagian 2.
//!
//! Setiap tipe di sini membungkus satu nilai dan di-serialize sebagai string atau angka JSON dengan aturan ketat:
//! hex wajib huruf kecil dengan panjang tepat; jumlah koin dan nonce wajib string desimal kanonis (tanpa nol di
//! depan, tanpa tanda, tanpa titik). Nilai yang melanggar ditolak saat deserialisasi, dan server memetakannya ke
//! kode `-32602`. Jumlah koin tidak pernah ditulis sebagai angka JSON.

use serde::{Deserialize, Serialize, Serializer};
use thiserror::Error;
use zevrigo_primitives::amount::Amount;
use zevrigo_primitives::hash::Hash32;
use zevrigo_primitives::params::Network;
use zevrigo_primitives::tx::OutputKind;

/// Galat saat mengurai nilai JSON RPC.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ValueError {
    /// Panjang string hex tidak sesuai jenis nilainya.
    #[error("panjang hex harus {expected} karakter, ditemukan {actual}")]
    HexLength {
        /// Panjang yang diwajibkan.
        expected: usize,
        /// Panjang input dalam byte.
        actual: usize,
    },
    /// Panjang string hex ganjil.
    #[error("panjang hex harus genap, ditemukan {actual}")]
    HexOddLength {
        /// Panjang input dalam byte.
        actual: usize,
    },
    /// Karakter di luar `0-9` dan `a-f` (huruf besar ditolak).
    #[error("karakter hex tidak sah pada indeks byte {index}")]
    HexChar {
        /// Indeks byte berbasis nol.
        index: usize,
    },
    /// Angka desimal kosong, memakai nol di depan, tanda, titik, spasi, atau karakter bukan digit.
    #[error("angka desimal tidak kanonis")]
    Decimal,
    /// Angka desimal melebihi batas tipenya.
    #[error("angka desimal melebihi batas")]
    DecimalOverflow,
    /// Nama jaringan bukan `mainnet`, `testnet`, atau `regtest`.
    #[error("nama jaringan tidak dikenal")]
    UnknownNetwork,
    /// Jenis output bukan `0` atau `1`.
    #[error("jenis output harus 0 atau 1, ditemukan {actual}")]
    UnknownOutputKind {
        /// Nilai yang ditemukan.
        actual: u8,
    },
    /// Jumlah payload `scan_utxos` di luar 1 sampai 64.
    #[error("jumlah payload harus 1 sampai 64, ditemukan {actual}")]
    PayloadCount {
        /// Jumlah payload yang diberikan.
        actual: usize,
    },
}

/// Nilai satu digit hex huruf kecil, atau `None` untuk karakter lain.
const fn lower_hex_value(ch: u8) -> Option<u8> {
    match ch {
        b'0'..=b'9' => Some(ch.wrapping_sub(b'0')),
        b'a'..=b'f' => Some(ch.wrapping_sub(b'a').wrapping_add(10)),
        _ => None,
    }
}

/// Mengurai string hex huruf kecil berpanjang genap (boleh kosong) menjadi byte.
///
/// # Errors
///
/// [`ValueError::HexOddLength`] untuk panjang ganjil, [`ValueError::HexChar`] untuk karakter selain `0-9`/`a-f`.
pub fn decode_hex(s: &str) -> Result<Vec<u8>, ValueError> {
    let input = s.as_bytes();
    if !input.len().is_multiple_of(2) {
        return Err(ValueError::HexOddLength {
            actual: input.len(),
        });
    }
    let mut out = Vec::with_capacity(input.len() / 2);
    let mut high: Option<u8> = None;
    for (index, &ch) in input.iter().enumerate() {
        let nibble = lower_hex_value(ch).ok_or(ValueError::HexChar { index })?;
        match high.take() {
            None => high = Some(nibble),
            Some(h) => out.push((h << 4) | nibble),
        }
    }
    Ok(out)
}

/// Menulis byte sebagai string hex huruf kecil.
#[must_use]
pub fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for &byte in bytes {
        for nibble in [byte >> 4, byte & 0x0f] {
            out.push(char::from_digit(u32::from(nibble), 16).unwrap_or('0'));
        }
    }
    out
}

/// Mengurai hex huruf kecil dengan panjang tepat `2 × N` karakter.
fn decode_fixed<const N: usize>(s: &str) -> Result<[u8; N], ValueError> {
    let expected = N.saturating_mul(2);
    let length_error = ValueError::HexLength {
        expected,
        actual: s.len(),
    };
    if s.len() != expected {
        return Err(length_error);
    }
    let bytes = decode_hex(s)?;
    <[u8; N]>::try_from(bytes).map_err(|_| length_error)
}

/// Mengurai angka desimal kanonis: hanya digit ASCII, tanpa nol di depan kecuali tepat `0`.
fn parse_decimal(s: &str) -> Result<u128, ValueError> {
    let input = s.as_bytes();
    let Some(&first) = input.first() else {
        return Err(ValueError::Decimal);
    };
    if first == b'0' && input.len() > 1 {
        return Err(ValueError::Decimal);
    }
    let mut value: u128 = 0;
    for &ch in input {
        if !ch.is_ascii_digit() {
            return Err(ValueError::Decimal);
        }
        let digit = u128::from(ch.wrapping_sub(b'0'));
        value = value
            .checked_mul(10)
            .and_then(|v| v.checked_add(digit))
            .ok_or(ValueError::DecimalOverflow)?;
    }
    Ok(value)
}

/// Data 32 byte sebagai hex huruf kecil tepat 64 karakter: txid, block_id, Merkle root, payload, dan chainwork
/// (256-bit big-endian).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Hex32([u8; 32]);

impl Hex32 {
    /// Membungkus 32 byte.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Mengembalikan 32 byte di dalamnya.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Mengembalikan nilai sebagai [`Hash32`].
    #[must_use]
    pub const fn to_hash32(self) -> Hash32 {
        Hash32::from_bytes(self.0)
    }
}

impl From<Hash32> for Hex32 {
    fn from(hash: Hash32) -> Self {
        Self(*hash.as_bytes())
    }
}

impl TryFrom<&str> for Hex32 {
    type Error = ValueError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        decode_fixed::<32>(s).map(Self)
    }
}

impl TryFrom<String> for Hex32 {
    type Error = ValueError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl From<Hex32> for String {
    fn from(value: Hex32) -> Self {
        encode_hex(&value.0)
    }
}

/// Data biner (encoding kanonis blok atau transaksi) sebagai hex huruf kecil berpanjang genap.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct HexBytes(Vec<u8>);

impl HexBytes {
    /// Membungkus byte.
    #[must_use]
    pub const fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Mengembalikan byte di dalamnya.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Mengambil byte di dalamnya.
    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl TryFrom<&str> for HexBytes {
    type Error = ValueError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        decode_hex(s).map(Self)
    }
}

impl TryFrom<String> for HexBytes {
    type Error = ValueError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl Serialize for HexBytes {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&encode_hex(&self.0))
    }
}

/// Jumlah koin sebagai string desimal dalam quanta, misalnya `"1910127426536917"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Quanta(Amount);

impl Quanta {
    /// Membungkus jumlah koin.
    #[must_use]
    pub const fn new(amount: Amount) -> Self {
        Self(amount)
    }

    /// Mengembalikan jumlah koin.
    #[must_use]
    pub const fn amount(self) -> Amount {
        self.0
    }
}

impl From<Amount> for Quanta {
    fn from(amount: Amount) -> Self {
        Self(amount)
    }
}

impl TryFrom<&str> for Quanta {
    type Error = ValueError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        parse_decimal(s).map(|q| Self(Amount::from_quanta(q)))
    }
}

impl TryFrom<String> for Quanta {
    type Error = ValueError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl From<Quanta> for String {
    fn from(value: Quanta) -> Self {
        value.0.as_quanta().to_string()
    }
}

/// Bilangan `u64` penuh sebagai string desimal kanonis (dipakai untuk nonce).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DecimalU64(u64);

impl DecimalU64 {
    /// Membungkus bilangan.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Mengembalikan bilangan.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl TryFrom<&str> for DecimalU64 {
    type Error = ValueError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let value = parse_decimal(s)?;
        u64::try_from(value)
            .map(Self)
            .map_err(|_| ValueError::DecimalOverflow)
    }
}

impl TryFrom<String> for DecimalU64 {
    type Error = ValueError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl From<DecimalU64> for String {
    fn from(value: DecimalU64) -> Self {
        value.0.to_string()
    }
}

/// Target compact `u32` sebagai hex huruf kecil tepat 8 karakter, misalnya `"207fffff"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CompactHex(u32);

impl CompactHex {
    /// Membungkus target compact.
    #[must_use]
    pub const fn new(bits: u32) -> Self {
        Self(bits)
    }

    /// Mengembalikan target compact.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl TryFrom<&str> for CompactHex {
    type Error = ValueError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        decode_fixed::<4>(s).map(|bytes| Self(u32::from_be_bytes(bytes)))
    }
}

impl TryFrom<String> for CompactHex {
    type Error = ValueError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl From<CompactHex> for String {
    fn from(value: CompactHex) -> Self {
        encode_hex(&value.0.to_be_bytes())
    }
}

/// Jaringan sebagai string `"mainnet"`, `"testnet"`, atau `"regtest"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct NetworkName(Network);

impl NetworkName {
    /// Membungkus jaringan.
    #[must_use]
    pub const fn new(network: Network) -> Self {
        Self(network)
    }

    /// Mengembalikan jaringan.
    #[must_use]
    pub const fn network(self) -> Network {
        self.0
    }
}

impl TryFrom<&str> for NetworkName {
    type Error = ValueError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "mainnet" => Ok(Self(Network::Mainnet)),
            "testnet" => Ok(Self(Network::Testnet)),
            "regtest" => Ok(Self(Network::Regtest)),
            _ => Err(ValueError::UnknownNetwork),
        }
    }
}

impl TryFrom<String> for NetworkName {
    type Error = ValueError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl From<NetworkName> for String {
    fn from(value: NetworkName) -> Self {
        let name = match value.0 {
            Network::Mainnet => "mainnet",
            Network::Testnet => "testnet",
            Network::Regtest => "regtest",
        };
        name.to_owned()
    }
}

/// Jenis output sebagai angka JSON `0` atau `1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct KindNumber(OutputKind);

impl KindNumber {
    /// Membungkus jenis output.
    #[must_use]
    pub const fn new(kind: OutputKind) -> Self {
        Self(kind)
    }

    /// Mengembalikan jenis output.
    #[must_use]
    pub const fn kind(self) -> OutputKind {
        self.0
    }
}

impl TryFrom<u8> for KindNumber {
    type Error = ValueError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        OutputKind::from_u8(value)
            .map(Self)
            .map_err(|_| ValueError::UnknownOutputKind { actual: value })
    }
}

impl From<KindNumber> for u8 {
    fn from(value: KindNumber) -> Self {
        value.0.to_u8()
    }
}

#[cfg(test)]
mod tests;
