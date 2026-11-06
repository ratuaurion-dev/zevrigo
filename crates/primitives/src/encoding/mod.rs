//! Encoding biner kanonis Zevrigo Coin (`ZVRG`) sesuai Spesifikasi Format v0.1 Bagian 2 dan 10.
//!
//! Setiap struktur protokol hanya memiliki tepat satu representasi byte yang sah:
//! - Varint memakai LEB128 tak bertanda yang **wajib minimal**.
//! - Seluruh byte input wajib terbaca habis ([`Reader::finish`]).
//! - Pembacaan panjang elemen koleksi (`vec<T>`) memeriksa batas maksimum terhadap sisa byte
//!   ([`Reader::read_vec_len`]) **sebelum** melakukan alokasi memori apa pun.

use crate::height::BlockHeight;

/// Ukuran minimum 1 elemen byte mentah (`u8`) untuk pelindung alokasi `vec<u8>`.
pub const MIN_BYTE_SIZE: usize = 1;

/// Ukuran minimum 1 kunci publik x-only (`[u8; 32]`) untuk pelindung alokasi `vec<bytes32>`.
pub const MIN_KEY_SIZE: usize = 32;

/// Ukuran minimum 1 [`Input`](crate::tx::Input) terenkode (`32` byte `prev_txid` + `1` byte `prev_index`).
pub const MIN_INPUT_SIZE: usize = 33;

/// Ukuran minimum 1 [`Output`](crate::tx::Output) terenkode (`1` byte `amount` + `1` byte `kind` + `32` byte `payload`).
pub const MIN_OUTPUT_SIZE: usize = 34;

/// Ukuran minimum 1 [`Witness`](crate::tx::Witness) terenkode (`1` byte `kind = 0` + `64` byte tanda tangan).
pub const MIN_WITNESS_SIZE: usize = 65;

/// Ukuran minimum 1 [`SigEntry`](crate::tx::SigEntry) terenkode (`1` byte `key_index` + `64` byte tanda tangan).
pub const MIN_SIG_ENTRY_SIZE: usize = 65;

/// Ukuran minimum 1 [`Transaction`](crate::tx::Transaction) terenkode (coinbase terkecil: `46` byte).
pub const MIN_TX_SIZE: usize = 46;

/// Tabel pergeseran bit dan batas nilai 7-bit per posisi byte untuk `varint<32>` (maksimum 5 byte).
const U32_VARINT_STEPS: [(u32, u8); 5] = [(0, 0x7f), (7, 0x7f), (14, 0x7f), (21, 0x7f), (28, 0x0f)];

/// Tabel pergeseran bit dan batas nilai 7-bit per posisi byte untuk `varint<64>` (maksimum 10 byte).
const U64_VARINT_STEPS: [(u32, u8); 10] = [
    (0, 0x7f),
    (7, 0x7f),
    (14, 0x7f),
    (21, 0x7f),
    (28, 0x7f),
    (35, 0x7f),
    (42, 0x7f),
    (49, 0x7f),
    (56, 0x7f),
    (63, 0x01),
];

/// Tabel pergeseran bit dan batas nilai 7-bit per posisi byte untuk `varint<128>` (maksimum 19 byte).
const U128_VARINT_STEPS: [(u32, u8); 19] = [
    (0, 0x7f),
    (7, 0x7f),
    (14, 0x7f),
    (21, 0x7f),
    (28, 0x7f),
    (35, 0x7f),
    (42, 0x7f),
    (49, 0x7f),
    (56, 0x7f),
    (63, 0x7f),
    (70, 0x7f),
    (77, 0x7f),
    (84, 0x7f),
    (91, 0x7f),
    (98, 0x7f),
    (105, 0x7f),
    (112, 0x7f),
    (119, 0x7f),
    (126, 0x03),
];

