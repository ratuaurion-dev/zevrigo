# Spesifikasi P2P Homeostasis Coin (HSIS) v0.1: transport libp2p, 5 protokol, sinkronisasi header lalu blok, siaran blok dan transaksi

> Diperbarui: 1 November 2026 · Pemilik: pencipta · Ditulis: Quick · Task: TSK-S05 · Status: 🟢 v0.1
> Lokasi di repo: `docs/reference/spesifikasi-p2p-v0.1.md`

| Status | Ringkasan |
|---|---|
| 🟢 Dikunci | rust-libp2p 0.57 (DEC-24): TCP, Noise, Yamux, request-response. 5 protokol dengan encoding kanonis sendiri, sinkronisasi header lalu blok, siaran blok dan transaksi, pemilihan chain berdasarkan chainwork, sanksi rekan |
| ⏳ Ditunda | Penemuan rekan otomatis (Kademlia, mDNS, seed DNS), relay dan NAT, pembatasan bandwidth, perlindungan eclipse, blok kompak, pemangkasan (TSK-M23) |
| ➡️ Langkah berikutnya | Semua task v0.1 selesai: TSK-M27, TSK-M14 (MOD-06), TSK-M28 (daemon), TSK-F04 (lima node, reorg dalam). Sebelum testnet: penemuan rekan, batas memori cabang, perlindungan eclipse |

P2P **bukan konsensus**. Aturan di dokumen ini bisa diubah lewat versi protokol baru tanpa memecah chain. Semua blok dan
transaksi yang diterima dari rekan tetap diperiksa penuh dengan MOD-04 (V1 sampai V10, T1 sampai T6).

---

## 1. Transport (DEC-24, DEC-10 bagian 5)

| Hal | Nilai |
|---|---|
| Pustaka | `libp2p` 0.57.1, tanpa fitur bawaan; fitur `tcp`, `noise`, `yamux`, `request-response`, `ed25519`, `macros`, ditambah `libp2p-swarm` 0.48.0 dan `libp2p-tcp` 0.45.0 dengan fitur `tokio` (runtime). Hanya di MOD-06. Fitur `tokio` libp2p sendiri tidak dipakai, karena rujukan lemahnya (`libp2p-quic?/tokio`, dns, mdns, upnp) memasukkan QUIC, TLS, dan DNS ke Cargo.lock walaupun tidak dikompilasi (TSK-M14, Q78) |
| Lapisan | TCP, lalu Noise XX (enkripsi dan autentikasi rekan), lalu Yamux (multipleks aliran) |
| Identitas rekan | Kunci Ed25519 libp2p, terpisah dari kunci wallet. 32 byte rahasia dari sumber acak sistem operasi saat node pertama kali berjalan, disimpan di `<datadir>/<jaringan>/p2p_key` (32 byte mentah, izin `0600`). `PeerId` diturunkan dari kunci ini |
| Port bawaan | regtest 28_800 · testnet 18_800 · mainnet 8_800 (RPC tetap `x_801`) |
| Alamat dengar bawaan | regtest `/ip4/127.0.0.1/tcp/28800`. Testnet dan mainnet ditetapkan bersama TSK-F06 |
| Penemuan rekan v0.1 | Hanya alamat yang ditulis operator: opsi `--connect <multiaddr>` boleh berulang. Node menyambung ulang ke alamat itu setiap 10 detik jika putus |
| Batas koneksi | Paling banyak 8 koneksi keluar dan 32 koneksi masuk; satu koneksi per `PeerId` |
| Koneksi menganggur | Ditutup setelah 300 detik tanpa aliran terbuka |

Fitur lain libp2p (gossipsub, kad, mdns, identify, ping, quic, dns, websocket, relay, autonat, dcutr) tidak dipakai di v0.1,
agar dependensi tetap seminimal mungkin.

---

## 2. Nama protokol

