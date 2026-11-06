//! Parameter per jaringan ([`ChainParams`]) dan blok genesis regtest dan testnet.
//!
//! Sesuai `docs/reference/spesifikasi-validasi-v0.1.md` Bagian 2, 10, dan 10b. Parameter testnet disetujui pencipta
//! 5 November 2026 (DEC-01, TSK-M06 bagian C2); cap waktu genesis testnet ditetapkan saat peluncuran (TSK-F06). Nilai
//! mainnet ditetapkan dari pengukuran testnet.

use homeostasis_crypto::{merkle_root, network_id, wtxid};
use homeostasis_pow::Target;
use homeostasis_primitives::params::GENESIS_ALLOCATION;
use homeostasis_primitives::{
    Block, BlockHeight, DecodeError, GENESIS_EXTRA_LEN, Hash32, Header, Network, Output,
    OutputKind, Transaction,
};

use crate::difficulty::{DifficultyError, decode_compact};

/// `pow_limit` regtest dalam format compact (`0x7fffff × 2^232`).
pub const REGTEST_POW_LIMIT_COMPACT: u32 = 0x207f_ffff;

/// Target genesis regtest dalam format compact (sama dengan `pow_limit` regtest).
pub const REGTEST_GENESIS_TARGET_COMPACT: u32 = REGTEST_POW_LIMIT_COMPACT;

/// Cap waktu genesis regtest: 28 Oktober 2026 00.00 UTC.
pub const REGTEST_GENESIS_TIMESTAMP: u64 = 1_793_145_600;

/// Kunci x-only penerima alokasi genesis regtest: kunci publik dari kunci rahasia `1`.
///
/// Sengaja publik, karena koin regtest tidak bernilai dan test perlu membelanjakan alokasi genesis.
pub const REGTEST_GENESIS_KEY: [u8; 32] = [
    0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b, 0x07,
    0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98,
];

/// `pow_limit` testnet dalam format compact: kerja 1_200 hash per blok, sekitar 10 hash per detik pada waktu blok
/// 120 detik (disetujui pencipta 5 November 2026).
pub const TESTNET_POW_LIMIT_COMPACT: u32 = 0x1f36_9d03;

/// Target genesis testnet dalam format compact (kesulitan awal): kerja 120_000 hash per blok, sekitar 1_000 hash per
/// detik (disetujui pencipta 5 November 2026).
pub const TESTNET_GENESIS_TARGET_COMPACT: u32 = 0x1e8b_cf64;

/// Cap waktu genesis testnet, sekaligus acuan ASERT.
///
/// Nilai `0` sengaja tidak sah untuk peluncuran: dengan acuan 1970, ASERT memberi `pow_limit` untuk blok sesudah
/// genesis. Nilai sebenarnya adalah saat peluncuran testnet publik, dan mengubahnya mengubah `network_id` testnet.
// TODO(param): cap waktu genesis testnet, ditetapkan saat peluncuran testnet publik (TSK-F06).
pub const TESTNET_GENESIS_TIMESTAMP: u64 = 0;

/// `extra` coinbase genesis testnet: ASCII `homeostasis testnet 1`, diisi nol sampai 32 byte. Tanpa jangkar blok
/// Bitcoin (khusus mainnet, Konsep L2).
pub const TESTNET_GENESIS_EXTRA: [u8; GENESIS_EXTRA_LEN] =
    *b"homeostasis testnet 1\0\0\0\0\0\0\0\0\0\0\0";

/// Penerima alokasi genesis testnet: koordinat x titik NUMS `H` BIP341, yang kunci rahasianya tidak diketahui siapa pun.
/// Alokasi `G` testnet tetap dihitung di pasokan, tetapi tidak bisa dibelanjakan.
pub const TESTNET_GENESIS_KEY: [u8; 32] = [
    0x50, 0x92, 0x9b, 0x74, 0xc1, 0xa0, 0x49, 0x54, 0xb7, 0x8b, 0x4b, 0x60, 0x35, 0xe9, 0x7a, 0x5e,
    0x07, 0x8a, 0x5a, 0x0f, 0x28, 0xec, 0x96, 0xd5, 0x47, 0xbf, 0xee, 0x9a, 0xce, 0x80, 0x3a, 0xc0,
];

/// `pow_limit` jaringan dalam format compact.
///
/// Mainnet memakai placeholder `0`, yang sengaja tidak sah sehingga [`pow_limit`] gagal.
#[must_use]
pub const fn pow_limit_compact(network: Network) -> u32 {
    match network {
        Network::Regtest => REGTEST_POW_LIMIT_COMPACT,
        Network::Testnet => TESTNET_POW_LIMIT_COMPACT,
        // TODO(param): pow_limit mainnet, ditetapkan di spesifikasi genesis yang diumumkan (L1).
        Network::Mainnet => 0,
    }
}

