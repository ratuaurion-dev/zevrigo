# MOD-15: Wallet App (`apps/wallet`)

- **ID:** `MOD-15`
- **Path:** `apps/wallet` (antarmuka), `apps/wallet/src-tauri` (crate Rust)
- **Paket:** `zevrigo-wallet` (bin), antarmuka npm `zevrigo-wallet`
- **Status:** 🟢 v0.1: aplikasi desktop Tauri 2 untuk testnet dan regtest (TSK-M42), installer Windows NSIS (TSK-F07)
- **Bergantung pada:** `MOD-01` (`Amount`, `Network`), `MOD-09` (frasa baru, alamat), `MOD-12` (pustaka CLI, DEC-31)
- **Dipakai oleh:** —

## Tujuan

Wallet desktop untuk pengguna akhir: membuat atau memulihkan wallet, melihat saldo dan riwayat, menerima (alamat + QR),
dan mengirim ZVRG, dengan kunci hanya di komputer pengguna.

## Tanggung Jawab

- Antarmuka (`apps/wallet/src`, React + Vite + Tailwind CSS, zustand): enam layar, dua bahasa, tanpa simulasi, tanpa
  sumber dari internet. Jumlah koin tidak pernah `number`: teks angka quanta di IPC, `bigint` di antarmuka.
- Perintah IPC (`src-tauri/src/commands`): `wallet_info`, `new_phrase`, `restore_wallet`, `receive_address`,
  `new_address`, `chain_status`, `balance`, `history`, `send`, `reveal_phrase`. Semua lewat pustaka MOD-12, sehingga
  file wallet dan tembolok riwayat sama dengan CLI. Pekerjaan lama berjalan di utas sendiri.
- Wallet baru: frasa 24 kata dibuat backend dari sumber acak sistem operasi (`getrandom`), baru disimpan sesudah
  pengguna mengisi ulang 3 kata acak dari catatannya. File wallet selalu dikunci password (Argon2id penuh, DEC-27).
- Galat dikirim dengan kode tetap (`wrong_password`, `node_unreachable`, `insufficient_funds`, ...) dan diterjemahkan.
- Folder data: `%APPDATA%\Zevrigo` (Windows) atau `~/.zevrigo`, atau `ZEVRIGO_WALLET_DATADIR`.
- CSP ketat di `tauri.conf.json`: hanya sumber aplikasi sendiri dan IPC.

## Bukan Tanggung Jawab

- Kriptografi, pemilihan koin, dan format file wallet (MOD-09, MOD-12). Mainnet (belum ada). Passphrase BIP39 tambahan
  (pakai CLI). Wallet mobile (TSK-F08). Layanan kasir (MOD-16).

## Uji

- Rust (`src-tauri/src/commands/tests.rs`): nama jaringan, teks quanta, kode galat, bentuk JSON saldo dan riwayat, frasa
  baru, alur file wallet (W1 → W5, alamat terakhir tetap, alamat baru W6, lihat frasa, password salah).
- Antarmuka (vitest, `src/lib/*.spec.ts`): jumlah ZVRG/quanta, alamat bech32m (vektor W5, W10, TESTNET_0), daftar kata
  BIP39 (sha256 sama dengan MOD-09), penggolongan riwayat, URI dan kode QR.

## Task terkait
- [TSK-M42](../tasks/TSK-M42-wallet-desktop.md) — 🟢 aplikasi wallet desktop
- [TSK-F07](../tasks/README.md) — 🟢 wallet desktop v0.1 (installer Windows, uji dengan node regtest 8 November 2026)
- [TSK-F08](../tasks/README.md)
