//! Pemilihan koin, biaya, kembalian, dan tanda tangan transaksi kirim (TSK-M16, Spesifikasi Wallet v0.1 Bagian 5).
//!
//! Alurnya dua langkah: [`plan`] memilih koin dan menghitung biaya serta kembalian tanpa kunci rahasia, lalu
//! [`sign_plan`] menandatangani setiap input dengan BIP340 atas `sighash_v1` (DEC-13). [`build_transaction`]
//! menjalankan keduanya. Semua langkah deterministik: `aux_rand` tanda tangan diberikan pemanggil. Wallet v0.1 hanya
//! membelanjakan output jenis 0 miliknya sendiri. Modul ini bukan aturan konsensus.

use std::collections::BTreeSet;

use homeostasis_crypto::{SecretKey, sighash_v1, sign, verify};
use homeostasis_primitives::encoding::{varint_u64_len, varint_u128_len};
use homeostasis_primitives::params::MAX_BLOCK_SIZE;
use homeostasis_primitives::tx::SIGNATURE_LEN;
use homeostasis_primitives::{
    Amount, DecodeError, Hash32, Input, OutPoint, Output, OutputKind, Transaction, Witness,
};

use crate::bip32::{Bip32Error, ExtendedPrivKey};

/// Ukuran witness jenis 0: 1 byte `kind` ditambah tanda tangan 64 byte (`65` byte).
pub const SINGLE_KEY_WITNESS_SIZE: usize = 65;

/// Bagian tetap ukuran output: 1 byte `kind` ditambah payload 32 byte (`33` byte), di luar varint jumlah.
const OUTPUT_FIXED_SIZE: usize = 33;

/// Panjang varint `u128` terpanjang (`19` byte).
const MAX_AMOUNT_VARINT_LEN: usize = 19;

/// Ukuran tetap badan transaksi biasa: `version` dan `tx_kind` (2 byte) ditambah penghitung output (1 byte, karena
/// output paling banyak 2).
const BODY_FIXED_SIZE: usize = 3;

/// Galat penyusunan dan tanda tangan transaksi kirim.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SendError {
    /// Jumlah kirim nol.
    #[error("jumlah kirim harus minimal 1 quanta")]
    ZeroAmount,
    /// Outpoint yang sama muncul lebih dari sekali di daftar koin.
    #[error("koin {txid}:{index} muncul lebih dari sekali")]
    DuplicateCoin {
        /// Txid outpoint.
        txid: Hash32,
        /// Indeks output outpoint.
        index: u32,
    },
    /// Koin yang sudah bisa dipakai tidak cukup untuk jumlah kirim ditambah biaya.
    #[error("saldo yang bisa dipakai {available} kurang dari kebutuhan {needed}")]
    InsufficientFunds {
        /// Total koin yang sudah bisa dipakai.
        available: Amount,
        /// Jumlah kirim ditambah biaya bila semua koin itu dipakai, tanpa kembalian.
        needed: Amount,
    },
    /// Transaksi lebih besar dari ukuran blok maksimum, sehingga tidak akan pernah bisa ditambang.
    #[error("ukuran transaksi {size} byte melebihi ukuran blok maksimum")]
    TooLarge {
        /// Ukuran encoding transaksi dalam byte.
        size: usize,
    },
    /// Penjumlahan jumlah koin atau perkalian biaya meluap.
    #[error("jumlah koin atau biaya meluap")]
    AmountOverflow,
    /// Kunci turunan `branch/index` sebuah koin tidak sama dengan payload koin itu.
    #[error("kunci turunan koin {txid}:{index} tidak cocok dengan payload-nya")]
    KeyMismatch {
        /// Txid outpoint.
        txid: Hash32,
        /// Indeks output outpoint.
        index: u32,
    },
    /// Turunan kunci BIP32 gagal.
    #[error("turunan kunci gagal: {0}")]
    Derivation(#[from] Bip32Error),
    /// Struktur transaksi ditolak MOD-01.
    #[error("struktur transaksi tidak sah: {0}")]
    Structure(#[from] DecodeError),
    /// Pembuatan atau pemeriksaan ulang tanda tangan gagal.
    #[error("pembuatan tanda tangan gagal")]
    Signing,
    /// Ukuran transaksi bertanda tangan tidak sama dengan ukuran di rencana (tidak terjadi pada rencana dari [`plan`]).
    #[error("ukuran transaksi bertanda tangan tidak sama dengan rencana")]
    SizeMismatch,
}

/// Satu UTXO jenis 0 milik wallet yang boleh dipilih: outpoint, jumlah, payload, tinggi paling awal yang boleh
/// membelanjakannya (`spendable_at` dari `scan_utxos`), dan jalur kunci `branch/index` di bawah kunci akun.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Coin {
    outpoint: OutPoint,
    amount: Amount,
    payload: [u8; 32],
    spendable_at: u64,
    branch: u32,
    index: u32,
}

