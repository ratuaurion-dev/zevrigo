# TSK-M39 · Explorer hanya data sistem (tanpa "apa dan siapa") · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-14 (`apps/explorer` Rust dan `web/`), MOD-17 (uji FTR-05) · **Untuk:** privasi pengguna, keamanan pencipta
- **Bergantung pada:** TSK-M38, DEC-28 · **Memblokir:** TSK-M33
- **Estimasi:** M · **Ditulis:** Quick, 3 November 2026

## Tujuan
Pencipta memutuskan (DEC-28) bahwa explorer tidak menampilkan "apa dan siapa" atau ID seseorang, karena itu mengancam
dan membuat orang tidak nyaman. Explorer menjadi "sistem saja": aturan, blok, ringkasan transaksi, dan pasokan hasil
tambang tetap terbuka; alamat, saldo, jumlah per penerima, jejak koin, dan alokasi genesis tidak ditampilkan.

## Isi
| Bagian | Perubahan |
|---|---|
| API (`api`) | `/address/…` dan `/allocation` dihapus (`404`); `/tx` hanya `input_count` dan `output_count` (tanpa alamat, payload, jumlah per output, rujukan input, pembelanja); `/supply` tanpa `genesis_allocation` dan `total_supply`; `/mempool` hanya `count`; `/search` tanpa alamat (`400`) |
| Pustaka (`server`) | `Shared::read_index` untuk membaca indeks langsung (operator dan uji), tidak lewat HTTP |
| Dependensi | MOD-09 (`homeostasis-wallet-core`) tidak dipakai lagi oleh explorer (Cargo.lock: satu rujukan dihapus) |
| Web | Halaman `/address/` dan `/allocation/` serta kode QR dihapus; menu "Alokasi pencipta" dihapus; halaman transaksi hanya ringkasan; Pasokan dan beranda tanpa alokasi genesis dan total beredar (kartu beranda diganti total biaya); mempool hanya jumlah; pencarian tanpa alamat; banner galat node dengan pesan umum; teks dua bahasa disesuaikan |
| npm | `qrcode` dan `@types/qrcode` dihapus beserta 28 paket di bawahnya (30 paket); versi paket lain tidak berubah |
| Indeks | Tidak berubah: alamat dan alokasi genesis tetap dicatat untuk pemeriksaan I1–I4 |

## Test (Rust tetap 380; vitest 14 → 15)
- `api`: `api_serves_indexed_chain` dan `api_rejects_bad_requests_and_reports_status` memastikan kunci tersembunyi tidak
  muncul dan jalur alamat/alokasi `404`.
- `server`: `http_server_serves_web_files_and_api` memeriksa ringkasan transaksi dan `404` alokasi.
- FTR-05: alamat, input/output, dan alokasi genesis diperiksa lewat `Shared::read_index`; API memastikan tidak menyajikannya.
- vitest baru: `mock data exposes no address, per-output amount, coin trail, or genesis allocation`.

## Batas
Data tetap ada di chain. Orang yang menjalankan node atau explorer sendiri tetap bisa membacanya. Explorer ini hanya
memastikan bukan penyelenggaranya yang memamerkan "apa dan siapa".

## Definition of done
- [x] `make check-all` dan `make web-check` lolos; laporan paket `docs/reference/paket-mod14-web-v0.2.txt` diperbarui
- [x] DEC-28; Spesifikasi Explorer v0.1 Bagian 5; Spesifikasi Explorer Web v0.2; MOD-14; FTR-05; README; AGENTS.md
