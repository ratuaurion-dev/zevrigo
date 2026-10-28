//! Struktur transaksi, output, kondisi, input, dan witness beserta encoding kanonisnya
//! sesuai Spesifikasi Format v0.1 Bagian 4, 5, dan 10.
//!
//! Seluruh tipe di modul ini memiliki field privat dan hanya dapat dibangun melalui konstruktor
//! yang memvalidasi aturan struktural protokol. Operasi hashing (`txid`, `wtxid`, hash kondisi)
//! tidak dilakukan di sini melainkan di `MOD-02` (`TSK-M21`).

use crate::amount::Amount;
use crate::encoding::{
    Decode, DecodeError, Encode, MIN_BYTE_SIZE, MIN_INPUT_SIZE, MIN_KEY_SIZE, MIN_OUTPUT_SIZE,
    MIN_SIG_ENTRY_SIZE, MIN_WITNESS_SIZE, Reader, varint_u32_len, varint_u64_len, varint_u128_len,
    write_bytes, write_u8, write_u64_le, write_varint_u32, write_varint_u64, write_varint_u128,
    write_vec,
};
use crate::hash::Hash32;
use crate::height::BlockHeight;

/// Nomor versi format [`Condition`] yang didukung saat ini (`1`).
pub const CONDITION_VERSION: u8 = 1;

/// Jumlah maksimum kunci publik di dalam satu [`Condition`] (`16`).
pub const MAX_CONDITION_KEYS: usize = 16;

/// Nomor versi format [`TxBody`] yang didukung saat ini (`1`).
pub const TX_VERSION: u8 = 1;

/// Panjang maksimum field `extra` pada transaksi coinbase (`32` byte).
pub const MAX_COINBASE_EXTRA_LEN: usize = 32;

/// Panjang byte kunci publik x-only BIP340 (`32` byte).
pub const PUBKEY_XONLY_LEN: usize = 32;

/// Panjang byte tanda tangan Schnorr BIP340 (`64` byte).
pub const SIGNATURE_LEN: usize = 64;

/// Jenis keluaran transaksi ([`Output`]) sesuai Spesifikasi Bagian 4.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OutputKind {
    /// Jenis `0`: kunci tunggal x-only BIP340 (termasuk kunci gabungan MuSig2 `n`-dari-`n`).
    SingleKey = 0,
    /// Jenis `1`: hash BLAKE3 atas encoding kanonis [`Condition`].
    ConditionHash = 1,
}

impl OutputKind {
    /// Mengembalikan representasi byte tag `kind` (`0` atau `1`).
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        match self {
            Self::SingleKey => 0,
            Self::ConditionHash => 1,
        }
    }

    /// Mengurai byte tag `kind` menjadi [`OutputKind`].
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError::UnknownOutputKind`] jika `value` bukan `0` atau `1`.
    pub const fn from_u8(value: u8) -> Result<Self, DecodeError> {
        match value {
            0 => Ok(Self::SingleKey),
            1 => Ok(Self::ConditionHash),
            actual => Err(DecodeError::UnknownOutputKind { actual }),
        }
    }
}

/// Keluaran transaksi UTXO (`amount >= 1` quanta, `kind` `0` atau `1`, `payload` 32 byte).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    amount: Amount,
    kind: OutputKind,
    payload: [u8; PUBKEY_XONLY_LEN],
}

impl Output {
    /// Membentuk [`Output`] baru setelah memvalidasi `amount >= 1` quanta.
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError::ZeroOutputAmount`] jika `amount == Amount::ZERO`.
    pub const fn new(
        amount: Amount,
        kind: OutputKind,
        payload: [u8; PUBKEY_XONLY_LEN],
    ) -> Result<Self, DecodeError> {
        if amount.as_quanta() == 0 {
            return Err(DecodeError::ZeroOutputAmount);
        }
        Ok(Self {
            amount,
            kind,
            payload,
        })
    }

    /// Mengembalikan nilai jumlah koin pada output ini.
    #[must_use]
    pub const fn amount(&self) -> Amount {
        self.amount
    }

    /// Mengembalikan jenis output (`0` kunci tunggal atau `1` hash kondisi).
    #[must_use]
    pub const fn kind(&self) -> OutputKind {
        self.kind
    }

    /// Mengembalikan payload 32 byte (kunci x-only atau hash kondisi).
    #[must_use]
    pub const fn payload(&self) -> &[u8; PUBKEY_XONLY_LEN] {
        &self.payload
    }

    /// Menghitung ukuran byte terenkode kanonis output ini (`34` sampai `52` byte).
    #[must_use]
    pub const fn encoded_size(&self) -> usize {
        // varint_u128_len <= 19, ditambah 1 byte kind + 32 byte payload <= 52 byte.
        varint_u128_len(self.amount.as_quanta())
            .saturating_add(1)
            .saturating_add(PUBKEY_XONLY_LEN)
    }
}

