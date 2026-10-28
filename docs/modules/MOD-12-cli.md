# MOD-12: CLI (`homeostasis-cli`)

- **ID:** `MOD-12`
- **Path:** `apps/cli`
- **Paket:** `homeostasis-cli` (bin)
- **Status:** ⚪ belum dimulai (kerangka R0)
- **Bergantung pada:** `MOD-01`, `MOD-08`
- **Dipakai oleh:** —

## Tujuan

Menyediakan alat baris perintah ringan bagi operator untuk memeriksa status dan berinteraksi dengan daemon `homeostasis-node` melalui JSON-RPC.

## Tanggung Jawab

- Pengiriman perintah administrasi baca/kirim transaksi ke endpoint RPC node dan pemformatan keluaran terminal secara deterministik (*diisi lebih rinci di R2*).

## Bukan Tanggung Jawab

- Validasi konsensus lokal (`MOD-04`) maupun pengelolaan kunci dompet pengguna (`MOD-09`/`MOD-15`).

## Task terkait
- [TSK-M19](../tasks/README.md)