impl Coin {
    /// Membentuk [`Coin`]. Kecocokan `payload` dengan kunci `branch/index` diperiksa saat [`sign_plan`].
    #[must_use]
    pub const fn new(
        outpoint: OutPoint,
        amount: Amount,
        payload: [u8; 32],
        spendable_at: u64,
        branch: u32,
        index: u32,
    ) -> Self {
        Self {
            outpoint,
            amount,
            payload,
            spendable_at,
            branch,
            index,
        }
    }

    /// Outpoint koin.
    #[must_use]
    pub const fn outpoint(&self) -> OutPoint {
        self.outpoint
    }

    /// Jumlah koin.
    #[must_use]
    pub const fn amount(&self) -> Amount {
        self.amount
    }

    /// Payload output (kunci x-only).
    #[must_use]
    pub const fn payload(&self) -> &[u8; 32] {
        &self.payload
    }

    /// Tinggi blok paling awal yang boleh membelanjakan koin ini.
    #[must_use]
    pub const fn spendable_at(&self) -> u64 {
        self.spendable_at
    }

    /// Cabang jalur kunci (`0` penerimaan, `1` kembalian).
    #[must_use]
    pub const fn branch(&self) -> u32 {
        self.branch
    }

    /// Indeks jalur kunci.
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }
}

/// Permintaan kirim: penerima, jumlah, tarif biaya, payload kembalian, dan tinggi ujung chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SendRequest {
    /// Jenis output penerima (dari alamat).
    pub recipient_kind: OutputKind,
    /// Payload output penerima (dari alamat).
    pub recipient_payload: [u8; 32],
    /// Jumlah yang diterima penerima (minimal 1 quanta).
    pub amount: Amount,
    /// Tarif biaya dalam quanta per byte encoding lengkap transaksi bertanda tangan (bawaan CLI 1, boleh 0).
    pub fee_rate: u128,
    /// Payload output kembalian (kunci x-only `branch 1` berikutnya yang belum dipakai).
    pub change_payload: [u8; 32],
    /// Tinggi ujung chain aktif. Koin boleh dipilih jika `spendable_at <= tip_height + 1`.
    pub tip_height: u64,
}

/// Rencana transaksi hasil [`plan`]: koin terpilih (urut sebagai input), output, biaya, kembalian, dan ukuran pasti
/// transaksi bertanda tangan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    coins: Vec<Coin>,
    outputs: Vec<Output>,
    fee: Amount,
    change: Option<Amount>,
    size: usize,
}

impl Plan {
    /// Koin terpilih, urut sebagai input transaksi.
    #[must_use]
    pub fn coins(&self) -> &[Coin] {
        &self.coins
    }

    /// Output transaksi: penerima, lalu kembalian jika ada.
    #[must_use]
    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    /// Biaya untuk penambang (`Σ input − Σ output`).
    #[must_use]
    pub const fn fee(&self) -> Amount {
        self.fee
    }

    /// Jumlah kembalian, atau `None` jika tanpa output kembalian.
    #[must_use]
    pub const fn change(&self) -> Option<Amount> {
        self.change
    }

    /// Ukuran encoding lengkap transaksi bertanda tangan dalam byte.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.size
    }
}

/// Ukuran transaksi tanpa kembalian untuk `count` input yang encoding-nya `inputs_size` byte.
fn size_without_change(count: usize, inputs_size: usize, recipient_size: usize) -> Option<usize> {
    let witnesses = count.checked_mul(SINGLE_KEY_WITNESS_SIZE)?;
    BODY_FIXED_SIZE
        .checked_add(varint_u64_len(u64::try_from(count).ok()?))?
        .checked_add(inputs_size)?
        .checked_add(recipient_size)?
        .checked_add(witnesses)
}

fn fee_for(fee_rate: u128, size: usize) -> Result<Amount, SendError> {
    let bytes = u128::try_from(size).map_err(|_| SendError::AmountOverflow)?;
    fee_rate
        .checked_mul(bytes)
        .map(Amount::from_quanta)
        .ok_or(SendError::AmountOverflow)
}

