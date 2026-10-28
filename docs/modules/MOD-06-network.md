# MOD-06: Network (`homeostasis-network`)

- **ID:** `MOD-06`
- **Path:** `crates/network`
- **Paket:** `homeostasis-network`
- **Status:** ⚪ belum dimulai (kerangka R0)
- **Bergantung pada:** `MOD-01`
- **Dipakai oleh:** `MOD-11`, `MOD-17`

## Tujuan

Menangani komunikasi *peer-to-peer* antar node menggunakan `rust-libp2p`.

## Tanggung Jawab

- Penemuan rekan (*peer discovery*), manajemen koneksi, pertukaran header/blok, dan penyebaran transaksi (*gossip*) (*diisi lebih rinci di R2*).

## Bukan Tanggung Jawab

- Validasi aturan konsensus atas blok atau transaksi yang diterima (`MOD-04`).
- Kebijakan penerimaan dan pengurutan biaya transaksi di antrean lokal (`MOD-07`).

## Task terkait
- [TSK-M14](../tasks/README.md)
