# Homeostasis Coin (HSIS): node, penambangan, kirim-terima lewat CLI, dan jaringan multi-node berjalan di regtest

> Diperbarui: 2 November 2026 · Pemilik: pencipta · Tahap: 🟢 node regtest (FTR-01), penambangan (FTR-02), kirim dan terima lewat CLI (FTR-03), jaringan multi-node dan reorg dalam (FTR-04), explorer minimal (FTR-05)

| Status | Ringkasan |
|---|---|
| 🟢 Berjalan baik | Spesifikasi format, validasi, dan RPC lengkap. Node, mempool, RPC, dan miner berjalan untuk regtest; FTR-01 sampai FTR-04 diuji ujung ke ujung. Wallet-core: frasa BIP39, kunci BIP32, alamat bech32m (TSK-M15), serta pemilihan koin dan tanda tangan transaksi (TSK-M16). CLI wallet regtest (TSK-M19). Cabang chain dan reorg di node (TSK-M27). P2P libp2p (TSK-M14) di daemon (TSK-M28): lima node uji tetap sinkron, terpisah lalu tersambung dengan reorg 10 blok, node offline dan node baru mengejar (TSK-F04). 343 test |
| 🔴 Masalah utama | Testnet publik (FTR-06) terblokir DEC-01: strategi PoW di tengah larangan unsafe dan float |
| ❓ Keputusan dibutuhkan | DEC-01 strategi PoW mainnet |
| ➡️ Langkah berikutnya | Antarmuka web explorer Next.js + Tailwind + shadcn/ui (DEC-26) selesai (TSK-S07, M31, M38). Pilihan pencipta berikutnya: paket Docker (TSK-M33), DEC-01 (PoW mainnet), atau FTR-07 wallet desktop. FTR-01 sampai FTR-05 selesai |

**Legenda status:** 🟢 selesai · 🟡 berjalan atau siap dikerjakan · 🔴 terblokir · ⚪ belum dimulai · ❓ perlu keputusan

## Navigasi
| Dokumen | Isi |
|---|---|
| [system-map.md](system-map.md) | Peta modul, dependensi, matriks fitur × modul |
| [modules/](modules/) | Kartu modul MOD-01 sampai MOD-17 |
| [features/README.md](features/README.md) | Fitur FTR-01 sampai FTR-10 |
| [tasks/README.md](tasks/README.md) | Backlog task: modul (TSK-M), fitur (TSK-F), spesifikasi (TSK-S), dan jalur kritis |
| [decisions/README.md](decisions/README.md) | Keputusan DEC-01 sampai DEC-25 |
| [glossary.md](glossary.md) | Istilah |
| [reference/](reference/) | Dokumen Konsep v1.1, Dokumen Aplikasi v0.1, Spesifikasi Format v0.1, Spesifikasi Validasi v0.1, Spesifikasi RPC v0.1, Spesifikasi Wallet v0.1, Spesifikasi P2P v0.1, dan Spesifikasi Explorer v0.1 (sumber kebenaran desain) |
