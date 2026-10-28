# Homeostasis Coin (HSIS): kerangka repo selesai, task kerja tersusun

> Diperbarui: 28 Oktober 2026 · Pemilik: pencipta · Tahap: 🟡 R2 (task kerja tersusun)

| Status | Ringkasan |
|---|---|
| 🟢 Berjalan baik | Struktur 17 modul lengkap dan lolos `make check-all` (R0, commit e31bed7) |
| 🔴 Masalah utama | Testnet publik (FTR-06) terblokir DEC-01: strategi PoW di tengah larangan unsafe dan float |
| ❓ Keputusan dibutuhkan | DEC-13 sighash · DEC-01 strategi PoW mainnet · DEC-10 bagian 2 (async, P2P, RPC, logging) |
| ➡️ Langkah berikutnya | Agent: TSK-M01, lalu TSK-M07 · Pencipta + Quick: DEC-13 dan TSK-S02 (aturan validasi) |

**Legenda status:** 🟢 selesai · 🟡 berjalan atau siap dikerjakan · 🔴 terblokir · ⚪ belum dimulai · ❓ perlu keputusan

## Navigasi
| Dokumen | Isi |
|---|---|
| [system-map.md](system-map.md) | Peta modul, dependensi, matriks fitur × modul |
| [modules/](modules/) | Kartu modul MOD-01 sampai MOD-17 |
| [features/README.md](features/README.md) | Fitur FTR-01 sampai FTR-10 |
| [tasks/README.md](tasks/README.md) | Backlog task: modul (TSK-M), fitur (TSK-F), spesifikasi (TSK-S), dan jalur kritis |
| [decisions/README.md](decisions/README.md) | Keputusan DEC-01 sampai DEC-13 |
| [glossary.md](glossary.md) | Istilah |
| [reference/](reference/) | Dokumen Konsep v1.1, Dokumen Aplikasi v0.1, dan Spesifikasi Format v0.1 (sumber kebenaran desain) |
