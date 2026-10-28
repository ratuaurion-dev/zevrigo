//! Struktur [`Header`] (96 byte tetap) dan [`Block`] beserta encoding kanonisnya
//! sesuai Spesifikasi Format v0.1 Bagian 6.1, 6.2, 7, dan 10.
//!
//! Perhitungan `block_id`, Merkle root, dan Proof of Work tidak dilakukan di modul ini.

use crate::encoding::{
    Decode, DecodeError, Encode, MIN_TX_SIZE, Reader, varint_u64_len, write_bytes, write_vec,
};
use crate::hash::Hash32;
use crate::height::BlockHeight;
use crate::params::MAX_BLOCK_SIZE;
use crate::tx::{Transaction, TxBody};

/// Ukuran tetap encoding biner [`Header`] dalam satuan byte (`96` byte).
pub const HEADER_SIZE: usize = 96;

/// Nomor versi format [`Header`] yang didukung saat ini (`1`).
pub const HEADER_VERSION: u32 = 1;

/// Panjang wajib field `extra` pada transaksi coinbase blok genesis (`32` byte jangkar Bitcoin).
pub const GENESIS_EXTRA_LEN: usize = 32;

/// Batas atas teoritis jumlah transaksi di dalam satu blok sah (`100_000 / 46 = 2_173`).
const MAX_TXS_PER_BLOCK: usize = MAX_BLOCK_SIZE / MIN_TX_SIZE;

/// Header blok berukuran tetap 96 byte sesuai Spesifikasi Bagian 6.1.
///
/// Nomor versi (`1`, `u32le`) ditulis saat encode dan wajib bernilai `1` saat decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    height: BlockHeight,
    prev_block_id: Hash32,
    merkle_root: Hash32,
    timestamp: u64,
    target: u32,
    nonce: u64,
}

impl Header {
    /// Membentuk [`Header`] baru dari keenam field protokolnya (versi dikunci pada `1`).
    #[must_use]
    pub const fn new(
        height: BlockHeight,
        prev_block_id: Hash32,
        merkle_root: Hash32,
        timestamp: u64,
        target: u32,
        nonce: u64,
    ) -> Self {
        Self {
            height,
            prev_block_id,
            merkle_root,
            timestamp,
            target,
            nonce,
        }
    }

    /// Mengembalikan tinggi blok.
    #[must_use]
    pub const fn height(&self) -> BlockHeight {
        self.height
    }

    /// Mengembalikan ID blok sebelumnya (`Hash32::ZERO` pada genesis).
    #[must_use]
    pub const fn prev_block_id(&self) -> Hash32 {
        self.prev_block_id
    }

    /// Mengembalikan Merkle root dari daftar `wtxid` blok.
    #[must_use]
    pub const fn merkle_root(&self) -> Hash32 {
        self.merkle_root
    }

    /// Mengembalikan cap waktu blok dalam detik Unix.
    #[must_use]
    pub const fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Mengembalikan target kesulitan dalam format compact `u32`.
    #[must_use]
    pub const fn target(&self) -> u32 {
        self.target
    }

    /// Mengembalikan nonce penambang.
    #[must_use]
    pub const fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Meng-encode [`Header`] ke array ukuran tetap `[u8; HEADER_SIZE]` (`96` byte).
    #[must_use]
    pub fn to_array(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0_u8; HEADER_SIZE];
        let version_bytes = HEADER_VERSION.to_le_bytes();
        let height_bytes = self.height.get().to_le_bytes();
        let timestamp_bytes = self.timestamp.to_le_bytes();
        let target_bytes = self.target.to_le_bytes();
        let nonce_bytes = self.nonce.to_le_bytes();

        let chunks = [
            version_bytes.as_slice(),
            height_bytes.as_slice(),
            self.prev_block_id.as_bytes().as_slice(),
            self.merkle_root.as_bytes().as_slice(),
            timestamp_bytes.as_slice(),
            target_bytes.as_slice(),
            nonce_bytes.as_slice(),
        ];

