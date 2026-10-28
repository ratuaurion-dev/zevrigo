# MOD-07: Mempool (`homeostasis-mempool`)

- **ID:** `MOD-07`
- **Path:** `crates/mempool`
- **Paket:** `homeostasis-mempool`
- **Status:** ⚪ belum dimulai (kerangka R0)
- **Bergantung pada:** `MOD-01`, `MOD-04`
- **Dipakai oleh:** `MOD-11`, `MOD-17`

## Tujuan

Mengelola antrean transaksi belum terkonfirmasi di memori sebelum dimasukkan ke dalam kandidat blok.

## Tanggung Jawab

- Pemeriksaan kelayakan transaksi terhadap aturan konsensus (`MOD-04`), pencegahan pengeluaran ganda di antrean, dan pemilihan transaksi bagi kandidat blok penambang (*diisi lebih rinci di R2*).

## Bukan Tanggung Jawab

- Mendefinisikan ulang aturan konsensus di luar `MOD-04`.
- Penyimpanan blok permanen di disk (`MOD-05`).
