//! Argumen baris perintah CLI (tanpa `clap`, DEC-10 bagian 2).

use std::path::PathBuf;

use thiserror::Error;
use zevrigo_primitives::{Amount, Network};
use zevrigo_rpc::transport::{DEFAULT_HOST, default_port};

/// Tarif biaya bawaan perintah `send`: 1 quanta per byte (Spesifikasi Wallet v0.1 Bagian 5).
pub const DEFAULT_FEE_RATE: u128 = 1;

/// Teks bantuan `--help`.
pub const USAGE: &str = concat!(
    "zevrigo-cli (MOD-12): alat baris perintah node dan wallet Zevrigo Coin (regtest dan testnet)\n",
    "\n",
    "Pemakaian:\n",
    "  zevrigo-cli [--rpc-url <url>] info\n",
    "  zevrigo-cli --datadir <folder> [--passphrase <teks>] [--rpc-url <url>] <perintah wallet> [opsi]\n",
    "  zevrigo-cli --help | --version\n",
    "\n",
    "Perintah:\n",
    "  info                                ringkasan chain dari node\n",
    "  wallet-create                       wallet baru 24 kata di <folder>/<jaringan>/wallet.json, dikunci password\n",
    "  wallet-restore --mnemonic \"<kata>\"  pulihkan wallet dari 12 sampai 24 kata, dikunci password\n",
    "  wallet-encrypt                      kunci wallet lama (tanpa password) dengan password\n",
    "  wallet-show-phrase                  tampilkan frasa pemulihan (butuh password)\n",
    "  address                             alamat penerimaan baru dan payload-nya (tanpa node)\n",
    "  balance                             saldo dari node lewat scan_utxos\n",
    "  send --to <alamat> --amount <ZVRG> [--fee-rate <quanta per byte>]\n",
    "                                      kirim ZVRG (tarif bawaan 1 quanta per byte, butuh password)\n",
    "\n",
    "Opsi umum:\n",
    "  --regtest             jaringan regtest (bawaan): alamat rzvrg1..., wallet di <folder>/regtest/\n",
    "  --testnet             jaringan testnet: alamat tzvrg1..., wallet di <folder>/testnet/\n",
    "  --rpc-url <url>       alamat RPC node (bawaan regtest http://127.0.0.1:28801, testnet http://127.0.0.1:18801)\n",
    "  --datadir <folder>    folder data wallet, wajib untuk perintah wallet\n",
    "  --passphrase <teks>   passphrase BIP39, ASCII cetak; tidak disimpan di file wallet\n",
    "\n",
    "Password wallet (minimal 8 karakter) diketik tanpa tampil di layar; untuk skrip boleh lewat variabel\n",
    "lingkungan ZEVRIGO_WALLET_PASSWORD. address dan balance tidak butuh password.",
);

/// Perintah hasil membaca argumen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Mencetak [`USAGE`].
    Help,
    /// Mencetak versi paket.
    Version,
    /// Menjalankan satu aksi.
    Run(Config),
}

/// Konfigurasi satu pemanggilan CLI.
#[derive(Clone, PartialEq, Eq)]
pub struct Config {
    /// Jaringan wallet dan node: regtest (bawaan) atau testnet.
    pub network: Network,
    /// Alamat RPC node.
    pub rpc_url: String,
    /// Folder data wallet. Selalu ada untuk aksi wallet, tidak ada untuk `info`.
    pub datadir: Option<PathBuf>,
    /// Passphrase BIP39 (kosong jika tidak diisi).
    pub passphrase: String,
    /// Aksi yang dijalankan.
    pub action: Action,
}

impl core::fmt::Debug for Config {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Config")
            .field("network", &self.network)
            .field("rpc_url", &self.rpc_url)
            .field("datadir", &self.datadir)
            .field("passphrase", &"<rahasia>")
            .field("action", &self.action)
            .finish()
    }
}

/// Aksi CLI.
#[derive(Clone, PartialEq, Eq)]
pub enum Action {
    /// `info`: ringkasan chain.
    Info,
    /// `wallet-create`: wallet baru 24 kata.
    WalletCreate,
    /// `wallet-restore`: pulihkan wallet dari frasa.
    WalletRestore {
        /// Frasa pemulihan apa adanya; dinormalkan saat diurai.
        mnemonic: String,
    },
    /// `wallet-encrypt`: kunci file wallet lama (versi 1, tanpa password) dengan password.
    WalletEncrypt,
    /// `wallet-show-phrase`: tampilkan frasa pemulihan.
    WalletShowPhrase,
    /// `address`: alamat penerimaan berikutnya.
    Address,
    /// `balance`: saldo dari node.
    Balance,
    /// `send`: kirim ZVRG.
    Send {
        /// Alamat penerima.
        to: String,
        /// Jumlah yang diterima penerima.
        amount: Amount,
        /// Tarif biaya dalam quanta per byte.
        fee_rate: u128,
    },
}