/// Galat saat men-decode byte biner atau memvalidasi aturan struktural tipe protokol di `MOD-01`.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum DecodeError {
    /// Data input habis sebelum seluruh field selesai dibaca.
    #[error("data terpotong sebelum struktur selesai dibaca")]
    UnexpectedEof,

    /// Varint memakai lebih banyak byte daripada representasi minimalnya (byte terakhir bernilai `0x00`).
    #[error("encoding varint tidak minimal")]
    NonMinimalVarint,

    /// Nilai atau panjang byte varint melampaui lebar tipe bilangan bulat sasaran.
    #[error("nilai varint melampaui lebar tipe sasaran")]
    VarintOverflow,

    /// Masih terdapat sisa byte yang belum terbaca sesudah struktur selesai di-decode.
    #[error("terdapat {remaining} sisa byte sesudah akhir struktur")]
    TrailingBytes {
        /// Jumlah byte tersisa pada buffer input.
        remaining: usize,
    },

    /// Jumlah elemen koleksi (`vec<T>`) mustahil dipenuhi oleh sisa byte input (pelindung alokasi).
    #[error(
        "jumlah elemen ({count}) mustahil untuk sisa {remaining} byte (minimum {min_elem_size} byte/elemen)"
    )]
    ImpossibleElementCount {
        /// Jumlah elemen yang diklaim di dalam stream.
        count: u64,
        /// Sisa byte aktual sesudah pembacaan panjang koleksi.
        remaining: usize,
        /// Ukuran minimum per elemen dalam satuan byte.
        min_elem_size: usize,
    },

    /// Nomor versi struktur tidak didukung oleh protokol v0.1.
    #[error("versi struktur tidak dikenal: {actual}")]
    UnsupportedVersion {
        /// Nilai versi aktual yang ditemukan.
        actual: u32,
    },

    /// Tag `tx_kind` pada badan transaksi bukan `0` (biasa) atau `1` (coinbase).
    #[error("tx_kind tidak dikenal: {actual}")]
    UnknownTxKind {
        /// Nilai byte `tx_kind` aktual yang ditemukan.
        actual: u8,
    },

    /// Tag `kind` pada output bukan `0` (kunci tunggal) atau `1` (hash kondisi).
    #[error("jenis output tidak dikenal: {actual}")]
    UnknownOutputKind {
        /// Nilai byte `kind` output aktual yang ditemukan.
        actual: u8,
    },

    /// Tag `kind` pada witness bukan `0` (kunci tunggal) atau `1` (kondisi).
    #[error("jenis witness tidak dikenal: {actual}")]
    UnknownWitnessKind {
        /// Nilai byte `kind` witness aktual yang ditemukan.
        actual: u8,
    },

    /// Output memiliki `amount == 0` quanta.
    #[error("amount output wajib paling sedikit 1 quanta")]
    ZeroOutputAmount,

    /// Transaksi biasa memiliki daftar input kosong.
    #[error("transaksi biasa wajib memiliki paling sedikit 1 input")]
    EmptyInputs,

    /// Transaksi memiliki daftar output kosong.
    #[error("transaksi wajib memiliki paling sedikit 1 output")]
    EmptyOutputs,

    /// Field `extra` pada coinbase melebihi batas maksimum 32 byte.
    #[error("extra coinbase terlalu panjang: {len} byte (maksimum 32 byte)")]
    CoinbaseExtraTooLong {
        /// Panjang `extra` aktual dalam byte.
        len: usize,
    },

    /// Struktur kondisi melanggar aturan `1 <= m <= n <= 16` atau urutan kunci tidak naik ketat.
    #[error(
        "kondisi tidak valid (aturan 1 <= m <= n <= 16 dan kunci urut naik ketat tanpa duplikat)"
    )]
    InvalidCondition,

    /// Daftar tanda tangan witness jenis 1 melanggar jumlah `m`, batas `< n`, atau urutan indeks naik ketat.
    #[error("daftar tanda tangan witness tidak valid")]
    InvalidWitnessSignatures,

    /// Jumlah witness pada konstruksi transaksi tidak cocok dengan jumlah input yang diharapkan.
    #[error("jumlah witness ({actual}) tidak sama dengan jumlah yang diharapkan ({expected})")]
    WitnessCountMismatch {
        /// Jumlah witness yang seharusnya ada (`inputs.len()` untuk biasa, `0` untuk coinbase).
        expected: usize,
        /// Jumlah witness aktual yang diberikan.
        actual: usize,
    },

    /// Ukuran encoding blok melampaui `MAX_BLOCK_SIZE` (`100_000` byte).
    #[error("ukuran blok ({size} byte) melampaui batas maksimum 100000 byte")]
    BlockTooLarge {
        /// Ukuran byte yang melampaui batas.
        size: usize,
    },

    /// Blok tidak memiliki satu pun transaksi.
    #[error("blok kosong (wajib memuat paling sedikit 1 transaksi)")]
    EmptyBlock,

    /// Transaksi pertama bukan coinbase, atau ada coinbase pada indeks `>= 1`.
    #[error("posisi coinbase di dalam blok tidak sah")]
    InvalidCoinbasePosition,

    /// Tinggi blok pada badan coinbase tidak sama dengan tinggi pada header blok.
    #[error("tinggi coinbase ({coinbase}) tidak sama dengan tinggi header ({header})")]
    CoinbaseHeightMismatch {
        /// Tinggi blok menurut header.
        header: BlockHeight,
        /// Tinggi blok menurut transaksi coinbase pertama.
        coinbase: BlockHeight,
    },

    /// Pada blok genesis (`height == 0`), panjang `extra` coinbase bukan tepat 32 byte.
    #[error("panjang extra coinbase genesis ({actual} byte) harus tepat 32 byte")]
    InvalidGenesisExtraLength {
        /// Panjang byte `extra` aktual pada coinbase genesis.
        actual: usize,
    },
}

