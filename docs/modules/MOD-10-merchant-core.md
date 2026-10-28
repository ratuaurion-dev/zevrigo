# MOD-10: Merchant Core (`homeostasis-merchant-core`)

- **ID:** `MOD-10`
- **Path:** `crates/merchant-core`
- **Paket:** `homeostasis-merchant-core`
- **Status:** ⚪ belum dimulai (kerangka R0)
- **Bergantung pada:** `MOD-01`, `MOD-09`
- **Dipakai oleh:** `MOD-16`, `MOD-17`

## Tujuan

Menyediakan logika inti pembuatan tagihan *watch-only* dan pelacakan status konfirmasi pembayaran bagi pelaku usaha.

## Tanggung Jawab

- Pembuatan alamat/kode tagihan unik berbasis kunci publik *watch-only*, pencocokan pembayaran masuk, dan pembaruan status lunas berdasarkan ambang konfirmasi (*diisi lebih rinci di R2*).

## Bukan Tanggung Jawab

- Menyimpan kunci privat pengeluaran dana (khusus dompet penuh `MOD-09`/`MOD-15`).
- Antarmuka kasir berbasis Tauri (`MOD-16`).

## Task terkait
- [TSK-M17](../tasks/README.md)