impl Action {
    /// Nama aksi seperti ditulis di baris perintah.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::WalletCreate => "wallet-create",
            Self::WalletRestore { .. } => "wallet-restore",
            Self::WalletEncrypt => "wallet-encrypt",
            Self::WalletShowPhrase => "wallet-show-phrase",
            Self::Address => "address",
            Self::Balance => "balance",
            Self::Send { .. } => "send",
        }
    }
}

impl core::fmt::Debug for Action {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::WalletRestore { .. } => f.write_str("WalletRestore(<rahasia>)"),
            Self::Send {
                to,
                amount,
                fee_rate,
            } => f
                .debug_struct("Send")
                .field("to", to)
                .field("amount", amount)
                .field("fee_rate", fee_rate)
                .finish(),
            Self::Info
            | Self::WalletCreate
            | Self::WalletEncrypt
            | Self::WalletShowPhrase
            | Self::Address
            | Self::Balance => f.write_str(self.name()),
        }
    }
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
    /// `--help` atau `--version` digabung dengan argumen lain.
    #[error("--help dan --version tidak boleh digabung dengan argumen lain")]
    Exclusive,
    /// Tidak ada perintah.
    #[error(
        "perintah wajib diisi (info, wallet-create, wallet-restore, wallet-encrypt, wallet-show-phrase, address, balance, send)"
    )]
    MissingAction,
    /// Lebih dari satu perintah.
    #[error("hanya satu perintah yang boleh: {0}")]
    ExtraAction(String),
    /// Opsi tidak berlaku untuk perintah ini.
    #[error("opsi {option} tidak berlaku untuk perintah {action}")]
    NotFor {
        /// Nama opsi.
        option: &'static str,
        /// Nama perintah.
        action: &'static str,
    },
    /// Opsi wajib untuk perintah ini tidak diisi.
    #[error("perintah {action} wajib memakai {option}")]
    MissingOption {
        /// Nama opsi.
        option: &'static str,
        /// Nama perintah.
        action: &'static str,
    },
    /// Jumlah bukan angka ZVRG yang sah, atau nol.
    #[error("jumlah ZVRG tidak sah: {0}")]
    InvalidAmount(String),
    /// Tarif biaya bukan bilangan bulat tak bertanda.
    #[error("tarif biaya tidak sah: {0}")]
    InvalidFeeRate(String),
    /// Lebih dari satu jaringan dipilih.
    #[error("hanya satu jaringan boleh dipilih: --regtest atau --testnet")]
    NetworkConflict,
}

/// Jenis aksi sebelum opsi aksi dibaca.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Info,
    WalletCreate,
    WalletRestore,
    WalletEncrypt,
    WalletShowPhrase,
    Address,
    Balance,
    Send,
}

