# MOD-15: Wallet App (`apps/wallet`)

- **ID:** `MOD-15`
- **Path:** `apps/wallet`
- **Paket:** *(belum dibuat — menunggu inisialisasi proyek Tauri)*
- **Status:** ⚪ belum dimulai (kerangka dokumentasi R0; terkait `DEC-04`, `DEC-05`)
- **Bergantung pada:** `MOD-08`, `MOD-09`
- **Dipakai oleh:** —

## Tujuan

Menyediakan aplikasi dompet lintas platform (desktop dan mobile) bagi pengguna akhir untuk menyimpan kunci secara lokal, memantau saldo, dan menandatangani transaksi HSIS.

## Tanggung Jawab

- Antarmuka pengguna grafis yang aman (tanpa menyimpan jumlah koin di tipe `number` JavaScript/TypeScript), menghubungkan `MOD-09` (`homeostasis-wallet-core`) dengan node melalui `MOD-08` (`homeostasis-rpc`) (*diisi lebih rinci di R2*).

## Bukan Tanggung Jawab

- Implementasi kriptografi mentah di luar `MOD-02`/`MOD-09` atau menjalankan layanan kasir *watch-only* khusus merchant (`MOD-16`).
