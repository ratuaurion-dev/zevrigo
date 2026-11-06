//! Alamat bech32m (BIP350): `bech32m(HRP, [kind] ‖ convertbits(payload, 8→5))` (DEC-11, Spesifikasi Format v0.1
//! Bagian 8). HRP `zvrg` (mainnet), `tzvrg` (testnet), `rzvrg` (regtest).
//!
//! Alamat bukan bagian konsensus: chain hanya menyimpan `kind` dan `payload` output.

use zevrigo_primitives::{Network, OutputKind};

/// Panjang payload output (`32` byte).
pub const PAYLOAD_LEN: usize = 32;

const CHARSET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
const BECH32M_CONST: u32 = 0x2bc8_30a3;
const GENERATORS: [u32; 5] = [
    0x3b6a_57b2,
    0x2650_8e6d,
    0x1ea1_19fa,
    0x3d42_33dd,
    0x2a14_62b3,
];
const CHECKSUM_LEN: usize = 6;
/// 256 bit payload = 52 kelompok 5 bit (4 bit sisa bernilai nol).
const PAYLOAD_GROUPS: usize = 52;
/// Bagian data setelah pemisah `1`: 1 kelompok `kind` + 52 kelompok payload + 6 kelompok checksum.
const DATA_LEN: usize = 59;

/// Galat penguraian alamat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AddressError {
    /// Alamat memakai huruf besar dan huruf kecil sekaligus.
    #[error("alamat memakai huruf besar dan kecil sekaligus")]
    MixedCase,
    /// Alamat tidak punya pemisah `1`.
    #[error("alamat tidak punya pemisah 1")]
    MissingSeparator,
    /// HRP alamat bukan milik jaringan ini.
    #[error("awalan alamat bukan milik jaringan ini")]
    WrongNetwork,
    /// Panjang bagian data bukan 59 karakter.
    #[error("panjang alamat tidak sah")]
    InvalidLength,
    /// Ada karakter di luar himpunan karakter bech32.
    #[error("karakter tidak sah di alamat")]
    InvalidCharacter,
    /// Checksum bech32m tidak cocok.
    #[error("checksum bech32m tidak cocok")]
    InvalidChecksum,
    /// Jenis output bukan 0 atau 1.
    #[error("jenis output {kind} tidak dikenal")]
    UnknownKind {
        /// Nilai `kind` di alamat.
        kind: u8,
    },
    /// Bit sisa payload tidak nol.
    #[error("bit sisa payload tidak nol")]
    InvalidPadding,
}

fn low_mask(bits: u32) -> u32 {
    1_u32.wrapping_shl(bits).wrapping_sub(1)
}

fn low_byte(value: u32) -> u8 {
    let [byte, ..] = value.to_le_bytes();
    byte
}

fn polymod(values: impl Iterator<Item = u8>) -> u32 {
    let mut chk: u32 = 1;
    for value in values {
        let mut top = chk.wrapping_shr(25);
        chk = (chk & 0x01ff_ffff).wrapping_shl(5) ^ u32::from(value);
        for generator in GENERATORS {
            if top & 1 == 1 {
                chk ^= generator;
            }
            top = top.wrapping_shr(1);
        }
    }
    chk
}

/// HRP yang diperluas (`hrp >> 5`, `0`, `hrp & 31`) diikuti `data`, masukan polymod BIP173/BIP350.
fn checksum_input<'a>(hrp: &'a str, data: &'a [u8]) -> impl Iterator<Item = u8> + 'a {
    hrp.bytes()
        .map(|byte| byte.wrapping_shr(5))
        .chain(core::iter::once(0))
        .chain(hrp.bytes().map(|byte| byte & 0x1f))
        .chain(data.iter().copied())
}

fn create_checksum(hrp: &str, data: &[u8]) -> [u8; CHECKSUM_LEN] {
    let mut value = polymod(checksum_input(hrp, data).chain([0_u8; CHECKSUM_LEN])) ^ BECH32M_CONST;
    let mut out = [0_u8; CHECKSUM_LEN];
    for slot in out.iter_mut().rev() {
        *slot = low_byte(value & 0x1f);
        value = value.wrapping_shr(5);
    }
    out
}

fn verify_checksum(hrp: &str, data: &[u8]) -> bool {
    polymod(checksum_input(hrp, data)) == BECH32M_CONST
}