/// Pembaca biner sekuensial tanpa salinan di atas slice `&'a [u8]`.
#[derive(Clone, Debug)]
pub struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    /// Membentuk [`Reader`] baru di atas slice byte `bytes`.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// Mengembalikan jumlah byte yang belum dibaca.
    #[must_use]
    pub const fn remaining(&self) -> usize {
        self.bytes.len()
    }

    /// Memastikan seluruh byte pada [`Reader`] telah habis dibaca.
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError::TrailingBytes`] jika masih ada sisa byte (`remaining() > 0`).
    pub const fn finish(self) -> Result<(), DecodeError> {
        if self.bytes.is_empty() {
            Ok(())
        } else {
            Err(DecodeError::TrailingBytes {
                remaining: self.bytes.len(),
            })
        }
    }

    /// Membaca tepat 1 byte (`u8`).
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError::UnexpectedEof`] jika buffer kosong.
    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        let (&first, rest) = self.bytes.split_first().ok_or(DecodeError::UnexpectedEof)?;
        self.bytes = rest;
        Ok(first)
    }

    /// Membaca array byte berukuran tetap `N`.
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError::UnexpectedEof`] jika sisa byte kurang dari `N`.
    pub fn read_bytes<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let (head, rest) = self
            .bytes
            .split_at_checked(N)
            .ok_or(DecodeError::UnexpectedEof)?;
        let array = <[u8; N]>::try_from(head).map_err(|_| DecodeError::UnexpectedEof)?;
        self.bytes = rest;
        Ok(array)
    }

    /// Membaca slice sepanjang `len` byte tanpa menyalin.
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError::UnexpectedEof`] jika sisa byte kurang dari `len`.
    pub fn read_slice(&mut self, len: usize) -> Result<&'a [u8], DecodeError> {
        let (head, rest) = self
            .bytes
            .split_at_checked(len)
            .ok_or(DecodeError::UnexpectedEof)?;
        self.bytes = rest;
        Ok(head)
    }

    /// Membaca bilangan bulat `u32` little-endian berukuran tetap (4 byte).
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError::UnexpectedEof`] jika sisa byte kurang dari 4.
    pub fn read_u32_le(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(self.read_bytes::<4>()?))
    }

    /// Membaca bilangan bulat `u64` little-endian berukuran tetap (8 byte).
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError::UnexpectedEof`] jika sisa byte kurang dari 8.
    pub fn read_u64_le(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_le_bytes(self.read_bytes::<8>()?))
    }

    /// Membaca `varint<32>` (LEB128 tak bertanda minimal, maksimum 5 byte).
    ///
    /// # Errors
    ///
    /// Mengembalikan:
    /// - [`DecodeError::UnexpectedEof`] jika byte terpotong sebelum byte penutup.
    /// - [`DecodeError::VarintOverflow`] jika panjang melebihi 5 byte atau bit di atas 32 bit tidak nol.
    /// - [`DecodeError::NonMinimalVarint`] jika byte terakhir bernilai `0x00` pada varint multi-byte.
    pub fn read_varint_u32(&mut self) -> Result<u32, DecodeError> {
        let mut acc: u32 = 0;
        for &(shift, max_low) in &U32_VARINT_STEPS {
            let byte = self.read_u8()?;
            let low = byte & 0x7f;
            let has_more = (byte & 0x80) != 0;

            if low > max_low || (max_low < 0x7f && has_more) {
                return Err(DecodeError::VarintOverflow);
            }
            if !has_more && shift > 0 && low == 0 {
                return Err(DecodeError::NonMinimalVarint);
            }

            let part = u32::from(low)
                .checked_shl(shift)
                .ok_or(DecodeError::VarintOverflow)?;
            acc |= part;

            if !has_more {
                return Ok(acc);
            }
        }
        Err(DecodeError::VarintOverflow)
    }

    /// Membaca `varint<64>` (LEB128 tak bertanda minimal, maksimum 10 byte).
    ///
    /// # Errors
    ///
    /// Mengembalikan:
    /// - [`DecodeError::UnexpectedEof`] jika byte terpotong sebelum byte penutup.
    /// - [`DecodeError::VarintOverflow`] jika panjang melebihi 10 byte atau bit di atas 64 bit tidak nol.
    /// - [`DecodeError::NonMinimalVarint`] jika byte terakhir bernilai `0x00` pada varint multi-byte.
    pub fn read_varint_u64(&mut self) -> Result<u64, DecodeError> {
        let mut acc: u64 = 0;
        for &(shift, max_low) in &U64_VARINT_STEPS {
            let byte = self.read_u8()?;
            let low = byte & 0x7f;
            let has_more = (byte & 0x80) != 0;

            if low > max_low || (max_low < 0x7f && has_more) {
                return Err(DecodeError::VarintOverflow);
            }
            if !has_more && shift > 0 && low == 0 {
                return Err(DecodeError::NonMinimalVarint);
            }

            let part = u64::from(low)
                .checked_shl(shift)
                .ok_or(DecodeError::VarintOverflow)?;
            acc |= part;

            if !has_more {
                return Ok(acc);
            }
        }
        Err(DecodeError::VarintOverflow)
    }

    /// Membaca `varint<128>` (LEB128 tak bertanda minimal, maksimum 19 byte).
    ///
    /// # Errors
    ///
    /// Mengembalikan:
    /// - [`DecodeError::UnexpectedEof`] jika byte terpotong sebelum byte penutup.
    /// - [`DecodeError::VarintOverflow`] jika panjang melebihi 19 byte atau bit di atas 128 bit tidak nol.
    /// - [`DecodeError::NonMinimalVarint`] jika byte terakhir bernilai `0x00` pada varint multi-byte.
    pub fn read_varint_u128(&mut self) -> Result<u128, DecodeError> {
        let mut acc: u128 = 0;
        for &(shift, max_low) in &U128_VARINT_STEPS {
            let byte = self.read_u8()?;
            let low = byte & 0x7f;
            let has_more = (byte & 0x80) != 0;

            if low > max_low || (max_low < 0x7f && has_more) {
                return Err(DecodeError::VarintOverflow);
            }
            if !has_more && shift > 0 && low == 0 {
                return Err(DecodeError::NonMinimalVarint);
            }

            let part = u128::from(low)
                .checked_shl(shift)
                .ok_or(DecodeError::VarintOverflow)?;
            acc |= part;

            if !has_more {
                return Ok(acc);
            }
        }
        Err(DecodeError::VarintOverflow)
    }

    /// Membaca jumlah elemen `vec<T>` (`varint<64>`) dan memvalidasinya terhadap sisa byte
    /// **sebelum** melakukan alokasi memori apa pun.
    ///
    /// Syarat lulus: `min_elem_size > 0` dan `jumlah <= remaining() / min_elem_size`.
    ///
    /// # Errors
    ///
    /// Mengembalikan galat varint jika pembacaan `varint<64>` gagal, atau
    /// [`DecodeError::ImpossibleElementCount`] jika jumlah elemen melebihi kapasitas sisa byte.
    pub fn read_vec_len(&mut self, min_elem_size: usize) -> Result<usize, DecodeError> {
        let count_u64 = self.read_varint_u64()?;
        let remaining = self.remaining();
        let max_possible =
            remaining
                .checked_div(min_elem_size)
                .ok_or(DecodeError::ImpossibleElementCount {
                    count: count_u64,
                    remaining,
                    min_elem_size,
                })?;
        let count =
            usize::try_from(count_u64).map_err(|_| DecodeError::ImpossibleElementCount {
                count: count_u64,
                remaining,
                min_elem_size,
            })?;
        if count > max_possible {
            return Err(DecodeError::ImpossibleElementCount {
                count: count_u64,
                remaining,
                min_elem_size,
            });
        }
        Ok(count)
    }
}

