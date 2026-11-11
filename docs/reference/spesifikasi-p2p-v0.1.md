# Spesifikasi P2P Zevrigo Coin (ZVRG) v0.1: transport libp2p, 5 protokol, sinkronisasi header lalu blok, siaran blok dan transaksi

> Diperbarui: 11 November 2026 · Pemilik: pencipta · Ditulis: Quick · Task: TSK-S05, TSK-M45 · Status: 🟢 v0.1
> Lokasi di repo: `docs/reference/spesifikasi-p2p-v0.1.md`

| Status | Ringkasan |
|---|---|
| 🟢 Dikunci | rust-libp2p 0.57 (DEC-24): TCP atau WebSocket (DEC-33), Noise, Yamux, request-response. 5 protokol dengan encoding kanonis sendiri, sinkronisasi header lalu blok, siaran blok dan transaksi, pemilihan chain berdasarkan chainwork, sanksi rekan, keepalive `status` 60 detik |
| ⏳ Ditunda | Penemuan rekan otomatis (Kademlia, mDNS, daftar rekan bertukar), relay dan NAT, pembatasan bandwidth, perlindungan eclipse, blok kompak, pemangkasan (TSK-M23) |
| ➡️ Langkah berikutnya | Semua task v0.1 selesai: TSK-M27, TSK-M14 (MOD-06), TSK-M28 (daemon), TSK-F04 (lima node, reorg dalam). Batas memori node (TSK-M34) selesai. Sebelum testnet: penemuan rekan, perlindungan eclipse |

P2P **bukan konsensus**. Aturan di dokumen ini bisa diubah lewat versi protokol baru tanpa memecah chain. Semua blok dan
transaksi yang diterima dari rekan tetap diperiksa penuh dengan MOD-04 (V1 sampai V10, T1 sampai T6).

---

## 1. Transport (DEC-24, DEC-10 bagian 5)

| Hal | Nilai |
|---|---|
| Pustaka | `libp2p` 0.57.1, tanpa fitur bawaan; fitur `tcp`, `noise`, `yamux`, `request-response`, `ed25519`, `macros`, `websocket` (sejak TSK-M45, DEC-10 bagian 12), ditambah `libp2p-swarm` 0.48.0 dan `libp2p-tcp` 0.45.0 dengan fitur `tokio` (runtime). Hanya di MOD-06. Fitur `tokio` libp2p sendiri tidak dipakai, karena rujukan lemahnya (`libp2p-quic?/tokio`, dns, mdns, upnp) memasukkan QUIC dan DNS ke Cargo.lock walaupun tidak dikompilasi (TSK-M14, Q78) |
| Lapisan | TCP, atau WebSocket di atas TCP (`/ws`; `/tls/.../ws` hanya sebagai klien), lalu Noise XX (enkripsi dan autentikasi rekan), lalu Yamux (multipleks aliran). Alamat `/ws` ditolak TCP lalu dicoba WebSocket. Noise tetap dipakai di atas WebSocket dan TLS, jadi rekan tetap diautentikasi lewat `PeerId`, bukan lewat sertifikat |
| Alamat DNS | `/dns4/<nama>/tcp/<port>`, `/dns6/...`, dan `/dns/...` (juga di dalam `/tls/sni/<nama>/ws`) di-resolve resolver sistem operasi (`tokio::net::lookup_host`) setiap kali didial, di bawah lapisan WebSocket, sehingga SNI TLS dan kepala `Host` WebSocket berisi nama. Tanpa `libp2p-dns` |
| Identitas rekan | Kunci Ed25519 libp2p, terpisah dari kunci wallet. 32 byte rahasia dari sumber acak sistem operasi saat node pertama kali berjalan, disimpan di `<datadir>/<jaringan>/p2p_key` (32 byte mentah, izin `0600`). `PeerId` diturunkan dari kunci ini |
| Port bawaan | regtest 28_800 · testnet 18_800 · mainnet 8_800 (RPC tetap `x_801`). WebSocket tanpa bawaan (mati); paket publik memakai `x_803` (regtest 28_803) |
| Alamat dengar bawaan | regtest `/ip4/127.0.0.1/tcp/28800`; dengan `--p2p-ws-port <port>` juga `/ip4/127.0.0.1/tcp/<port>/ws`. Testnet dan mainnet ditetapkan bersama TSK-F06 |
| Seed di belakang proxy TLS (DEC-33) | Seed mendengar `/ws` tanpa TLS di 127.0.0.1; proxy (cloudflared, Cloudflare Tunnel) memutus TLS dan meneruskan WebSocket ke port itu. Peserta bergabung dengan `--connect /dns4/<nama>/tcp/443/tls/sni/<nama>/ws/p2p/<PeerId seed>`; TLS diperiksa terhadap sertifikat akar `webpki-roots` |
| Penemuan rekan v0.1 | Hanya alamat yang ditulis operator: opsi `--connect <multiaddr>` boleh berulang (IP atau nama `/dns*`, TCP atau WebSocket). Node menyambung ulang ke alamat itu setiap 10 detik jika putus |
| Batas koneksi | Paling banyak 8 koneksi keluar dan 32 koneksi masuk; satu koneksi per `PeerId` |
| Koneksi menganggur | Ditutup setelah 300 detik tanpa aliran terbuka. Keepalive `status` setiap 60 detik (Bagian 4) membuat koneksi ke rekan aktif tidak pernah menganggur |