/// `pow_limit` jaringan sebagai [`Target`] 256-bit.
///
/// # Errors
///
/// [`DifficultyError::InvalidCompact`] untuk mainnet sampai `pow_limit`-nya ditetapkan.
pub fn pow_limit(network: Network) -> Result<Target, DifficultyError> {
    decode_compact(pow_limit_compact(network))
}

/// Blok genesis regtest (Spesifikasi Validasi Bagian 10).
///
/// Header: tinggi 0, `prev_block_id` nol, cap waktu [`REGTEST_GENESIS_TIMESTAMP`], target
/// [`REGTEST_GENESIS_TARGET_COMPACT`], nonce 0. Coinbase: `extra` 32 byte nol dan satu output alokasi
/// pencipta `G` ke [`REGTEST_GENESIS_KEY`]. PoW genesis tidak diperiksa.
///
/// # Errors
///
/// [`DecodeError`] dari konstruktor MOD-01. Tidak terjadi untuk nilai tetap ini, dan dibuktikan oleh test.
pub fn regtest_genesis() -> Result<Block, DecodeError> {
    genesis_block(
        vec![0_u8; GENESIS_EXTRA_LEN],
        REGTEST_GENESIS_KEY,
        REGTEST_GENESIS_TIMESTAMP,
        REGTEST_GENESIS_TARGET_COMPACT,
    )
}

/// Blok genesis testnet (Spesifikasi Validasi Bagian 10b): bentuk sama dengan genesis regtest, dengan `extra`
/// [`TESTNET_GENESIS_EXTRA`], alokasi `G` ke [`TESTNET_GENESIS_KEY`], cap waktu [`TESTNET_GENESIS_TIMESTAMP`], dan target
/// [`TESTNET_GENESIS_TARGET_COMPACT`]. PoW genesis tidak diperiksa.
///
/// # Errors
///
/// [`DecodeError`] dari konstruktor MOD-01. Tidak terjadi untuk nilai tetap ini, dan dibuktikan oleh test.
pub fn testnet_genesis() -> Result<Block, DecodeError> {
    genesis_block(
        TESTNET_GENESIS_EXTRA.to_vec(),
        TESTNET_GENESIS_KEY,
        TESTNET_GENESIS_TIMESTAMP,
        TESTNET_GENESIS_TARGET_COMPACT,
    )
}

/// Genesis tinggi 0: coinbase dengan `extra` dan satu output alokasi `G` ke `key`, nonce 0.
fn genesis_block(
    extra: Vec<u8>,
    key: [u8; 32],
    timestamp: u64,
    target: u32,
) -> Result<Block, DecodeError> {
    let allocation = Output::new(GENESIS_ALLOCATION, OutputKind::SingleKey, key)?;
    let coinbase = Transaction::coinbase(BlockHeight::GENESIS, extra, vec![allocation])?;
    let root = merkle_root(&[wtxid(&coinbase)]).ok_or(DecodeError::EmptyBlock)?;
    let header = Header::new(
        BlockHeight::GENESIS,
        Hash32::ZERO,
        root,
        timestamp,
        target,
        0,
    );
    Block::new(header, vec![coinbase])
}

/// Genesis yang dikenal (regtest atau testnet) dengan `block_id` = `network_id`, atau `None`. Node memilih genesis
/// dengan fungsi ini, sehingga genesis selalu cocok dengan `network_id` parameternya.
///
/// # Errors
///
/// Sama dengan [`regtest_genesis`].
pub fn known_genesis(network_id: &Hash32) -> Result<Option<Block>, DecodeError> {
    for genesis in [regtest_genesis()?, testnet_genesis()?] {
        if network_id_of(&genesis) == *network_id {
            return Ok(Some(genesis));
        }
    }
    Ok(None)
}

fn network_id_of(genesis: &Block) -> Hash32 {
    network_id(genesis.header())
}

/// ID jaringan regtest: `block_id` genesis regtest.
///
/// # Errors
///
/// Sama dengan [`regtest_genesis`].
pub fn regtest_network_id() -> Result<Hash32, DecodeError> {
    Ok(network_id(regtest_genesis()?.header()))
}

