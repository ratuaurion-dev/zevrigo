# MOD-16: Merchant App (`apps/merchant`)

- **ID:** `MOD-16`
- **Path:** `apps/merchant`
- **Paket:** *(belum dibuat — menunggu inisialisasi proyek Tauri)*
- **Status:** ⚪ belum dimulai (kerangka dokumentasi R0; terkait `DEC-04`, `DEC-05`, `DEC-06`)
- **Bergantung pada:** `MOD-08`, `MOD-10`
- **Dipakai oleh:** —

## Tujuan

Menyediakan aplikasi kasir lintas platform (desktop dan mobile) berbasis *watch-only* bagi pelaku usaha untuk menerima pembayaran HSIS secara aman.

## Tanggung Jawab

- Pembuatan tagihan dan tampilan kode QR pembayaran menggunakan `MOD-10` (`homeostasis-merchant-core`), pemantauan status konfirmasi melalui `MOD-08` (`homeostasis-rpc`), serta tampilan nilai tagihan tanpa tipe float (*diisi lebih rinci di R2*).

## Bukan Tanggung Jawab

- Menyimpan kunci privat atau menandatangani pengeluaran dana (`MOD-09`/`MOD-15`).
