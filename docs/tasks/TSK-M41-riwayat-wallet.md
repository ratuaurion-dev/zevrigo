# TSK-M41 · Riwayat transaksi wallet dan sumber acak lintas platform · 🟢 · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-12 (apps/cli), MOD-17 (uji FTR-03) · **Untuk:** TSK-F07 (wallet desktop, DEC-31)
- **Bergantung pada:** TSK-M36, DEC-10 bagian 11 · **Memblokir:** TSK-F07
- **Estimasi:** M

## Tujuan
Layar Riwayat wallet desktop butuh daftar transaksi wallet. Node sengaja tidak mencatat alamat (DEC-28), jadi riwayat
dihitung di sisi wallet dari blok. Wallet desktop memakai pustaka CLI (DEC-31), dan harus berjalan di Windows, sehingga
sumber acak tidak boleh lagi membaca `/dev/urandom`.

## Isi
1. Modul `history` (`apps/cli/src/history`): `WalletKeys` (payload kedua cabang, gap limit 20 yang diperluas saat indeks
   terpakai ditemukan), `HistoryState::apply_block` (masuk, keluar, biaya bila semua input milik wallet),
   `HistoryState::pending_entries` (dari `scan_mempool`), tembolok `history.json` (`load`, `save`, `identity`).
2. `Cli::history`: membaca blok sesudah ujung tembolok (atau dari genesis bila tembolok tidak sah atau ujungnya tidak lagi di
   chain aktif), memeriksa `prev_block_id` bersambung (`CliError::ChainChanged`), menyimpan tembolok dan indeks terpakai.
3. Perintah `history` di CLI (`format_history`).
4. `os_random` memakai `getrandom::fill` (DEC-10 bagian 11).
5. Spesifikasi Wallet Bagian 6, MOD-12, DEC-31.

## Vektor dan uji
- Unit (`history/tests.rs`): coinbase, belanja dengan kembalian dan biaya 0,1 ZVRG, input asing (tanpa biaya), gap limit
  19 → 35 → 60 (tidak dikenali), indeks tersimpan, jumlah txid salah, entri tertunda, tembolok bolak-balik dan penolakan
  (jaringan, wallet, versi, bukan JSON).
- Ujung ke ujung (FTR-03): riwayat saat transaksi tertunda (102 entri A, 1 entri B), riwayat akhir (103 entri A, 2 entri B,
  biaya sama dengan CLI), panggilan kedua dari tembolok sama, wallet yang dipulihkan dari frasa sama.

## Kriteria penerimaan
- [x] `make check-all` lolos (jumlah test bertambah sesuai skrip langkah)
- [x] Node regtest + CLI `history` sungguhan menampilkan coinbase yang ditambang
- [x] Tanpa `unwrap`/`expect`/`panic` di kode non-test, tanpa float, tanpa `unsafe`
