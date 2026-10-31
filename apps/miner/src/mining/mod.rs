//! Loop penambangan: template dari node, cari nonce dengan PoW dev regtest (MOD-03), lalu kirim blok.
//!
//! Satu putaran ([`Miner::round`]) mencoba paling banyak `attempts` nonce untuk satu template. Jika belum ketemu,
//! putaran berikutnya mengambil template baru, yang memuat transaksi mempool dan cap waktu terbaru dari node,
//! sehingga ruang pencarian berganti. Panggilan RPC memakai runtime tokio satu thread milik miner; pencarian nonce
//! berjalan di luar runtime.

use homeostasis_pow::{Target, mine_regtest};
use homeostasis_primitives::{Block, Decode, Encode};
use homeostasis_rpc::{Hex32, KindNumber, RpcClient, SubmitStatus};
use thiserror::Error;
use tokio::runtime::Runtime;

/// Jumlah nonce yang dicoba untuk satu template sebelum template diambil ulang.
pub const ATTEMPTS_PER_TEMPLATE: u64 = 1_000_000;

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

/// Miner yang terhubung ke satu node.
pub struct Miner {
    client: RpcClient,
    runtime: Runtime,
    payout_kind: KindNumber,
    payout_payload: Hex32,
    attempts: u64,
}

impl Miner {
    /// Menyiapkan miner untuk node di `rpc_url`. Belum ada koneksi sampai putaran pertama.
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
        })
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
        let Some(found) = mine_regtest(header, &target, self.attempts) else {
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
