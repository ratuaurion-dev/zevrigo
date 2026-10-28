# Keputusan Desain & Arsitektur (`docs/decisions/`)

Folder ini mencatat keputusan arsitektur dan teknis proyek Homeostasis Coin (`DEC-xx`), termasuk perubahan struktur repositori, dependensi antar modul, maupun pemilihan komponen utama.

Legenda: 🟢 selesai/diputuskan · 🟡 berjalan · 🔴 terblokir · ⚪ belum dimulai · ❓ perlu keputusan

## Daftar Keputusan

| ID | Keputusan | Menghambat | Status |
|---|---|---|---|
| DEC-01 | Strategi PoW di tengah larangan `unsafe` dan float: isolasi RandomX, ganti PoW, atau RandomX Rust murni | Implementasi MOD-03 | ❓ |
| DEC-02 | Lisensi: MIT atau Apache 2.0 | Rilis publik | ❓ |
| DEC-03 | Mesin penyimpanan node: redb atau RocksDB | Implementasi MOD-05 | ❓ |
| DEC-04 | UI mobile: Tauri 2 di semua platform atau UI native | MOD-15, MOD-16 | ❓ |
| DEC-05 | Jalur distribusi mobile | Rilis mobile | ❓ |
| DEC-06 | Tampilan Rupiah di aplikasi merchant | MOD-16 | ❓ |
| DEC-07 | Penerbit rilis (code signing dan akun toko aplikasi) | Rilis | ❓ |
| DEC-08 | Identitas repo dan commit: **diputuskan** memakai `ratuaurion-dev` (keterkaitan dengan identitas pencipta disengaja) | — | 🟢 diputuskan (28 Oktober 2026) |
| DEC-09 | **Disetujui:** antarmuka PoW di MOD-03 dengan implementasi dev berbasis BLAKE3 yang HANYA sah di jaringan regtest. Node wajib menolak PoW dev di testnet dan mainnet, dan penolakan itu dibuktikan dengan test. Tujuannya supaya FTR-01 sampai FTR-04 bisa dibangun tanpa menunggu DEC-01 | TSK-M05 | 🟢 diputuskan 28 Oktober 2026 |
| DEC-10 | Dependensi eksternal yang diizinkan. **Bagian 1 disetujui:** `blake3` (hash, MOD-02) dan `k256` (Schnorr BIP340 di secp256k1, MOD-02). Kode proyek tetap bebas `unsafe` dan float. Saat pertama kali memakai sebuah dependensi, agent wajib melaporkan versi, lisensi, fitur yang diaktifkan, dan apakah pustaka itu memakai `unsafe` di dalamnya. **Bagian 2 terbuka:** async, P2P, RPC, logging | Bagian 2: TSK-M12, M14, M18 | 🟡 sebagian diputuskan 28 Oktober 2026 |