impl Encode for Output {
    fn encode_to(&self, out: &mut Vec<u8>) {
        write_varint_u128(out, self.amount.as_quanta());
        write_u8(out, self.kind.to_u8());
        write_bytes(out, &self.payload);
    }
}

impl Decode for Output {
    fn decode_from(r: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let quanta = r.read_varint_u128()?;
        if quanta == 0 {
            return Err(DecodeError::ZeroOutputAmount);
        }
        let kind = OutputKind::from_u8(r.read_u8()?)?;
        let payload = r.read_bytes::<PUBKEY_XONLY_LEN>()?;
        Self::new(Amount::from_quanta(quanta), kind, payload)
    }
}

/// Kondisi pembelanjaan output jenis `1` (`m`-dari-`n` kunci x-only dengan `lock_height` opsional).
///
/// Nomor versi (`1`) tidak disimpan sebagai field, melainkan selalu ditulis saat encode dan
/// diverifikasi saat decode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Condition {
    m: u8,
    keys: Vec<[u8; PUBKEY_XONLY_LEN]>,
    lock_height: BlockHeight,
}

impl Condition {
    /// Membentuk [`Condition`] baru setelah memvalidasi aturan kanonis:
    /// - `1 <= n <= 16` (dengan `n = keys.len()`)
    /// - `1 <= m <= n`
    /// - `keys` urut naik ketat secara bytewise (tanpa duplikat).
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError::InvalidCondition`] jika salah satu aturan dilanggar.
    pub fn new(
        m: u8,
        keys: Vec<[u8; PUBKEY_XONLY_LEN]>,
        lock_height: BlockHeight,
    ) -> Result<Self, DecodeError> {
        let n = keys.len();
        if n == 0 || n > MAX_CONDITION_KEYS {
            return Err(DecodeError::InvalidCondition);
        }
        if m == 0 || usize::from(m) > n {
            return Err(DecodeError::InvalidCondition);
        }
        for pair in keys.windows(2) {
            if let [left, right] = pair
                && left >= right
            {
                return Err(DecodeError::InvalidCondition);
            }
        }
        Ok(Self {
            m,
            keys,
            lock_height,
        })
    }

    /// Mengembalikan ambang jumlah tanda tangan minimum (`m`).
    #[must_use]
    pub const fn m(&self) -> u8 {
        self.m
    }

    /// Mengembalikan total kunci publik (`n`), dijamin berada pada rentang `1..=16`.
    #[must_use]
    pub fn n(&self) -> u8 {
        u8::try_from(self.keys.len()).unwrap_or(0)
    }

    /// Mengembalikan daftar kunci publik x-only 32 byte yang terurut naik ketat.
    #[must_use]
    pub fn keys(&self) -> &[[u8; PUBKEY_XONLY_LEN]] {
        &self.keys
    }

    /// Mengembalikan tinggi blok kunci waktu (`0` berarti tanpa kunci waktu).
    #[must_use]
    pub const fn lock_height(&self) -> BlockHeight {
        self.lock_height
    }

    /// Menghitung ukuran byte terenkode kanonis kondisi ini.
    #[must_use]
    pub fn encoded_size(&self) -> usize {
        let key_count_u64 = u64::try_from(self.keys.len()).unwrap_or(u64::MAX);
        let keys_bytes = self.keys.len().saturating_mul(PUBKEY_XONLY_LEN);
        // 1 (version) + 1 (m) + varint(n) + n*32 + 8 (lock_height)
        2_usize
            .saturating_add(varint_u64_len(key_count_u64))
            .saturating_add(keys_bytes)
            .saturating_add(8)
    }
}

impl Encode for Condition {
    fn encode_to(&self, out: &mut Vec<u8>) {
        write_u8(out, CONDITION_VERSION);
        write_u8(out, self.m);
        let key_count = u64::try_from(self.keys.len()).unwrap_or(u64::MAX);
        write_varint_u64(out, key_count);
        for key in &self.keys {
            write_bytes(out, key);
        }
        write_u64_le(out, self.lock_height.get());
    }
}

impl Decode for Condition {
    fn decode_from(r: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let version = r.read_u8()?;
        if version != CONDITION_VERSION {
            return Err(DecodeError::UnsupportedVersion {
                actual: u32::from(version),
            });
        }
        let m = r.read_u8()?;
        // Pelindung alokasi wajib diperiksa terlebih dahulu terhadap sisa byte.
        let key_count = r.read_vec_len(MIN_KEY_SIZE)?;
        // Sesudah lolos pelindung alokasi, periksa batas protokol 1 <= m <= n <= 16.
        if key_count == 0 || key_count > MAX_CONDITION_KEYS {
            return Err(DecodeError::InvalidCondition);
        }
        if m == 0 || usize::from(m) > key_count {
            return Err(DecodeError::InvalidCondition);
        }
        let mut keys = Vec::with_capacity(key_count);
        for _ in 0..key_count {
            keys.push(r.read_bytes::<PUBKEY_XONLY_LEN>()?);
        }
        let lock_height = BlockHeight::new(r.read_u64_le()?);
        Self::new(m, keys, lock_height)
    }
}

/// Masukan transaksi biasa yang menunjuk satu UTXO sebelumnya (`prev_txid` + `prev_index`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    prev_txid: Hash32,
    prev_index: u32,
}

impl Input {
    /// Membentuk [`Input`] baru.
    #[must_use]
    pub const fn new(prev_txid: Hash32, prev_index: u32) -> Self {
        Self {
            prev_txid,
            prev_index,
        }
    }

    /// Mengembalikan `txid` transaksi asal output yang dibelanjakan.
    #[must_use]
    pub const fn prev_txid(&self) -> Hash32 {
        self.prev_txid
    }

    /// Mengembalikan indeks output berbasis nol di dalam transaksi asal.
    #[must_use]
    pub const fn prev_index(&self) -> u32 {
        self.prev_index
    }

    /// Menghitung ukuran byte terenkode kanonis input ini (`33` sampai `37` byte).
    #[must_use]
    pub const fn encoded_size(&self) -> usize {
        32_usize.saturating_add(varint_u32_len(self.prev_index))
    }
}

impl Encode for Input {
    fn encode_to(&self, out: &mut Vec<u8>) {
        write_bytes(out, self.prev_txid.as_bytes());
        write_varint_u32(out, self.prev_index);
    }
}

impl Decode for Input {
    fn decode_from(r: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let prev_txid = Hash32::from_bytes(r.read_bytes::<32>()?);
        let prev_index = r.read_varint_u32()?;
        Ok(Self::new(prev_txid, prev_index))
    }
}

/// Isi badan transaksi biasa (`tx_kind = 0`): minimal 1 input dan minimal 1 output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegularTxBody {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
}

impl RegularTxBody {
    /// Membentuk [`RegularTxBody`] setelah memastikan `inputs` dan `outputs` tidak kosong.
    ///
    /// # Errors
    ///
    /// Mengembalikan:
    /// - [`DecodeError::EmptyInputs`] jika `inputs` kosong.
    /// - [`DecodeError::EmptyOutputs`] jika `outputs` kosong.
    pub fn new(inputs: Vec<Input>, outputs: Vec<Output>) -> Result<Self, DecodeError> {
        if inputs.is_empty() {
            return Err(DecodeError::EmptyInputs);
        }
        if outputs.is_empty() {
            return Err(DecodeError::EmptyOutputs);
        }
        Ok(Self { inputs, outputs })
    }

    /// Mengembalikan slice daftar input transaksi.
    #[must_use]
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    /// Mengembalikan slice daftar output transaksi.
    #[must_use]
    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }
}

/// Isi badan transaksi coinbase (`tx_kind = 1`): tinggi blok, `extra` `0..=32` byte, dan minimal 1 output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoinbaseTxBody {
    height: BlockHeight,
    extra: Vec<u8>,
    outputs: Vec<Output>,
}

impl CoinbaseTxBody {
    /// Membentuk [`CoinbaseTxBody`] setelah memastikan `extra.len() <= 32` dan `outputs` tidak kosong.
    ///
    /// # Errors
    ///
    /// Mengembalikan:
    /// - [`DecodeError::CoinbaseExtraTooLong`] jika `extra.len() > 32`.
    /// - [`DecodeError::EmptyOutputs`] jika `outputs` kosong.
    pub fn new(
        height: BlockHeight,
        extra: Vec<u8>,
        outputs: Vec<Output>,
    ) -> Result<Self, DecodeError> {
        if extra.len() > MAX_COINBASE_EXTRA_LEN {
            return Err(DecodeError::CoinbaseExtraTooLong { len: extra.len() });
        }
        if outputs.is_empty() {
            return Err(DecodeError::EmptyOutputs);
        }
        Ok(Self {
            height,
            extra,
            outputs,
        })
    }

    /// Mengembalikan tinggi blok yang tertulis pada badan coinbase.
    #[must_use]
    pub const fn height(&self) -> BlockHeight {
        self.height
    }

    /// Mengembalikan byte `extra` coinbase (`0..=32` byte).
    #[must_use]
    pub fn extra(&self) -> &[u8] {
        &self.extra
    }

    /// Mengembalikan slice daftar output coinbase.
    #[must_use]
    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }
}

/// Badan transaksi (`TxBody`) yang menjadi preimage `txid` sesuai Spesifikasi Bagian 5.1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TxBody {
    /// Transaksi biasa (`tx_kind = 0`).
    Regular(RegularTxBody),
    /// Transaksi coinbase (`tx_kind = 1`).
    Coinbase(CoinbaseTxBody),
}

impl TxBody {
    /// Mengembalikan `true` jika badan ini adalah transaksi coinbase.
    #[must_use]
    pub const fn is_coinbase(&self) -> bool {
        matches!(self, Self::Coinbase(_))
    }

    /// Mengembalikan `true` jika badan ini adalah transaksi biasa.
    #[must_use]
    pub const fn is_regular(&self) -> bool {
        matches!(self, Self::Regular(_))
    }

    /// Mengembalikan daftar output pada badan transaksi ini.
    #[must_use]
    pub fn outputs(&self) -> &[Output] {
        match self {
            Self::Regular(reg) => reg.outputs(),
            Self::Coinbase(cb) => cb.outputs(),
        }
    }

    /// Menghitung ukuran byte terenkode kanonis badan transaksi (termasuk `version` dan `tx_kind`).
    pub fn encoded_size(&self) -> Option<usize> {
        // 1 byte version + 1 byte tx_kind
        let mut total: usize = 2;
        match self {
            Self::Regular(reg) => {
                let in_len = u64::try_from(reg.inputs().len()).ok()?;
                total = total.checked_add(varint_u64_len(in_len))?;
                for input in reg.inputs() {
                    total = total.checked_add(input.encoded_size())?;
                }
                let out_len = u64::try_from(reg.outputs().len()).ok()?;
                total = total.checked_add(varint_u64_len(out_len))?;
                for output in reg.outputs() {
                    total = total.checked_add(output.encoded_size())?;
                }
            }
            Self::Coinbase(cb) => {
                total = total.checked_add(8)?;
                let extra_len = u64::try_from(cb.extra().len()).ok()?;
                total = total.checked_add(varint_u64_len(extra_len))?;
                total = total.checked_add(cb.extra().len())?;
                let out_len = u64::try_from(cb.outputs().len()).ok()?;
                total = total.checked_add(varint_u64_len(out_len))?;
                for output in cb.outputs() {
                    total = total.checked_add(output.encoded_size())?;
                }
            }
        }
        Some(total)
    }
}

impl Encode for TxBody {
    fn encode_to(&self, out: &mut Vec<u8>) {
        write_u8(out, TX_VERSION);
        match self {
            Self::Regular(reg) => {
                write_u8(out, 0);
                write_vec(out, reg.inputs());
                write_vec(out, reg.outputs());
            }
            Self::Coinbase(cb) => {
                write_u8(out, 1);
                write_u64_le(out, cb.height().get());
                let extra_len = u64::try_from(cb.extra().len()).unwrap_or(u64::MAX);
                write_varint_u64(out, extra_len);
                write_bytes(out, cb.extra());
                write_vec(out, cb.outputs());
            }
        }
    }
}

impl Decode for TxBody {
    fn decode_from(r: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let version = r.read_u8()?;
        if version != TX_VERSION {
            return Err(DecodeError::UnsupportedVersion {
                actual: u32::from(version),
            });
        }
        let tx_kind = r.read_u8()?;
        match tx_kind {
            0 => {
                let input_count = r.read_vec_len(MIN_INPUT_SIZE)?;
                if input_count == 0 {
                    return Err(DecodeError::EmptyInputs);
                }
                let mut inputs = Vec::with_capacity(input_count);
                for _ in 0..input_count {
                    inputs.push(Input::decode_from(r)?);
                }

                let output_count = r.read_vec_len(MIN_OUTPUT_SIZE)?;
                if output_count == 0 {
                    return Err(DecodeError::EmptyOutputs);
                }
                let mut outputs = Vec::with_capacity(output_count);
                for _ in 0..output_count {
                    outputs.push(Output::decode_from(r)?);
                }

                Ok(Self::Regular(RegularTxBody::new(inputs, outputs)?))
            }
            1 => {
                let height = BlockHeight::new(r.read_u64_le()?);
                let extra_len = r.read_vec_len(MIN_BYTE_SIZE)?;
                if extra_len > MAX_COINBASE_EXTRA_LEN {
                    return Err(DecodeError::CoinbaseExtraTooLong { len: extra_len });
                }
                let extra = r.read_slice(extra_len)?.to_vec();

                let output_count = r.read_vec_len(MIN_OUTPUT_SIZE)?;
                if output_count == 0 {
                    return Err(DecodeError::EmptyOutputs);
                }
                let mut outputs = Vec::with_capacity(output_count);
                for _ in 0..output_count {
                    outputs.push(Output::decode_from(r)?);
                }

                Ok(Self::Coinbase(CoinbaseTxBody::new(height, extra, outputs)?))
            }
            actual => Err(DecodeError::UnknownTxKind { actual }),
        }
    }
}

/// Pasangan indeks kunci dan tanda tangan 64 byte di dalam witness kondisi (`kind = 1`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SigEntry {
    key_index: u8,
    sig: [u8; SIGNATURE_LEN],
}

impl SigEntry {
    /// Membentuk [`SigEntry`] baru.
    #[must_use]
    pub const fn new(key_index: u8, sig: [u8; SIGNATURE_LEN]) -> Self {
        Self { key_index, sig }
    }

    /// Mengembalikan indeks kunci berbasis nol di dalam daftar kunci [`Condition`].
    #[must_use]
    pub const fn key_index(&self) -> u8 {
        self.key_index
    }

    /// Mengembalikan tanda tangan Schnorr BIP340 64 byte.
    #[must_use]
    pub const fn sig(&self) -> &[u8; SIGNATURE_LEN] {
        &self.sig
    }

    /// Ukuran byte terenkode tetap untuk satu [`SigEntry`] (`65` byte).
    #[must_use]
    pub const fn encoded_size(&self) -> usize {
        MIN_SIG_ENTRY_SIZE
    }
}

impl Encode for SigEntry {
    fn encode_to(&self, out: &mut Vec<u8>) {
        write_u8(out, self.key_index);
        write_bytes(out, &self.sig);
    }
}

impl Decode for SigEntry {
    fn decode_from(r: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let key_index = r.read_u8()?;
        let sig = r.read_bytes::<SIGNATURE_LEN>()?;
        Ok(Self::new(key_index, sig))
    }
}

/// Isi witness jenis `1`: pengungkapan [`Condition`] beserta tepat `m` entri [`SigEntry`] terurut.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionWitness {
    condition: Condition,
    sigs: Vec<SigEntry>,
}

impl ConditionWitness {
    /// Membentuk [`ConditionWitness`] setelah memvalidasi:
    /// - `sigs.len() == usize::from(condition.m())`
    /// - setiap `key_index < condition.n()`
    /// - `key_index` urut naik ketat tanpa duplikat.
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError::InvalidWitnessSignatures`] jika salah satu syarat dilanggar.
    pub fn new(condition: Condition, sigs: Vec<SigEntry>) -> Result<Self, DecodeError> {
        if sigs.len() != usize::from(condition.m()) {
            return Err(DecodeError::InvalidWitnessSignatures);
        }
        let n = condition.n();
        for entry in &sigs {
            if entry.key_index() >= n {
                return Err(DecodeError::InvalidWitnessSignatures);
            }
        }
        for pair in sigs.windows(2) {
            if let [left, right] = pair
                && left.key_index() >= right.key_index()
            {
                return Err(DecodeError::InvalidWitnessSignatures);
            }
        }
        Ok(Self { condition, sigs })
    }

    /// Mengembalikan referensi ke [`Condition`] yang diungkapkan.
    #[must_use]
    pub const fn condition(&self) -> &Condition {
        &self.condition
    }

    /// Mengembalikan slice daftar tanda tangan [`SigEntry`].
    #[must_use]
    pub fn sigs(&self) -> &[SigEntry] {
        &self.sigs
    }
}

