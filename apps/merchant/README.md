# MOD-16: Aplikasi Merchant (`apps/merchant`)

- **ID Modul:** `MOD-16`
- **Fungsi:** Aplikasi pelaku usaha berbasis *watch-only* untuk membuat tagihan pembayaran HSIS, menghasilkan kode pembayaran, dan memantau status konfirmasi tanpa menyimpan kunci privat di perangkat kasir.
- **Platform Target:** Desktop (Linux, Windows, macOS), Android, dan iOS.
- **Dependensi Rencana:** `MOD-08` (`homeostasis-rpc`), `MOD-10` (`homeostasis-merchant-core`).

## Status Kerangka (R0)

Proyek Tauri pada folder ini **belum diinisialisasi** di tahap R0 (belum ada `Cargo.toml` maupun `package.json`). Inisialisasi proyek Tauri akan dikerjakan melalui task tersendiri setelah prasyarat Node.js/Tauri disiapkan.

Catatan wajib untuk kode TypeScript nanti: jumlah koin **dilarang** disimpan pada tipe `number` (floating point IEEE 754); wajib memakai `string` atau `bigint`.

## Keputusan Terkait

- **DEC-04:** Pemilihan pendekatan UI mobile (Tauri 2 di semua platform atau UI native).
- **DEC-05:** Jalur distribusi aplikasi mobile.
- **DEC-06:** Mekanisme tampilan ekuivalen Rupiah di aplikasi merchant.
