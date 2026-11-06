//! Pesan P2P v0.1 (Spesifikasi P2P Bagian 3): `Status`, `GetHeaders`, `Headers`, `GetBlocks`, `Blocks`, `Ack`, serta
//! `Block` dan `Transaction` dengan encoding kanonis Spesifikasi Format v0.1 (DEC-12).
//!
//! Setiap pesan punya tepat satu encoding sah. Batas jumlah elemen diperiksa sesudah jumlahnya dibaca dan sebelum
//! elemennya dibaca, dan setiap byte wajib terbaca habis. Pengirim wajib mematuhi batas; [`Wire::to_wire`] tidak
//! memeriksanya, penerima menolak lewat [`Wire::from_wire`].

use thiserror::Error;
use zevrigo_primitives::encoding::{
    write_bytes, write_u8, write_u64_le, write_varint_u64, write_vec,
};
use zevrigo_primitives::{
    Block, BlockHeight, Decode, DecodeError, Encode, HEADER_SIZE, Hash32, Header, Reader,
    Transaction,
};

/// Panjang isi pesan paling besar di dalam bingkai (`2_097_152` byte, 2 MiB).
pub const MAX_MESSAGE_SIZE: usize = 2_097_152;

/// Versi pesan `Status` yang didukung (`1`).
pub const STATUS_VERSION: u8 = 1;

/// Ukuran tetap encoding `Status`: 1 + 32 + 8 + 32 + 32 = 105 byte.
pub const STATUS_SIZE: usize = 105;

/// Jumlah ID locator paling banyak di `GetHeaders` (`64`).
pub const MAX_LOCATOR: usize = 64;

/// Jumlah header paling banyak di `Headers` (`2_000`).
pub const MAX_HEADERS: usize = 2_000;

/// Jumlah ID blok paling banyak di `GetBlocks` dan entri di `Blocks` (`16`).
pub const MAX_BLOCK_IDS: usize = 16;

/// Ukuran satu ID (`bytes32`).
const ID_SIZE: usize = 32;

/// Pesan rusak dari rekan (Spesifikasi P2P Bagian 3 dan 7). Setiap varian adalah pelanggaran yang disanksi.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum MessageError {
    /// Encoding tidak kanonis menurut Spesifikasi Format v0.1 (termasuk data terpotong dan sisa byte).
    #[error("encoding pesan tidak sah: {0}")]
    Decode(#[from] DecodeError),
    /// Jumlah ID locator di `GetHeaders` di luar 1 sampai 64.
    #[error("locator berisi {count} ID (wajib 1 sampai 64)")]
    LocatorCount {
        /// Jumlah ID yang diklaim.
        count: usize,
    },
    /// `Headers` berisi lebih dari 2_000 header.
    #[error("Headers berisi {count} header (paling banyak 2000)")]
    TooManyHeaders {
        /// Jumlah header yang diklaim.
        count: usize,
    },
    /// Jumlah ID di `GetBlocks` di luar 1 sampai 16.
    #[error("GetBlocks berisi {count} ID (wajib 1 sampai 16)")]
    IdCount {
        /// Jumlah ID yang diklaim.
        count: usize,
    },
    /// `Blocks` berisi lebih dari 16 entri.
    #[error("Blocks berisi {count} entri (paling banyak 16)")]
    TooManyEntries {
        /// Jumlah entri yang diklaim.
        count: usize,
    },
    /// Tag entri `Blocks` bukan `00` atau `01`.
    #[error("tag entri Blocks tidak dikenal: {tag}")]
    UnknownEntryTag {
        /// Tag yang ditemukan.
        tag: u8,
    },
    /// `Ack.status` di luar 0 sampai 4.
    #[error("status Ack tidak dikenal: {status}")]
    UnknownAckStatus {
        /// Nilai yang ditemukan.
        status: u8,
    },
    /// Panjang isi bingkai melebihi 2 MiB.
    #[error("panjang pesan {len} byte melebihi batas 2097152 byte")]
    FrameTooLarge {
        /// Panjang yang diklaim atau yang akan ditulis.
        len: usize,
    },
    /// Awalan panjang bingkai bukan `varint<32>` minimal.
    #[error("awalan panjang pesan bukan varint<32> minimal")]
    FrameLength,
    /// Aliran berakhir sebelum awalan panjang atau isi pesan lengkap.
    #[error("aliran berakhir sebelum pesan lengkap")]
    Truncated,
}

/// Encoding biner pesan P2P (isi bingkai).
pub trait Wire: Sized {
    /// Encoding kanonis pesan ini.
    fn to_wire(&self) -> Vec<u8>;