        let mut offset: usize = 0;
        for chunk in chunks {
            let next_offset = offset.saturating_add(chunk.len());
            if let Some(target) = buf.get_mut(offset..next_offset) {
                target.copy_from_slice(chunk);
            }
            offset = next_offset;
        }
        buf
    }

    /// Men-decode [`Header`] dari array ukuran tetap `[u8; HEADER_SIZE]` (`96` byte).
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError::UnsupportedVersion`] jika versi bukan `1`.
    pub fn from_array(bytes: &[u8; HEADER_SIZE]) -> Result<Self, DecodeError> {
        Self::from_bytes(bytes.as_slice())
    }
}

impl Encode for Header {
    fn encode_to(&self, out: &mut Vec<u8>) {
        write_bytes(out, &self.to_array());
    }
}

impl Decode for Header {
    fn decode_from(r: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let version = r.read_u32_le()?;
        if version != HEADER_VERSION {
            return Err(DecodeError::UnsupportedVersion { actual: version });
        }
        let height = BlockHeight::new(r.read_u64_le()?);
        let prev_block_id = Hash32::from_bytes(r.read_bytes::<32>()?);
        let merkle_root = Hash32::from_bytes(r.read_bytes::<32>()?);
        let timestamp = r.read_u64_le()?;
        let target = r.read_u32_le()?;
        let nonce = r.read_u64_le()?;
        Ok(Self::new(
            height,
            prev_block_id,
            merkle_root,
            timestamp,
            target,
            nonce,
        ))
    }
}

/// Blok penuh Homeostasis Coin (`Header` + daftar [`Transaction`]) sesuai Spesifikasi Bagian 6.2 dan 7.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    header: Header,
    txs: Vec<Transaction>,
}

impl Block {
    /// Membentuk [`Block`] baru setelah memvalidasi seluruh invarian struktural tingkat blok:
    /// - Paling sedikit 1 transaksi.
    /// - Transaksi pertama wajib coinbase; seluruh transaksi berikutnya wajib transaksi biasa.
    /// - `height` pada coinbase wajib sama dengan `header.height()`.
    /// - Jika `header.height() == 0` (genesis), `extra` coinbase wajib tepat 32 byte.
    /// - Ukuran total encoding biner blok wajib `<= MAX_BLOCK_SIZE` (`100_000` byte).
    ///
    /// # Errors
    ///
    /// Mengembalikan varian [`DecodeError`] yang bersesuaian jika salah satu aturan dilanggar.
    pub fn new(header: Header, txs: Vec<Transaction>) -> Result<Self, DecodeError> {
        Self::validate_structure(&header, &txs)?;
        let size = Self::compute_encoded_size(&txs)?;
        if size > MAX_BLOCK_SIZE {
            return Err(DecodeError::BlockTooLarge { size });
        }
        Ok(Self { header, txs })
    }

    /// Mengembalikan referensi ke [`Header`] blok.
    #[must_use]
    pub const fn header(&self) -> &Header {
        &self.header
    }

    /// Mengembalikan slice daftar transaksi di dalam blok.
    #[must_use]
    pub fn txs(&self) -> &[Transaction] {
        &self.txs
    }

    fn validate_structure(header: &Header, txs: &[Transaction]) -> Result<(), DecodeError> {
        let Some((first_tx, rest_txs)) = txs.split_first() else {
            return Err(DecodeError::EmptyBlock);
        };
        let TxBody::Coinbase(coinbase) = first_tx.body() else {
            return Err(DecodeError::InvalidCoinbasePosition);
        };
        if rest_txs.iter().any(|tx| !tx.is_regular()) {
            return Err(DecodeError::InvalidCoinbasePosition);
        }
        if coinbase.height() != header.height() {
            return Err(DecodeError::CoinbaseHeightMismatch {
                header: header.height(),
                coinbase: coinbase.height(),
            });
        }
        if header.height() == BlockHeight::GENESIS && coinbase.extra().len() != GENESIS_EXTRA_LEN {
            return Err(DecodeError::InvalidGenesisExtraLength {
                actual: coinbase.extra().len(),
            });
        }
        Ok(())
    }

    fn compute_encoded_size(txs: &[Transaction]) -> Result<usize, DecodeError> {
        let tx_count_u64 = u64::try_from(txs.len())
            .map_err(|_| DecodeError::BlockTooLarge { size: usize::MAX })?;
        let mut size = HEADER_SIZE
            .checked_add(varint_u64_len(tx_count_u64))
            .ok_or(DecodeError::BlockTooLarge { size: usize::MAX })?;
        for tx in txs {
            let tx_size = tx
                .encoded_size()
                .ok_or(DecodeError::BlockTooLarge { size: usize::MAX })?;
            size = size
                .checked_add(tx_size)
                .ok_or(DecodeError::BlockTooLarge { size: usize::MAX })?;
        }
        Ok(size)
    }
}

impl Encode for Block {
    fn encode_to(&self, out: &mut Vec<u8>) {
        self.header.encode_to(out);
        write_vec(out, &self.txs);
    }
}

impl Decode for Block {
    fn decode_from(r: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let before = r.remaining();
        let header = Header::decode_from(r)?;
        let tx_count = r.read_vec_len(MIN_TX_SIZE)?;
        if tx_count == 0 {
            return Err(DecodeError::EmptyBlock);
        }
        let initial_cap = tx_count.min(MAX_TXS_PER_BLOCK);
        let mut txs = Vec::with_capacity(initial_cap);
        for _ in 0..tx_count {
            txs.push(Transaction::decode_from(r)?);
        }
        let after = r.remaining();
        let size = before
            .checked_sub(after)
            .ok_or(DecodeError::UnexpectedEof)?;
        if size > MAX_BLOCK_SIZE {
            return Err(DecodeError::BlockTooLarge { size });
        }
        Self::validate_structure(&header, &txs)?;
        Ok(Self { header, txs })
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() > MAX_BLOCK_SIZE {
            return Err(DecodeError::BlockTooLarge { size: bytes.len() });
        }
        let mut r = Reader::new(bytes);
        let block = Self::decode_from(&mut r)?;
        r.finish()?;
        Ok(block)
    }
}

#[cfg(test)]
mod tests;
