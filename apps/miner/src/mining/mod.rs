//! Loop penambangan: template dari node, cari nonce dengan PoW dev regtest atau RandomX (MOD-03), lalu kirim blok.
//!
//! RandomX (testnet, TSK-M06 bagian C3) memakai kunci `pow_key` dari template, sehingga miner tidak menulis ulang aturan
//! kunci. VM RandomX disimpan selama kuncinya sama; mode cepat menyusun dataset sekitar 2 GiB setiap kunci berganti.
//!
//! Satu putaran ([`Miner::round`]) mencoba paling banyak `attempts` nonce untuk satu template. Jika belum ketemu,
//! putaran berikutnya mengambil template baru, yang memuat transaksi mempool dan cap waktu terbaru dari node,
//! sehingga ruang pencarian berganti. Panggilan RPC memakai runtime tokio satu thread milik miner; pencarian nonce
//! berjalan di luar runtime.

use std::cell::RefCell;

use homeostasis_pow::{RandomxMiner, RandomxMode, Target, mine_regtest};
use homeostasis_primitives::{Block, Decode, Encode, Hash32, Header, Network};
use homeostasis_rpc::{Hex32, KindNumber, RpcClient, SubmitStatus};
use thiserror::Error;
use tokio::runtime::Runtime;

/// Jumlah nonce PoW dev regtest yang dicoba untuk satu template sebelum template diambil ulang.
pub const ATTEMPTS_PER_TEMPLATE: u64 = 1_000_000;

/// Jumlah nonce RandomX untuk satu template: beberapa detik di mode cepat, sekitar satu menit di mode ringan, sehingga
/// template (cap waktu, transaksi mempool, ujung chain) cukup sering diperbarui.
pub const RANDOMX_ATTEMPTS_PER_TEMPLATE: u64 = 2_000;

/// PoW yang dipakai miner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MinerPow {
    /// PoW dev regtest (DEC-09).
    Regtest,
    /// RandomX testnet dan mainnet (DEC-01) dengan mode VM ini.
    Randomx(RandomxMode),
}

/// Galat miner. Biner keluar dengan kode 1.
#[derive(Debug, Error)]
pub enum MinerError {
    /// Runtime tokio tidak bisa dibuat.
    #[error("runtime gagal dibuat: {0}")]
    Runtime(#[from] std::io::Error),
    /// Panggilan RPC gagal (node tidak bisa dihubungi atau membalas error).
    #[error("RPC gagal: {message}")]
    Rpc {
        /// Keterangan dari klien RPC.
        message: String,
    },
    /// Template dari node tidak bisa dipakai.
    #[error("template blok tidak sah: {message}")]
    BadTemplate {
        /// Keterangan galat.
        message: String,
    },
    /// Node menolak blok hasil tambang sebagai tidak sah. Ini menandakan bug di miner atau node.
    #[error("node menolak blok: aturan {reason}")]
    Rejected {
        /// Nama aturan yang dilanggar, dari node.
        reason: String,
    },
    /// Node berjalan di jaringan lain dari yang dipilih.
    #[error("node berjalan di jaringan {actual}, miner untuk {expected}")]
    WrongNetwork {
        /// Jaringan yang dipilih untuk miner.
        expected: String,
        /// Jaringan node.
        actual: String,
    },
    /// Pustaka RandomX gagal (alokasi cache, dataset, atau VM).
    #[error("RandomX gagal: {message}")]
    Pow {
        /// Keterangan galat.
        message: String,
    },
}

/// Hasil satu putaran.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Round {
    /// Blok ditemukan dan diterima node.
    Accepted {
        /// Tinggi blok.
        height: u64,
        /// ID blok dari node.
        block_id: Option<Hex32>,
        /// Nonce yang memenuhi target.
        nonce: u64,
    },
    /// Blok ditemukan, tetapi node tidak memasukkannya ke chain (`duplicate`, `orphan`, atau `deferred`), misalnya
    /// karena ujung chain sudah berganti.
    Stale {
        /// Status dari node.
        status: SubmitStatus,
    },
    /// Nonce belum ditemukan dalam batas percobaan untuk template ini.
    Exhausted,
}

/// Galat RPC sebagai teks, agar tipe galat klien tidak ikut terekspos.
fn rpc_error(error: impl std::fmt::Display) -> MinerError {
    MinerError::Rpc {
        message: error.to_string(),
    }
}

/// Galat template sebagai teks.
fn bad_template(message: impl std::fmt::Display) -> MinerError {
    MinerError::BadTemplate {
        message: message.to_string(),
    }
}

/// Galat RandomX sebagai teks.
fn pow_error(error: impl std::fmt::Display) -> MinerError {
    MinerError::Pow {
        message: error.to_string(),
    }
}

/// Nama jaringan seperti di RPC.
fn network_name(network: Network) -> String {
    String::from(homeostasis_rpc::NetworkName::new(network))
}

/// Miner yang terhubung ke satu node. Tidak `Send` jika memakai RandomX (VM pustaka milik satu utas).
pub struct Miner {
    client: RpcClient,
    runtime: Runtime,
    payout_kind: KindNumber,
    payout_payload: Hex32,
    attempts: u64,
    pow: MinerPow,
    /// VM RandomX untuk kunci terakhir.
    randomx: RefCell<Option<RandomxMiner>>,
}

