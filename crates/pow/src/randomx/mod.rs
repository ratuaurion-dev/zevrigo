//! PoW RandomX untuk testnet dan mainnet (DEC-01): mode ringan (cache 256 MiB) untuk verifikasi, rumus tinggi kunci,
//! layanan satu utas pemilik cache ([`RandomxService`], TSK-M06 bagian B), dan penambang mode ringan atau cepat
//! ([`RandomxMiner`], bagian C3).
//!
//! Hash dihitung pustaka C++ resmi RandomX lewat crate `randomx-rs` (DEC-10 bagian 10), dibangun dari salinan di
//! `vendor/randomx-rs`. `unsafe` dan floating point
//! hanya ada di dalam pustaka itu; modul ini tetap tanpa keduanya. Hash PoW dari 96 byte header dibaca sebagai bilangan
//! 256-bit big-endian dan harus ≤ target, sama dengan PoW regtest.
//!
//! Varian ZVRG (Konsep Bagian 5.10, DEC-01): salt Argon2 `RandomZVRG\x01`, satu-satunya perbedaan dari RandomX resmi
//! (vendor/README.md). Pemilihan jaringan dan sumber kunci adalah aturan konsensus MOD-04 (TSK-M06 bagian B).

use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::thread;

use randomx_rs::{RandomXCache, RandomXDataset, RandomXError, RandomXFlag, RandomXVM};
use zevrigo_primitives::block::Header;
use zevrigo_primitives::hash::Hash32;
use zevrigo_primitives::height::BlockHeight;
use zevrigo_primitives::params::{RANDOMX_KEY_INTERVAL, RANDOMX_KEY_LAG};

use crate::target::Target;

/// Tinggi terakhir yang masih memakai kunci genesis (`2_048 + 64`).
const LAST_GENESIS_KEY_HEIGHT: u64 = RANDOMX_KEY_INTERVAL + RANDOMX_KEY_LAG;

/// Jarak dari blok yang diperiksa ke batas jeda: jeda `64` blok dihitung dari induk (`64 + 1`).
const LAG_FROM_BLOCK: u64 = RANDOMX_KEY_LAG + 1;

/// Topeng kelipatan interval kunci (`!(2_048 - 1)`).
const INTERVAL_MASK: u64 = !(RANDOMX_KEY_INTERVAL - 1);

/// Jumlah kunci yang pemeriksanya disimpan [`RandomxService`]: kunci aktif dan kunci sebelumnya, supaya reorg melewati
/// batas kunci tidak membangun cache ulang. Masing-masing 256 MiB.
pub const SERVICE_KEYS: usize = 2;

/// Galat PoW RandomX.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RandomxError {
    /// Pustaka RandomX gagal (alokasi cache atau VM, atau input kosong).
    #[error("pustaka RandomX gagal: {0}")]
    Library(String),
    /// Hash dari pustaka tidak 32 byte.
    #[error("hash RandomX tidak 32 byte")]
    HashLength,
    /// Hash PoW berada di atas target kesulitan.
    #[error("hash PoW RandomX di atas target")]
    AboveTarget {
        /// Nilai hash PoW yang dihasilkan.
        pow_hash: Hash32,
        /// Target kesulitan yang harus dipenuhi.
        target: Target,
    },
    /// Utas layanan RandomX tidak bisa dimulai atau sudah berhenti.
    #[error("layanan RandomX gagal: {0}")]
    Service(String),
}

fn library(error: RandomXError) -> RandomxError {
    RandomxError::Library(error.to_string())
}

/// Tinggi blok kunci RandomX untuk blok di tinggi `height`.
///
/// Sampai tinggi `2_112` kuncinya blok genesis (tinggi 0). Sesudahnya, kelipatan `2_048` terbesar yang tidak lebih dari
/// `height − 65`, sehingga blok kunci sudah tertimbun paling sedikit 64 blok sampai induk. Rumus sama dengan Monero:
/// kunci berganti setiap 2_048 blok dengan jeda 64 blok (Konsep Bagian 5.10).
#[must_use]
pub const fn key_height(height: BlockHeight) -> BlockHeight {
    let value = height.get();
    if value <= LAST_GENESIS_KEY_HEIGHT {
        return BlockHeight::GENESIS;
    }
    match value.checked_sub(LAG_FROM_BLOCK) {
        Some(base) => BlockHeight::new(base & INTERVAL_MASK),
        None => BlockHeight::GENESIS,
    }
}

/// Pemeriksa PoW RandomX mode ringan untuk satu kunci.
///
/// Kunci adalah `block_id` blok di [`key_height`]. Membuat pemeriksa menyiapkan cache 256 MiB (Argon2d), jadi satu
/// pemeriksa dipakai untuk semua blok dengan kunci yang sama. Tipe ini tidak `Send`: pemiliknya satu utas.
pub struct RandomxVerifier {
    key: Hash32,
    vm: RandomXVM,
}