/// ID jaringan testnet: `block_id` genesis testnet.
///
/// # Errors
///
/// Sama dengan [`testnet_genesis`].
pub fn testnet_network_id() -> Result<Hash32, DecodeError> {
    Ok(network_id(testnet_genesis()?.header()))
}

/// Galat saat menyusun [`ChainParams`].
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ParamsError {
    /// Parameter jaringan ini belum ditetapkan (mainnet: sesudah testnet).
    #[error("parameter jaringan {network:?} belum ditetapkan")]
    Unavailable {
        /// Jaringan yang diminta.
        network: Network,
    },
    /// Target compact parameter jaringan tidak sah.
    #[error("target parameter jaringan tidak sah: {0}")]
    Difficulty(#[from] DifficultyError),
    /// Blok genesis tidak bisa dibentuk.
    #[error("blok genesis tidak sah: {0}")]
    Genesis(#[from] DecodeError),
}

/// Parameter konsensus satu jaringan (Spesifikasi Validasi Bagian 2).
///
/// Aturan sama di semua jaringan. Yang berbeda hanya fungsi PoW (dipilih dari [`Network`]), `pow_limit`,
/// dan genesis (target, cap waktu, dan `network_id`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChainParams {
    network: Network,
    pow_limit: Target,
    genesis_target: Target,
    genesis_timestamp: u64,
    network_id: Hash32,
}

impl ChainParams {
    /// Menyusun parameter dari nilai mentah.
    ///
    /// Dipakai test, dan nanti testnet serta mainnet setelah parameternya ditetapkan (`TODO(param)` di
    /// [`pow_limit_compact`]).
    #[must_use]
    pub const fn new(
        network: Network,
        pow_limit: Target,
        genesis_target: Target,
        genesis_timestamp: u64,
        network_id: Hash32,
    ) -> Self {
        Self {
            network,
            pow_limit,
            genesis_target,
            genesis_timestamp,
            network_id,
        }
    }

    /// Parameter regtest: `pow_limit` [`REGTEST_POW_LIMIT_COMPACT`] dan genesis [`regtest_genesis`].
    ///
    /// # Errors
    ///
    /// [`ParamsError`] jika konstanta regtest tidak sah. Tidak terjadi, dan dibuktikan oleh test.
    pub fn regtest() -> Result<Self, ParamsError> {
        let genesis = regtest_genesis()?;
        let header = genesis.header();
        Ok(Self::new(
            Network::Regtest,
            pow_limit(Network::Regtest)?,
            decode_compact(header.target())?,
            header.timestamp(),
            network_id(header),
        ))
    }

    /// Parameter testnet: `pow_limit` [`TESTNET_POW_LIMIT_COMPACT`] dan genesis [`testnet_genesis`].
    ///
    /// # Errors
    ///
    /// [`ParamsError`] jika konstanta testnet tidak sah. Tidak terjadi, dan dibuktikan oleh test.
    pub fn testnet() -> Result<Self, ParamsError> {
        let genesis = testnet_genesis()?;
        let header = genesis.header();
        Ok(Self::new(
            Network::Testnet,
            pow_limit(Network::Testnet)?,
            decode_compact(header.target())?,
            header.timestamp(),
            network_id(header),
        ))
    }

    /// Parameter jaringan `network`: [`ChainParams::regtest`] atau [`ChainParams::testnet`].
    ///
    /// # Errors
    ///
    /// [`ParamsError::Unavailable`] untuk mainnet, yang parameternya belum ditetapkan.
    pub fn for_network(network: Network) -> Result<Self, ParamsError> {
        match network {
            Network::Regtest => Self::regtest(),
            Network::Testnet => Self::testnet(),
            Network::Mainnet => Err(ParamsError::Unavailable { network }),
        }
    }

    /// Jaringan parameter ini.
    #[must_use]
    pub const fn network(&self) -> Network {
        self.network
    }

    /// Target terbesar (kesulitan terendah) yang boleh dipakai.
    #[must_use]
    pub const fn pow_limit(&self) -> Target {
        self.pow_limit
    }

    /// Target genesis, sekaligus target acuan ASERT.
    #[must_use]
    pub const fn genesis_target(&self) -> Target {
        self.genesis_target
    }

    /// Cap waktu genesis, sekaligus cap waktu acuan ASERT.
    #[must_use]
    pub const fn genesis_timestamp(&self) -> u64 {
        self.genesis_timestamp
    }

    /// ID jaringan: `block_id` genesis. Dipakai di sighash v1 (DEC-13).
    #[must_use]
    pub const fn network_id(&self) -> Hash32 {
        self.network_id
    }
}

#[cfg(test)]
mod tests;
