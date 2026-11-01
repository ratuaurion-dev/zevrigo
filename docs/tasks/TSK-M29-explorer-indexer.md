# TSK-M29 · Indexer SQLite MOD-14: skema, sinkronisasi tahan reorg, pemeriksaan I1–I4, query · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-14 (apps/explorer) · **Untuk:** FTR-05
- **Bergantung pada:** TSK-S06, DEC-25 · **Memblokir:** TSK-M30, TSK-F05
- **Estimasi:** M · **Ditulis:** Quick (kode dan test), 2 November 2026

## Tujuan
Pustaka `homeostasis_explorer` dengan indeks SQLite sesuai Spesifikasi Explorer v0.1 Bagian 2 sampai 4. API HTTP,
program, dan adaptor RPC menyusul di TSK-M30. API `rusqlite` 0.41.1 diperiksa Quick dari file .crate resmi.

## Isi
| Modul | Isi |
|---|---|
| `source` | Trait `ChainSource` (`tip`, `block_id_at`, `block`, `mempool`), `SourceBlock` (byte + txid dari node), `SourceError` |
| `db` | Skema `STRICT` (`meta`, `blocks`, `txs`, `outputs`, `mempool`), `DB_FILE`, `SCHEMA_VERSION`, `amount_to_blob`/`amount_from_blob` (16 byte big-endian) |
| `index` | `Indexer::{open, open_in_memory, sync_step, halted, network_id}`, `SyncReport`, `IndexError` |
| `query` | `tip`, `block_at`, `block_by_id`, `blocks_before`, `tx`, `address`, `supply`, `allocation`, `mempool`, `in_mempool`, `confirmations` |

## Keputusan teknis (Quick, tanpa DEC)
- **Sumber data lewat trait.** Indexer tidak bergantung pada RPC; adaptor `RpcClient` ditulis di TSK-M30. Test memakai
  chain buatan tanpa PoW dan tanpa tanda tangan sah, karena explorer memang tidak memeriksa keduanya.
- **Genesis:** I4 untuk tinggi 0 = coinbase tepat alokasi genesis (`GENESIS_ALLOCATION`), subsidy 0; block_id genesis
  wajib sama dengan `network_id` (selain itu `GenesisMismatch`, indexer berhenti).
- **Indeks tambahan** `outputs (spent_txid)` untuk daftar input transaksi (Spesifikasi Explorer Bagian 2 diperbarui).
- **Riwayat alamat** dihitung di Rust dari semua output alamat itu (terbaru dulu: tinggi, lalu posisi), halaman mulai 0.
  Cukup untuk regtest; indeks per transaksi menyusul jika alamat sangat ramai.
- **Galat sumber** (node mati) tidak menghentikan indexer; galat pemeriksaan menghentikannya, dan langkah berikutnya
  mengembalikan galat yang sama tanpa membaca node.
- **Fitur `fallible_uint` rusqlite** (ditambah di Q82b): tanpa fitur ini rusqlite 0.41 tidak menerima `u64`/`usize`
  sebagai parameter atau kolom. Konversinya lewat `i64::try_from` dan galat jika di luar jangkauan, sesuai Spesifikasi
  Explorer Bagian 2. Tanpa paket baru; Cargo.lock sama dengan percobaan pertama Q82.
- `homeostasis-storage` dilepas dari dependensi MOD-14 (DEC-25 bagian 2). `homeostasis-wallet-core` baru ditambah di
  TSK-M30 (alamat bech32m di API).

## Test (explorer 3 → 10, workspace 343 → 350)
- `db`: `index_file_survives_reopen_and_rejects_other_network_or_schema`, `amount_blobs_and_confirmations`.
- `index`: `sync_indexes_genesis_new_blocks_and_mempool` (laporan langkah, mempool, blok hilang, node mati),
  `reorg_rolls_back_and_follows_new_chain` (lepas 2 blok lalu 3 blok, transaksi dan status alokasi genesis ikut mundur
  dan maju), `bad_blocks_halt_indexer_and_keep_last_valid_index` (I1, I2, I3 tiga kasus, I4 dua kasus, genesis lain).
- `query`: `queries_follow_indexed_chain` (blok, transaksi beserta input dan pembelanja, alamat penerima dan alamat
  alokasi, pasokan sama dengan jadwal MOD-04, rata-rata 120 detik, alokasi genesis), `address_history_pages_newest_first`.

## Dependensi (DEC-10 bagian 6)
Paket baru di Cargo.lock beserta lisensi, build script, dan jumlah baris `unsafe`:
[reference/paket-mod14-v0.1.txt](../reference/paket-mod14-v0.1.txt), dibuat skrip gerbang Q82.
