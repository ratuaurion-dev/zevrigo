//! Alat upacara genesis jaringan demo regtest (Rancangan Ekosistem Demo v0.1 Bagian 4.2, 4.3; TSK-M62).
//!
//! Membelanjakan output alokasi genesis `G` regtest (10.000.000 ZVRG, kunci rahasia `1` yang sengaja publik, Spesifikasi
//! Validasi Bagian 10) ke kantong-kantong treasury dalam satu transaksi. Hanya untuk regtest: kunci `1` hanya memiliki `G`
//! di genesis regtest, dan alat ini memeriksa jaringan node, `network_id`, serta output `G` sebelum menyusun transaksi.
//!
//! - Jumlah semua kantong wajib tepat `G`. Biaya (tarif quanta per byte dikali ukuran transaksi) diambil dari kantong
//!   terakhir.
//! - Tanpa `--kirim` alat hanya menyusun dan menandatangani transaksi, lalu mencetak rencana dan txid. Dengan `--kirim`,
//!   transaksi dikirim ke node lewat `send_raw_transaction` dan masuk blok berikutnya.
//! - `aux_rand` tanda tangan BIP340 bernilai nol: kunci `1` publik, jadi tidak ada rahasia yang perlu dilindungi.
//!
//! Peluncuran yang aman (seed ditutup sampai upacara selesai) ada di Rancangan Ekosistem Demo v0.1 Bagian 4.3.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use thiserror::Error;
use zevrigo_crypto::{PublicKey, SecretKey, sighash_v1, sign, txid, verify};
use zevrigo_primitives::params::GENESIS_ALLOCATION;
use zevrigo_primitives::{
    Amount, Encode, Hash32, Input, Network, Output, OutputKind, Transaction, Witness,
};
use zevrigo_rpc::{ChainInfo, Hex32, RpcClient, UtxoInfo, reject_reason};
use zevrigo_wallet_core::encode_address;

use crate::cli::UpacaraConfig;

/// Payload (kunci publik x-only) output `G` di genesis regtest: kunci dari kunci rahasia `1` (Spesifikasi Validasi
/// Bagian 10).
pub const GENESIS_PAYLOAD: [u8; 32] = [
    0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b, 0x07,
    0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98,
];

/// `network_id` regtest = `block_id` genesis regtest (Spesifikasi Validasi Bagian 10).
pub const REGTEST_NETWORK_ID: [u8; 32] = [
    0x8b, 0x9e, 0xb3, 0x18, 0x72, 0x50, 0xbb, 0xdf, 0xe1, 0x57, 0x7e, 0xbc, 0xb6, 0x68, 0xc3, 0x00,
    0x03, 0x21, 0x3a, 0x9e, 0xe6, 0xa4, 0x42, 0x44, 0x27, 0xb0, 0x8e, 0x38, 0xda, 0x36, 0x4d, 0xab,
];

/// Jumlah kantong paling banyak dalam satu upacara.
pub const MAX_POCKETS: usize = 16;

/// Tarif biaya bawaan: 1 quanta per byte, sama dengan CLI (Spesifikasi Wallet v0.1 Bagian 5).
pub const DEFAULT_FEE_RATE: u128 = 1;

/// Kunci rahasia `1` pemilik `G` regtest.
fn genesis_secret() -> Result<SecretKey, UpacaraError> {
    let mut bytes = [0_u8; 32];
    if let Some(last) = bytes.last_mut() {
        *last = 1;
    }
    SecretKey::from_bytes(&bytes).map_err(|_| UpacaraError::Sign)
}

/// Satu kantong tujuan: alamat regtest jenis 0 dan jumlahnya.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pocket {
    /// Payload alamat (kunci publik x-only).
    pub payload: [u8; 32],
    /// Jumlah yang diminta untuk kantong ini (sebelum biaya untuk kantong terakhir).
    pub amount: Amount,
}

/// Output `G` yang akan dibelanjakan, dari `scan_utxos`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenesisUtxo {
    /// Txid coinbase genesis.
    pub txid: Hash32,
    /// Jumlah output (wajib [`GENESIS_ALLOCATION`]).
    pub amount: Amount,
    /// Tinggi blok paling awal yang boleh membelanjakannya (100 untuk coinbase genesis).
    pub spendable_at: u64,
}

/// Transaksi upacara yang sudah ditandatangani.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ceremony {
    /// Transaksi lengkap.
    pub tx: Transaction,
    /// Txid transaksi.
    pub txid: Hash32,
    /// Ukuran encoding kanonis dalam byte.
    pub size: usize,
    /// Biaya yang diambil dari kantong terakhir.
    pub fee: Amount,
    /// Jumlah yang benar-benar diterima setiap kantong, urut seperti masukan.
    pub received: Vec<Amount>,
}

