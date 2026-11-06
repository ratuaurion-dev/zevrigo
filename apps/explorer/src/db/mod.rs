//! Skema SQLite dan encoding nilai tersimpan (Spesifikasi Explorer v0.1 Bagian 2, DEC-25).
//!
//! Jumlah koin disimpan sebagai BLOB 16 byte big-endian dari `u128` quanta, karena `INTEGER` SQLite paling banyak 63
//! bit dan `REAL` adalah float (dilarang). Fungsi agregat SQL tidak pernah dipakai pada kolom jumlah; penjumlahan di
//! Rust dengan `checked_add`.

use rusqlite::{Connection, OptionalExtension, params};
use zevrigo_primitives::Amount;

/// Versi skema indeks (`1`).
pub const SCHEMA_VERSION: u32 = 1;

/// Nama file indeks di folder jaringan: `<datadir>/<jaringan>/explorer.sqlite`.
pub const DB_FILE: &str = "explorer.sqlite";

/// Kunci `meta`: versi skema (`u32` little-endian).
pub(crate) const META_SCHEMA_VERSION: &str = "schema_version";

/// Kunci `meta`: `network_id` (32 byte).
pub(crate) const META_NETWORK_ID: &str = "network_id";

/// Kunci `meta`: total subsidy tambang di indeks (16 byte big-endian).
pub(crate) const META_MINED_TOTAL: &str = "mined_total";

/// Skema indeks. Semua tabel `STRICT`; dijalankan setiap kali indeks dibuka.
pub(crate) const SCHEMA: &str = "
PRAGMA journal_mode = WAL;
CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value BLOB NOT NULL
) STRICT;
CREATE TABLE IF NOT EXISTS blocks (
    height INTEGER PRIMARY KEY,
    block_id BLOB NOT NULL UNIQUE,
    prev_block_id BLOB NOT NULL,
    merkle_root BLOB NOT NULL,
    timestamp INTEGER NOT NULL,
    target INTEGER NOT NULL,
    nonce BLOB NOT NULL,
    size INTEGER NOT NULL,
    tx_count INTEGER NOT NULL,
    subsidy BLOB NOT NULL,
    fees BLOB NOT NULL
) STRICT;
CREATE TABLE IF NOT EXISTS txs (
    txid BLOB PRIMARY KEY,
    height INTEGER NOT NULL,
    position INTEGER NOT NULL,
    size INTEGER NOT NULL,
    is_coinbase INTEGER NOT NULL,
    fee BLOB NOT NULL
) STRICT;
CREATE INDEX IF NOT EXISTS txs_by_height ON txs (height, position);
CREATE TABLE IF NOT EXISTS outputs (
    txid BLOB NOT NULL,
    idx INTEGER NOT NULL,
    height INTEGER NOT NULL,
    kind INTEGER NOT NULL,
    payload BLOB NOT NULL,
    amount BLOB NOT NULL,
    spent_txid BLOB,
    spent_input INTEGER,
    spent_height INTEGER,
    PRIMARY KEY (txid, idx)
) STRICT;
CREATE INDEX IF NOT EXISTS outputs_by_payload ON outputs (payload, height);
CREATE INDEX IF NOT EXISTS outputs_by_spender ON outputs (spent_txid);
CREATE INDEX IF NOT EXISTS outputs_by_spent_height ON outputs (spent_height);
CREATE TABLE IF NOT EXISTS mempool (
    txid BLOB PRIMARY KEY
) STRICT;
";

/// Encoding BLOB jumlah koin: 16 byte big-endian dari `u128` quanta.
#[must_use]
pub const fn amount_to_blob(amount: Amount) -> [u8; 16] {
    amount.as_quanta().to_be_bytes()
}

/// Kebalikan [`amount_to_blob`].
#[must_use]
pub const fn amount_from_blob(bytes: [u8; 16]) -> Amount {
    Amount::from_quanta(u128::from_be_bytes(bytes))
}

/// Membaca nilai `meta` berukuran tepat `N` byte.
pub(crate) fn read_meta<const N: usize>(
    conn: &Connection,
    key: &str,
) -> rusqlite::Result<Option<[u8; N]>> {
    conn.query_row(
        "SELECT value FROM meta WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
}

/// Menulis atau mengganti nilai `meta`.
pub(crate) fn write_meta(conn: &Connection, key: &str, value: &[u8]) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