Setiap protokol memakai nama `/hsis/<net>/<nama>/1`. `<net>` adalah 16 karakter hex huruf kecil pertama dari `network_id`
(block_id genesis). Untuk regtest: `/hsis/b933d7c548dc7845/status/1`. Rekan dari jaringan lain tidak bisa membuka aliran
sama sekali, karena negosiasi protokol libp2p gagal.

| Protokol | Permintaan | Jawaban | Dipakai untuk |
|---|---|---|---|
| `status` | `Status` | `Status` | Pertukaran ujung chain saat terhubung |
| `headers` | `GetHeaders` | `Headers` | Sinkronisasi header |
| `blocks` | `GetBlocks` | `Blocks` | Mengambil badan blok |
| `block` | `Block` (encoding kanonis) | `Ack` | Siaran blok baru |
| `tx` | `Transaction` (encoding kanonis) | `Ack` | Siaran transaksi baru |

---

## 3. Encoding pesan

- Setiap pesan dikirim sebagai `varint<32>(panjang) ‖ isi` di aliran request-response. Panjang maksimum 2_097_152 byte
  (2 MiB). Panjang lebih besar, varint tidak minimal, atau aliran yang berakhir sebelum isi lengkap: rekan disanksi (Bagian 7).
- Isi memakai encoding kanonis Spesifikasi Format v0.1 Bagian 2 (DEC-12): varint LEB128 minimal, `vec<T>` = `varint(n) ‖ T…`,
  dan setiap byte wajib terbaca habis. `Header` (96 byte), `Block`, dan `Transaction` memakai encoding Bagian 5 dan 6.

| Pesan | Isi | Batas |
|---|---|---|
| `Status` | `version: u8` (= 1) ‖ `network_id: bytes32` ‖ `tip_height: u64le` ‖ `tip_id: bytes32` ‖ `chainwork: bytes32` (256-bit big-endian) | 105 byte tetap (1 + 32 + 8 + 32 + 32) |
| `GetHeaders` | `locator: vec<bytes32>` ‖ `stop: bytes32` | locator 1 sampai 64 ID; `stop` nol berarti tanpa batas |
| `Headers` | `headers: vec<Header>` | 0 sampai 2_000 header |
| `GetBlocks` | `ids: vec<bytes32>` | 1 sampai 16 ID |
| `Blocks` | `entries: vec<Entry>`, `Entry` = `00` (tidak punya) atau `01 ‖ Block` | tepat sebanyak `ids`, urutan sama |
| `Ack` | `status: u8` | lihat Bagian 6 |

`Status.version` selain 1 atau `network_id` yang berbeda dari milik node: koneksi ditutup tanpa sanksi.

Koreksi 1 November 2026 (TSK-M14): ukuran `Status` adalah 105 byte; versi pertama dokumen ini salah menulis 73 byte.

---

## 4. Saat terhubung

1. Kedua sisi mengirim `status` begitu koneksi terbentuk dan menjawab `status` dengan ujung chain aktifnya sendiri.
2. Jika `chainwork` rekan lebih besar daripada chainwork ujung sendiri, node memulai sinkronisasi header dengan rekan itu
   (Bagian 5). Jika sama atau lebih kecil, node menunggu siaran.
3. Node menyimpan ujung terakhir yang diketahui per rekan dan memperbaruinya dari `status`, `block`, dan `Headers`.

---

## 5. Sinkronisasi: header dulu, lalu blok

**Locator.** ID blok chain aktif di tinggi `t, t−1, t−2, …, t−9`, lalu jaraknya digandakan (`t−11, t−15, t−23, …`), ditutup
genesis; paling banyak 64 ID (cukup untuk chain setinggi 2^53 blok).

**Menjawab `GetHeaders`.** Cari ID locator pertama yang ada di chain aktif sendiri (genesis selalu ada). Kirim header
chain aktif sesudah blok itu, berurutan, sampai 2_000 header, ujung, atau `stop`. Jika tidak ada ID yang dikenal, kirim
`Headers` kosong.

**Memproses `Headers`.**
1. Header pertama wajib menyambung ke blok yang dikenal, dan setiap header berikutnya wajib menyambung ke header
   sebelumnya. Jika tidak: sanksi.
