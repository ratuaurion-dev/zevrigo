//! MOD-01: Tipe dasar Zevrigo Coin (`ZVRG`): jumlah (`u128` quanta), tinggi blok, hash 32 byte,
//! parameter chain, serta struktur transaksi, blok, dan encoding biner kanonis.

#![forbid(unsafe_code)]
#![deny(
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]

pub mod amount;
pub mod block;
pub mod encoding;
pub mod hash;
pub mod height;
pub mod params;
pub mod tx;
pub mod utxo;

pub use amount::{Amount, AmountParseError, QUANTA_PER_ZVRG};
pub use block::{Block, GENESIS_EXTRA_LEN, HEADER_SIZE, HEADER_VERSION, Header};
pub use encoding::{Decode, DecodeError, Encode, Reader};
pub use hash::{Hash32, HashParseError};
pub use height::BlockHeight;
pub use params::Network;
pub use tx::{
    CoinbaseTxBody, Condition, ConditionWitness, Input, Output, OutputKind, RegularTxBody,
    SigEntry, Transaction, TxBody, Witness,
};
pub use utxo::{OutPoint, UtxoEntry};

/// Identitas modul sesuai peta arsitektur di `AGENTS.md`.
pub const MODULE_ID: &str = "MOD-01";

#[cfg(test)]
mod tests;