fn check_size(size: usize) -> Result<(), SendError> {
    if size > MAX_BLOCK_SIZE {
        return Err(SendError::TooLarge { size });
    }
    Ok(())
}

/// Memilih koin dan menghitung biaya serta kembalian (Spesifikasi Wallet v0.1 Bagian 5).
///
/// - Koin dengan `spendable_at <= tip_height + 1` diurutkan jumlah menurun, lalu outpoint naik, dan diambil
///   berurutan sampai cukup untuk jumlah kirim ditambah biaya tanpa kembalian.
/// - Biaya = `fee_rate × ukuran` encoding lengkap transaksi bertanda tangan (witness jenis 0 selalu 65 byte).
/// - Jika sisa nol, tanpa kembalian. Jika sisa cukup untuk membayar output kembalian ditambah minimal 1 quanta,
///   kembalian dibuat ke `change_payload`. Jika tidak cukup, sisa itu ikut menjadi biaya.
/// - Pada batas panjang varint jumlah kembalian, biaya bisa lebih tinggi dari `fee_rate × ukuran` (paling banyak
///   selisih panjang varint itu dikali tarif), agar kembalian tetap ditentukan secara unik.
///
/// # Errors
///
/// [`SendError::ZeroAmount`], [`SendError::DuplicateCoin`], [`SendError::InsufficientFunds`],
/// [`SendError::TooLarge`], atau [`SendError::AmountOverflow`].
pub fn plan(coins: &[Coin], request: &SendRequest) -> Result<Plan, SendError> {
    let recipient = Output::new(
        request.amount,
        request.recipient_kind,
        request.recipient_payload,
    )
    .map_err(|_| SendError::ZeroAmount)?;
    let mut seen = BTreeSet::new();
    for coin in coins {
        if !seen.insert(coin.outpoint) {
            return Err(SendError::DuplicateCoin {
                txid: coin.outpoint.txid(),
                index: coin.outpoint.index(),
            });
        }
    }
    // Sengaja saturating: pada tinggi u64::MAX semua koin dianggap sudah bisa dipakai.
    let limit = request.tip_height.saturating_add(1);
    let mut eligible: Vec<&Coin> = coins
        .iter()
        .filter(|coin| coin.spendable_at <= limit)
        .collect();
    eligible.sort_by(|left, right| {
        right
            .amount
            .cmp(&left.amount)
            .then_with(|| left.outpoint.cmp(&right.outpoint))
    });

    let recipient_size = recipient.encoded_size();
    let mut total = Amount::ZERO;
    let mut inputs_size: usize = 0;
    let mut needed = request.amount;
    let mut selected: Vec<Coin> = Vec::new();
    for &coin in &eligible {
        selected.push(coin.clone());
        let count = selected.len();
        total = total
            .checked_add(coin.amount)
            .ok_or(SendError::AmountOverflow)?;
        let input = Input::new(coin.outpoint.txid(), coin.outpoint.index());
        inputs_size = inputs_size
            .checked_add(input.encoded_size())
            .ok_or(SendError::AmountOverflow)?;
        let base = size_without_change(count, inputs_size, recipient_size)
            .ok_or(SendError::TooLarge { size: usize::MAX })?;
        check_size(base)?;
        let base_fee = fee_for(request.fee_rate, base)?;
        needed = request
            .amount
            .checked_add(base_fee)
            .ok_or(SendError::AmountOverflow)?;
        let Some(remainder) = total.checked_sub(needed) else {
            continue;
        };
        return finish(
            selected, recipient, request, total, base, base_fee, remainder,
        );
    }
    Err(SendError::InsufficientFunds {
        available: total,
        needed,
    })
}