Fitur lain libp2p (gossipsub, kad, mdns, identify, ping, quic, dns, relay, autonat, dcutr) tidak dipakai di v0.1, agar
dependensi tetap seminimal mungkin. Ping libp2p tidak diperlukan karena keepalive memakai protokol `status` yang sudah ada.

---

## 2. Nama protokol

Setiap protokol memakai nama `/zvrg/<net>/<nama>/1`. `<net>` adalah 16 karakter hex huruf kecil pertama dari `network_id`
(block_id genesis). Untuk regtest: `/zvrg/8b9eb3187250bbdf/status/1`. Rekan dari jaringan lain tidak bisa membuka aliran
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
4. **Keepalive (TSK-M45, DEC-33):** setiap 60 detik node mengirim `status` ke setiap rekan yang terhubung, dan rekan
   menjawab seperti di langkah 1. Alasannya: Cloudflare menutup WebSocket yang diam sekitar 100 detik, dan nilai itu tidak
   bisa diubah. Jawaban memperbarui ujung rekan (langkah 2 dan 3 berlaku lagi). Permintaan keepalive yang tidak dijawab
   dihitung seperti permintaan lain (Bagian 7: 3 kali berturut-turut, putus tanpa blokir).

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

**Presync header (TSK-M35 bagian B).** Jika indeks header penuh (`Full`, Bagian 9) sebelum cabang rekan terbukti lebih
berat, node menjalankan presync dengan rekan itu. Tahap hitung: header diminta dengan locator chain aktif lalu
`GetHeaders` lanjutan, setiap header diperiksa V2, V3, dan V5 sampai V8 terhadap header sebelumnya tanpa disimpan; yang
diingat hanya ujung, chainwork, 11 cap waktu terakhir, dan ID header di setiap tinggi kelipatan 1_000 (komitmen).
Tahap unduh ulang dimulai begitu chainwork melebihi ujung chain aktif (ujung itu juga jadi komitmen): header diminta
lagi sesudah titik cabang, wajib menyambung dan naik satu tinggi, ditampung paling banyak 1_000, dan disimpan di indeks
header sebagai header terbukti setiap kali ID di tinggi komitmen cocok (ID header mengikat induknya, jadi seluruh
tampungan sama dengan yang dihitung). Header terbukti tidak dihitung dalam batas indeks dan tidak dibuang selama ada
header terbukti yang lebih berat daripada chain aktif; sesudah itu kembali menjadi header biasa. Header yang tidak
menyambung atau tidak sah di tahap hitung disanksi; pesan kosong, V8, ID yang tidak cocok, atau rekan yang berhenti
sebelum ujung menghentikan presync tanpa sanksi. Selama presync, sinkronisasi header biasa dengan rekan itu tidak
dimulai; permintaan yang gagal menghentikan presync, lalu sinkronisasi dicoba lagi pada selang berikutnya.

**Memproses `Blocks`.** Setiap blok diproses seperti blok dari `block` (Bagian 6). `Entry` `00` untuk ID yang sudah
diumumkan rekan dihitung pelanggaran ringan: rekan itu dilewati dan blok diminta dari rekan lain.