    /// Membaca pesan dari isi bingkai lengkap.
    ///
    /// # Errors
    ///
    /// [`MessageError`] jika encoding tidak sah, batas Bagian 3 dilanggar, atau ada sisa byte.
    fn from_wire(bytes: &[u8]) -> Result<Self, MessageError>;
}

/// Pesan `Status`: ujung chain aktif pengirim (protokol `status`, permintaan dan jawaban).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    /// Versi pesan; selain [`STATUS_VERSION`] koneksi ditutup tanpa sanksi.
    pub version: u8,
    /// `network_id` (block_id genesis) jaringan pengirim.
    pub network_id: Hash32,
    /// Tinggi ujung chain aktif.
    pub tip_height: BlockHeight,
    /// block_id ujung chain aktif.
    pub tip_id: Hash32,
    /// Chainwork kumulatif ujung, bilangan 256 bit big-endian.
    pub chainwork: [u8; 32],
}

impl Status {
    /// `Status` versi 1.
    #[must_use]
    pub const fn new(
        network_id: Hash32,
        tip_height: BlockHeight,
        tip_id: Hash32,
        chainwork: [u8; 32],
    ) -> Self {
        Self {
            version: STATUS_VERSION,
            network_id,
            tip_height,
            tip_id,
            chainwork,
        }
    }

    /// `true` jika versi 1 dan `network_id` sama dengan milik node.
    #[must_use]
    pub fn is_compatible(&self, network_id: &Hash32) -> bool {
        self.version == STATUS_VERSION && self.network_id == *network_id
    }
}

impl Wire for Status {
    fn to_wire(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(STATUS_SIZE);
        write_u8(&mut out, self.version);
        write_bytes(&mut out, self.network_id.as_bytes());
        write_u64_le(&mut out, self.tip_height.get());
        write_bytes(&mut out, self.tip_id.as_bytes());
        write_bytes(&mut out, &self.chainwork);
        out
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, MessageError> {
        let mut r = Reader::new(bytes);
        let version = r.read_u8()?;
        let network_id = Hash32::from_bytes(r.read_bytes::<ID_SIZE>()?);
        let tip_height = BlockHeight::new(r.read_u64_le()?);
        let tip_id = Hash32::from_bytes(r.read_bytes::<ID_SIZE>()?);
        let chainwork = r.read_bytes::<32>()?;
        r.finish()?;
        Ok(Self {
            version,
            network_id,
            tip_height,
            tip_id,
            chainwork,
        })
    }
}

/// Pesan `GetHeaders` (protokol `headers`, permintaan).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GetHeaders {
    /// ID blok locator, 1 sampai 64, dari ujung ke genesis.
    pub locator: Vec<Hash32>,
    /// Berhenti sesudah blok ini; nol berarti tanpa batas.
    pub stop: Hash32,
}

impl Wire for GetHeaders {
    fn to_wire(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_ids(&mut out, &self.locator);
        write_bytes(&mut out, self.stop.as_bytes());
        out
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, MessageError> {
        let mut r = Reader::new(bytes);
        let count = r.read_vec_len(ID_SIZE)?;
        if count == 0 || count > MAX_LOCATOR {
            return Err(MessageError::LocatorCount { count });
        }
        let locator = read_ids(&mut r, count)?;
        let stop = Hash32::from_bytes(r.read_bytes::<ID_SIZE>()?);
        r.finish()?;
        Ok(Self { locator, stop })
    }
}

/// Pesan `Headers` (protokol `headers`, jawaban): 0 sampai 2_000 header berurutan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Headers {
    /// Header berurutan dari yang terendah.
    pub headers: Vec<Header>,
}

impl Wire for Headers {
    fn to_wire(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_vec(&mut out, &self.headers);
        out
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, MessageError> {
        let mut r = Reader::new(bytes);
        let count = r.read_vec_len(HEADER_SIZE)?;
        if count > MAX_HEADERS {
            return Err(MessageError::TooManyHeaders { count });
        }
        let mut headers = Vec::with_capacity(count);
        for _ in 0..count {
            headers.push(Header::decode_from(&mut r)?);
        }
        r.finish()?;
        Ok(Self { headers })
    }
}

/// Pesan `GetBlocks` (protokol `blocks`, permintaan): 1 sampai 16 block_id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GetBlocks {
    /// block_id yang diminta, berurutan.
    pub ids: Vec<Hash32>,
}

