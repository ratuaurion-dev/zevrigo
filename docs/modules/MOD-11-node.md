# MOD-11: Node Daemon (`homeostasis-node`)

- **ID:** `MOD-11`
- **Path:** `apps/node`
- **Paket:** `homeostasis-node` (bin)
- **Status:** 🟢 regtest satu node (TSK-M18, TSK-F01), cabang chain + reorg (TSK-M27), dan P2P di daemon (TSK-M28); uji 4 node menyusul (TSK-F04)
- **Bergantung pada:** `MOD-01`, `MOD-02`, `MOD-04`, `MOD-05`, `MOD-06`, `MOD-07`, `MOD-08` (MOD-01 dan MOD-02 sejak DEC-19)
- **Dipakai oleh:** —

## Tujuan

Menjalankan daemon node penuh Homeostasis Coin (baik mode arsip maupun *pruned*) yang menyatukan konsensus, penyimpanan, jaringan P2P, mempool, dan layanan RPC.

## Tanggung Jawab

- `Node<S: ChainStore>` (TSK-M18 bagian A): genesis regtest, penerimaan blok V1–V10 lewat MOD-04 lalu `apply_block`, pemeriksaan ulang mempool, template blok untuk miner, dan implementasi `RpcBackend` (MOD-08) untuk 8 metode.
- Daemon (TSK-M18 bagian B, TSK-M28, TSK-M06 C2): `homeostasis-node --regtest|--testnet --datadir <folder> [--rpc-port <port>] [--p2p-port <port>] [--connect <multiaddr>]...`, chain di `<folder>/<jaringan>/chain.redb`, kunci rekan di `<folder>/<jaringan>/p2p_key`, port bawaan testnet RPC 18801 dan P2P 18800, log ke stderr, server RPC dan P2P di 127.0.0.1, stdout `rpc: …` lalu `p2p: …`, berhenti bersih dengan SIGINT atau SIGTERM.
- Cabang chain, pemilihan chain berdasarkan chainwork, dan reorg (TSK-M27); sinkronisasi header lalu blok, siaran, dan opsi `--p2p-port`/`--connect` (TSK-M28), sesuai [spesifikasi-p2p-v0.1.md](../reference/spesifikasi-p2p-v0.1.md).
- Batas blok cabang, indeks header, dan catatan blok tidak sah (`NodeLimits`, TSK-M34).
- PoW RandomX testnet dan mainnet (TSK-M06 bagian B, DEC-01): kunci = `block_id` leluhur di tinggi kunci pada chain blok itu sendiri (chain aktif, blok cabang, indeks header, atau ID yang dicatat tahap hitung presync), V2 sampai V4 sesudah induk dikenal. Hash dihitung `Randomx` (`Node::open_with_limits` memulai layanan MOD-03 lewat MOD-04; `Node::open_with_pow` untuk penghitung pilihan). Hash yang gagal dihitung menjadi galat internal, bukan blok tidak sah. Regtest tidak berubah. Genesis dipilih dari `network_id` parameter (`known_genesis`: regtest atau testnet, TSK-M06 C2).
- Badan blok cabang di penyimpanan MOD-05, bukan di memori; blok yang dilepas saat reorg langsung dipindah ke sana, dan badan blok cabang dibaca satu per satu saat disambungkan (TSK-M35 bagian A). Presync header (`Presync`, `Node::presync_headers`) untuk cabang yang sangat dalam: header diperiksa tanpa disimpan sampai cabang terbukti lebih berat, lalu diunduh ulang dan disimpan sebagai header terbukti (TSK-M35 bagian B).

## Bukan Tanggung Jawab

- Menulis aturan validasi konsensus sendiri di luar `MOD-04`.
- Melakukan perhitungan hash Proof of Work penambangan (`MOD-13`) atau menyimpan kunci privat dompet pengguna (`MOD-15`).

## Task terkait
- [TSK-M18](../tasks/TSK-M18-node.md) — 🟢 selesai (termasuk TSK-F01)
- [TSK-M27](../tasks/TSK-M27-cabang-reorg.md) — 🟢 selesai (blok cabang, pemilihan chain berdasarkan chainwork, reorg)
- [TSK-M28](../tasks/TSK-M28-p2p-daemon.md) — 🟢 selesai (indeks header, sinkronisasi header lalu blok, siaran, `Relay`, opsi P2P daemon)
- [TSK-M34](../tasks/TSK-M34-batas-memori-node.md) — 🟢 selesai (batas memori blok cabang, indeks header, catatan blok tidak sah)
- [TSK-M35](../tasks/TSK-M35-cabang-di-disk.md) — 🟢 selesai (badan blok cabang di penyimpanan, presync header)