/// Menulis satu byte (`u8`) ke buffer `out`.
pub fn write_u8(out: &mut Vec<u8>, value: u8) {
    out.push(value);
}

/// Menulis bilangan bulat `u32` dalam urutan byte little-endian (4 byte) ke `out`.
pub fn write_u32_le(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// Menulis bilangan bulat `u64` dalam urutan byte little-endian (8 byte) ke `out`.
pub fn write_u64_le(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// Menulis slice byte mentah apa adanya ke `out`.
pub fn write_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(bytes);
}

/// Menulis `varint<32>` minimal (1 sampai 5 byte) ke `out`.
pub fn write_varint_u32(out: &mut Vec<u8>, mut value: u32) {
    loop {
        let low = u8::try_from(value & 0x7f).unwrap_or(0);
        value = value.wrapping_shr(7);
        if value == 0 {
            out.push(low);
            break;
        }
        out.push(low | 0x80);
    }
}

/// Menulis `varint<64>` minimal (1 sampai 10 byte) ke `out`.
pub fn write_varint_u64(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let low = u8::try_from(value & 0x7f).unwrap_or(0);
        value = value.wrapping_shr(7);
        if value == 0 {
            out.push(low);
            break;
        }
        out.push(low | 0x80);
    }
}

/// Menulis `varint<128>` minimal (1 sampai 19 byte) ke `out`.
pub fn write_varint_u128(out: &mut Vec<u8>, mut value: u128) {
    loop {
        let low = u8::try_from(value & 0x7f).unwrap_or(0);
        value = value.wrapping_shr(7);
        if value == 0 {
            out.push(low);
            break;
        }
        out.push(low | 0x80);
    }
}

