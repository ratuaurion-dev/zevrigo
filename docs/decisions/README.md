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
