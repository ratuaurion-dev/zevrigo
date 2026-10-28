# Dokumentasi Homeostasis Coin (HSIS)

## Ringkasan 30 Detik

- **Nama Proyek:** Homeostasis Coin
- **Kode Aset:** `HSIS` (`1 HSIS = 10^18 quanta`, disimpan sebagai `u128`)
- **Arsitektur:** Blockchain Layer 1 Proof of Work dari nol dalam Rust — model UTXO minimal tanpa VM serbaguna, hanya mendukung transaksi bawaan (transfer, multi-tanda tangan MuSig2, dan kunci waktu), tanpa kunci admin atau hak istimewa.
- **Status Saat Ini:** `R0: kerangka` (struktur monorepo lengkap dengan kode minimal tanpa logika protokol).

## Legenda Status

🟢 selesai · 🟡 berjalan · 🔴 terblokir · ⚪ belum dimulai · ❓ perlu keputusan

## Navigasi Dokumentasi

- [Peta Sistem & Dependensi (`system-map.md`)](system-map.md) — tabel 17 modul, grafik dependensi Mermaid, dan matriks fitur × modul
- [Kartu Modul (`modules/`)](modules/) — spesifikasi tanggung jawab dan batas tiap modul (`MOD-01` s.d. `MOD-17`)
- [Kartu Fitur (`features/README.md`)](features/README.md) — spesifikasi kemampuan lintas modul (`FTR-xx`, diisi di R2)
- [Daftar Tugas (`tasks/README.md`)](tasks/README.md) — rincian unit kerja (`TSK-xxx`, diisi di R2)
- [Keputusan Arsitektur (`decisions/README.md`)](decisions/README.md) — catatan keputusan terbuka dan yang telah ditetapkan (`DEC-01` s.d. `DEC-08`)
- [Glosarium (`glossary.md`)](glossary.md) — definisi ringkas istilah protokol dan aplikasi HSIS
- [Dokumen Referensi (`reference/README.md`)](reference/README.md) — tempat penyimpanan Dokumen Konsep v1.1 dan Dokumen Aplikasi v0.1
