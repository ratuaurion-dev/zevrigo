---
judul: Ringkasan tokenomik Zevrigo v0.1
ringkasan: Pasokan tambang 100 juta ZVRG yang turun mulus setiap blok, hadiah ekor 2 ZVRG selamanya, dan alokasi genesis 10 juta ZVRG yang dibagi dengan kunci waktu di chain.
tanggal: 2026-11-15
kategori: Tokenomik
penulis: Badak Besi
---
Parameter pasokan Zevrigo ditetapkan pada 7 November 2026 dan menjadi bagian dari aturan konsensus. Artinya, setiap node memeriksa sendiri jumlah koin di setiap blok. Tidak ada pihak yang bisa menerbitkan koin di luar aturan ini.

## Parameter

| Parameter | Nilai |
|---|---|
| Satuan | 1 ZVRG = 10^18 quanta, dihitung sebagai bilangan bulat |
| Waktu blok | 120 detik, 262.800 blok per tahun |
| Pasokan tambang | 100.000.000 ZVRG |
| Alokasi genesis | 10.000.000 ZVRG di blok 0, di luar pasokan tambang |
| Hadiah ekor | 2 ZVRG per blok, selamanya |
| Slow start | 10.080 blok pertama (sekitar 14 hari) |
| Kematangan hadiah blok | 100 blok |
| Biaya transaksi | Seluruhnya untuk penambang, tanpa pembakaran |

## Hadiah blok

Hadiah dasar dihitung dari sisa pasokan tambang: `(100.000.000 ZVRG − yang sudah tertambang) / 2.274.842`, dibulatkan ke bawah. Hadiah tidak pernah lebih kecil dari 2 ZVRG.

Selama 10.080 blok pertama, hadiah dinaikkan bertahap dari hampir nol sampai penuh. Tujuannya agar penambang awal tidak mendapat terlalu banyak saat jaringan masih sepi. Hadiah blok penuh pertama sekitar 43,96 ZVRG, lalu turun sedikit di setiap blok. Tidak ada *halving* mendadak.

## Jadwal pasokan

| Akhir tahun | Hasil tambang | Total dengan alokasi genesis |
|---|---|---|
| 1 | 10,71 juta | 20,71 juta |
| 2 | 20,45 juta | 30,45 juta |
| 5 | 43,75 juta | 53,75 juta |
| 10 | 68,43 juta | 78,43 juta |
| 20 | 90,06 juta | 100,06 juta |
| 50 | 107,66 juta | 117,66 juta |

Setengah pasokan tambang keluar dalam sekitar 6 tahun. Hadiah ekor dimulai di blok 7.034.557, sekitar tahun ke-27.

## Mengapa ada hadiah ekor

Sesudah pasokan tambang hampir habis, setiap blok tetap menerbitkan tepat 2 ZVRG. Dengan begitu penambang tetap dibayar, dan keamanan chain tidak hanya bergantung pada biaya transaksi. Akibatnya pasokan total tidak punya batas keras. Pertambahannya tetap 525.600 ZVRG per tahun, jadi persentasenya terus mengecil, sekitar 0,4% di tahun ke-50.

## Alokasi genesis

Alokasi genesis 10.000.000 ZVRG adalah satu output di blok 0. Dari sinilah kantong untuk pendanaan awal, likuiditas, operasional, dan bagian pencipta akan diambil. Porsi tiap kategori untuk mainnet **belum diputuskan**. Delapan kantong di jaringan demo hanya contoh.

Jadwal kunci untuk penerima ditegakkan aturan konsensus yang sudah ada, bukan janji:

1. Bagian yang langsung cair dikirim sebagai output biasa.
2. Setiap tahap vesting dikirim sebagai output terkunci (jenis 1) dengan tinggi buka sendiri.
3. Node menolak transaksi yang membelanjakan output terkunci sebelum tingginya tercapai.

Sesudah dikirim, koin itu milik penerima dan tidak bisa ditarik kembali. Siapa pun bisa memeriksa jumlah dan tinggi buka setiap output terkunci di chain.

Di jaringan demo, satu bulan direncanakan dipercepat menjadi 30 blok, supaya jadwal vesting bisa dicoba dalam hitungan jam. Fitur ini sedang dikerjakan.

## Yang tidak ada

- Tidak ada koin pra-alokasi selain alokasi genesis. Publik mendapat ZVRG dengan menambang tanpa izin.
- Tidak ada harga yang ditetapkan. Koin di jaringan demo tidak bernilai dan tidak dijual.
- Penawaran aset kripto di Indonesia diawasi OJK. Bentuk perjanjian apa pun akan diperiksa ahli hukum lebih dulu.

Rincian lengkap ada di dokumen `docs/reference/tokenomik-v0.1.md` di [repositori Zevrigo](https://github.com/ratuaurion-dev/zevrigo).
