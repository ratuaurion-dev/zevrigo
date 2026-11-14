# MOD-15: Wallet App (`apps/wallet`)

- **ID:** `MOD-15`
- **Path:** `apps/wallet` (antarmuka), `apps/wallet/src-tauri` (crate Rust)
- **Paket:** `zevrigo-wallet` (pustaka `zevrigo_wallet_lib` + bin), antarmuka npm `zevrigo-wallet`
- **Status:** 🟢 v0.1: aplikasi desktop Tauri 2 untuk testnet dan regtest (TSK-M42), installer Windows NSIS (TSK-F07); 🟢 0.2.0 versi demo (TSK-M61); 🟡 Android (TSK-M58 bagian 1: kode)
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
- Android (TSK-M58 bagian 1): isi aplikasi di pustaka `zevrigo_wallet_lib` (`src-tauri/src/lib.rs`, crate-type
  `staticlib`, `cdylib`, `rlib`); `run` adalah `tauri::mobile_entry_point`, program desktop memanggil `try_run`. Folder data
  Android = folder data aplikasi (`app_data_dir`, ikut terhapus saat aplikasi dihapus; frasa 24 kata tetap satu-satunya
  cadangan). HTTPS ke gerbang RPC diperiksa terhadap daftar akar sistem Android (rustls-platform-verifier, pilihan pencipta
  14 November 2026), disiapkan saat mulai dengan `init_with_env` dan konteks Activity dari `WebviewWindow::with_webview` +
  `jni_handle().exec` (tanpa kode `unsafe` di repo). Layar sempit memakai menu bawah lima tombol (`md:hidden`).
- Versi demo 0.2.0 (TSK-M61, DEC-34): jaringan bawaan regtest dengan node bawaan gerbang demo publik
  `https://zevrigo-rpc.ratuaurion.store` (`wallet_default_rpc_url`; testnet tetap node lokal), pita "REGTEST" berisi
  peringatan koin demo, catatan privasi di Pengaturan, pesan offline khusus gerbang demo. Installer menampilkan
  `src-tauri/PRIVASI-DAN-LISENSI.txt` (`bundle.licenseFile`). Rilis dibangun `.github/workflows/rilis-wallet.yml` dari tag
  `wallet-v<versi>` (rilis draf + SHA256SUMS.txt); belum ditandatangani (DEC-07, `docs/code-signing-policy.md`).

## Bukan Tanggung Jawab

- Kriptografi, pemilihan koin, dan format file wallet (MOD-09, MOD-12). Mainnet (belum ada). Passphrase BIP39 tambahan
  (pakai CLI). iOS. Layanan kasir (MOD-16).

## Uji

- Rust (`src-tauri/src/commands/tests.rs`): nama jaringan, teks quanta, kode galat, bentuk JSON saldo dan riwayat, frasa
  baru, alur file wallet (W1 → W5, alamat terakhir tetap, alamat baru W6, lihat frasa, password salah).
- Antarmuka (vitest, `src/lib/*.spec.ts`): jumlah ZVRG/quanta, alamat bech32m (vektor W5, W10, TESTNET_0), daftar kata
  BIP39 (sha256 sama dengan MOD-09), penggolongan riwayat, URI dan kode QR.

## Task terkait
- [TSK-M42](../tasks/TSK-M42-wallet-desktop.md) — 🟢 aplikasi wallet desktop
- [TSK-F07](../tasks/README.md) — 🟢 wallet desktop v0.1 (installer Windows, uji dengan node regtest 8 November 2026)
- [TSK-M61](../tasks/TSK-M61-wallet-demo.md) — 🟢 versi demo 0.2.0 dan workflow rilis
- [TSK-M58](../tasks/TSK-M58-apk-wallet.md) — 🟡 APK Wallet Android (bagian 1 kode, bagian 2 APK)
- [TSK-F08](../tasks/README.md)