impl RandomxVerifier {
    /// Menyiapkan cache dan VM mode ringan untuk `key`, dengan flag yang dianjurkan pustaka untuk CPU ini (JIT dan
    /// AES perangkat keras jika ada; hasil hash sama untuk semua flag).
    ///
    /// # Errors
    ///
    /// [`RandomxError::Library`] jika cache atau VM tidak bisa dialokasikan.
    pub fn new(key: Hash32) -> Result<Self, RandomxError> {
        let vm = light_vm(key.as_bytes())?;
        Ok(Self { key, vm })
    }

    /// Kunci pemeriksa ini.
    #[must_use]
    pub const fn key(&self) -> Hash32 {
        self.key
    }

    /// Hash PoW RandomX dari 96 byte header.
    ///
    /// # Errors
    ///
    /// [`RandomxError::Library`] atau [`RandomxError::HashLength`] jika pustaka gagal.
    pub fn hash(&self, header: &Header) -> Result<Hash32, RandomxError> {
        hash_with(&self.vm, &header.to_array())
    }

    /// Memeriksa bahwa hash PoW RandomX header ≤ `target`.
    ///
    /// # Errors
    ///
    /// [`RandomxError::AboveTarget`] jika hash di atas target, atau galat dari [`RandomxVerifier::hash`].
    pub fn verify(&self, header: &Header, target: &Target) -> Result<(), RandomxError> {
        let pow_hash = self.hash(header)?;
        if target.is_met_by(&pow_hash) {
            Ok(())
        } else {
            Err(RandomxError::AboveTarget {
                pow_hash,
                target: *target,
            })
        }
    }
}

/// Mode VM RandomX untuk penambang.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RandomxMode {
    /// Mode ringan: cache 256 MiB, sama dengan pemeriksa node. Lambat, untuk perangkat kecil.
    Light,
    /// Mode cepat: dataset sekitar 2 GiB disusun dari cache sekali per kunci (lama, satu utas), lalu hash jauh lebih
    /// cepat. Hasil hash sama dengan mode ringan.
    Fast,
}

/// Penambang RandomX untuk satu kunci (TSK-M06 bagian C3). Tidak `Send`: pemiliknya satu utas.
///
/// Pustaka `randomx-rs` tidak menandai cache dan dataset aman dipakai bersama antar utas, jadi satu penambang memakai
/// satu utas. Beberapa utas mode cepat butuh dataset bersama dan menunggu perubahan pustaka itu.
pub struct RandomxMiner {
    key: Hash32,
    mode: RandomxMode,
    vm: RandomXVM,
}

impl RandomxMiner {
    /// Menyiapkan VM untuk `key`. Mode cepat menyusun dataset dulu, yang bisa makan waktu sekitar satu menit.
    ///
    /// # Errors
    ///
    /// [`RandomxError::Library`] jika cache, dataset, atau VM tidak bisa dialokasikan.
    pub fn new(key: Hash32, mode: RandomxMode) -> Result<Self, RandomxError> {
        let vm = match mode {
            RandomxMode::Light => light_vm(key.as_bytes())?,
            RandomxMode::Fast => fast_vm(key.as_bytes())?,
        };
        Ok(Self { key, mode, vm })
    }

    /// Kunci penambang ini.
    #[must_use]
    pub const fn key(&self) -> Hash32 {
        self.key
    }

    /// Mode VM penambang ini.
    #[must_use]
    pub const fn mode(&self) -> RandomxMode {
        self.mode
    }

    /// Hash PoW RandomX dari 96 byte header.
    ///
    /// # Errors
    ///
    /// [`RandomxError::Library`] atau [`RandomxError::HashLength`] jika pustaka gagal.
    pub fn hash(&self, header: &Header) -> Result<Hash32, RandomxError> {
        hash_with(&self.vm, &header.to_array())
    }

    /// Mencari nonce agar hash RandomX ≤ `target`, mulai dari `header.nonce()`, paling banyak `max_attempts`
    /// percobaan dengan `checked_add` (sama dengan `mine_regtest`). `None` jika `max_attempts == 0`, semua percobaan
    /// gagal, atau nonce meluap.
    ///
    /// # Errors
    ///
    /// Galat [`RandomxMiner::hash`].
    pub fn mine(
        &self,
        header: &Header,
        target: &Target,
        max_attempts: u64,
    ) -> Result<Option<Header>, RandomxError> {
        let start_nonce = header.nonce();
        for i in 0..max_attempts {
            let Some(nonce) = start_nonce.checked_add(i) else {
                return Ok(None);
            };
            let candidate = Header::new(
                header.height(),
                header.prev_block_id(),
                header.merkle_root(),
                header.timestamp(),
                header.target(),
                nonce,
            );
            if target.is_met_by(&self.hash(&candidate)?) {
                return Ok(Some(candidate));
            }
        }
        Ok(None)
    }
}