impl Kind {
    fn parse(word: &str) -> Option<Self> {
        match word {
            "info" => Some(Self::Info),
            "wallet-create" => Some(Self::WalletCreate),
            "wallet-restore" => Some(Self::WalletRestore),
            "wallet-encrypt" => Some(Self::WalletEncrypt),
            "wallet-show-phrase" => Some(Self::WalletShowPhrase),
            "address" => Some(Self::Address),
            "balance" => Some(Self::Balance),
            "send" => Some(Self::Send),
            _ => None,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::WalletCreate => "wallet-create",
            Self::WalletRestore => "wallet-restore",
            Self::WalletEncrypt => "wallet-encrypt",
            Self::WalletShowPhrase => "wallet-show-phrase",
            Self::Address => "address",
            Self::Balance => "balance",
            Self::Send => "send",
        }
    }
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

/// Menolak opsi yang diisi tetapi tidak berlaku untuk aksi ini.
fn reject<T>(
    slot: &Option<T>,
    allowed: bool,
    option: &'static str,
    kind: Kind,
) -> Result<(), ArgError> {
    if slot.is_some() && !allowed {
        return Err(ArgError::NotFor {
            option,
            action: kind.name(),
        });
    }
    Ok(())
}

/// Mengambil opsi wajib.
fn require<T>(slot: Option<T>, option: &'static str, kind: Kind) -> Result<T, ArgError> {
    slot.ok_or(ArgError::MissingOption {
        option,
        action: kind.name(),
    })
}

/// Alamat RPC bawaan jaringan: regtest `http://127.0.0.1:28801`, testnet `http://127.0.0.1:18801`.
#[must_use]
pub fn default_rpc_url(network: Network) -> String {
    format!("http://{DEFAULT_HOST}:{}", default_port(network))
}

/// Mencatat pilihan jaringan dari `flag`. Opsi yang sama dua kali atau dua jaringan berbeda ditolak.
fn choose_network(
    slot: &mut Option<Network>,
    chosen: Network,
    flag: &'static str,
) -> Result<(), ArgError> {
    match *slot {
        Some(previous) if previous == chosen => Err(ArgError::Duplicate(flag)),
        Some(_) => Err(ArgError::NetworkConflict),
        None => {
            *slot = Some(chosen);
            Ok(())
        }
    }
}

/// Opsi yang sudah dibaca, sebelum dicocokkan dengan aksi.
#[derive(Default)]
struct Options {
    network: Option<Network>,
    rpc_url: Option<String>,
    datadir: Option<PathBuf>,
    passphrase: Option<String>,
    mnemonic: Option<String>,
    to: Option<String>,
    amount: Option<Amount>,
    fee_rate: Option<u128>,
}

/// Membaca argumen (tanpa nama program). Tanpa argumen sama dengan `--help`. Opsi boleh ditulis sebelum atau
/// sesudah perintah.
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
    let mut options = Options::default();
    let mut kind: Option<Kind> = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--regtest" => choose_network(&mut options.network, Network::Regtest, "--regtest")?,
            "--testnet" => choose_network(&mut options.network, Network::Testnet, "--testnet")?,
            "--rpc-url" => {
                let value = value_of(&mut iter, "--rpc-url")?;
                set_once(&mut options.rpc_url, value, "--rpc-url")?;
            }
            "--datadir" => {
                let value = value_of(&mut iter, "--datadir")?;
                set_once(&mut options.datadir, PathBuf::from(value), "--datadir")?;
            }
            "--passphrase" => {
                let value = value_of(&mut iter, "--passphrase")?;
                set_once(&mut options.passphrase, value, "--passphrase")?;
            }
            "--mnemonic" => {
                let value = value_of(&mut iter, "--mnemonic")?;
                set_once(&mut options.mnemonic, value, "--mnemonic")?;
            }
            "--to" => {
                let value = value_of(&mut iter, "--to")?;
                set_once(&mut options.to, value, "--to")?;
            }
            "--amount" => {
                let value = value_of(&mut iter, "--amount")?;
                let amount = Amount::from_zvrg_str(&value)
                    .ok()
                    .filter(|amount| *amount != Amount::ZERO)
                    .ok_or_else(|| ArgError::InvalidAmount(value.clone()))?;
                set_once(&mut options.amount, amount, "--amount")?;
            }
            "--fee-rate" => {
                let value = value_of(&mut iter, "--fee-rate")?;
                let rate = value
                    .parse::<u128>()
                    .map_err(|_| ArgError::InvalidFeeRate(value.clone()))?;
                set_once(&mut options.fee_rate, rate, "--fee-rate")?;
            }
            "--help" | "-h" | "--version" => return Err(ArgError::Exclusive),
            word => {
                let Some(parsed) = Kind::parse(word) else {
                    return Err(ArgError::Unknown(arg));
                };
                if kind.is_some() {
                    return Err(ArgError::ExtraAction(arg));
                }
                kind = Some(parsed);
            }
        }
    }
    let kind = kind.ok_or(ArgError::MissingAction)?;
    build(kind, options).map(Command::Run)
}

/// Mencocokkan opsi dengan aksi.
fn build(kind: Kind, options: Options) -> Result<Config, ArgError> {
    let wallet = kind != Kind::Info;
    reject(&options.datadir, wallet, "--datadir", kind)?;
    reject(&options.passphrase, wallet, "--passphrase", kind)?;
    reject(
        &options.mnemonic,
        kind == Kind::WalletRestore,
        "--mnemonic",
        kind,
    )?;
    let send = kind == Kind::Send;
    reject(&options.to, send, "--to", kind)?;
    reject(&options.amount, send, "--amount", kind)?;
    reject(&options.fee_rate, send, "--fee-rate", kind)?;
    let datadir = if wallet {
        Some(require(options.datadir, "--datadir", kind)?)
    } else {
        None
    };
    let action = match kind {
        Kind::Info => Action::Info,
        Kind::WalletCreate => Action::WalletCreate,
        Kind::WalletRestore => Action::WalletRestore {
            mnemonic: require(options.mnemonic, "--mnemonic", kind)?,
        },
        Kind::WalletEncrypt => Action::WalletEncrypt,
        Kind::WalletShowPhrase => Action::WalletShowPhrase,
        Kind::Address => Action::Address,
        Kind::Balance => Action::Balance,
        Kind::Send => Action::Send {
            to: require(options.to, "--to", kind)?,
            amount: require(options.amount, "--amount", kind)?,
            fee_rate: options.fee_rate.unwrap_or(DEFAULT_FEE_RATE),
        },
    };
    let network = options.network.unwrap_or(Network::Regtest);
    Ok(Config {
        network,
        rpc_url: options.rpc_url.unwrap_or_else(|| default_rpc_url(network)),
        datadir,
        passphrase: options.passphrase.unwrap_or_default(),
        action,
    })
}

#[cfg(test)]
mod tests;
