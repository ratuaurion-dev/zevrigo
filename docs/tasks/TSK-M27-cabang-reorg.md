# TSK-M27 · Blok cabang, pemilihan chain berdasarkan chainwork, dan reorg di MOD-11 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-11 (apps/node) · **Untuk:** FTR-04
- **Bergantung pada:** TSK-M11, TSK-M18, TSK-S05 · **Memblokir:** TSK-M28, TSK-F04
- **Estimasi:** M · **Ditulis:** Quick (kode dan test), 1 November 2026

## Tujuan
Node mengikuti chain sah dengan chainwork terbanyak, tanpa batas kedalaman reorg dan tanpa checkpoint (AGENTS.md,
Spesifikasi P2P v0.1 Bagian 5). Sebelumnya blok yang induknya bukan ujung chain aktif dilaporkan `orphan` dan dibuang.

## Perilaku `submit_block` (juga dipakai P2P di TSK-M28)
1. V1 decode. Blok yang sudah dikenal (chain aktif atau cabang) → `duplicate`. Blok yang pernah dinyatakan tidak sah
   → `invalid` dengan aturan yang sama.
2. V2 sampai V4. Gagal → `invalid` (tidak dicatat, karena blok seperti ini murah dibuat).
3. Induk tidak sah → `invalid` `V5` (`INVALID_ANCESTOR_RULE`), dicatat. Induk tidak dikenal → `orphan`, tidak disimpan.
4. V5 sampai V7 terhadap induk, di chain aktif maupun cabang: tinggi induk, dan cap waktu sampai 11 blok terakhir
   mengikuti cabang induk itu. Gagal → `invalid`, dicatat.
5. V8 gagal → `deferred`, tidak disimpan.
6. Induk = ujung chain aktif: V9 dan V10 lalu `apply_block` seperti sebelumnya (`accepted`); gagal → `invalid`, dicatat.
7. Selain itu blok disimpan sebagai blok cabang (`accepted`, Spesifikasi RPC 3.5: "masuk chain aktif atau cabang yang
   dikenal") dengan chainwork = chainwork induk + kerja blok.
8. Selama ada blok cabang dengan chainwork lebih besar daripada ujung chain aktif (jika sama, yang lebih dulu diterima),
   node reorg ke sana: `undo_tip` sampai titik cabang, lalu setiap blok cabang diperiksa V9 dan V10 dan diterapkan.
   Jika satu blok tidak sah: blok itu ditandai dengan aturannya, keturunannya dibuang dengan `V5`, blok cabang yang
   sudah tersambung dilepas lagi, dan chain lama diterapkan kembali. Jika blok yang dikirim ikut tidak sah, hasilnya
   `invalid`.
9. Setelah reorg berhasil: blok chain lama menjadi blok cabang, dan mempool disusun ulang: transaksi biasa dari blok
   yang dilepas (urut chain lama), lalu isi mempool lama, masing-masing diperiksa terhadap ujung baru.

## Keputusan teknis (Quick, tanpa DEC)
- **Blok cabang di memori node, bukan di MOD-05.** `ChainStore` tetap hanya chain aktif (tanpa migrasi skema redb);
  `undo_tip` memang menghapus badan blok. Blok cabang hilang saat node dibuka ulang dan diterima lagi dari rekan lewat
  sinkronisasi (TSK-M28). Tanpa batas jumlah di v0.1; penyimpanan cabang di MOD-05 dan batas memori menyusul sebelum
  testnet (dicatat di Spesifikasi P2P Bagian 9 dan 10).
- Reorg tidak atomik terhadap kegagalan penyimpanan di tengah jalan: setiap `apply_block`/`undo_tip` tetap atomik, jadi
  penyimpanan selalu berisi chain sah; node yang dibuka ulang melanjutkan dari chain itu.
- `get_block` tetap hanya untuk chain aktif (Spesifikasi RPC 3.3).
- Tanpa dependensi baru; Cargo.lock dan MOD-05 tidak berubah.

## Test (node 14 → 18, workspace 306 → 310)
`apps/node/src/chain/tests.rs`:
- `heavier_branch_reorgs_and_back`: chain A tiga blok, cabang B dari genesis; B3 menyamai (A tetap aktif), B4 reorg
  sedalam 3; A4 menyamai (B tetap aktif), A5 reorg kembali sedalam 4. Diperiksa tinggi, ujung, chainwork, `mined_total`,
  `get_block_hash` setiap tinggi, `scan_utxos`, `duplicate` untuk blok cabang, `get_block` blok cabang `NotFound`.
- `reorg_returns_transactions_to_mempool`: transaksi yang membelanjakan alokasi genesis di blok 100 A kembali ke mempool
  setelah cabang B101 menang, lalu masuk template dan blok 102.
- `invalid_branch_is_marked_and_active_chain_restored`: B2 mengklaim coinbase +1 quanta (V10); reorg di B3 gagal,
  chain A dipulihkan, B2 `V10`, B3 dan B4 `V5`, B1 tetap cabang; cabang sah C di atas B1 lalu menang.
- `memory_and_redb_agree_after_reorg_and_reopen`: skenario reorg sama di `MemoryStore` dan `RedbStore`, lalu node
  dibuka ulang dari penyimpanan redb dengan chain aktif utuh.
- Uji lama tetap; komentar uji `orphan` diperbarui (induk tidak dikenal).
