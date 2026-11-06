# MOD-15: Aplikasi Wallet (`apps/wallet`)

- **ID Modul:** `MOD-15`
- **Fungsi:** Aplikasi dompet pengguna untuk mengelola kunci, melihat saldo UTXO, serta menyusun dan mengirim transaksi ZVRG (transfer, multi-tanda tangan MuSig2, dan kunci waktu).
- **Platform Target:** Desktop (Linux, Windows, macOS), Android, dan iOS.
- **Dependensi Rencana:** `MOD-08` (`zevrigo-rpc`), `MOD-09` (`zevrigo-wallet-core`).

## Status Kerangka (R0)

Proyek Tauri pada folder ini **belum diinisialisasi** di tahap R0 (belum ada `Cargo.toml` maupun `package.json`). Inisialisasi proyek Tauri akan dikerjakan melalui task tersendiri setelah prasyarat Node.js/Tauri disiapkan.

Catatan wajib untuk kode TypeScript nanti: jumlah koin **dilarang** disimpan pada tipe `number` (floating point IEEE 754); wajib memakai `string` atau `bigint`.

## Keputusan Terkait

- **DEC-04:** Pemilihan pendekatan UI mobile (Tauri 2 di semua platform atau UI native).
- **DEC-05:** Jalur distribusi aplikasi mobile.
