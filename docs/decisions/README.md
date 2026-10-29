# Keputusan Desain & Arsitektur (`docs/decisions/`)

Folder ini mencatat keputusan arsitektur dan teknis proyek Homeostasis Coin (`DEC-xx`), termasuk perubahan struktur repositori, dependensi antar modul, maupun pemilihan komponen utama.

Legenda: 🟢 selesai/diputuskan · 🟡 berjalan · 🔴 terblokir · ⚪ belum dimulai · ❓ perlu keputusan

## Daftar Keputusan

| ID | Keputusan | Menghambat | Status |
|---|---|---|---|
| DEC-01 | Strategi PoW di tengah larangan `unsafe` dan float: isolasi RandomX, ganti PoW, atau RandomX Rust murni | Implementasi MOD-03 | ❓ |
| DEC-02 | Lisensi: MIT atau Apache 2.0 | Rilis publik | ❓ |
| DEC-03 | **Disetujui:** mesin penyimpanan persisten redb seri 4.x (4.4.0, lisensi MIT OR Apache-2.0, MSRV 1.90, Rust murni tanpa C/C++), hanya di MOD-05 (TSK-M11). `MemoryStore` (TSK-M10) menjadi model acuan, dan `RedbStore` diuji diferensial terhadapnya. Fitur: `default-features = false` ditambah `std` | — | 🟢 diputuskan 29 Oktober 2026 |
| DEC-04 | UI mobile: Tauri 2 di semua platform atau UI native | MOD-15, MOD-16 | ❓ |
| DEC-05 | Jalur distribusi mobile | Rilis mobile | ❓ |
| DEC-06 | Tampilan Rupiah di aplikasi merchant | MOD-16 | ❓ |
| DEC-07 | Penerbit rilis (code signing dan akun toko aplikasi) | Rilis | ❓ |
| DEC-08 | Identitas repo dan commit: **diputuskan** memakai `ratuaurion-dev` (keterkaitan dengan identitas pencipta disengaja) | — | 🟢 diputuskan (28 Oktober 2026) |
| DEC-09 | **Disetujui:** antarmuka PoW di MOD-03 dengan implementasi dev berbasis BLAKE3 yang HANYA sah di jaringan regtest. Node wajib menolak PoW dev di testnet dan mainnet, dan penolakan itu dibuktikan dengan test. Tujuannya supaya FTR-01 sampai FTR-04 bisa dibangun tanpa menunggu DEC-01 | TSK-M05 | 🟢 diputuskan 28 Oktober 2026 |
| DEC-10 | Dependensi eksternal yang diizinkan. **Bagian 1 disetujui:** `blake3` (hash, MOD-02) dan `k256` (Schnorr BIP340 di secp256k1, MOD-02). **Bagian 3 disetujui (29 Oktober 2026):** `redb` 4.4.0, khusus MOD-05 (DEC-03), dipakai mulai TSK-M11. Kode proyek tetap bebas `unsafe` dan float. Saat pertama kali memakai sebuah dependensi, agent wajib melaporkan versi, lisensi, fitur yang diaktifkan, dan apakah pustaka itu memakai `unsafe` di dalamnya. **Bagian 2 terbuka:** async, P2P, RPC, logging | Bagian 2: TSK-M12, M14, M18 | 🟡 sebagian diputuskan 28–29 Oktober 2026 |
| DEC-11 | **Disetujui:** alamat bech32m dengan HRP per jaringan (hsis, thsis, rhsis); dua jenis output: 0 = kunci tunggal atau MuSig2 n-dari-n, 1 = kondisi m-dari-n dengan kunci waktu. Rincian: reference/spesifikasi-format-v0.1.md Bagian 4 dan 8 | TSK-M03 | 🟢 diputuskan 28 Oktober 2026 |
| DEC-12 | **Disetujui:** encoding biner tulis sendiri dan kanonis; varint LEB128 minimal; setiap byte wajib terbaca habis. Rincian: Bagian 2 | TSK-M03 | 🟢 diputuskan 28 Oktober 2026 |
| DEC-13 | Cakupan tanda tangan (sighash): satu mode (usulan di Bagian 9) atau beberapa mode | TSK-M09, TSK-M16 | ❓ |
| DEC-14 | **Disetujui:** MOD-03 pow bergantung pada MOD-02 crypto. PoW dev regtest (DEC-09) memakai `hash(Domain::RegtestPow, …)` dari MOD-02, sehingga BLAKE3 hanya dipanggil di satu crate. PoW mainnet nanti juga butuh BLAKE3 (kunci RandomX). Alternatif yang ditolak: MOD-03 memakai crate blake3 langsung | — | 🟢 diputuskan 29 Oktober 2026 |
| DEC-15 | **Disetujui:** pembagian PoW dan target. MOD-03 (TSK-M05) menerima target sebagai `Target` 256-bit big-endian dan hanya memeriksa `hash ≤ target`. Konversi target compact `u32` di header ke `Target`, rumus ASERT, dan chainwork ada di MOD-04 (TSK-M08, menunggu TSK-S02) | — | 🟢 diputuskan 29 Oktober 2026 |