/// Hasil menjalankan upacara terhadap node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// Tinggi ujung chain node saat upacara disusun.
    pub tip_height: u64,
    /// Output `G` yang dibelanjakan.
    pub genesis: GenesisUtxo,
    /// Kantong, urut seperti masukan.
    pub pockets: Vec<Pocket>,
    /// Transaksi upacara.
    pub ceremony: Ceremony,
    /// `true` jika transaksi sudah diterima node (`--kirim`).
    pub sent: bool,
}

/// Galat upacara genesis. Biner keluar dengan kode 1.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum UpacaraError {
    /// Node tidak bisa dihubungi atau membalas galat.
    #[error("RPC gagal: {0}")]
    Rpc(String),
    /// Node bukan regtest.
    #[error("node berjalan di jaringan {0}, upacara hanya untuk regtest")]
    WrongNetwork(String),
    /// Genesis node bukan genesis regtest yang diketahui.
    #[error("genesis node bukan genesis regtest (network_id {0})")]
    WrongGenesis(String),
    /// Output `G` tidak ada di set UTXO.
    #[error("output alokasi genesis tidak ada di set UTXO: sudah dibelanjakan di chain ini")]
    GenesisMissing,
    /// Output kunci `1` yang ditemukan bukan output `G` genesis.
    #[error(
        "output kunci genesis tidak sesuai alokasi genesis (tinggi, indeks, jenis, atau jumlah)"
    )]
    GenesisMismatch,
    /// `G` belum matang di blok berikutnya.
    #[error(
        "alokasi genesis belum matang: tinggi node {height}, baru bisa dibelanjakan di blok {spendable_at}"
    )]
    NotMature {
        /// Tinggi ujung chain node.
        height: u64,
        /// Tinggi blok paling awal.
        spendable_at: u64,
    },
    /// Tidak ada kantong, atau lebih dari [`MAX_POCKETS`].
    #[error("jumlah kantong {0} tidak sah (1 sampai 16)")]
    PocketCount(usize),
    /// Dua kantong memakai alamat yang sama.
    #[error("alamat kantong dipakai dua kali: {0}")]
    DuplicatePocket(String),
    /// Jumlah kantong tidak sama dengan `G`.
    #[error("jumlah kantong {total} tidak sama dengan alokasi genesis {expected}")]
    Total {
        /// Jumlah semua kantong.
        total: String,
        /// Alokasi genesis.
        expected: String,
    },
    /// Biaya tidak muat di kantong terakhir.
    #[error("biaya {fee} quanta tidak muat di kantong terakhir")]
    FeeTooLarge {
        /// Biaya dalam quanta.
        fee: u128,
    },
    /// Transaksi tidak bisa disusun.
    #[error("transaksi upacara tidak bisa disusun: {0}")]
    Build(String),
    /// Tanda tangan gagal dibuat atau tidak lolos verifikasi.
    #[error("tanda tangan upacara gagal")]
    Sign,
    /// Node menolak transaksi.
    #[error("node menolak transaksi upacara: {0}")]
    Rejected(String),
}

/// Alamat bech32m regtest kantong.
#[must_use]
pub fn pocket_address(pocket: &Pocket) -> String {
    encode_address(Network::Regtest, OutputKind::SingleKey, &pocket.payload)
}

/// Output kantong dengan jumlah `amount`.
fn pocket_output(payload: [u8; 32], amount: Amount) -> Result<Output, UpacaraError> {
    Output::new(amount, OutputKind::SingleKey, payload)
        .map_err(|e| UpacaraError::Build(e.to_string()))
}

/// Transaksi satu input (`G`) ke `outputs`, dengan `sig` sebagai witness.
fn transaction(
    genesis: &GenesisUtxo,
    outputs: Vec<Output>,
    sig: [u8; 64],
) -> Result<Transaction, UpacaraError> {
    Transaction::regular(
        vec![Input::new(genesis.txid, 0)],
        outputs,
        vec![Witness::single_key(sig)],
    )
    .map_err(|e| UpacaraError::Build(e.to_string()))
}

