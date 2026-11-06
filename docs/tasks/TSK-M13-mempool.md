# TSK-M13 · Mempool di MOD-07 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-07 (crates/mempool) · **Untuk:** FTR-02, FTR-03
- **Bergantung pada:** TSK-M09, DEC-18 · **Memblokir:** TSK-M18, TSK-F01, TSK-F03
- **Estimasi:** S · **Ditulis:** Quick (kode dan test), 31 Oktober 2026

## Tujuan
Antrean transaksi belum terkonfirmasi untuk RPC `send_raw_transaction`, `get_mempool`, `get_block_template`
(Spesifikasi RPC 3.4, 3.6, 3.7). Aturan konsensus tidak ditulis ulang: setiap transaksi diperiksa dengan
`check_transaction` (MOD-04, T1 sampai T6) dengan tinggi = ujung chain + 1.

## API (`zevrigo_mempool`)
| Item | Isi |
|---|---|
| `Mempool::new(max_bytes)`, `Default` | Batas bawaan `DEFAULT_MAX_BYTES` = 10_000_000 byte |
| `add(tx, txid, height, network_id, chain)` | Berurutan: `Duplicate`, `TooLarge` (> 100_000 byte), `Full`, `Conflict` (input sudah dibelanjakan transaksi lain di mempool, tanpa penggantian), lalu `Invalid(TxError)`. View = output mempool, lalu `chain: UtxoLookup`. Gagal berarti isi tidak berubah. Mengembalikan biaya |
| `revalidate(height, network_id, chain)` | Setelah blok baru atau reorg: semua transaksi diperiksa ulang sesuai urutan kedatangan; yang masuk blok, bentrok dengan blok, atau bergantung pada yang terbuang ikut terbuang. Mengembalikan txid yang terbuang |
| `select_for_block(max_bytes)` | Urut kedatangan, anak hanya jika induknya di mempool ikut terpilih |
| `txids()` | Urut naik bytewise (`get_mempool`) |
| `get`, `contains`, `spender`, `in_arrival_order`, `len`, `is_empty`, `total_bytes`, `max_bytes` | Akses baca |
| `MempoolError::rule()`, `rule_name(&TxError)` | Nama aturan untuk RPC `1002`: `T1` sampai `T6`; bentrok mempool = `T1`; galat struktur (coinbase, jumlah witness, indeks input) = `struktur`; penolakan antrean (`Duplicate`, `TooLarge`, `Full`) = `None` |

## Keputusan teknis (Quick, tanpa DEC selain DEC-18)
- txid diberikan pemanggil (MOD-02), sama seperti MOD-05, sehingga dependensi produksi tetap MOD-01 dan MOD-04.
- Tanpa penggantian transaksi (RBF), tanpa biaya minimum, tanpa pengusiran saat penuh: cukup untuk regtest dan
  bukan konsensus. Bisa ditambah nanti tanpa memecah chain.
- Penyimpanan `BTreeMap` (txid, nomor urut, outpoint pembelanja), tanpa `HashMap`, sehingga urutan deterministik.
- Test memakai MOD-02 (`sign`, `sighash_v1`, `txid`) sebagai dev-dependency (DEC-18), agar alur T6 diuji dengan
  tanda tangan asli.

## Test (mempool 1 → 9, workspace 221 → 229)
- `pool/tests.rs`: `accepts_valid_transaction_and_reports_fee`, `rejects_duplicate_and_conflicting_spend`,
  `rejects_consensus_invalid_with_rule_names` (T1, T2 termasuk matang di tinggi 101, T5, T6, coinbase),
  `chained_spend_and_block_selection`, `full_and_too_large_are_rejected`, `revalidate_after_block_and_reorg`,
  `rule_name_covers_all_tx_errors`.
- `tests.rs`: `module_id_matches` (lama), `crate_root_reexports_are_accessible`.