/// Menentukan kembalian untuk koin terpilih yang sudah cukup membayar jumlah kirim ditambah biaya tanpa kembalian.
fn finish(
    coins: Vec<Coin>,
    recipient: Output,
    request: &SendRequest,
    total: Amount,
    base: usize,
    base_fee: Amount,
    remainder: Amount,
) -> Result<Plan, SendError> {
    if remainder != Amount::ZERO {
        let after_send = total
            .checked_sub(request.amount)
            .ok_or(SendError::AmountOverflow)?;
        for len in 1..=MAX_AMOUNT_VARINT_LEN {
            let estimate = base
                .checked_add(OUTPUT_FIXED_SIZE)
                .and_then(|size| size.checked_add(len))
                .ok_or(SendError::TooLarge { size: usize::MAX })?;
            let fee = fee_for(request.fee_rate, estimate)?;
            // Biaya naik dengan len, jadi bila kembalian sudah habis, len yang lebih panjang juga tidak cukup.
            let Some(change) = after_send
                .checked_sub(fee)
                .filter(|change| *change != Amount::ZERO)
            else {
                break;
            };
            let change_len = varint_u128_len(change.as_quanta());
            if change_len > len {
                continue;
            }
            let size = base
                .checked_add(OUTPUT_FIXED_SIZE)
                .and_then(|size| size.checked_add(change_len))
                .ok_or(SendError::TooLarge { size: usize::MAX })?;
            check_size(size)?;
            let change_output = Output::new(change, OutputKind::SingleKey, request.change_payload)?;
            return Ok(Plan {
                coins,
                outputs: vec![recipient, change_output],
                fee,
                change: Some(change),
                size,
            });
        }
    }
    let fee = base_fee
        .checked_add(remainder)
        .ok_or(SendError::AmountOverflow)?;
    Ok(Plan {
        coins,
        outputs: vec![recipient],
        fee,
        change: None,
        size: base,
    })
}

/// Menandatangani rencana: input ke-`i` ditandatangani kunci `branch/index` koin ke-`i` di bawah kunci akun
/// `account`, dengan BIP340 atas `sighash_v1(network_id, tx, i, output semua koin)` dan `aux_rand` dari pemanggil.
///
/// Setiap tanda tangan diverifikasi ulang, dan ukuran transaksi dicocokkan dengan rencana.
///
/// # Errors
///
/// [`SendError::KeyMismatch`] bila kunci turunan tidak sama dengan payload koin, [`SendError::Derivation`],
/// [`SendError::Structure`], [`SendError::Signing`], atau [`SendError::SizeMismatch`].
pub fn sign_plan(
    plan: &Plan,
    account: &ExtendedPrivKey,
    network_id: &Hash32,
    aux_rand: &[u8; 32],
) -> Result<Transaction, SendError> {
    let inputs: Vec<Input> = plan
        .coins
        .iter()
        .map(|coin| Input::new(coin.outpoint.txid(), coin.outpoint.index()))
        .collect();
    let mut spent = Vec::with_capacity(plan.coins.len());
    for coin in &plan.coins {
        spent.push(Output::new(
            coin.amount,
            OutputKind::SingleKey,
            coin.payload,
        )?);
    }
    let placeholders = vec![Witness::single_key([0_u8; SIGNATURE_LEN]); plan.coins.len()];
    let unsigned = Transaction::regular(inputs.clone(), plan.outputs.clone(), placeholders)?;

    let mut witnesses = Vec::with_capacity(plan.coins.len());
    for (position, coin) in plan.coins.iter().enumerate() {
        let key = account.derive_path(&[coin.branch, coin.index])?;
        let secret = SecretKey::from_bytes(key.secret_bytes()).map_err(|_| SendError::Signing)?;
        let public = secret.public_key();
        if *public.as_bytes() != coin.payload {
            return Err(SendError::KeyMismatch {
                txid: coin.outpoint.txid(),
                index: coin.outpoint.index(),
            });
        }
        let input_index =
            u32::try_from(position).map_err(|_| SendError::TooLarge { size: plan.size })?;
        let msg = sighash_v1(network_id, &unsigned, input_index, &spent)
            .map_err(|_| SendError::Signing)?;
        let signature = sign(&secret, msg.as_bytes(), aux_rand).map_err(|_| SendError::Signing)?;
        if !verify(&public, msg.as_bytes(), &signature) {
            return Err(SendError::Signing);
        }
        witnesses.push(Witness::single_key(*signature.as_bytes()));
    }
    let tx = Transaction::regular(inputs, plan.outputs.clone(), witnesses)?;
    if tx.encoded_size() != Some(plan.size) {
        return Err(SendError::SizeMismatch);
    }
    Ok(tx)
}

/// Menjalankan [`plan`] lalu [`sign_plan`].
///
/// # Errors
///
/// Galat dari [`plan`] atau [`sign_plan`].
pub fn build_transaction(
    coins: &[Coin],
    request: &SendRequest,
    account: &ExtendedPrivKey,
    network_id: &Hash32,
    aux_rand: &[u8; 32],
) -> Result<(Plan, Transaction), SendError> {
    let chosen = plan(coins, request)?;
    let tx = sign_plan(&chosen, account, network_id, aux_rand)?;
    Ok((chosen, tx))
}

#[cfg(test)]
mod tests;
