# TSK-M26 · Pencarian UTXO per payload untuk `scan_utxos` di MOD-05 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-05 (crates/storage) · **Untuk:** FTR-03
- **Bergantung pada:** TSK-M11, TSK-S04 · **Memblokir:** TSK-M12 (metode `scan_utxos`), TSK-F03
- **Estimasi:** XS · **Ditulis:** Quick (kode dan test), 31 Oktober 2026

## Tujuan
Satu metode baru di trait `ChainStore` agar RPC `scan_utxos` (Spesifikasi RPC 3.8) bisa mendaftar UTXO milik
sekumpulan payload, di `MemoryStore` dan `RedbStore` dengan hasil yang sama persis.

## API
`fn utxos_by_payloads(&self, payloads: &[[u8; 32]]) -> Result<Vec<(OutPoint, UtxoEntry)>, StoreError>`
- Hasil: semua UTXO yang `payload` output-nya ada di `payloads`, urut naik `OutPoint` (`txid`, lalu `index`).
- `payloads` kosong → daftar kosong. Payload ganda tidak menggandakan hasil.
- Batas 1 sampai 64 payload dan perhitungan `confirmations`/`spendable_at` adalah urusan MOD-08, bukan MOD-05.

## Keputusan teknis (Quick, tanpa DEC)
- **Pindai penuh, tanpa indeks payload.** Skema redb tetap versi 1 (tanpa migrasi), dan biayanya sebanding dengan ukuran
  set UTXO, sama seperti `scantxoutset` di Bitcoin Core. Indeks payload bisa ditambah nanti lewat skema versi 2 jika
  explorer atau wallet membutuhkannya.
- `MemoryStore`: filter `BTreeMap` yang sudah urut. `RedbStore`: satu read transaction, `ReadableTable::iter`
  (redb 4.4.0), lalu diurutkan dengan `sort_unstable_by_key` agar kontrak urutan tidak bergantung pada backend.
- Fungsi bantu `codec::decode_outpoint_key` (kebalikan `outpoint_key`).
- Tanpa dependensi baru; Cargo.lock tidak berubah.

## Test (storage 23 → 26, workspace 196 → 199)
- `codec/tests.rs`: `decode_outpoint_key_inverts_outpoint_key`.
- `memory/tests.rs`: `utxos_by_payloads_filters_sorts_and_follows_undo` (filter, urutan, payload ganda, payload
  yang sudah dibelanjakan, daftar kosong, dan hasil setelah `undo_tip`).
- `redb_store/tests.rs`: `utxos_by_payloads_matches_memory_store` (skenario blok 1 TSK-M11, termasuk output yang
  dibelanjakan di blok yang sama). `assert_same_state` kini juga membandingkan `utxos_by_payloads` untuk payload
  `0..=4` dan `0x99`, sehingga uji diferensial `prop_differential_against_memory_store` ikut memeriksanya.
