# TSK-F04 · FTR-04 Jaringan multi-node dan uji reorg dalam · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task fitur · **Fitur:** FTR-04 · **Modul:** MOD-04, 05, 06, 07, 08, 11 (diuji di MOD-17)
- **Bergantung pada:** TSK-F02, TSK-M14, TSK-M27, TSK-M28 · **Memblokir:** TSK-F06
- **Estimasi:** M · **Ditulis:** Quick (uji), 1 November 2026

## Kriteria selesai (docs/features/README.md)
4 node atau lebih tetap sinkron; uji reorg dalam (tanpa batas kedalaman) lolos; node yang sempat offline mengejar chain
dengan kerja terbanyak.

## Bukti
`tests/integration/tests/ftr04_multi_node.rs` · `five_nodes_partition_reorg_and_catch_up`, di `make check-all`. Lima
`Node<MemoryStore>` di satu proses, masing-masing dengan `P2pService` dan `Network` sungguhan (TCP, Noise, Yamux di
127.0.0.1, port bebas) dan backend `Relay` (blok ditambang lewat `get_block_template`/`submit_block` seperti miner).
Jam semua node sama dan tetap (`genesis + 120 × 1_000` detik), sehingga target tetap `pow_limit` dan setiap blok
berkerja sama.
1. **Sinkron:** A dan C mendengar; B dan D menyambung ke A dan C (A dan C hanya terhubung lewat B dan D). A menambang
   98 blok, C menambang blok 99: keempat node berujung sama.
2. **Terpisah:** sesi P2P B dan D dihentikan lalu dijalankan lagi dengan `--connect` ke A saja dan ke C saja, sehingga
   jaringan menjadi {A, B} dan {C, D}. Transaksi yang membelanjakan alokasi genesis dikirim ke A dan masuk blok 100
   cabang A. A menambang 10 blok (tinggi 109), C menambang 15 (tinggi 114). B mengikuti A, D mengikuti C, blok 99 tetap
   sama, transaksi tidak bocor ke {C, D}.
3. **Tersambung lagi (reorg dalam):** B tersambung ke A dan C. B lalu A reorg sedalam 10 blok ke cabang C (A lewat
   siaran blok, `Ack 1`, sinkronisasi header lalu `GetBlocks`). UTXO cabang A hilang, transaksi dari blok yang dilepas
   kembali ke mempool A dan B. Blok 115 dari A memuat transaksi itu dan sampai ke keempat node lewat B.
4. **Node yang sempat offline:** sesi D berhenti selama C menambang 5 blok (116 sampai 120); D tetap di 115, lalu
   tersambung lagi ke C dan mengejar sampai 120.
5. **Node baru:** E (hanya genesis) tersambung ke A dan mengejar 120 blok lewat header lalu `GetBlocks` (8 kali minta);
   `get_chain_info` E sama persis dengan C.

## Keputusan teknis (Quick, tanpa DEC)
- **Satu proses, bukan biner.** MOD-17 tidak bisa menjalankan biner `zevrigo-node` (`CARGO_BIN_EXE_*` hanya ada
  untuk uji di paket biner itu), jadi node dan layanan P2P dijalankan sebagai pustaka (DEC-20). Uji dua biner sungguhan
  sudah ada di `apps/node/tests/regtest_node.rs` (TSK-M28).
- **"Offline" = sesi P2P dihentikan**, isi node tetap (seperti daemon yang dibuka ulang dari `chain.redb`). Blok cabang dan
  indeks header di memori ikut tetap; perilaku setelah buka ulang sungguhan (cabang hilang) dicatat di Spesifikasi P2P
  Bagian 9.
- Kedalaman reorg 10 blok cukup untuk membuktikan reorg tanpa batas di jalur P2P; reorg lebih dalam diuji di MOD-11
  (TSK-M27, tanpa batas di kode).
- Tanpa dependensi baru; fitur `sync` tokio dicatat eksplisit di dev-dependency MOD-17. Cargo.lock tidak berubah.

## Test (workspace 342 → 343, MOD-17 3 → 4)
`ftr04_multi_node::five_nodes_partition_reorg_and_catch_up`.