impl Miner {
    /// Menyiapkan miner PoW dev regtest untuk node di `rpc_url`. Belum ada koneksi sampai putaran pertama.
    ///
    /// # Errors
    ///
    /// [`MinerError::Runtime`] jika runtime gagal dibuat, [`MinerError::Rpc`] jika `rpc_url` tidak sah.
    pub fn connect(
        rpc_url: &str,
        payout_kind: KindNumber,
        payout_payload: Hex32,
        attempts: u64,
    ) -> Result<Self, MinerError> {
        Self::connect_with_pow(
            rpc_url,
            payout_kind,
            payout_payload,
            attempts,
            MinerPow::Regtest,
        )
    }

    /// Seperti [`Miner::connect`], dengan PoW `pow`.
    ///
    /// # Errors
    ///
    /// Sama dengan [`Miner::connect`].
    pub fn connect_with_pow(
        rpc_url: &str,
        payout_kind: KindNumber,
        payout_payload: Hex32,
        attempts: u64,
        pow: MinerPow,
    ) -> Result<Self, MinerError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let client = runtime
            .block_on(async { RpcClient::new(rpc_url) })
            .map_err(rpc_error)?;
        Ok(Self {
            client,
            runtime,
            payout_kind,
            payout_payload,
            attempts,
            pow,
            randomx: RefCell::new(None),
        })
    }

    /// PoW miner ini.
    #[must_use]
    pub const fn pow(&self) -> MinerPow {
        self.pow
    }

    /// Memastikan node berjalan di jaringan `expected` (`get_chain_info`), agar miner tidak menambang dengan PoW yang
    /// salah.
    ///
    /// # Errors
    ///
    /// [`MinerError::Rpc`] jika node tidak bisa dihubungi, [`MinerError::WrongNetwork`] jika jaringannya lain.
    pub fn check_network(&self, expected: Network) -> Result<(), MinerError> {
        let info = self
            .runtime
            .block_on(self.client.get_chain_info())
            .map_err(rpc_error)?;
        let actual = info.network.network();
        if actual == expected {
            Ok(())
        } else {
            Err(MinerError::WrongNetwork {
                expected: network_name(expected),
                actual: network_name(actual),
            })
        }
    }

    /// Mencari nonce RandomX dengan kunci `key`; VM dibuat ulang hanya jika kunci atau mode berganti.
    fn mine_randomx(
        &self,
        mode: RandomxMode,
        key: Hash32,
        header: &Header,
        target: &Target,
    ) -> Result<Option<Header>, MinerError> {
        let mut slot = self.randomx.borrow_mut();
        let reuse = slot
            .as_ref()
            .is_some_and(|miner| miner.key() == key && miner.mode() == mode);
        if !reuse {
            // VM lama dilepas dulu, agar dataset lama (mode cepat, sekitar 2 GiB) tidak bertahan bersama yang baru.
            *slot = None;
            *slot = Some(RandomxMiner::new(key, mode).map_err(pow_error)?);
        }
        let Some(miner) = slot.as_ref() else {
            return Err(pow_error("VM RandomX tidak tersedia"));
        };
        miner.mine(header, target, self.attempts).map_err(pow_error)
    }

    /// Satu putaran: ambil template, cari nonce, lalu kirim blok jika ketemu.
    ///
    /// # Errors
    ///
    /// [`MinerError::Rpc`], [`MinerError::BadTemplate`], atau [`MinerError::Rejected`] jika node menyatakan blok
    /// tidak sah.
    pub fn round(&self) -> Result<Round, MinerError> {
        let template = self
            .runtime
            .block_on(
                self.client
                    .get_block_template(self.payout_kind, self.payout_payload),
            )
            .map_err(rpc_error)?;
        let block = Block::from_bytes(template.hex.as_slice()).map_err(bad_template)?;
        let header = block.header();
        if header.height().get() != template.height || header.target() != template.target.get() {
            return Err(bad_template("header tidak sesuai dengan isi template"));
        }
        let target = Target::from_be_bytes(*template.full_target.bytes());
        let found = match self.pow {
            MinerPow::Regtest => mine_regtest(header, &target, self.attempts),
            MinerPow::Randomx(mode) => {
                self.mine_randomx(mode, template.pow_key.to_hash32(), header, &target)?
            }
        };
        let Some(found) = found else {
            return Ok(Round::Exhausted);
        };
        let mined = Block::new(found, block.txs().to_vec()).map_err(bad_template)?;
        let result = self
            .runtime
            .block_on(self.client.submit_block(&mined.to_bytes()))
            .map_err(rpc_error)?;
        match result.status {
            SubmitStatus::Accepted => Ok(Round::Accepted {
                height: template.height,
                block_id: result.block_id,
                nonce: found.nonce(),
            }),
            SubmitStatus::Invalid => Err(MinerError::Rejected {
                reason: result.reason.unwrap_or_else(|| "tidak disebut".to_owned()),
            }),
            status @ (SubmitStatus::Duplicate | SubmitStatus::Orphan | SubmitStatus::Deferred) => {
                Ok(Round::Stale { status })
            }
        }
    }

    /// Menambang sampai `blocks` blok diterima node (`0` berarti terus). `on_round` dipanggil setiap putaran.
    /// Mengembalikan jumlah blok yang diterima.
    ///
    /// # Errors
    ///
    /// Galat pertama dari [`Miner::round`].
    pub fn run(&self, blocks: u64, mut on_round: impl FnMut(&Round)) -> Result<u64, MinerError> {
        let mut accepted: u64 = 0;
        while blocks == 0 || accepted < blocks {
            let round = self.round()?;
            if matches!(round, Round::Accepted { .. }) {
                accepted = accepted.saturating_add(1);
            }
            on_round(&round);
        }
        Ok(accepted)
    }
}

#[cfg(test)]
mod tests;
