---
judul: Panduan: mencoba wallet Zevrigo dan faucet demo
ringkasan: Pasang wallet desktop, simpan 24 kata pemulihan, minta 100 ZVRG demo dari faucet, lalu kirim transaksi pertama Anda. Sekitar 10 menit.
tanggal: 2026-11-15
kategori: Panduan
penulis: Badak Besi
---
Panduan ini untuk siapa saja yang ingin mencoba Zevrigo tanpa menjalankan node sendiri. Yang dibutuhkan hanya komputer Windows dan koneksi internet. Semua koin di panduan ini adalah koin demo regtest yang tidak bernilai.

## 1. Pasang wallet desktop

Buka [halaman Unduh](/unduh/), lalu unduh installer `zevrigo-wallet-0.2.0-x64-setup.exe`. Berkasnya disimpan di halaman rilis GitHub proyek, bersama berkas `SHA256SUMS.txt`. Sebelum memasang, cocokkan sidik berkasnya di PowerShell:

```powershell
Get-FileHash .\zevrigo-wallet-0.2.0-x64-setup.exe -Algorithm SHA256
```

Angka yang keluar harus sama persis dengan sidik SHA-256 di halaman Unduh dan di `SHA256SUMS.txt`. Kalau berbeda, jangan dipasang.

Installer belum ditandatangani, jadi Windows SmartScreen akan menampilkan peringatan. Pilih **Info selengkapnya**, lalu **Tetap jalankan**. Sebelum memasang, installer menampilkan kebijakan privasi dan lisensi.

## 2. Buat wallet dan simpan 24 kata

1. Buka Zevrigo Wallet, pilih **Wallet baru**, lalu **Buat frasa pemulihan**.
2. Tulis 24 kata yang muncul di kertas, dengan urutan yang benar. Jangan memotret layar dan jangan menyimpannya di chat.
3. Isi beberapa kata yang diminta dari catatan Anda, lalu buat password minimal 8 karakter.
4. Tekan **Simpan wallet**.

24 kata itu adalah satu-satunya cadangan wallet Anda. Siapa pun yang memegangnya bisa memakai koin Anda. Jangan pernah membagikannya, termasuk kepada orang yang mengaku dari tim Zevrigo.

Password hanya mengunci frasa di komputer ini. Kalau password lupa, wallet bisa dipulihkan dengan 24 kata lewat **Pulihkan dari frasa**.

## 3. Pastikan wallet terhubung

Wallet 0.2.0 langsung memakai jaringan **Regtest**, dengan node bawaan gerbang demo `https://zevrigo-rpc.ratuaurion.store`. Di bar status atas harus tertulis **Terhubung** beserta tinggi blok. Pita kuning REGTEST mengingatkan bahwa ini jaringan demo.

## 4. Minta koin demo dari faucet

1. Buka menu **Terima**, lalu salin alamat Anda. Alamat regtest selalu diawali `rzvrg1q`.
2. Buka [halaman faucet](https://zevrigo.ratuaurion.store/faucet/), tempel alamatnya, lalu tekan **Kirim koin**.
3. Tunggu sekitar 2 menit sampai transaksinya masuk blok. Saldo di **Ringkasan** akan bertambah 100 ZVRG.

Batas faucet: 100 ZVRG per permintaan, sekali per pengunjung dan sekali per alamat setiap hari, dan paling banyak 100 permintaan per hari untuk semua pengunjung. Jatah diisi ulang setiap pukul 00.00 UTC, atau 07.00 WIB.

## 5. Kirim transaksi pertama

1. Buka menu **Kirim**, isi alamat tujuan dan jumlah, lalu masukkan password.
2. Biaya bawaan adalah 1 quanta per byte, sekitar 190 quanta untuk transaksi biasa. Angka itu sangat kecil, karena 1 ZVRG sama dengan 10^18 quanta.
3. Transaksi masuk blok dalam sekitar 2 menit, dan bisa dilihat di **Riwayat**.

## Soal privasi

Gerbang RPC melihat alamat yang dicek wallet Anda, karena dari situlah wallet membaca saldo. Gerbang dan faucet tidak menyimpan alamat IP. Faucet menyimpan alamat penerima hanya sampai akhir hari, untuk menegakkan batas satu kali per alamat. Wallet tidak memakai analitik.

Kalau ingin tidak bergantung pada gerbang, Anda bisa menjalankan node sendiri lalu mengganti alamat node di **Pengaturan**.

Versi Android sedang diuji. Kabarnya ada di [artikel wallet Android](/artikel/kabar-wallet-android/).