2. Setiap header diperiksa V2, V3, dan V5 sampai V8 Spesifikasi Validasi (target sah, PoW, sambungan dan tinggi, target
   ASERT, MTP, batas masa depan) terhadap induknya. V4 (Merkle root) diperiksa saat badan blok tiba. Header yang gagal V8 menghentikan pemrosesan tanpa sanksi (jam lokal bisa tertinggal); gagal lainnya: sanksi.
3. Header yang lolos masuk indeks header node beserta chainwork kumulatifnya. Jika jumlahnya 2_000, minta lagi dengan
   locator dari header terakhir.
4. Setelah header habis, jika ada ujung header dengan chainwork lebih besar daripada ujung chain aktif, ambil badan blok
   chain itu yang belum dimiliki dengan `GetBlocks`, 16 ID sekali minta, berurutan dari yang terendah.

**Memproses `Blocks`.** Setiap blok diproses seperti blok dari `block` (Bagian 6). `Entry` `00` untuk ID yang sudah
diumumkan rekan dihitung pelanggaran ringan: rekan itu dilewati dan blok diminta dari rekan lain.

**Pemilihan chain.** Chain aktif adalah chain sah dengan chainwork terbanyak, tanpa batas kedalaman reorg dan tanpa checkpoint
(AGENTS.md). Jika chainwork sama, chain yang lebih dulu diterima tetap aktif. Reorg: `undo_tip` sampai titik cabang, lalu
setiap blok chain baru diperiksa V1 sampai V10 dan diterapkan. Jika salah satu blok chain baru tidak sah, blok itu dan
keturunannya ditandai tidak sah, dan node kembali ke chain dengan chainwork terbanyak yang masih sah. Transaksi dari blok
yang dilepas dikembalikan ke mempool jika masih sah.

---

## 6. Siaran

- Saat node menerima blok yang menjadi ujung chain aktif baru (dari miner lewat RPC atau dari rekan), node mengirim `block`
  ke semua rekan yang belum diketahui punya blok itu, kecuali pengirimnya.
- Saat transaksi diterima mempool (dari RPC atau dari rekan), node mengirim `tx` ke semua rekan kecuali pengirimnya.
- Penerima mengabaikan blok atau transaksi yang sudah dikenal (berdasarkan block_id atau txid), lalu menjawab `Ack`.

| `Ack.status` | Arti | Tindakan node |
|---|---|---|
| `0` | Diterima atau sudah dikenal | — |
| `1` | Induk blok belum dikenal | Penerima memulai sinkronisasi header dengan pengirim |
| `2` | Ditunda (V8: cap waktu terlalu jauh di masa depan untuk jam penerima) | Pengirim boleh mengirim ulang nanti |
| `3` | Tidak sah (V1 sampai V10, T1 sampai T6) | Penerima memberi sanksi kepada pengirim |
| `4` | Ditolak kebijakan mempool (penuh, terlalu besar, bentrok input) | — (bukan pelanggaran) |

Blok yang induknya belum dikenal tidak disimpan; penerima mengandalkan sinkronisasi header.

Rincian `tx` (TSK-M28): input yang tidak ada di view chain + mempool, coinbase yang belum matang, dan kunci waktu kondisi
yang belum terpenuhi bergantung pada view atau jam penerima, jadi dijawab `4` tanpa sanksi. Pelanggaran T1 sampai T6
lain dan galat struktur dijawab `3`. Jawaban `Blocks` yang berisi blok lain dari ID yang diminta dihitung pesan rusak. Transaksi dengan input yang
belum ada di view chain + mempool ditolak dengan status `4`, tanpa sanksi.

---

## 7. Sanksi rekan

