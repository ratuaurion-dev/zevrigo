# Homeostasis Coin (HSIS): node dan penambangan regtest berjalan, menuju transaksi lewat CLI

> Diperbarui: 31 Oktober 2026 · Pemilik: pencipta · Tahap: 🟢 node regtest (FTR-01) dan penambangan (FTR-02) · 🟡 menuju FTR-03

| Status | Ringkasan |
|---|---|
| 🟢 Berjalan baik | Spesifikasi format, validasi, dan RPC lengkap. Node, mempool, RPC, dan miner berjalan untuk regtest; FTR-01 dan FTR-02 diuji ujung ke ujung (247 test) |
| 🔴 Masalah utama | Testnet publik (FTR-06) terblokir DEC-01: strategi PoW di tengah larangan unsafe dan float |
| ❓ Keputusan dibutuhkan | DEC-01 strategi PoW mainnet |
| ➡️ Langkah berikutnya | Wallet-core (TSK-M15, M16) sesuai Spesifikasi Wallet v0.1, CLI (TSK-M19), lalu TSK-F03 |

**Legenda status:** 🟢 selesai · 🟡 berjalan atau siap dikerjakan · 🔴 terblokir · ⚪ belum dimulai · ❓ perlu keputusan

## Navigasi
| Dokumen | Isi |
|---|---|
| [system-map.md](system-map.md) | Peta modul, dependensi, matriks fitur × modul |
| [modules/](modules/) | Kartu modul MOD-01 sampai MOD-17 |
| [features/README.md](features/README.md) | Fitur FTR-01 sampai FTR-10 |
| [tasks/README.md](tasks/README.md) | Backlog task: modul (TSK-M), fitur (TSK-F), spesifikasi (TSK-S), dan jalur kritis |
| [decisions/README.md](decisions/README.md) | Keputusan DEC-01 sampai DEC-22 |
| [glossary.md](glossary.md) | Istilah |
| [reference/](reference/) | Dokumen Konsep v1.1, Dokumen Aplikasi v0.1, Spesifikasi Format v0.1, Spesifikasi Validasi v0.1, Spesifikasi RPC v0.1, dan Spesifikasi Wallet v0.1 (sumber kebenaran desain) |
