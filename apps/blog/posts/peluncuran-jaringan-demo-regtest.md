---
judul: Jaringan demo Zevrigo diluncurkan, dengan upacara genesis di blok 100
ringkasan: Pada 14 November 2026 chain demo regtest dimulai lagi dari nol. Alokasi genesis 10 juta ZVRG dibagi ke 8 kantong dalam satu transaksi yang bisa Anda periksa sendiri.
tanggal: 2026-11-14
kategori: Pengumuman
penulis: Badak Besi
sampul: /sampul/kisi-batu.jpg
unggulan: ya
---
Pada 14 November 2026 dini hari, jaringan demo Zevrigo dijalankan ulang dari blok genesis. Chain uji sebelumnya, sekitar 870 blok, dihentikan. Chain baru dibuat dengan satu tujuan: siapa pun bisa mencoba wallet, meminta koin demo, dan mengirim transaksi tanpa harus menjalankan node sendiri.

## Ini jaringan demo, bukan mainnet

Jaringan ini memakai mode **regtest**. Koin ZVRG di sini adalah koin demo: tidak bernilai, tidak dijual, dan tidak akan ditukar ke mainnet. Chain demo juga bisa direset. Mainnet Zevrigo belum ada, dan testnet publik baru menyusul.

## Apa yang berjalan

- Lima node: seed, penambang, treasury, faucet, dan pengamat. Empat node operator tersambung ke seed lewat alamat publik, sama seperti peserta lain.
- [Explorer](https://zevrigo.ratuaurion.store), yang menampilkan blok, ringkasan transaksi, dan pasokan.
- Gerbang RPC di `zevrigo-rpc.ratuaurion.store`, dipakai wallet yang tidak menjalankan node sendiri.
- [Faucet demo](https://zevrigo.ratuaurion.store/faucet/), yang membagikan 100 ZVRG demo per pengunjung per hari.
- Penambang yang membuat satu blok kira-kira setiap 2 menit.

Untuk sementara semua layanan berjalan dari satu komputer pengembang lewat Cloudflare Tunnel. Kalau komputer itu mati, jaringan demo ikut berhenti sampai dinyalakan lagi.

## Mengapa peluncurannya tertutup

Blok 0 regtest memuat satu output alokasi genesis sebesar 10.000.000 ZVRG. Kunci rahasianya sengaja publik, yaitu angka `1`, karena regtest adalah jaringan uji. Output ini baru boleh dibelanjakan mulai blok 100. Kalau jaringan dibiarkan terbuka saat chain mendekati blok 100, siapa pun bisa memindahkan alokasi itu lebih dulu.

Karena itu urutan peluncurannya:

1. Semua layanan publik dimatikan, dan data chain lama dicadangkan.
2. Seed dijalankan sendiri tanpa koneksi luar, lalu menambang 99 blok.
3. Satu transaksi upacara membelanjakan alokasi genesis ke 8 kantong dan masuk ke blok 100.
4. Delapan kantong diperiksa langsung di chain, lalu semua layanan dibuka kembali.

## Delapan kantong

Transaksi upacara `f7bd4b482887165856137e6e64da59d8fddf5aaec649cd0da9285acaa8bb9cfa` punya satu input dan delapan output, dengan biaya 462 quanta. Biaya itu diambil dari kantong terakhir.

| Kantong | Jumlah (ZVRG) | Alamat |
|---|---|---|
| Pre-Seed | 1.000.000 | `rzvrg1qljvaslfc3mr0fz2vafppmttkrde6y0u0r7g92fal6lmvznnpnwxsd5g23c` |
| Seed | 1.000.000 | `rzvrg1qx7tkc95n4sdtwzpeckljrqte86wrnmkuntj33jwdfhduj0088gzsat7ctf` |
| Private | 1.500.000 | `rzvrg1q9vwa9ft8amm8tx58depj8ut6vsxeq8yt9y3jhh43kv6g0890k78q78yhf0` |
| Genesis Contributors | 1.500.000 | `rzvrg1q2uhf8tv0tj8frj84w0te5rlc0u4k7hmmj76ptsgruht6hjauf8zs8jq02h` |
| Likuiditas | 1.000.000 | `rzvrg1q2uuk3cykvgrstararzlpfsn803gde2de7vw5ckutnfw80qehsgysj3x8xj` |
| Operasional | 1.500.000 | `rzvrg1qcmpr3uxamudfegpnurac6mgp942r6tknpfcx3z96hhlef9rygdvsckyus6` |
| Pencipta | 2.000.000 | `rzvrg1q00tvqn5hmwzfl0qy4t95vxl509qaa6tga9j4tfg657d8u27tj07qha3m03` |
| Faucet | 500.000 dikurangi biaya | `rzvrg1qyxnvc28tl47kpamrex8z9sfws78ude9leyks6wjy32qd60y83y6s8cfl7y` |

Nama dan jumlah kantong ini adalah **contoh demo**, untuk mencoba alat distribusi dan halaman treasury nanti. Porsi alokasi untuk mainnet belum diputuskan.

Faucet tidak memegang seluruh kantong Faucet. Kantong itu tetap di wallet treasury, dan faucet diisi bertahap. Isi ulang pertama sebesar 10.000 ZVRG dikirim pada 14 November 2026, dan sisanya kembali ke kantong yang sama.

## Memeriksa sendiri

Transaksi upacara ada di [blok 100 di explorer](https://zevrigo.ratuaurion.store/block/?id=100). Anda juga bisa bertanya langsung ke gerbang RPC. Contoh di bawah memeriksa kantong Pencipta. Nilai `payloads` adalah kunci publik di dalam alamat kantong itu.

```bash
curl -s https://zevrigo-rpc.ratuaurion.store \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"scan_utxos","params":{"payloads":["7bd6c04e97db849fbc04aacb461bf47941dee968e96555a51aa79a7e2bcb93fc"]}}'
```

Selama kantong itu belum dipakai, jawabannya berisi satu output dengan `amount` `2000000000000000000000000` dan `txid` transaksi upacara. Jumlah koin selalu ditulis dalam quanta: 1 ZVRG sama dengan 10^18 quanta, jadi angka itu adalah 2.000.000 ZVRG.

## Batas yang perlu diketahui

- Regtest memakai Proof-of-Work pengembangan yang sangat murah. Node lain yang tersambung dengan chain yang kerjanya lebih banyak bisa menimpa chain demo, dan di chain itu alokasi genesis kembali ke kunci publik. Ini batas yang kami terima untuk jaringan demo.
- Explorer hanya menampilkan data sistem: blok, ringkasan transaksi, dan pasokan. Saldo dan alamat orang lain tidak ditampilkan.
- Hadiah 100 blok pertama masuk ke alamat penambang demo.

Mau mencobanya? Ikuti [panduan mencoba wallet dan faucet](/artikel/panduan-wallet-dan-faucet/).
