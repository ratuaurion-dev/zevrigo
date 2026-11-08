# MOD-15: Wallet desktop (`apps/wallet`)

Wallet desktop Zevrigo Coin untuk testnet dan regtest: Tauri 2 dengan antarmuka React (DEC-31, TSK-M42).

- `src/`: antarmuka (React 19, Vite, Tailwind CSS, zustand). Enam layar: wallet baru / pulihkan, Ringkasan, Terima
  (alamat + QR), Kirim, Riwayat, Pengaturan. Dua bahasa (Indonesia, Inggris). Tanpa simulasi dan tanpa sumber dari
  internet (font sistem, ikon dibundel).
- `src-tauri/`: crate Rust `zevrigo-wallet` (anggota workspace). Perintah IPC di `src/commands` memakai pustaka MOD-12
  `zevrigo-cli`, sehingga file wallet sama dengan CLI: `<folder data>/<jaringan>/wallet.json` (terenkripsi Argon2id +
  XChaCha20-Poly1305, DEC-27) dan tembolok riwayat `history.json`.

Folder data bawaan: `%APPDATA%\Zevrigo` (Windows) atau `~/.zevrigo` (sistem lain); ganti dengan variabel
`ZEVRIGO_WALLET_DATADIR`. Untuk memakai wallet yang sama dari CLI: `zevrigo-cli --testnet --datadir <folder data> ...`.

Aturan jumlah koin: tidak pernah memakai `number` (floating point). Lewat IPC jumlah berupa teks angka quanta, di
antarmuka berupa `bigint` (`src/lib/quanta.ts`).

## Perintah

```bash
cd apps/wallet
npm ci --ignore-scripts --no-audit --no-fund   # paket dari package-lock.json
npm run check                                   # typecheck, test (vitest), build ke dist/
npm run tauri build                             # aplikasi rilis dan installer (Windows: NSIS)
```

Dari akar repo: `make wallet-check`. Crate Rust ikut `make check-all`; di Linux butuh paket WebKitGTK
(`libwebkit2gtk-4.1-dev`, `librsvg2-dev`, `libxdo-dev`, `libayatana-appindicator3-dev`).

## Installer Windows

Dibangun di Windows (DEC-31 pilihan A) dengan Rust MSVC, Node.js 22.12 atau lebih baru, Microsoft C++ Build Tools, dan
WebView2. Pakai salinan bersih di luar repo (misalnya `git archive HEAD`), karena `node_modules` versi Windows berbeda dari
versi Linux dan `.cargo/config.toml` lokal memakai jalur Linux. Di PowerShell, dari `apps/wallet` salinan itu:

```powershell
$env:CARGO_TARGET_DIR = 'C:\zevrigo-build\target'
npm ci --ignore-scripts --no-audit --no-fund
npm run check
npm run tauri -- build -- --locked
```

Hasilnya `<CARGO_TARGET_DIR>\release\bundle\nsis\Zevrigo Wallet_0.1.0_x64-setup.exe`. Installer belum ditandatangani
(DEC-07), sehingga Windows SmartScreen menampilkan peringatan.

Wallet butuh node yang berjalan di jaringan yang sama, misalnya `zevrigo-node --testnet` (RPC bawaan
`http://127.0.0.1:18801`); alamat node bisa diubah di Pengaturan. Untuk uji coba sebelum testnet publik, pilih **Regtest**
di bilah status dan jalankan `zevrigo-node --regtest` (RPC `http://127.0.0.1:28801`). Node di WSL2 bisa dihubungi dari
wallet Windows lewat `127.0.0.1`.