/// Menyusun dan menandatangani transaksi upacara (fungsi murni, tanpa node).
///
/// Jumlah semua kantong wajib tepat [`GENESIS_ALLOCATION`] dan sama dengan `genesis.amount`; alamat tidak boleh ganda.
/// Biaya = `fee_rate` × ukuran transaksi dengan jumlah penuh, diambil dari kantong terakhir. Mengurangi jumlah tidak
/// pernah menambah ukuran encoding, sehingga biaya selalu paling sedikit `fee_rate` × ukuran akhir. Tanda tangan diperiksa
/// ulang dengan kunci publik `G` sebelum dikembalikan.
///
/// # Errors
///
/// [`UpacaraError`] untuk kantong, jumlah, biaya, susunan, atau tanda tangan yang tidak sah.
pub fn build_ceremony(
    genesis: &GenesisUtxo,
    pockets: &[Pocket],
    network_id: &Hash32,
    fee_rate: u128,
) -> Result<Ceremony, UpacaraError> {
    if pockets.is_empty() || pockets.len() > MAX_POCKETS {
        return Err(UpacaraError::PocketCount(pockets.len()));
    }
    let mut seen = BTreeSet::new();
    for pocket in pockets {
        if !seen.insert(pocket.payload) {
            return Err(UpacaraError::DuplicatePocket(pocket_address(pocket)));
        }
    }
    let total = pockets
        .iter()
        .try_fold(Amount::ZERO, |sum, pocket| sum.checked_add(pocket.amount))
        .ok_or_else(|| UpacaraError::Total {
            total: "meluap".to_owned(),
            expected: GENESIS_ALLOCATION.to_string(),
        })?;
    if total != GENESIS_ALLOCATION || genesis.amount != GENESIS_ALLOCATION {
        return Err(UpacaraError::Total {
            total: total.to_string(),
            expected: GENESIS_ALLOCATION.to_string(),
        });
    }
    let full: Vec<Output> = pockets
        .iter()
        .map(|pocket| pocket_output(pocket.payload, pocket.amount))
        .collect::<Result<_, _>>()?;
    let full_size = transaction(genesis, full, [0; 64])?.to_bytes().len();
    let size_quanta =
        u128::try_from(full_size).map_err(|_| UpacaraError::FeeTooLarge { fee: u128::MAX })?;
    let fee_quanta = size_quanta
        .checked_mul(fee_rate)
        .ok_or(UpacaraError::FeeTooLarge { fee: u128::MAX })?;
    let fee = Amount::from_quanta(fee_quanta);
    let mut received: Vec<Amount> = pockets.iter().map(|pocket| pocket.amount).collect();
    let last = received.last_mut().ok_or(UpacaraError::PocketCount(0))?;
    *last = last
        .checked_sub(fee)
        .filter(|left| *left != Amount::ZERO)
        .ok_or(UpacaraError::FeeTooLarge { fee: fee_quanta })?;
    let outputs: Vec<Output> = pockets
        .iter()
        .zip(&received)
        .map(|(pocket, amount)| pocket_output(pocket.payload, *amount))
        .collect::<Result<_, _>>()?;
    let unsigned = transaction(genesis, outputs.clone(), [0; 64])?;
    let spent = pocket_output(GENESIS_PAYLOAD, genesis.amount)?;
    let msg = sighash_v1(network_id, &unsigned, 0, std::slice::from_ref(&spent))
        .map_err(|e| UpacaraError::Build(e.to_string()))?;
    let secret = genesis_secret()?;
    let sig = sign(&secret, msg.as_bytes(), &[0; 32]).map_err(|_| UpacaraError::Sign)?;
    let public = PublicKey::from_bytes(&GENESIS_PAYLOAD).map_err(|_| UpacaraError::Sign)?;
    if secret.public_key() != public || !verify(&public, msg.as_bytes(), &sig) {
        return Err(UpacaraError::Sign);
    }
    let tx = transaction(genesis, outputs, *sig.as_bytes())?;
    let size = tx.to_bytes().len();
    Ok(Ceremony {
        txid: txid(&tx),
        size,
        fee,
        received,
        tx,
    })
}

fn rpc(error: impl core::fmt::Display) -> UpacaraError {
    UpacaraError::Rpc(error.to_string())
}

/// Memilih output `G` dari hasil `scan_utxos` untuk [`GENESIS_PAYLOAD`].
///
/// # Errors
///
/// [`UpacaraError::GenesisMissing`] jika tidak ada output, [`UpacaraError::GenesisMismatch`] jika output kunci itu
/// bukan output genesis (tinggi 0, indeks 0, coinbase, jenis 0, jumlah `G`).
pub fn genesis_from_scan(utxos: &[UtxoInfo]) -> Result<GenesisUtxo, UpacaraError> {
    let found = utxos
        .iter()
        .find(|utxo| utxo.height == 0)
        .ok_or(if utxos.is_empty() {
            UpacaraError::GenesisMissing
        } else {
            UpacaraError::GenesisMismatch
        })?;
    let amount = found.amount.amount();
    let genuine = found.index == 0
        && found.is_coinbase
        && found.kind.kind() == OutputKind::SingleKey
        && *found.payload.bytes() == GENESIS_PAYLOAD
        && amount == GENESIS_ALLOCATION;
    if !genuine {
        return Err(UpacaraError::GenesisMismatch);
    }
    Ok(GenesisUtxo {
        txid: found.txid.to_hash32(),
        amount,
        spendable_at: found.spendable_at,
    })
}

