# Fitur: 10 fitur, 5 pertama membentuk jalur menuju testnet

> Diperbarui: 8 November 2026 · Status keseluruhan: 🟡 FTR-01 sampai FTR-05 dan FTR-07 (v0.1) selesai

Fitur adalah kemampuan yang terasa bagi pengguna dan selalu melibatkan lebih dari satu modul.
Fitur dikerjakan lewat task fitur (TSK-F), dan baru boleh dimulai setelah semua task modul yang dibutuhkannya selesai.

| ID | Fitur | Modul utama | Task | Status |
|---|---|---|---|---|
| FTR-01 | Node regtest tunggal memvalidasi dan menyimpan chain sendiri | MOD-01..05, MOD-08, MOD-11 | TSK-F01 | 🟢 |
| FTR-02 | Menambang blok di regtest | MOD-01, 03, 04, 08, 11, 13 | TSK-F02 | 🟢 |
| FTR-03 | Kirim dan terima ZVRG lewat CLI di regtest | MOD-01, 02, 04, 07, 08, 09, 11, 12 | TSK-F03 | 🟢 |
| FTR-04 | Jaringan multi-node dan uji reorg dalam | MOD-04, 05, 06, 11, 17 | TSK-F04 | 🟢 |
| FTR-05 | Explorer minimal (hanya data sistem sejak DEC-28) | MOD-01, 02, 04, 08, 14 (DEC-25, DEC-28) | TSK-F05, TSK-M39 | 🟢 |
| FTR-06 | Testnet publik | semua modul inti, MOD-11..14 | TSK-F06 | 🔴 keputusan pencipta (waktu luncur, seed node) |
| FTR-07 | Wallet desktop (P0) | MOD-09, 12, 15 | TSK-F07 | 🟢 v0.1 (testnet dan regtest, installer Windows) |
| FTR-08 | Wallet mobile | MOD-09, 15 | TSK-F08 | ⚪ |
| FTR-09 | Aplikasi pelaku usaha | MOD-08, 09, 10, 16 | TSK-F09 | ⚪ |
| FTR-10 | Reproducible build dan rilis | semua aplikasi | TSK-F10 | ⚪ |

## Kartu fitur

### FTR-01 · Node regtest tunggal · 🟢
- **Nilai:** bukti pertama bahwa aturan chain bekerja end-to-end di satu mesin.
- **Selesai bila:** node regtest membuat genesis, menerima blok valid, menolak blok tidak valid, menyimpannya, dan menjawab RPC `get_chain_info` serta `get_block`.
- **Catatan:** regtest memakai PoW dev (DEC-09), bukan PoW mainnet.

### FTR-02 · Menambang di regtest · 🟢
- **Nilai:** koin lahir hanya dari penambangan, sesuai rumus pasokan.
- **Selesai bila:** miner mengambil template blok, menemukan blok, node menerimanya, dan coinbase sama persis dengan `block_subsidy` (TSK-M07) ditambah biaya transaksi.

### FTR-03 · Kirim dan terima lewat CLI · 🟢
- **Nilai:** transaksi pertama antar dua wallet.
- **Selesai bila:** wallet CLI membuat alamat, menerima coinbase yang sudah matang (100 blok), mengirim ke wallet lain, dan transaksi terkonfirmasi.

### FTR-04 · Jaringan multi-node · 🟢
- **Nilai:** konsensus terbukti di banyak node, termasuk saat jaringan terpisah lalu tersambung lagi.
- **Selesai bila:** 4 node atau lebih tetap sinkron; uji reorg dalam (tanpa batas kedalaman) lolos; node yang sempat offline mengejar chain dengan kerja terbanyak.

### FTR-05 · Explorer minimal · 🟢
- **Nilai:** siapa pun bisa memeriksa chain (blok, transaksi, pasokan sesuai jadwal) tanpa perlu percaya pada node atau
  pencipta, tanpa explorer menampilkan "apa dan siapa".
- **Selesai bila (diubah DEC-28, 3 November 2026):** API explorer menampilkan blok, ringkasan transaksi, dan statistik
  pasokan hasil tambang, tanpa alamat, jumlah per penerima, jejak koin, dan alokasi genesis; indeks tetap mencatat semuanya
  dengan benar (I1–I4); indexer tahan reorg.

### FTR-06 · Testnet publik · 🔴
- **Terblokir oleh:** keputusan pencipta: waktu peluncuran (cap waktu genesis testnet `TESTNET_GENESIS_TIMESTAMP`,
  mengubah `network_id`) dan lokasi seed node. TSK-M06 (PoW RandomX, DEC-01) dan parameter testnet sudah selesai.
- **Selesai bila:** cakupan minimum L6 (Dokumen Konsep 5.11) tercapai.

### FTR-07 · Wallet desktop · 🟢 v0.1
- **Nilai:** pengguna memegang, menerima, dan mengirim ZVRG lewat aplikasi, dengan kunci hanya di komputer sendiri.
- **Selesai bila (v0.1, testnet dan regtest):** installer Windows NSIS dibangun dari commit bersih dengan Cargo.lock dan
  package-lock.json tetap; wallet terpasang terhubung ke node, menerima hadiah tambang yang matang, mengirim ke wallet lain,
  dan transaksi terkonfirmasi. Tercapai 8 November 2026: installer dari `1313ce4`, uji pencipta dengan node regtest di WSL
  (kirim 0,1 ZVRG ke wallet CLI, terkonfirmasi di blok 111).
- **Belum (menyusul):** tanda tangan installer (DEC-07), build installer reproducible (TSK-F10), keychain sistem operasi,
  mainnet.

### FTR-08 sampai FTR-10 · ⚪
Dipecah menjadi task nanti. Rujukan: Dokumen Aplikasi Bagian 5, 8, dan 11.