/// Menghitung panjang byte kanonis `varint<32>` untuk `value` tanpa alokasi.
#[must_use]
pub const fn varint_u32_len(mut value: u32) -> usize {
    let mut len: usize = 1;
    while value >= 0x80 {
        value = value.wrapping_shr(7);
        // Maksimum iterasi adalah 4 kali (total len <= 5), sehingga saturating_add tidak pernah jenuh.
        len = len.saturating_add(1);
    }
    len
}

/// Menghitung panjang byte kanonis `varint<64>` untuk `value` tanpa alokasi.
#[must_use]
pub const fn varint_u64_len(mut value: u64) -> usize {
    let mut len: usize = 1;
    while value >= 0x80 {
        value = value.wrapping_shr(7);
        // Maksimum iterasi adalah 9 kali (total len <= 10), sehingga saturating_add tidak pernah jenuh.
        len = len.saturating_add(1);
    }
    len
}

/// Menghitung panjang byte kanonis `varint<128>` untuk `value` tanpa alokasi.
#[must_use]
pub const fn varint_u128_len(mut value: u128) -> usize {
    let mut len: usize = 1;
    while value >= 0x80 {
        value = value.wrapping_shr(7);
        // Maksimum iterasi adalah 18 kali (total len <= 19), sehingga saturating_add tidak pernah jenuh.
        len = len.saturating_add(1);
    }
    len
}

/// Menulis koleksi `vec<T>` (`varint<64>` jumlah elemen diikuti setiap elemen secara berurutan).
pub fn write_vec<T: Encode>(out: &mut Vec<u8>, items: &[T]) {
    let count = u64::try_from(items.len()).unwrap_or(u64::MAX);
    write_varint_u64(out, count);
    for item in items {
        item.encode_to(out);
    }
}

/// Membaca koleksi `vec<T>` dengan pelindung alokasi `min_elem_size`.
///
/// # Errors
///
/// Mengembalikan galat dari [`Reader::read_vec_len`] atau dari `T::decode_from` elemennya.
pub fn read_vec<T: Decode>(
    r: &mut Reader<'_>,
    min_elem_size: usize,
) -> Result<Vec<T>, DecodeError> {
    let count = r.read_vec_len(min_elem_size)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        items.push(T::decode_from(r)?);
    }
    Ok(items)
}

/// Trait serialisasi ke representasi byte kanonis Zevrigo Coin.
pub trait Encode {
    /// Menambahkan representasi byte kanonis `self` ke dalam `out`.
    fn encode_to(&self, out: &mut Vec<u8>);

    /// Mengalokasikan `Vec<u8>` baru berisi representasi byte kanonis `self`.
    #[must_use]
    fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.encode_to(&mut out);
        out
    }
}

/// Trait deserialisasi dari representasi byte kanonis Zevrigo Coin.
pub trait Decode: Sized {
    /// Membaca nilai `Self` dari posisi saat ini pada [`Reader`].
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError`] jika encoding tidak sah atau terpotong.
    fn decode_from(r: &mut Reader<'_>) -> Result<Self, DecodeError>;

    /// Membaca nilai `Self` dari slice byte penuh dan memastikan tidak ada sisa byte ([`Reader::finish`]).
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError`] jika encoding tidak sah, terpotong, atau menyisakan byte.
    fn from_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        let value = Self::decode_from(&mut r)?;
        r.finish()?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests;
