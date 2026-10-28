# MOD-11: Node Daemon (`homeostasis-node`)

- **ID:** `MOD-11`
- **Path:** `apps/node`
- **Paket:** `homeostasis-node` (bin)
- **Status:** ⚪ belum dimulai (kerangka R0)
- **Bergantung pada:** `MOD-04`, `MOD-05`, `MOD-06`, `MOD-07`, `MOD-08`
- **Dipakai oleh:** —

## Tujuan

Menjalankan daemon node penuh Homeostasis Coin (baik mode arsip maupun *pruned*) yang menyatukan konsensus, penyimpanan, jaringan P2P, mempool, dan layanan RPC.

## Tanggung Jawab

- Orkestrasi sinkronisasi blok, pemrosesan rantai terpanjang berdasarkan kerja kumulatif, pengelolaan mempool, dan penyediaan endpoint JSON-RPC (*diisi lebih rinci di R2*).

## Bukan Tanggung Jawab

- Menulis aturan validasi konsensus sendiri di luar `MOD-04`.
- Melakukan perhitungan hash Proof of Work penambangan (`MOD-13`) atau menyimpan kunci privat dompet pengguna (`MOD-15`).

## Task terkait
- [TSK-M18](../tasks/README.md)