| Pelanggaran | Sanksi |
|---|---|
| Blok atau header tidak sah (selain V8), transaksi tidak sah (T1 sampai T6, struktur) | Putus dan diblokir 24 jam |
| Pesan rusak: panjang di atas 2 MiB, encoding tidak kanonis, batas Bagian 3 dilanggar | Putus dan diblokir 24 jam |
| `Headers` yang tidak menyambung | Putus dan diblokir 24 jam |
| `Entry` `00` untuk blok yang sudah diumumkan | Dilewati untuk blok itu |
| Tidak menjawab dalam 30 detik | Permintaan dianggap gagal; setelah 3 kali berturut-turut, putus (tanpa blokir) |

Blokir disimpan di memori per `PeerId` dan per alamat IP, sehingga hilang saat node dimulai ulang. Alamat loopback
(`127.0.0.0/8`, `::1`) hanya diblokir per `PeerId`, supaya beberapa node di satu mesin (uji TSK-F04) tidak saling
memblokir (TSK-M14). Rekan dari alamat yang
ditulis lewat `--connect` juga bisa diblokir.

---

## 8. Opsi daemon (MOD-11, TSK-M28)

`homeostasis-node --regtest --datadir <folder> [--rpc-port <port>] [--p2p-port <port>] [--connect <multiaddr>]…`

- `--p2p-port 0` memilih port bebas. Setelah baris `rpc: http://<alamat>`, stdout mencetak satu baris
  `p2p: /ip4/<alamat>/tcp/<port>/p2p/<PeerId>`, sehingga test dan skrip bisa menyambungkan node lain.
- Log rekan (terhubung, putus, sanksi) ke stderr lewat `tracing`.

---

## 9. Batas v0.1 dan risiko yang diketahui

- Tanpa penemuan rekan otomatis: jaringan uji publik (TSK-F06) butuh seed node dan Kademlia atau daftar rekan bertukar.
- Perlindungan eclipse dan sybil minimal (batas koneksi, sanksi). Wajib ditinjau ulang sebelum mainnet.
- Blok dikirim utuh ke setiap rekan; cukup untuk blok 100_000 byte setiap 120 detik, tetapi boros jika rekan banyak.
- Ring (kode C dan assembly) masuk lewat Noise (DEC-24). Reproducible build (TSK-F10) wajib mengunci versi compiler C.
- Blok cabang disimpan di memori node, tanpa batas jumlah, dan hilang saat node dibuka ulang (diterima lagi dari rekan
  lewat sinkronisasi). Penyimpanan cabang di MOD-05 dan batas memori menyusul sebelum testnet.

---

## 10. Lokasi kode dan task turunan

| Isi | Modul | Task |
|---|---|---|
| Blok cabang (di memori node pada v0.1; MOD-05 tetap hanya chain aktif), pemilihan chain berdasarkan chainwork, reorg, blok yang induknya belum dikenal | MOD-11 | TSK-M27 |
| Indeks header tanpa badan blok (untuk sinkronisasi header dulu) | MOD-11 | TSK-M28 |
| Swarm libp2p, 5 protokol, codec pesan, kunci rekan, batas koneksi, sanksi | MOD-06 | TSK-M14 |
| Sinkronisasi header lalu blok, siaran, opsi daemon | MOD-11 | TSK-M28 |
| 4 node atau lebih tetap sinkron, jaringan terpisah lalu tersambung (reorg dalam), node yang tertinggal mengejar | MOD-17 | TSK-F04 |

---

## 11. Keputusan

| ID | Keputusan | Status |
|---|---|---|
| DEC-24 | Transport P2P memakai rust-libp2p 0.57 (TCP, Noise, Yamux, request-response), sesuai AGENTS.md. Konsekuensi: ring dan generasi kedua pustaka RustCrypto ikut masuk lewat MOD-06 | 🟢 diputuskan pencipta 1 November 2026 |
| DEC-10 bagian 5 | `libp2p` 0.57.1 tanpa fitur bawaan, fitur `tcp`, `noise`, `yamux`, `request-response`, `ed25519`, `macros`, serta `libp2p-swarm` dan `libp2p-tcp` dengan fitur `tokio`, hanya di MOD-06 | 🟢 1 November 2026 (konsekuensi DEC-24) |
