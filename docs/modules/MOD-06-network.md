# MOD-06: Network (`homeostasis-network`)

- **ID:** `MOD-06`
- **Path:** `crates/network`
- **Paket:** `homeostasis-network`
- **Status:** 🟡 spesifikasi selesai (TSK-S05, DEC-24); kode menunggu TSK-M14
- **Bergantung pada:** `MOD-01`
- **Dipakai oleh:** `MOD-11`, `MOD-17`

## Tujuan

Menangani komunikasi *peer-to-peer* antar node menggunakan `rust-libp2p` 0.57 (DEC-24, DEC-10 bagian 5), sesuai [spesifikasi-p2p-v0.1.md](../reference/spesifikasi-p2p-v0.1.md).

## Tanggung Jawab

- Swarm libp2p (TCP, Noise, Yamux), kunci identitas rekan, batas koneksi, dan sambung ulang ke alamat `--connect`.
- Lima protokol request-response (`status`, `headers`, `blocks`, `block`, `tx`) dengan codec encoding kanonis dan batas ukuran 2 MiB.
- Sanksi rekan (putus dan blokir 24 jam). Keputusan sinkronisasi dan pemilihan chain ada di MOD-11.

## Bukan Tanggung Jawab

- Validasi aturan konsensus atas blok atau transaksi yang diterima (`MOD-04`).
- Kebijakan penerimaan dan pengurutan biaya transaksi di antrean lokal (`MOD-07`).

## Task terkait
- [TSK-M14](../tasks/README.md) — 🟡 siap setelah TSK-M27
