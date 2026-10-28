# MOD-13: Miner (`homeostasis-miner`)

- **ID:** `MOD-13`
- **Path:** `apps/miner`
- **Paket:** `homeostasis-miner` (bin)
- **Status:** ⚪ belum dimulai (kerangka R0; menunggu `DEC-01`)
- **Bergantung pada:** `MOD-01`, `MOD-03`, `MOD-08`
- **Dipakai oleh:** —

## Tujuan

Menyediakan aplikasi penambang CPU mandiri yang mengambil templat blok dari node melalui RPC, mencari solusi Proof of Work, dan mengirimkan blok yang ditemukan.

## Tanggung Jawab

- Pengambilan kandidat blok lewat `MOD-08`, eksekusi pencarian *nonce* multi-utas memakai `MOD-03`, dan pengiriman blok sah kembali ke node (*diisi lebih rinci di R2*).

## Bukan Tanggung Jawab

- Validasi penuh rantai atau penyimpanan UTXO (`MOD-04`, `MOD-05`).
