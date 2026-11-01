# MOD-11: Node Daemon (`homeostasis-node`)

- **ID:** `MOD-11`
- **Path:** `apps/node`
- **Paket:** `homeostasis-node` (bin)
- **Status:** 🟢 regtest satu node (TSK-M18, TSK-F01) dan cabang chain + reorg (TSK-M27); P2P menyusul (TSK-M28, TSK-F04)
- **Bergantung pada:** `MOD-01`, `MOD-02`, `MOD-04`, `MOD-05`, `MOD-06`, `MOD-07`, `MOD-08` (MOD-01 dan MOD-02 sejak DEC-19)
- **Dipakai oleh:** —

## Tujuan

Menjalankan daemon node penuh Homeostasis Coin (baik mode arsip maupun *pruned*) yang menyatukan konsensus, penyimpanan, jaringan P2P, mempool, dan layanan RPC.

## Tanggung Jawab

- `Node<S: ChainStore>` (TSK-M18 bagian A): genesis regtest, penerimaan blok V1–V10 lewat MOD-04 lalu `apply_block`, pemeriksaan ulang mempool, template blok untuk miner, dan implementasi `RpcBackend` (MOD-08) untuk 8 metode.
- Daemon (TSK-M18 bagian B): `homeostasis-node --regtest --datadir <folder> [--rpc-port <port>]`, chain di `<folder>/regtest/chain.redb`, log ke stderr, server RPC di 127.0.0.1, berhenti bersih dengan SIGINT atau SIGTERM.
- Cabang chain, pemilihan chain berdasarkan chainwork, dan reorg (TSK-M27); sinkronisasi header lalu blok, siaran, dan opsi `--p2p-port`/`--connect` (TSK-M28), sesuai [spesifikasi-p2p-v0.1.md](../reference/spesifikasi-p2p-v0.1.md).

## Bukan Tanggung Jawab

- Menulis aturan validasi konsensus sendiri di luar `MOD-04`.
- Melakukan perhitungan hash Proof of Work penambangan (`MOD-13`) atau menyimpan kunci privat dompet pengguna (`MOD-15`).

## Task terkait
- [TSK-M18](../tasks/TSK-M18-node.md) — 🟢 selesai (termasuk TSK-F01)
- [TSK-M27](../tasks/TSK-M27-cabang-reorg.md) — 🟢 selesai (blok cabang, pemilihan chain berdasarkan chainwork, reorg)
