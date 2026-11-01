# Homeostasis Coin (HSIS): node, penambangan, dan kirim-terima lewat CLI berjalan di regtest

> Diperbarui: 1 November 2026 · Pemilik: pencipta · Tahap: 🟢 node regtest (FTR-01), penambangan (FTR-02), kirim dan terima lewat CLI (FTR-03)

| Status | Ringkasan |
|---|---|
| 🟢 Berjalan baik | Spesifikasi format, validasi, dan RPC lengkap. Node, mempool, RPC, dan miner berjalan untuk regtest; FTR-01 sampai FTR-03 diuji ujung ke ujung. Wallet-core: frasa BIP39, kunci BIP32, alamat bech32m (TSK-M15), serta pemilihan koin dan tanda tangan transaksi (TSK-M16). CLI wallet regtest (TSK-M19). 306 test |
| 🔴 Masalah utama | Testnet publik (FTR-06) terblokir DEC-01: strategi PoW di tengah larangan unsafe dan float |
| ❓ Keputusan dibutuhkan | DEC-01 strategi PoW mainnet |
| ➡️ Langkah berikutnya | FTR-04 jaringan multi-node (spesifikasi P2P TSK-S05, lalu TSK-M14) atau FTR-05 explorer |

**Legenda status:** 🟢 selesai · 🟡 berjalan atau siap dikerjakan · 🔴 terblokir · ⚪ belum dimulai · ❓ perlu keputusan

## Navigasi
| Dokumen | Isi |
|---|---|
| [system-map.md](system-map.md) | Peta modul, dependensi, matriks fitur × modul |
| [modules/](modules/) | Kartu modul MOD-01 sampai MOD-17 |
| [features/README.md](features/README.md) | Fitur FTR-01 sampai FTR-10 |
| [tasks/README.md](tasks/README.md) | Backlog task: modul (TSK-M), fitur (TSK-F), spesifikasi (TSK-S), dan jalur kritis |
| [decisions/README.md](decisions/README.md) | Keputusan DEC-01 sampai DEC-23 |
| [glossary.md](glossary.md) | Istilah |
| [reference/](reference/) | Dokumen Konsep v1.1, Dokumen Aplikasi v0.1, Spesifikasi Format v0.1, Spesifikasi Validasi v0.1, Spesifikasi RPC v0.1, dan Spesifikasi Wallet v0.1 (sumber kebenaran desain) |
