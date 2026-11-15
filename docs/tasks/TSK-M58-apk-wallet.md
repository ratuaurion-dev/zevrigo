# TSK-M58 · APK Wallet Android (Tauri 2) · 🟡 bagian 1 dan 2 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-15 (`apps/wallet`) · **Untuk:** FTR-11 (DEC-34)
- **Bergantung pada:** TSK-M56 (alat build Android), TSK-M60 (HTTPS di MOD-08), TSK-M61 (wallet demo)
- **Ditulis:** Quick, 14 November 2026 (pencipta: "build wallet untuk versi android saja dulu"; HP uji Android arm64; HTTPS
  lewat daftar akar sistem Android)

## Bagian 1: kode (Q136)
| Bagian | Isi |
|---|---|
| Pustaka | `src-tauri/src/lib.rs` (`zevrigo_wallet_lib`, crate-type `staticlib`, `cdylib`, `rlib`): `try_run` (Builder, setup, 10 perintah IPC yang sama) dan `run` (`#[cfg_attr(mobile, tauri::mobile_entry_point)]`). `main.rs` desktop hanya memanggil `try_run` |
| Folder data | Desktop tidak berubah (`AppState::from_env`). Android: `app.path().app_data_dir()` lewat `AppState::new`, dibuat jika belum ada |
| HTTPS Android | Pilihan pencipta: daftar akar sistem Android. `rustls_platform_verifier::android::init_with_env(env, context)` dipanggil sekali di `setup`, dengan konteks Activity dari `WebviewWindow::with_webview` dan `PlatformWebview::jni_handle().exec` (API aman Tauri/wry; repo tetap tanpa `unsafe`). Ketergantungan langsung `rustls-platform-verifier = "=0.5.3"` hanya untuk `target_os = "android"`, paket yang sama dengan yang dibawa MOD-08 (DEC-10 bagian 14); Cargo.lock hanya bertambah satu rujukan |
| Tampilan HP | Layar di bawah `md` (768 px): menu atas disembunyikan, menu bawah lima tombol (tinggi 56 px, aman untuk area gestur), jarak tepi 16 px, alamat node dipatahkan. `viewport-fit=cover` |
| Gerbang | `make check-all` (desktop tidak berubah), `make wallet-check`, dan `cargo clippy --target aarch64-linux-android -p zevrigo-wallet --lib -- -D warnings` dengan clang NDK |

## Bagian 2: APK uji (Q137)
`npm run tauri android init` (proyek Gradle di `src-tauri/gen/android`), repository Maven lokal komponen Kotlin
rustls-platform-verifier-android 0.1.1 dan dependensinya di Gradle, `tauri android build --apk --debug --target aarch64`,
SHA-256 APK, lalu dipasang pencipta di HP. Dilanjutkan: kunci tanda tangan rilis (DEC-04 demo), tombol faucet, daftar
investor demo, output jenis 1.

## Hasil bagian 2 (Q137, Q137b)
APK uji `zevrigo-wallet-0.2.0-android-arm64-debug.apk` (201.907.294 byte, SHA-256
`85832eb1c550027cdc90977fb7c2ec5169a8b1746320f5089a6722fbf7e28a80`, minSdk 24, targetSdk 37, kunci debug). Lolos uji pencipta di
HP Android pada 15 November 2026. Tidak dibagikan.

## Bagian 3 (berikutnya): APK rilis
Build rilis (`tauri android build --apk --target aarch64`), keystore rilis dibuat pencipta di luar repo (password diketik
pencipta), SHA-256 diumumkan di blog `/unduh/` (`apps/blog/lib/unduhan.ts`) dan rilis GitHub.
