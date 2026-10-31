# MOD-11: Node Daemon (`homeostasis-node`)

- **ID:** `MOD-11`
- **Path:** `apps/node`
- **Paket:** `homeostasis-node` (bin)
- **Status:** 🟡 sedang berjalan (TSK-M18 bagian A selesai: logika chain dan backend RPC; bagian B: daemon)
- **Bergantung pada:** `MOD-01`, `MOD-02`, `MOD-04`, `MOD-05`, `MOD-06`, `MOD-07`, `MOD-08` (MOD-01 dan MOD-02 sejak DEC-19)
- **Dipakai oleh:** —

## Tujuan

Menjalankan daemon node penuh Homeostasis Coin (baik mode arsip maupun *pruned*) yang menyatukan konsensus, penyimpanan, jaringan P2P, mempool, dan layanan RPC.

## Tanggung Jawab

- `Node<S: ChainStore>` (TSK-M18 bagian A): genesis regtest, penerimaan blok V1–V10 lewat MOD-04 lalu `apply_block`, pemeriksaan ulang mempool, template blok untuk miner, dan implementasi `RpcBackend` (MOD-08) untuk 8 metode.
- Daemon (bagian B): argumen, folder data, logging, server RPC lokal, berhenti bersih.
- Sinkronisasi P2P, pemilihan chain berdasarkan chainwork, dan reorg menyusul di TSK-F04.

## Bukan Tanggung Jawab

- Menulis aturan validasi konsensus sendiri di luar `MOD-04`.
- Melakukan perhitungan hash Proof of Work penambangan (`MOD-13`) atau menyimpan kunci privat dompet pengguna (`MOD-15`).

## Task terkait
- [TSK-M18](../tasks/TSK-M18-node.md) — 🟡 bagian A selesai
