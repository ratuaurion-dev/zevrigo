---
judul: Mengapa Zevrigo memakai Proof-of-Work RandomX
ringkasan: Penambangan terbuka untuk komputer biasa, tanpa daftar validator. Ini alasan memilih RandomX, cara kerjanya di Zevrigo, dan batas yang kami terima.
tanggal: 2026-11-15
kategori: Protokol
penulis: Badak Besi
---
Setiap blockchain harus menjawab satu pertanyaan: siapa yang boleh menambah blok berikutnya? Zevrigo menjawabnya dengan Proof-of-Work. Siapa pun boleh menambang, tanpa izin dan tanpa daftar validator. Blok yang sah adalah blok yang hash-nya memenuhi target kesulitan, dan chain yang diikuti adalah chain dengan kerja kumulatif terbanyak.

## Aturan dasar

| Hal | Aturan |
|---|---|
| Model transaksi | UTXO: koin berupa output yang dibelanjakan utuh, dengan kembalian ke output baru |
| Waktu blok target | 120 detik |
| Penyesuaian kesulitan | ASERT di setiap blok, dengan waktu paruh 720 blok |
| Pemilihan chain | Chain sah dengan kerja kumulatif terbanyak, tanpa checkpoint |
| Ukuran blok | Paling besar 100.000 byte |
| Hash umum | BLAKE3 |
| Tanda tangan | Schnorr BIP340 di kurva secp256k1 |
| Alamat | bech32m, misalnya `rzvrg1q…` untuk regtest |

## Mengapa RandomX

RandomX adalah algoritma Proof-of-Work yang dirancang untuk prosesor komputer biasa. Setiap percobaan hash menjalankan program acak, dan saat menambang memakai data sekitar 2 GB di memori. Pola kerja seperti ini paling cocok dikerjakan CPU umum, sehingga keunggulan perangkat khusus jauh lebih kecil dibanding algoritma lain.

Kami juga menimbang beberapa pilihan lain:

- **SHA256d atau BLAKE3 biasa**: mudah dibuat, tetapi perangkat khusus (ASIC) untuk hash semacam ini sudah umum, sehingga penambangan cepat terpusat.
- **Equihash dan Cuckoo**: ASIC-nya sudah tersedia.
- **Argon2 atau yespower**: pemeriksaan bloknya lambat, dan pustakanya untuk Rust belum kuat.
- **RandomX versi Rust murni**: belum ada yang matang.

## Cara Zevrigo memakai RandomX

Zevrigo memakai kode C++ resmi RandomX lewat pustaka `randomx-rs`. Satu-satunya perubahan adalah garam (salt) Argon2 di dalam RandomX, yang diganti menjadi `RandomZVRG\x01`. Dengan begitu hash Zevrigo berbeda dari hash jaringan lain yang memakai RandomX standar.

- Kunci RandomX diganti setiap 2.048 blok, memakai blok yang sudah tertimbun 64 blok.
- Node memeriksa blok dalam mode ringan, dengan cache sekitar 256 MiB.
- Penambang memakai mode cepat, dengan data sekitar 2 GB.

Kode Zevrigo sendiri ditulis tanpa `unsafe` dan tanpa bilangan pecahan. Satu-satunya pengecualian ada di dalam pustaka RandomX resmi, yang memang membutuhkannya.

## Batas yang kami terima

RandomX tidak membuat jaringan kebal dari penambang besar. CPU dan server sewaan yang biasa menambang Monero bisa dengan mudah pindah ke Zevrigo, dan perubahan salt hanya penghalang kecil. Pada 2025, Monero sendiri mengalami reorganisasi 18 blok yang dikaitkan dengan kelompok penambang Qubic. Risiko yang sama berlaku untuk chain kecil mana pun yang memakai algoritma populer.

## Jaringan demo tidak memakai RandomX

Jaringan demo regtest memakai fungsi Proof-of-Work pengembangan yang sangat ringan, supaya uji dan latihan bisa berjalan cepat di satu komputer. RandomX dipakai di testnet dan mainnet. Testnet publik belum diluncurkan, dan mainnet menyusul sesudah testnet.

Aturan pasokan koin dijelaskan di [ringkasan tokenomik v0.1](/artikel/ringkasan-tokenomik-v0-1/).
