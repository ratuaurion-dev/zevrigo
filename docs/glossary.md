# Glosarium Zevrigo Coin (ZVRG)

- **ZVRG:** Satuan utama koin pada jaringan Zevrigo Coin yang setara dengan tepat `10^18` quanta.
- **quanta:** Satuan terkecil yang tak terbagi pada Zevrigo Coin, disimpan di seluruh kode konsensus sebagai bilangan bulat `u128`.
- **UTXO:** Model pencatatan saldo berbasis keluaran transaksi yang belum dibelanjakan (*Unspent Transaction Output*) tanpa konsep akun global.
- **genesis:** Blok pertama pada rantai (`h = 0`) yang tidak memiliki hadiah tambang dan hanya memuat alokasi pencipta `G`.
- **alokasi pencipta:** Alokasi awal sebesar `4_380_000 ZVRG` (`G`) yang diterbitkan satu kali di blok genesis di luar pasokan tambang utama `S` tanpa kunci waktu.
- **pasokan ekor:** Hadiah blok tetap sebesar tepat `1 ZVRG` (`T`) per blok yang berlaku selamanya ketika perhitungan hadiah dasar turun ke atau di bawah `T`.
- **slow start:** Masa peningkatan hadiah blok secara bertahap dan linier pada tinggi blok `1 ≤ h < 10_080` sebesar `base(M) × h / 10_080` sebelum mencapai hadiah dasar penuh.
- **ASERT:** Algoritma penyesuaian kesulitan * per-blok* berbasis eksponensial dengan waktu paruh `720` blok untuk menjaga waktu blok rata-rata di sekitar target `120` detik.
- **RandomX:** Algoritma Proof of Work ramah CPU yang pada protokol ZVRG menggunakan varian salt Argon2 tersendiri serta pergantian kunci setiap `2_048` blok dengan jeda `64` blok.
- **reorg:** Penggantian cabang rantai aktif oleh cabang sah lain yang memiliki kerja kumulatif Proof of Work lebih besar, tanpa batas kedalaman buatan maupun checkpoint.
- **konfirmasi:** Jumlah blok sah yang telah dibangun di atas blok tempat suatu transaksi tercatat, termasuk blok transaksi itu sendiri sebagai konfirmasi pertama.
- **watch-only:** Mode dompet atau kasir yang hanya menyimpan kunci publik untuk membuat alamat tagihan dan memantau pembayaran masuk tanpa kemampuan menandatangani pengeluaran dana.
- **node pruned:** Node penuh yang memvalidasi seluruh sejarah blok sejak genesis namun memangkas data blok lama yang telah dibelanjakan demi menghemat ruang penyimpanan, sambil mempertahankan set UTXO terkini.
- **node arsip:** Node penuh yang memvalidasi dan menyimpan seluruh blok dari genesis hingga puncak rantai tanpa pemangkasan untuk melayani sinkronisasi historis dan indeks explorer.
