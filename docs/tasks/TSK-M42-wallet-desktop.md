# TSK-M42 · Wallet desktop Tauri 2 di atas pustaka CLI · 🟢 · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-15 (apps/wallet) · **Untuk:** TSK-F07 (wallet desktop, DEC-31)
- **Bergantung pada:** TSK-M41, DEC-10 bagian 11, DEC-31 · **Memblokir:** installer Windows (TSK-F07)
- **Estimasi:** L

## Tujuan
Aplikasi wallet desktop untuk testnet dan regtest dari tampilan Google AI Studio yang dibersihkan (DEC-31): seluruh
simulasi dihapus, semua data dari wallet sungguhan lewat pustaka MOD-12, dibangun dan diuji di WSL; installer Windows di
langkah berikutnya.

## Isi
1. Crate `zevrigo-wallet` (`apps/wallet/src-tauri`, anggota workspace): Tauri 2.12.3, tauri-build 2.7.2. Sepuluh
   perintah IPC (lihat MOD-15), galat dengan kode tetap, folder data `%APPDATA%\Zevrigo` atau `~/.zevrigo`.
2. Antarmuka (`apps/wallet/src`): React 19.3.1, Vite 8.3.6, Tailwind CSS 4.3.3, zustand 5.1.0, qrcode 1.5.4,
   lucide-react 1.66.0, @tauri-apps/api 2.12.3; dev: @tauri-apps/cli 2.12.2, TypeScript 7.0.2, vitest 5.0.5, dan
   @types/node 26.6.6 (tipe saja, versi sama dengan explorer, untuk test dan konfigurasi).
3. Dihapus dari zip: frasa dari `Date.now`, checksum dan alamat palsu, simulator dan tombol tambang, penampil kode
   Rust, jalur `m/44'/919'`, Mainnet, RPC :8545, password demo, Google Fonts, paket AI Studio, `src-tauri` lama.
4. Dibuat baru: wallet baru (frasa dari backend, periksa 3 kata, password) dan pulihkan, alamat terakhir tanpa menaikkan
   indeks, riwayat dari TSK-M41, kirim dengan konfirmasi password, lihat frasa.
5. `make wallet-check`, CI memasang WebKitGTK dan Node.js, daftar dependensi baru
   `docs/reference/dependensi-wallet-desktop-v0.1.md`.

## Vektor dan uji
- Rust: 7 test (`commands::tests`), termasuk W1 → W5 → W6 dengan Argon2id penuh.
- Antarmuka: 14 test vitest; alamat regtest jenis 1, jenis 2, kurang satu simbol, dan bit sisa bukan nol dihitung
  dari payload W5 dengan checksum bech32m yang benar.

## Kriteria penerimaan
- [x] `make check-all` lolos (429 test), `npm run check` lolos (14 test, build `dist/`)
- [x] Cargo.lock hanya bertambah, kecuali paling banyak `toml_datetime` 1.1.2 → 1.1.3 (patch, alat build, disatukan
  dengan `toml` 1.2 milik Tauri); tidak ada paket lama lain yang hilang atau berubah; lisensi paket baru diperiksa
- [x] Tanpa `unwrap`/`expect`/`panic` di kode Rust non-test, tanpa float untuk jumlah koin, tanpa `unsafe`