impl Wire for GetBlocks {
    fn to_wire(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_ids(&mut out, &self.ids);
        out
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, MessageError> {
        let mut r = Reader::new(bytes);
        let count = r.read_vec_len(ID_SIZE)?;
        if count == 0 || count > MAX_BLOCK_IDS {
            return Err(MessageError::IdCount { count });
        }
        let ids = read_ids(&mut r, count)?;
        r.finish()?;
        Ok(Self { ids })
    }
}

/// Pesan `Blocks` (protokol `blocks`, jawaban): satu entri per ID yang diminta, urutan sama.
///
/// Entri `00` = tidak punya, `01 ‖ Block` = punya. Jumlah entri yang wajib sama dengan jumlah ID diperiksa layanan
/// jaringan, karena butuh permintaannya.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blocks {
    /// Entri per ID: `None` jika pengirim tidak punya blok itu.
    pub entries: Vec<Option<Block>>,
}

impl Wire for Blocks {
    fn to_wire(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_varint_u64(
            &mut out,
            u64::try_from(self.entries.len()).unwrap_or(u64::MAX),
        );
        for entry in &self.entries {
            match entry {
                None => write_u8(&mut out, 0),
                Some(block) => {
                    write_u8(&mut out, 1);
                    block.encode_to(&mut out);
                }
            }
        }
        out
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, MessageError> {
        let mut r = Reader::new(bytes);
        let count = r.read_vec_len(1)?;
        if count > MAX_BLOCK_IDS {
            return Err(MessageError::TooManyEntries { count });
        }
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            match r.read_u8()? {
                0 => entries.push(None),
                1 => entries.push(Some(Block::decode_from(&mut r)?)),
                tag => return Err(MessageError::UnknownEntryTag { tag }),
            }
        }
        r.finish()?;
        Ok(Self { entries })
    }
}

/// Pesan `Ack` (jawaban protokol `block` dan `tx`, Spesifikasi P2P Bagian 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ack {
    /// `0`: diterima atau sudah dikenal.
    Accepted,
    /// `1`: induk blok belum dikenal; penerima memulai sinkronisasi header dengan pengirim.
    UnknownParent,
    /// `2`: ditunda (V8, cap waktu terlalu jauh di masa depan untuk jam penerima).
    Deferred,
    /// `3`: tidak sah (V1 sampai V10, T1 sampai T6); penerima memberi sanksi kepada pengirim.
    Invalid,
    /// `4`: ditolak kebijakan mempool (bukan pelanggaran).
    Rejected,
}

impl Ack {
    /// Nilai byte `status`.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        match self {
            Self::Accepted => 0,
            Self::UnknownParent => 1,
            Self::Deferred => 2,
            Self::Invalid => 3,
            Self::Rejected => 4,
        }
    }

    /// Mengurai byte `status`.
    ///
    /// # Errors
    ///
    /// [`MessageError::UnknownAckStatus`] jika di luar 0 sampai 4.
    pub const fn from_u8(status: u8) -> Result<Self, MessageError> {
        match status {
            0 => Ok(Self::Accepted),
            1 => Ok(Self::UnknownParent),
            2 => Ok(Self::Deferred),
            3 => Ok(Self::Invalid),
            4 => Ok(Self::Rejected),
            status => Err(MessageError::UnknownAckStatus { status }),
        }
    }
}

impl Wire for Ack {
    fn to_wire(&self) -> Vec<u8> {
        vec![self.to_u8()]
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, MessageError> {
        let mut r = Reader::new(bytes);
        let ack = Self::from_u8(r.read_u8()?)?;
        r.finish()?;
        Ok(ack)
    }
}

impl Wire for Block {
    fn to_wire(&self) -> Vec<u8> {
        self.to_bytes()
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, MessageError> {
        Ok(<Self as Decode>::from_bytes(bytes)?)
    }
}

impl Wire for Transaction {
    fn to_wire(&self) -> Vec<u8> {
        self.to_bytes()
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, MessageError> {
        Ok(<Self as Decode>::from_bytes(bytes)?)
    }
}

/// Menulis `vec<bytes32>`.
fn write_ids(out: &mut Vec<u8>, ids: &[Hash32]) {
    write_varint_u64(out, u64::try_from(ids.len()).unwrap_or(u64::MAX));
    for id in ids {
        write_bytes(out, id.as_bytes());
    }
}

/// Membaca `count` ID 32 byte (jumlahnya sudah dibaca dan diperiksa).
fn read_ids(r: &mut Reader<'_>, count: usize) -> Result<Vec<Hash32>, MessageError> {
    let mut ids = Vec::with_capacity(count);
    for _ in 0..count {
        ids.push(Hash32::from_bytes(r.read_bytes::<ID_SIZE>()?));
    }
    Ok(ids)
}

#[cfg(test)]
mod tests;
