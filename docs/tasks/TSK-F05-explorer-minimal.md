# TSK-F05 · FTR-05 Explorer minimal · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task fitur · **Fitur:** FTR-05 · **Modul:** MOD-01, 02, 04, 08, 09, 14 (diuji di MOD-17 dengan MOD-11, 13)
- **Bergantung pada:** TSK-F01, TSK-M29, TSK-M30, DEC-25 · **Memblokir:** TSK-F06
- **Estimasi:** S · **Ditulis:** Quick (uji), 2 November 2026

## Kriteria selesai (docs/features/README.md)
API explorer menampilkan blok, transaksi, alamat, statistik pasokan, dan halaman alokasi genesis; indexer tahan reorg.

## Bukti
`tests/integration/tests/ftr05_explorer.rs` · `explorer_follows_node_through_reorg_and_genesis_spend`, di
`make check-all`. Node A dan B (`Node<MemoryStore>`), masing-masing dengan `RpcServer` dan `Miner` sungguhan, di satu
proses. Explorer memakai indeks SQLite di file sementara (`Indexer::open`) dan membaca node A lewat RPC (`RpcSource`),
lewat `Shared` yang sama dengan program `homeostasis-explorer`: `sync_once` untuk satu langkah sinkronisasi dan
`respond` untuk API. Jam kedua node sama dan tetap (`genesis + 120 × 1_000` detik), sehingga target tetap `pow_limit`.
1. **Sebelum sinkron:** `/status` tanpa tinggi indeks, jalur lain `503 not_ready`.
2. **Alokasi utuh:** A menambang 99 blok, B menerima blok yang sama. Explorer di tinggi 99, ujung dan chainwork sama
   dengan node; `/allocation` satu output genesis, `unspent` = alokasi, belum dibelanjakan.
3. **Mempool:** transaksi yang membelanjakan alokasi genesis (biaya 1_000) dikirim ke A; `/mempool` memuatnya, `/tx` dan
   `/search` menjawab `404` dengan `in_mempool: true`.
4. **Dibelanjakan:** A menambang blok 100 sampai 102. `/tx` (tinggi 100, biaya, input alokasi, output ke kunci 3),
   `/block/100` (biaya, 2 transaksi), `/allocation` (`unspent` 0, `spent_by` = transaksi itu di tinggi 100),
   `/address` penerima, `/supply` (sama dengan `get_chain_info` node, `matches_schedule`), `/search` blok, transaksi,
   alamat.
5. **Reorg:** B menambang 5 blok di atas blok 99; blok-bloknya dikirim ke A lewat RPC `submit_block`, A reorg 3 blok dan
   transaksi kembali ke mempool. Explorer mundur: blok 100 = blok B, blok 100 lama `404`, transaksi `404` dengan
   `in_mempool: true`, alokasi kembali utuh, saldo penerima 0, hadiah B 5 UTXO, biaya total 0.
6. **Buka ulang:** explorer ditutup lalu dibuka lagi dari file indeks; tinggi dan ujung tetap tanpa sinkronisasi.
7. **Dibelanjakan lagi:** A menambang blok 105 berisi transaksi itu. Setiap blok 0 sampai 105 di indeks sama dengan
   chain aktif node; `/blocks` per 20; `/supply` sama dengan jadwal MOD-04 dan node.
8. **Node mati:** server RPC A dihentikan; langkah sinkronisasi mencatat `last_error`, indeks terakhir tetap dilayani.

## Keputusan teknis (Quick, tanpa DEC)
- **Reorg lewat RPC, bukan P2P.** Blok cabang B dikirim ke A dengan `submit_block` (cara yang sama dengan miner); reorg
  lewat P2P sudah diuji di TSK-F04. Yang diuji di sini adalah explorer mengikuti reorg node.
- **API lewat `Shared::respond`, tanpa server HTTP.** Handler axum hanya meneruskan metode, jalur, dan query ke fungsi itu;
  lapisan HTTP sudah diuji di TSK-M30 (`http_server_answers_and_syncs_from_node`). Dengan begitu axum tetap hanya
  dipakai MOD-14.
- Dev-dependency MOD-17: `homeostasis-explorer` lewat `path` (DEC-25 bagian 3) dan `serde_json` (sudah di workspace).
  Cargo.lock bertambah 2 rujukan, tetap 349 paket.

## Test (workspace 356 → 357, MOD-17 4 → 5)
`ftr05::explorer_follows_node_through_reorg_and_genesis_spend`.