/// Dataset dan VM mode cepat untuk `key`: cache dengan flag anjuran pustaka, dataset penuh, VM `FLAG_FULL_MEM`.
fn fast_vm(key: &[u8]) -> Result<RandomXVM, RandomxError> {
    let flags = RandomXFlag::get_recommended_flags();
    let cache = RandomXCache::new(flags, key).map_err(library)?;
    let dataset = RandomXDataset::new(flags, cache, 0).map_err(library)?;
    RandomXVM::new(flags | RandomXFlag::FLAG_FULL_MEM, None, Some(dataset)).map_err(library)
}

/// Cache dan VM mode ringan untuk kunci sembarang panjang (vektor uji resmi memakai kunci teks).
pub(crate) fn light_vm(key: &[u8]) -> Result<RandomXVM, RandomxError> {
    let flags = RandomXFlag::get_recommended_flags();
    let cache = RandomXCache::new(flags, key).map_err(library)?;
    RandomXVM::new(flags, Some(cache), None).map_err(library)
}

/// Hash RandomX dari `input` sembarang dengan `vm`.
pub(crate) fn hash_with(vm: &RandomXVM, input: &[u8]) -> Result<Hash32, RandomxError> {
    let output = vm.calculate_hash(input).map_err(library)?;
    let bytes: [u8; 32] = output
        .as_slice()
        .try_into()
        .map_err(|_| RandomxError::HashLength)?;
    Ok(Hash32::from_bytes(bytes))
}

/// Layanan RandomX mode ringan di satu utas sendiri (TSK-M06 bagian B).
///
/// Cache dan VM pustaka RandomX tidak boleh berpindah utas, jadi satu utas memiliki semuanya dan menjawab permintaan hash
/// lewat kanal, berurutan. Utas menyimpan pemeriksa untuk [`SERVICE_KEYS`] kunci terakhir, dibuat saat kunci itu pertama
/// kali diminta. Handle ini `Send` dan `Sync`; utasnya berhenti setelah handle dilepas.
pub struct RandomxService {
    sender: Sender<Request>,
}

/// Satu permintaan hash ke utas layanan.
struct Request {
    key: Hash32,
    header: Header,
    reply: SyncSender<Result<Hash32, RandomxError>>,
}

impl RandomxService {
    /// Memulai utas layanan. Belum ada cache yang dibuat.
    ///
    /// # Errors
    ///
    /// [`RandomxError::Service`] jika utas tidak bisa dibuat.
    pub fn start() -> Result<Self, RandomxError> {
        let (sender, receiver) = mpsc::channel::<Request>();
        // Utas tidak ditunggu: utas berhenti sendiri setelah semua handle dilepas.
        let _worker = thread::Builder::new()
            .name("zevrigo-randomx".to_owned())
            .spawn(move || serve(&receiver))
            .map_err(|error| RandomxError::Service(error.to_string()))?;
        Ok(Self { sender })
    }

    /// Hash PoW RandomX dari 96 byte `header` dengan kunci `key`, dihitung utas layanan.
    ///
    /// # Errors
    ///
    /// [`RandomxError::Service`] jika utas sudah berhenti, atau galat [`RandomxVerifier`] untuk kunci itu.
    pub fn hash(&self, key: Hash32, header: &Header) -> Result<Hash32, RandomxError> {
        let (reply, answer) = mpsc::sync_channel(1);
        self.sender
            .send(Request {
                key,
                header: *header,
                reply,
            })
            .map_err(|_| stopped())?;
        answer.recv().map_err(|_| stopped())?
    }
}

fn stopped() -> RandomxError {
    RandomxError::Service("utas RandomX sudah berhenti".to_owned())
}

/// Isi utas layanan: menjawab permintaan sampai semua handle dilepas.
fn serve(receiver: &Receiver<Request>) {
    let mut verifiers: Vec<RandomxVerifier> = Vec::new();
    while let Ok(request) = receiver.recv() {
        let result = hash_cached(&mut verifiers, request.key, &request.header);
        // Peminta yang sudah pergi tidak menghentikan layanan.
        let _sent = request.reply.send(result);
    }
}

/// Hash dengan pemeriksa untuk `key`; pemeriksa yang baru dipakai di depan, paling banyak [`SERVICE_KEYS`].
fn hash_cached(
    verifiers: &mut Vec<RandomxVerifier>,
    key: Hash32,
    header: &Header,
) -> Result<Hash32, RandomxError> {
    let verifier = match verifiers.iter().position(|verifier| verifier.key() == key) {
        Some(index) => verifiers.remove(index),
        None => {
            verifiers.truncate(SERVICE_KEYS.saturating_sub(1));
            RandomxVerifier::new(key)?
        }
    };
    let result = verifier.hash(header);
    verifiers.insert(0, verifier);
    verifiers.truncate(SERVICE_KEYS);
    result
}

#[cfg(test)]
mod tests;