**Pemilihan chain.** Chain aktif adalah chain sah dengan chainwork terbanyak, tanpa batas kedalaman reorg dan tanpa checkpoint
(AGENTS.md). Jika chainwork sama, chain yang lebih dulu diterima tetap aktif. Reorg: `undo_tip` sampai titik cabang, lalu
setiap blok chain baru diperiksa V1 sampai V10 dan diterapkan. Jika salah satu blok chain baru tidak sah, blok itu dan
keturunannya ditandai tidak sah, dan node kembali ke chain dengan chainwork terbanyak yang masih sah. Transaksi dari blok
yang dilepas dikembalikan ke mempool jika masih sah, berurutan, paling banyak sebesar batas mempool.

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
memblokir (TSK-M14). Rekan yang masuk lewat Cloudflare Tunnel tampak dari 127.0.0.1 (koneksi dari cloudflared), jadi juga
hanya diblokir per `PeerId`; IP asli peserta tidak terlihat oleh node. Rekan dari alamat yang ditulis lewat `--connect`
juga bisa diblokir.

---

## 8. Opsi daemon (MOD-11, TSK-M28)

`zevrigo-node --regtest --datadir <folder> [--rpc-port <port>] [--p2p-port <port>] [--p2p-ws-port <port>] [--connect <multiaddr>]…`

- `--p2p-port 0` memilih port bebas. Setelah baris `rpc: http://<alamat>`, stdout mencetak satu baris
  `p2p: /ip4/<alamat>/tcp/<port>/p2p/<PeerId>`, sehingga test dan skrip bisa menyambungkan node lain.
- `--p2p-ws-port <port>` (TSK-M45) menambah alamat dengar WebSocket `/ip4/127.0.0.1/tcp/<port>/ws` (`0` = port bebas) dan
  mencetak satu baris `p2p-ws: /ip4/127.0.0.1/tcp/<port>/ws/p2p/<PeerId>`. Tanpa opsi ini node tidak mendengar WebSocket.
- `--connect` menerima alamat IP maupun nama, TCP maupun WebSocket, misalnya
  `/dns4/zevrigo-seed.ratuaurion.store/tcp/443/tls/sni/zevrigo-seed.ratuaurion.store/ws/p2p/<PeerId>`.
- Log rekan (terhubung, putus, sanksi) ke stderr lewat `tracing`.

---

## 9. Batas v0.1 dan risiko yang diketahui

- Tanpa penemuan rekan otomatis: jaringan uji publik (TSK-F06) butuh seed node dan Kademlia atau daftar rekan bertukar.
- Perlindungan eclipse dan sybil minimal (batas koneksi, sanksi). Wajib ditinjau ulang sebelum mainnet.
- Blok dikirim utuh ke setiap rekan; cukup untuk blok 100_000 byte setiap 120 detik, tetapi boros jika rekan banyak.
- Ring (kode C dan assembly) masuk lewat Noise (DEC-24). Reproducible build (TSK-F10) wajib mengunci versi compiler C.
- Badan blok cabang disimpan di penyimpanan node (MOD-05, tabel `side_blocks`, TSK-M35 bagian A); di memori hanya
  header dan ringkasannya. Blok yang dilepas saat reorg langsung dipindah ke sana dan badan blok cabang dibaca satu per
  satu saat disambungkan, sehingga reorg sedalam apa pun tidak menahan badan blok di memori. Indeks blok cabang, indeks
  header, dan catatan blok tidak sah tetap di memori dan hilang saat node dibuka ulang; badan blok cabang lama dibuang
  saat node dibuka, dan cabang diterima lagi dari rekan lewat sinkronisasi. **Batas (TSK-M34, `NodeLimits`, kebijakan
  node, bukan konsensus):**

  | Data | Batas bawaan | Jika penuh |
  |---|---|---|
  | Blok cabang | 10_000 blok dan 64 MiB encoding (di penyimpanan) | Dibuang daun (blok cabang tanpa anak cabang) dengan chainwork paling kecil, jika sama yang paling akhir diterima, sampai dalam batas. Blok cabang di jalur menuju header yang lebih berat daripada chain aktif tidak pernah dibuang, sehingga cabang yang sedang diunduh selalu bisa disambungkan |
  | Indeks header tanpa badan blok | 100_000 header (sekitar 20 MB) | Header daun dengan chainwork paling kecil dibuang jika lebih ringan daripada header baru dan bukan induknya; jika tidak ada, pesan `Headers` berhenti tanpa sanksi (`Full`) dan sinkronisasi header berlanjut setelah badan blok diunduh |
  | Catatan blok tidak sah | 10_000 ID | Catatan tertua dilupakan; blok itu diperiksa ulang jika datang lagi |

  Blok yang dibuang dilupakan (bukan `duplicate` jika dikirim lagi). Pembuangan blok cabang dan header keturunan blok
  tidak sah memakai satu peta induk ke anak, tidak berulang per tingkat.