/// Menjalankan upacara terhadap node `client`: memeriksa jaringan, genesis, dan kematangan `G`, menyusun transaksi
/// ([`build_ceremony`]), lalu mengirimnya jika `send`.
///
/// # Errors
///
/// [`UpacaraError`] dari pemeriksaan, penyusunan, atau node.
pub async fn run(
    client: &RpcClient,
    pockets: &[Pocket],
    fee_rate: u128,
    send: bool,
) -> Result<Report, UpacaraError> {
    let info: ChainInfo = client.get_chain_info().await.map_err(rpc)?;
    if info.network.network() != Network::Regtest {
        return Err(UpacaraError::WrongNetwork(String::from(info.network)));
    }
    let network_id = client.get_block_hash(0).await.map_err(rpc)?;
    if *network_id.bytes() != REGTEST_NETWORK_ID {
        return Err(UpacaraError::WrongGenesis(String::from(network_id)));
    }
    let utxos = client
        .scan_utxos(&[Hex32::new(GENESIS_PAYLOAD)])
        .await
        .map_err(rpc)?;
    let genesis = genesis_from_scan(&utxos)?;
    if info.height.saturating_add(1) < genesis.spendable_at {
        return Err(UpacaraError::NotMature {
            height: info.height,
            spendable_at: genesis.spendable_at,
        });
    }
    let ceremony = build_ceremony(&genesis, pockets, &network_id.to_hash32(), fee_rate)?;
    if send {
        let result = client
            .send_raw_transaction(&ceremony.tx.to_bytes())
            .await
            .map_err(|error| match reject_reason(&error) {
                Some(reason) => UpacaraError::Rejected(reason),
                None => rpc(error),
            })?;
        if result.txid.to_hash32() != ceremony.txid {
            return Err(UpacaraError::Rpc(
                "node mengembalikan txid yang berbeda".to_owned(),
            ));
        }
    }
    Ok(Report {
        tip_height: info.height,
        genesis,
        pockets: pockets.to_vec(),
        ceremony,
        sent: send,
    })
}

/// Menjalankan mode `upacara` dari baris perintah dan mengembalikan teks keluarannya ([`format_report`]).
///
/// # Errors
///
/// [`UpacaraError`] dari runtime, klien RPC, atau [`run`].
pub fn run_command(config: &UpacaraConfig) -> Result<String, UpacaraError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(rpc)?;
    let client = RpcClient::new(&config.node_url).map_err(rpc)?;
    let report = runtime.block_on(run(&client, &config.pockets, config.fee_rate, config.send))?;
    Ok(format_report(&report))
}

/// Teks keluaran upacara.
#[must_use]
pub fn format_report(report: &Report) -> String {
    let mut out = String::new();
    let ceremony = &report.ceremony;
    let _ = writeln!(out, "upacara genesis regtest (jaringan demo)");
    let _ = writeln!(
        out,
        "tinggi node: {} (G bisa dibelanjakan mulai blok {})",
        report.tip_height, report.genesis.spendable_at
    );
    let _ = writeln!(
        out,
        "input: {}:0 {}",
        String::from(Hex32::from(report.genesis.txid)),
        report.genesis.amount
    );
    let count = report.pockets.len();
    for (number, (pocket, amount)) in report.pockets.iter().zip(&ceremony.received).enumerate() {
        let note = if number.saturating_add(1) == count {
            " (dikurangi biaya)"
        } else {
            ""
        };
        let _ = writeln!(
            out,
            "kantong {}: {} {}{note}",
            number.saturating_add(1),
            pocket_address(pocket),
            amount
        );
    }
    let _ = writeln!(
        out,
        "biaya: {} quanta ({} byte)",
        ceremony.fee.as_quanta(),
        ceremony.size
    );
    let _ = writeln!(out, "txid: {}", String::from(Hex32::from(ceremony.txid)));
    let status = if report.sent {
        "dikirim, masuk blok berikutnya"
    } else {
        "rencana saja, belum dikirim (tambahkan --kirim untuk mengirim)"
    };
    let _ = writeln!(out, "status: {status}");
    out
}

#[cfg(test)]
mod tests;
