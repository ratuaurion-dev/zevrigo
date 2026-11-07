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

Wallet butuh node yang berjalan di jaringan yang sama, misalnya `zevrigo-node --testnet` (RPC bawaan
`http://127.0.0.1:18801`); alamat node bisa diubah di Pengaturan.