- **Cabang yang sangat dalam (TSK-M35 bagian B):** cabang bersaing yang lebih dari 100_000 blok di bawah ujungnya belum
  lebih berat daripada chain aktif disinkronkan lewat presync header (Bagian 5), tanpa menyimpan header murah. Memori
  presync per rekan: 40 byte per 1_000 header ditambah tampungan unduh ulang paling banyak 1_000 header. Jumlah header
  yang bisa diperiksa dibatasi aturan cap waktu (V7, V8) dan ASERT, bukan oleh batas node. Aturan konsensus tetap tanpa
  batas reorg dan tanpa checkpoint.

---

## 10. Lokasi kode dan task turunan

| Isi | Modul | Task |
|---|---|---|
| Blok cabang (indeks di memori node; badannya di MOD-05 sejak TSK-M35), pemilihan chain berdasarkan chainwork, reorg, blok yang induknya belum dikenal | MOD-11 | TSK-M27 |
| Indeks header tanpa badan blok (untuk sinkronisasi header dulu) | MOD-11 | TSK-M28 |
| Batas memori blok cabang, indeks header, dan catatan blok tidak sah (`NodeLimits`) | MOD-11 | TSK-M34 |
| Badan blok cabang di penyimpanan (bagian A); presync header untuk cabang yang sangat dalam (bagian B) | MOD-05, MOD-11 | TSK-M35 |
| Swarm libp2p, 5 protokol, codec pesan, kunci rekan, batas koneksi, sanksi | MOD-06 | TSK-M14 |
| Sinkronisasi header lalu blok, siaran, opsi daemon | MOD-11 | TSK-M28 |
| Transport WebSocket, alamat `/dns*`, keepalive `status` (MOD-06); opsi `--p2p-ws-port` (MOD-11); seed lewat Cloudflare Tunnel | MOD-06, MOD-11 | TSK-M45 |
| 4 node atau lebih tetap sinkron, jaringan terpisah lalu tersambung (reorg dalam), node yang tertinggal mengejar | MOD-17 | TSK-F04 |

---

## 11. Keputusan

| ID | Keputusan | Status |
|---|---|---|
| DEC-24 | Transport P2P memakai rust-libp2p 0.57 (TCP, Noise, Yamux, request-response), sesuai AGENTS.md. Konsekuensi: ring dan generasi kedua pustaka RustCrypto ikut masuk lewat MOD-06 | 🟢 diputuskan pencipta 1 November 2026 |
| DEC-10 bagian 5 | `libp2p` 0.57.1 tanpa fitur bawaan, fitur `tcp`, `noise`, `yamux`, `request-response`, `ed25519`, `macros`, serta `libp2p-swarm` dan `libp2p-tcp` dengan fitur `tokio`, hanya di MOD-06 | 🟢 1 November 2026 (konsekuensi DEC-24) |
| DEC-33 | Seed publik sementara lewat Cloudflare Tunnel memakai WebSocket libp2p; keepalive `status` 60 detik; nama `/dns*` di-resolve resolver sistem; regtest dulu | 🟢 diputuskan pencipta 11 November 2026 |
| DEC-10 bagian 12 | Fitur `websocket` libp2p 0.57.1 di MOD-06: +6 paket (libp2p-websocket, futures-rustls, rustls dengan ring, rustls-pki-types, rustls-webpki, webpki-roots), reference/paket-mod06-websocket-v0.1.txt | 🟢 11 November 2026 (konsekuensi DEC-33) |