/// Data saksi pembelanjaan input pada transaksi biasa sesuai Spesifikasi Bagian 5.2.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Witness {
    /// Jenis `0`: satu tanda tangan Schnorr BIP340 64 byte.
    SingleKey([u8; SIGNATURE_LEN]),
    /// Jenis `1`: pengungkapan kondisi dan daftar tanda tangan tervalidasi.
    Condition(ConditionWitness),
}

impl Witness {
    /// Membentuk witness jenis `0` (kunci tunggal) dari tanda tangan 64 byte.
    #[must_use]
    pub const fn single_key(sig: [u8; SIGNATURE_LEN]) -> Self {
        Self::SingleKey(sig)
    }

    /// Membentuk witness jenis `1` (kondisi) setelah memvalidasi daftar [`SigEntry`].
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError::InvalidWitnessSignatures`] jika `sigs` tidak sesuai `condition`.
    pub fn condition(condition: Condition, sigs: Vec<SigEntry>) -> Result<Self, DecodeError> {
        Ok(Self::Condition(ConditionWitness::new(condition, sigs)?))
    }

    /// Menghitung ukuran byte terenkode kanonis witness ini.
    #[must_use]
    pub fn encoded_size(&self) -> usize {
        match self {
            Self::SingleKey(_) => MIN_WITNESS_SIZE,
            Self::Condition(cw) => {
                let sig_count = u64::try_from(cw.sigs().len()).unwrap_or(u64::MAX);
                let sigs_bytes = cw.sigs().len().saturating_mul(MIN_SIG_ENTRY_SIZE);
                1_usize
                    .saturating_add(cw.condition().encoded_size())
                    .saturating_add(varint_u64_len(sig_count))
                    .saturating_add(sigs_bytes)
            }
        }
    }
}

impl Encode for Witness {
    fn encode_to(&self, out: &mut Vec<u8>) {
        match self {
            Self::SingleKey(sig) => {
                write_u8(out, 0);
                write_bytes(out, sig);
            }
            Self::Condition(cw) => {
                write_u8(out, 1);
                cw.condition().encode_to(out);
                write_vec(out, cw.sigs());
            }
        }
    }
}

impl Decode for Witness {
    fn decode_from(r: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let kind = r.read_u8()?;
        match kind {
            0 => Ok(Self::SingleKey(r.read_bytes::<SIGNATURE_LEN>()?)),
            1 => {
                let condition = Condition::decode_from(r)?;
                let sig_count = r.read_vec_len(MIN_SIG_ENTRY_SIZE)?;
                if sig_count != usize::from(condition.m()) {
                    return Err(DecodeError::InvalidWitnessSignatures);
                }
                let mut sigs = Vec::with_capacity(sig_count);
                for _ in 0..sig_count {
                    sigs.push(SigEntry::decode_from(r)?);
                }
                Self::condition(condition, sigs)
            }
            actual => Err(DecodeError::UnknownWitnessKind { actual }),
        }
    }
}

/// Transaksi lengkap Homeostasis Coin (`body` + `witnesses`) sesuai Spesifikasi Bagian 5.
///
/// Menyediakan dua mode serialisasi:
/// - **Badan saja** ([`Transaction::encode_body_to`], [`Transaction::body_to_bytes`]) sebagai bahan `txid`.
/// - **Lengkap** ([`Encode::encode_to`], [`Encode::to_bytes`]) sebagai bahan `wtxid` dan penyimpanan blok.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    body: TxBody,
    witnesses: Vec<Witness>,
}

impl Transaction {
    /// Membentuk [`Transaction`] dari [`TxBody`] dan daftar [`Witness`].
    ///
    /// - Untuk transaksi biasa: `witnesses.len()` wajib sama dengan `inputs.len()`.
    /// - Untuk coinbase: `witnesses` wajib kosong.
    ///
    /// # Errors
    ///
    /// Mengembalikan [`DecodeError::WitnessCountMismatch`] jika jumlah witness tidak sesuai.
    pub fn from_parts(body: TxBody, witnesses: Vec<Witness>) -> Result<Self, DecodeError> {
        let expected = match &body {
            TxBody::Regular(reg) => reg.inputs().len(),
            TxBody::Coinbase(_) => 0,
        };
        if witnesses.len() != expected {
            return Err(DecodeError::WitnessCountMismatch {
                expected,
                actual: witnesses.len(),
            });
        }
        Ok(Self { body, witnesses })
    }

    /// Membentuk transaksi biasa dari daftar input, output, dan witness.
    ///
    /// # Errors
    ///
    /// Mengembalikan galat validasi [`RegularTxBody::new`] atau [`DecodeError::WitnessCountMismatch`].
    pub fn regular(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        witnesses: Vec<Witness>,
    ) -> Result<Self, DecodeError> {
        let body = TxBody::Regular(RegularTxBody::new(inputs, outputs)?);
        Self::from_parts(body, witnesses)
    }

    /// Membentuk transaksi coinbase tanpa witness.
    ///
    /// # Errors
    ///
    /// Mengembalikan galat validasi [`CoinbaseTxBody::new`].
    pub fn coinbase(
        height: BlockHeight,
        extra: Vec<u8>,
        outputs: Vec<Output>,
    ) -> Result<Self, DecodeError> {
        let body = TxBody::Coinbase(CoinbaseTxBody::new(height, extra, outputs)?);
        Self::from_parts(body, Vec::new())
    }

    /// Mengembalikan referensi ke badan transaksi ([`TxBody`]).
    #[must_use]
    pub const fn body(&self) -> &TxBody {
        &self.body
    }

    /// Mengembalikan slice witness transaksi (kosong untuk coinbase).
    #[must_use]
    pub fn witnesses(&self) -> &[Witness] {
        &self.witnesses
    }

    /// Mengembalikan `true` jika transaksi ini adalah coinbase.
    #[must_use]
    pub const fn is_coinbase(&self) -> bool {
        self.body.is_coinbase()
    }

    /// Mengembalikan `true` jika transaksi ini adalah transaksi biasa.
    #[must_use]
    pub const fn is_regular(&self) -> bool {
        self.body.is_regular()
    }

    /// Menulis encoding kanonis **badan saja** (tanpa witness, bahan `txid`) ke dalam `out`.
    pub fn encode_body_to(&self, out: &mut Vec<u8>) {
        self.body.encode_to(out);
    }

    /// Mengembalikan encoding kanonis **badan saja** (tanpa witness, bahan `txid`) sebagai `Vec<u8>`.
    #[must_use]
    pub fn body_to_bytes(&self) -> Vec<u8> {
        self.body.to_bytes()
    }

    /// Menghitung ukuran byte terenkode lengkap transaksi ini (`body + witnesses`).
    pub fn encoded_size(&self) -> Option<usize> {
        let mut total = self.body.encoded_size()?;
        for witness in &self.witnesses {
            total = total.checked_add(witness.encoded_size())?;
        }
        Some(total)
    }
}

impl Encode for Transaction {
    fn encode_to(&self, out: &mut Vec<u8>) {
        self.body.encode_to(out);
        for witness in &self.witnesses {
            witness.encode_to(out);
        }
    }
}

impl Decode for Transaction {
    fn decode_from(r: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let body = TxBody::decode_from(r)?;
        let witnesses = match &body {
            TxBody::Regular(reg) => {
                let count = reg.inputs().len();
                let max_possible = r.remaining().checked_div(MIN_WITNESS_SIZE).unwrap_or(0);
                let initial_cap = count.min(max_possible);
                let mut list = Vec::with_capacity(initial_cap);
                for _ in 0..count {
                    list.push(Witness::decode_from(r)?);
                }
                list
            }
            TxBody::Coinbase(_) => Vec::new(),
        };
        Self::from_parts(body, witnesses)
    }
}

#[cfg(test)]
mod tests;
