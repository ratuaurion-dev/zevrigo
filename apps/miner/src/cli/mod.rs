//! Argumen baris perintah miner (tanpa `clap`, DEC-10 bagian 2).

use homeostasis_primitives::{Network, OutputKind};
use homeostasis_rpc::transport::{DEFAULT_HOST, default_port};
use homeostasis_rpc::{Hex32, KindNumber};
use thiserror::Error;

/// Teks bantuan `--help`.
pub const USAGE: &str = concat!(
    "homeostasis-miner (MOD-13): penambang CPU Homeostasis Coin (PoW dev, hanya regtest)\n",
    "\n",
    "Pemakaian:\n",
    "  homeostasis-miner --payout <payload hex 64> [--payout-kind 0|1] [--rpc-url <url>] [--blocks <n>]\n",
    "  homeostasis-miner --help | --version\n",
    "\n",
    "Opsi:\n",
    "  --payout <hex>      payload output penerima hadiah: kunci x-only (jenis 0) atau hash kondisi (jenis 1)\n",
    "  --payout-kind <k>   jenis output penerima, 0 atau 1 (bawaan 0)\n",
    "  --rpc-url <url>     alamat RPC node (bawaan http://127.0.0.1:28801)\n",
    "  --blocks <n>        berhenti setelah n blok diterima node (bawaan 0 = terus sampai Ctrl-C)",
);

/// Perintah hasil membaca argumen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Mencetak [`USAGE`].
    Help,
    /// Mencetak versi paket.
    Version,
    /// Menambang.
    Run(MinerConfig),
}

/// Konfigurasi miner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MinerConfig {
    /// Alamat RPC node.
    pub rpc_url: String,
    /// Jenis output penerima hadiah.
    pub payout_kind: KindNumber,
    /// Payload output penerima hadiah.
    pub payout_payload: Hex32,
    /// Jumlah blok yang ditambang sebelum berhenti; `0` berarti terus.
    pub blocks: u64,
}

/// Argumen tidak sah. Biner keluar dengan kode 2.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ArgError {
    /// Argumen tidak dikenali.
    #[error("argumen tidak dikenali: {0}")]
    Unknown(String),
    /// Opsi butuh nilai, tetapi nilainya tidak ada, kosong, atau berupa opsi lain.
    #[error("opsi {0} butuh nilai")]
    MissingValue(&'static str),
    /// Opsi disebut lebih dari sekali.
    #[error("opsi {0} disebut lebih dari sekali")]
    Duplicate(&'static str),
    /// Payload bukan hex huruf kecil 64 karakter.
    #[error("payload penerima tidak sah: {0}")]
    InvalidPayout(String),
    /// Jenis output bukan 0 atau 1.
    #[error("jenis output penerima harus 0 atau 1: {0}")]
    InvalidKind(String),
    /// Jumlah blok bukan bilangan bulat tak bertanda.
    #[error("jumlah blok tidak sah: {0}")]
    InvalidBlocks(String),
    /// Penerima hadiah tidak diisi.
    #[error("penerima hadiah wajib diisi: --payout <hex>")]
    MissingPayout,
    /// `--help` atau `--version` digabung dengan opsi lain.
    #[error("--help dan --version tidak boleh digabung dengan opsi lain")]
    Exclusive,
}

/// Membaca nilai opsi berikutnya. Nilai kosong atau yang diawali `--` ditolak.
fn value_of(
    iter: &mut impl Iterator<Item = String>,
    option: &'static str,
) -> Result<String, ArgError> {
    match iter.next() {
        Some(value) if !value.is_empty() && !value.starts_with("--") => Ok(value),
        _ => Err(ArgError::MissingValue(option)),
    }
}

/// Menyimpan nilai opsi yang hanya boleh muncul sekali.
fn set_once<T>(slot: &mut Option<T>, value: T, option: &'static str) -> Result<(), ArgError> {
    if slot.is_some() {
        return Err(ArgError::Duplicate(option));
    }
    *slot = Some(value);
    Ok(())
}

/// Alamat RPC bawaan regtest: `http://127.0.0.1:28801`.
#[must_use]
pub fn default_rpc_url() -> String {
    format!("http://{DEFAULT_HOST}:{}", default_port(Network::Regtest))
}

/// Membaca argumen (tanpa nama program). Tanpa argumen sama dengan `--help`.
///
/// # Errors
///
/// [`ArgError`] untuk argumen yang tidak sah.
pub fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Result<Command, ArgError> {
    let args: Vec<String> = args.into_iter().collect();
    match args.as_slice() {
        [] => return Ok(Command::Help),
        [only] if only.as_str() == "--help" || only.as_str() == "-h" => return Ok(Command::Help),
        [only] if only.as_str() == "--version" => return Ok(Command::Version),
        _ => {}
    }
    let mut payout: Option<Hex32> = None;
    let mut kind: Option<KindNumber> = None;
    let mut rpc_url: Option<String> = None;
    let mut blocks: Option<u64> = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--payout" => {
                let value = value_of(&mut iter, "--payout")?;
                let parsed = Hex32::try_from(value.as_str())
                    .map_err(|_| ArgError::InvalidPayout(value.clone()))?;
                set_once(&mut payout, parsed, "--payout")?;
            }
            "--payout-kind" => {
                let value = value_of(&mut iter, "--payout-kind")?;
                let parsed = value
                    .parse::<u8>()
                    .ok()
                    .and_then(|number| KindNumber::try_from(number).ok())
                    .ok_or_else(|| ArgError::InvalidKind(value.clone()))?;
                set_once(&mut kind, parsed, "--payout-kind")?;
            }
            "--rpc-url" => {
                let value = value_of(&mut iter, "--rpc-url")?;
                set_once(&mut rpc_url, value, "--rpc-url")?;
            }
            "--blocks" => {
                let value = value_of(&mut iter, "--blocks")?;
                let parsed = value
                    .parse::<u64>()
                    .map_err(|_| ArgError::InvalidBlocks(value.clone()))?;
                set_once(&mut blocks, parsed, "--blocks")?;
            }
            "--help" | "-h" | "--version" => return Err(ArgError::Exclusive),
            _ => return Err(ArgError::Unknown(arg)),
        }
    }
    let payout_payload = payout.ok_or(ArgError::MissingPayout)?;
    let payout_kind = kind.unwrap_or(KindNumber::new(OutputKind::SingleKey));
    Ok(Command::Run(MinerConfig {
        rpc_url: rpc_url.unwrap_or_else(default_rpc_url),
        payout_kind,
        payout_payload,
        blocks: blocks.unwrap_or(0),
    }))
}

#[cfg(test)]
mod tests;