/// convertbits 8 → 5 dengan padding nol.
fn to_groups(payload: &[u8; PAYLOAD_LEN]) -> Vec<u8> {
    let mut out = Vec::with_capacity(PAYLOAD_GROUPS);
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    for &byte in payload {
        // bits < 5 sebelum ditambah 8 bit, jadi cukup 12 bit terakhir yang disimpan.
        acc = (acc.wrapping_shl(8) | u32::from(byte)) & 0x0fff;
        bits = bits.wrapping_add(8);
        while bits >= 5 {
            bits = bits.wrapping_sub(5);
            out.push(low_byte(acc.wrapping_shr(bits) & 0x1f));
        }
    }
    if bits > 0 {
        out.push(low_byte(acc.wrapping_shl(5_u32.wrapping_sub(bits)) & 0x1f));
    }
    out
}

/// convertbits 5 → 8 tanpa padding: bit sisa wajib kurang dari 5 dan bernilai nol.
fn from_groups(groups: &[u8]) -> Result<[u8; PAYLOAD_LEN], AddressError> {
    let mut out = Vec::with_capacity(PAYLOAD_LEN);
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    for &group in groups {
        // bits < 8 sebelum ditambah 5 bit, jadi cukup 12 bit terakhir yang disimpan.
        acc = (acc.wrapping_shl(5) | u32::from(group)) & 0x0fff;
        bits = bits.wrapping_add(5);
        while bits >= 8 {
            bits = bits.wrapping_sub(8);
            out.push(low_byte(acc.wrapping_shr(bits)));
        }
    }
    if bits >= 5 || acc & low_mask(bits) != 0 {
        return Err(AddressError::InvalidPadding);
    }
    out.try_into().map_err(|_| AddressError::InvalidLength)
}

/// Menyusun alamat bech32m huruf kecil untuk output `kind` dengan `payload` di jaringan `network`.
#[must_use]
pub fn encode_address(network: Network, kind: OutputKind, payload: &[u8; PAYLOAD_LEN]) -> String {
    let hrp = network.hrp();
    let mut data = Vec::with_capacity(DATA_LEN);
    data.push(kind.to_u8());
    data.extend(to_groups(payload));
    let checksum = create_checksum(hrp, &data);
    data.extend_from_slice(&checksum);
    let mut out = String::from(hrp);
    out.push('1');
    for value in data {
        if let Some(&symbol) = CHARSET.get(usize::from(value)) {
            out.push(char::from(symbol));
        }
    }
    out
}

/// Mengurai alamat bech32m untuk jaringan `network` menjadi `(kind, payload)`.
///
/// Aturan: huruf kecil semua atau huruf besar semua (tidak campuran), HRP wajib milik `network`, checksum bech32m
/// wajib cocok, `kind` wajib 0 atau 1, payload tepat 32 byte, dan bit sisa wajib nol.
///
/// # Errors
///
/// [`AddressError`] sesuai aturan yang dilanggar.
pub fn decode_address(
    network: Network,
    address: &str,
) -> Result<(OutputKind, [u8; PAYLOAD_LEN]), AddressError> {
    let has_lower = address.bytes().any(|byte| byte.is_ascii_lowercase());
    let has_upper = address.bytes().any(|byte| byte.is_ascii_uppercase());
    if has_lower && has_upper {
        return Err(AddressError::MixedCase);
    }
    let lower = address.to_ascii_lowercase();
    let (hrp, data_part) = lower
        .rsplit_once('1')
        .ok_or(AddressError::MissingSeparator)?;
    if hrp != network.hrp() {
        return Err(AddressError::WrongNetwork);
    }
    if data_part.len() != DATA_LEN {
        return Err(AddressError::InvalidLength);
    }
    let mut data = Vec::with_capacity(DATA_LEN);
    for symbol in data_part.bytes() {
        let value = CHARSET
            .iter()
            .position(|&candidate| candidate == symbol)
            .ok_or(AddressError::InvalidCharacter)?;
        data.push(u8::try_from(value).map_err(|_| AddressError::InvalidCharacter)?);
    }
    if !verify_checksum(hrp, &data) {
        return Err(AddressError::InvalidChecksum);
    }
    let (&kind_value, rest) = data.split_first().ok_or(AddressError::InvalidLength)?;
    let groups = rest
        .get(..PAYLOAD_GROUPS)
        .ok_or(AddressError::InvalidLength)?;
    let kind = OutputKind::from_u8(kind_value)
        .map_err(|_| AddressError::UnknownKind { kind: kind_value })?;
    let payload = from_groups(groups)?;
    Ok((kind, payload))
}

#[cfg(test)]
mod tests;
