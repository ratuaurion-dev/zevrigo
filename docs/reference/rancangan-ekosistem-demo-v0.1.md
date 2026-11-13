# Rancangan Ekosistem Demo Zevrigo Coin (ZVRG) v0.1: jaringan demo regtest untuk pemula

> Diperbarui: 12 November 2026 (urutan tahap 1 dan 2, gerbang RPC) · Pemilik: pencipta · Ditulis: Quick · Task: TSK-S08, TSK-M53 · Status: 🟢 v0.1 (DEC-34, DEC-35, DEC-36)
> Lokasi di repo: `docs/reference/rancangan-ekosistem-demo-v0.1.md`

| Status | Ringkasan |
|---|---|
| 🟢 Diputuskan | Regtest di komputer pencipta, 5 node lewat seed publik, statistik jaringan di explorer, upacara genesis + treasury 8 kantong + vesting dipercepat (1 bulan = 30 blok) untuk 40 peserta sungguhan, halaman treasury rinci (DEC-35), faucet 100 ZVRG per hari, pendaftaran lewat APK Wallet, dua APK Tauri 2 diunduh dari situs, layanan publik terpisah per hostname, aplikasi `apps/demo` (MOD-18, DEC-36) |
| 🟢 Disetujui | Teks persetujuan peserta versi 1 (Bagian 10, pencipta 12 November 2026). Rincian kecil diputuskan di task masing-masing |
| ⚠️ Terbesar | Kunci alokasi genesis regtest publik: peluncuran wajib tertutup (Bagian 4.3). Output jenis 1 belum didukung wallet dan CLI (Bagian 4.4) |

Dokumen ini tidak mengubah konsensus, parameter protokol, atau jaringan testnet dan mainnet. Semuanya hanya untuk jaringan
demo regtest.

---

## 1. Tujuan dan prinsip

Jaringan demo yang terasa seperti jaringan sungguhan, agar pemula memahami cara kerja Zevrigo dengan mencoba sendiri:
melihat blok, memiliki wallet, menerima koin, mengirim, menambang, menjalankan node, dan melihat koin terkunci terbuka sesuai
aturan chain. Pesertanya orang sungguhan, dan semuanya berjalan online.

Prinsip:
1. **Koin demo tidak bernilai.** Tertulis di setiap halaman dan aplikasi. Gratis, tanpa harga, tanpa pembayaran, tanpa janji
   ditukar ke testnet atau mainnet. Kategori penerima memakai nama "Investor Demo …" (tokenomik v0.1 Bagian 8).
2. **Jujur tentang batas.** Regtest bisa ditimpa, komputer pencipta bisa mati, dan statistik hanya yang terlihat dari seed.
3. **Tanpa perubahan konsensus.** Vesting, treasury, dan faucet memakai aturan yang sudah ada.
4. **Kunci milik pengguna.** Wallet HP menyimpan kunci di HP; server hanya meneruskan transaksi yang sudah ditandatangani.
5. **Berguna untuk testnet.** Output jenis 1, alat distribusi, statistik, layanan publik, dan APK dipakai lagi nanti.

## 2. Keputusan pencipta (12 November 2026)

| Hal | Keputusan |
|---|---|
| Jaringan | Regtest. ASERT berlaku di regtest (Spesifikasi Validasi Bagian 2), jadi kesulitan naik bila ada yang menambang terus |
| Tempat | Komputer pencipta dulu (sebelum VPS), lewat Cloudflare Tunnel |
| Node | 5 node di Docker, ringan: seed, penambang, treasury, faucet, pengamat. Semua tersambung lewat seed publik |
| Statistik | Node terhubung, node aktif, node operator dan peserta (PeerId resmi), alamat penambang berbeda dalam 720 blok |
| Chain | Chain publik di-reset sekali saat peluncuran; PeerId seed tetap |
| Hadiah penambang demo | Ke satu alamat wallet sungguhan yang tetap |
| Treasury | Upacara genesis memindah `G` ke 8 kantong (Bagian 4.2); halaman treasury termasuk rincian per penerima (DEC-35) |
| Vesting | Dipercepat, 1 bulan = 30 blok; 10 peserta sungguhan per kategori, siapa cepat dia dapat; daftar lewat tombol di APK Wallet |
| Faucet | 100 ZVRG per pengunjung per hari, dari halaman explorer dan tombol APK, satu layanan di node faucet |
| Android | Dua APK terpisah dengan Tauri 2 (DEC-04 untuk demo): Wallet, dan Explorer (antarmuka dibundel + API publik) |
| Distribusi APK | Unduh dari situs + SHA-256, juga di catatan rilis GitHub (DEC-05 untuk demo) |
| Layanan publik | Terpisah per hostname: gerbang RPC, faucet, pendaftaran; satu aplikasi `apps/demo` (MOD-18) dengan tiga mode |

## 3. Arsitektur

```
                         Internet (peserta: browser, APK, node sendiri)
                                        │ HTTPS / WSS
                                Cloudflare (TLS, DDoS)
                                        │ tunnel keluar (tanpa port router)
  Komputer pencipta · WSL · Docker, project zevrigo-publik
  ┌───────────────────────────────────────────────────────────────────────┐
  │ namespace node SEED                                                   │
  │   node seed (--p2p-ws-port 28803) · cloudflared · explorer 28802     │
  │   gerbang RPC 28804 · pendaftaran 28806                               │
  ├───────────────────────────────────────────────────────────────────────┤
  │ node PENAMBANG  + miner-berkala (hadiah ke alamat penambang tetap)    │
  │ node TREASURY   + wallet treasury (dipakai operator lewat CLI)        │
  │ node FAUCET     + faucet 28805 lewat jaringan Docker (wallet panas)   │
  │ node PENGAMAT   (hanya menerima siaran)                               │
  │   keempatnya: --connect /dns4/zevrigo-seed.ratuaurion.store/…/ws     │
  └───────────────────────────────────────────────────────────────────────┘
```

Blok dari node penambang dan transaksi dari node treasury dan faucet keluar ke internet, masuk lewat Cloudflare ke seed, lalu
disiarkan ke semua rekan. Pemula bisa melihat di explorer bahwa blok "datang dari jaringan".

## 4. Komponen

### 4.1 Lima node
- Satu image `zevrigo:regtest`, volume data per node, `restart: unless-stopped`, tanpa port host.
- Empat node baru tersambung ke seed publik dengan alamat Spesifikasi Docker v0.1 Bagian 7. Mereka memakai 4 dari 32 slot
  masuk seed; pemakai APK tidak memakai slot P2P (lewat gerbang RPC).
- Regtest tidak memakai RandomX. Batas `mem_limit` dan `cpus` ditulis dari hasil ukur `docker stats`, bukan tebakan.
- Jika internet putus, node terpisah dari seed lalu reorg saat tersambung lagi (diterangkan di panduan sebagai pelajaran).
- PeerId tetap selama volume tetap. Daftar 5 PeerId operator menjadi konfigurasi explorer.
- `miner-berkala` (TSK-M46, saat ini di namespace seed dengan hadiah ke kunci NUMS) pindah ke node penambang dengan
  `--payout` = payload alamat wallet penambang. Jeda 120 detik.

### 4.2 Treasury dan porsi kantong `G` (contoh demo, bukan porsi resmi)

| Kantong | ZVRG | Porsi |
|---|---|---|
| Investor Demo: Pre-Seed | 1.000.000 | 10% |
| Investor Demo: Seed | 1.000.000 | 10% |
| Investor Demo: Private | 1.500.000 | 15% |
| Investor Demo: Genesis Contributors | 1.500.000 | 15% |
| Likuiditas | 1.000.000 | 10% |
| Operasional & ekosistem | 1.500.000 | 15% |
| Pencipta | 2.000.000 | 20% |
| Faucet | 500.000 | 5% |
| **Jumlah** | **10.000.000** | **100%** |

Porsi resmi `G` tetap ditetapkan di tokenomik (Bagian 6.2). Wallet treasury dipegang operator (CLI di node treasury,
berpassword, frasa disimpan di luar komputer). Layanan faucet hanya memegang wallet panas berisi stok kecil (bawaan diusulkan
10.000 ZVRG), diisi ulang operator dari kantong Faucet: contoh wallet panas dan dingin untuk pemula.

### 4.3 Upacara genesis dan peluncuran tertutup
Blok genesis regtest tidak berubah (Spesifikasi Validasi Bagian 10). Upacara adalah satu transaksi yang membelanjakan output
`G` ke 8 alamat kantong setelah `G` matang (blok 100).

Kunci rahasia output `G` regtest adalah `1` dan sengaja publik, sehingga siapa pun bisa membelanjakan `G` begitu matang. Jika
seed terbuka saat chain baru mencapai blok 100, orang lain bisa memindah `G` lebih dulu. Urutan peluncuran:
1. Hentikan seed dan explorer publik (route tunnel tidak menjawab).
2. Reset chain: hapus `regtest/chain.redb` dan indeks explorer di semua volume, simpan `regtest/p2p_key`.
3. Tambang 99 blok tertutup (G boleh dibelanjakan di blok 100), kirim transaksi upacara dengan
   `zevrigo-demo upacara --kantong <alamat>=<ZVRG>... --kirim` (TSK-M62; tanpa `--kirim` untuk memeriksa rencana lebih
   dulu), tambang 1 blok lagi.
4. Periksa 8 kantong di indeks, lalu buka kembali seed dan explorer.

Batas yang diumumkan: siapa pun yang tersambung kemudian tetap bisa menimpa chain dengan chain yang kerjanya lebih banyak
(PoW dev murah), dan di chain itu `G` kembali ke kunci publik. Chain uji yang sekarang berjalan (tinggi di atas 100 sejak
`miner-berkala`) tidak dipakai untuk upacara.

### 4.4 Vesting dipercepat
Satu bulan demo = 30 blok (±1 jam pada 120 detik per blok). `B` = tinggi blok distribusi.
`lock_height = B + (cliff + k) × 30`, `k = 1 … jumlah bulan vesting`. Kunci dihitung dengan tinggi blok, bukan jam: bila
peserta ramai menambang, koin terbuka lebih cepat.

| Kategori (10 peserta) | Per peserta | Cair | Cliff | Vesting | Per output terkunci | Output per peserta | Terbuka pertama / penuh |
|---|---|---|---|---|---|---|---|
| Pre-Seed | 100.000 | 5.000 (5%) | 12 | 24 | 95.000 / 24 (sisa ke output terakhir) | 25 | B+390 (±13 j) / B+1.080 (±36 j) |
| Seed | 100.000 | 10.000 (10%) | 6 | 18 | 5.000 | 19 | B+210 (±7 j) / B+720 (±24 j) |
| Private | 150.000 | 37.500 (25%) | 3 | 12 | 9.375 | 13 | B+120 (±4 j) / B+450 (±15 j) |
| Genesis Contributors | 150.000 | 75.000 (50%) | 0 | 6 | 12.500 | 7 | B+30 (±1 j) / B+180 (±6 j) |

- Total 640 output untuk 40 peserta; empat kantong Investor Demo habis tepat.
- Pembagian dalam quanta (bilangan bulat); sisa pembagian masuk output terakhir.
- Output terkunci = jenis 1 dengan kondisi m = 1, n = 1 dan kunci penerima (tokenomik v0.1 Bagian 6.4). Node menolak
  pembelanjaan sebelum `lock_height` (Spesifikasi Validasi T4).
- **Belum ada di kode:** wallet-core (MOD-09) dan CLI (MOD-12) belum bisa membuat, memindai, menampilkan, dan membelanjakan
  output jenis 1; alat distribusi (daftar penerima menjadi transaksi) belum ada. Konsensus sudah mendukung.

### 4.5 Statistik jaringan di explorer
Butuh metode RPC baru (usul `get_peer_info`: PeerId, arah, waktu tersambung, `status` terakhir, tinggi dan ujung rekan) dan
API explorer baru (usul `/api/v1/network`).

| Angka | Definisi |
|---|---|
| Node terhubung | Rekan yang tersambung ke seed sekarang, per PeerId, ditambah seed sendiri |
| Node aktif | Terhubung, `status` diterima dalam 120 detik terakhir, dan ujung chain sama dengan seed |
| Node operator / peserta | Berdasarkan daftar PeerId resmi |
| Alamat penambang | Jumlah payload penerima coinbase yang berbeda dalam 720 blok terakhir (angka saja) |
| Porsi alamat terbesar | Persen blok dari payload penerima terbanyak (tanpa menampilkan payloadnya) |
| Kesulitan | Target dan rata-rata waktu blok (sudah ada di `/supply`) |

Label di halaman: "dilihat dari seed", "alamat penambang, bukan jumlah orang". Data peer adalah data sistem (DEC-28).

### 4.6 Halaman treasury (DEC-35)
Hanya (a) 8 kantong `G` berlabel dan (b) alamat yang menerima distribusi dari kantong itu. Halaman alamat umum tetap tidak ada.
Isi: transaksi upacara genesis; saldo tiap kantong; per peserta ("Pre-Seed #3"): alamat, total, sudah terbuka, masih terkunci,
tinggi buka berikutnya; grafik koin terkunci per tinggi buka (tokenomik v0.1 Bagian 7). Tanpa nama dan tanpa kontak.

### 4.7 Faucet
- Satu layanan di namespace node faucet (memakai RPC node faucet lewat loopback), dua pintu: halaman di explorer dan tombol
  di APK Wallet. Batas 100 ZVRG per pengunjung per hari dan per alamat.
- `cloudflared` (namespace seed) menjangkau faucet lewat jaringan Docker internal; route publik ke `http://node-faucet:28805`.
  Faucet mendengar di antarmuka jaringan Docker, bukan port host, dan mempercayai `CF-Connecting-IP` hanya dari alamat
  namespace seed. Cara menetapkan alamat itu (nama layanan saat berjalan atau alamat tetap jaringan compose) dipilih di
  task faucet. Konektor `cloudflared` kedua dengan token yang sama tidak dipakai, karena Cloudflare membagi permintaan ke
  semua konektor.
- Pengunjung dikenali tanpa menyimpan IP: hash alamat dengan kunci acak yang berganti setiap hari, hanya di memori (pola
  TSK-M40, TSK-M43). Batas per alamat disimpan di volume faucet.
- Anti-bot: Cloudflare Turnstile di web; di APK diperiksa di task APK Wallet. Jika tidak bisa, faucet APK hanya untuk alamat
  terdaftar atau dengan batas lebih kecil.
- Untuk pemula: faucet hanya ada di demo; di mainnet tidak ada koin gratis (tokenomik v0.1 Bagian 5).
- Bagian A (TSK-M54): `zevrigo-demo faucet` sudah ada (wallet panas regtest dengan frasa di volume faucet, `GET`/`POST
  /api/v1/faucet`, batas per pengunjung, per alamat, dan harian, kiriman berantai lewat mempool, CORS hanya untuk explorer,
  `CF-Connecting-IP` hanya dari `--trust-proxy-host`). Bagian B: layanan di `compose.publik.yaml`, route
  `zevrigo-faucet.ratuaurion.store`, halaman faucet di explorer, dan anti-bot.
- Pengisian (keputusan pencipta 13 November 2026): upacara mengirim 500.000 ZVRG ke kantong Faucet di wallet treasury, bukan
  ke wallet panas layanan faucet. Faucet diisi ulang bertahap (misalnya 10.000 ZVRG) dengan `zevrigo-cli send --dari
  <alamat kantong Faucet>` (TSK-M63), sehingga kantong lain tidak tersentuh dan sisa kantong Faucet terlihat di explorer.

### 4.8 Pendaftaran Investor Demo
- Tombol "Daftar Investor Demo" di APK Wallet, aktif setelah peserta mencentang persetujuan (Bagian 10).
- Wallet menandatangani pesan pendaftaran (Schnorr, kunci alamat itu) berisi kategori, alamat, waktu, dan versi teks
  persetujuan. Format pesan baru dengan domain terpisah dari sighash transaksi, sehingga tanda tangannya tidak bisa dipakai
  untuk transaksi.
- Siapa cepat dia dapat: satu pendaftaran per pengunjung per hari, satu slot per alamat, antrean tiap kategori ditutup saat
  10 slot terisi. Operator boleh menolak yang mencurigakan; penolakan tidak menghapus slot orang lain.
- Operator memeriksa antrean lalu menjalankan alat distribusi dari wallet treasury.

### 4.9 Gerbang RPC untuk wallet peserta (TSK-M53)
`zevrigo-demo rpc` (MOD-18) di namespace node seed, `127.0.0.1:28804`, hostname `zevrigo-rpc.ratuaurion.store`. Dipakai
wallet desktop dan APK Wallet peserta yang tidak menjalankan node sendiri.
- Satu permintaan JSON-RPC 2.0 per `POST`. Hanya metode aman yang diteruskan ke RPC node seed: `get_chain_info`,
  `get_block_hash`, `get_block`, `get_mempool`, `scan_utxos`, `scan_mempool`, `send_raw_transaction`. Hasil atau galat node
  (kode, pesan, `data`) dikembalikan apa adanya, sehingga wallet bekerja sama seperti ke node sendiri.
- `get_block_template`, `submit_block`, dan metode lain: `-32601` tanpa dikirim ke node (menambang tetap lewat node sendiri).
  Batch: `-32600`. Badan di atas 256 KiB: `413`. Selain `POST`: `405`. Node tidak menjawab: `-32603` tanpa rincian.
- Batas laju per pengunjung 20 permintaan per detik (kapasitas 80) dan bersama 400 per detik, diperiksa sebelum badan dibaca;
  pengunjung dikenali dari `CF-Connecting-IP` hanya untuk koneksi dari loopback, alamat tidak disimpan dan tidak dicatat.
- Server melihat alamat yang dicek wallet; ini diumumkan. Riwayat wallet memindai blok (`get_block` per blok), sehingga makin
  berat seiring chain tumbuh (±720 blok per hari); dipantau, metode bantu menyusul bila perlu.
- Klien RPC MOD-08 mendukung `https://` sejak TSK-M60, sehingga CLI dan wallet MOD-15 bisa memakai gerbang ini lewat
  isian alamat node (`--rpc-url`, Pengaturan wallet).

### 4.10 Dua APK

| | APK Explorer | APK Wallet |
|---|---|---|
| Isi | Antarmuka explorer (static export) dibundel, alamat API bisa diatur | Wallet MOD-15 untuk Android |
| Butuh | CORS di API explorer untuk origin aplikasi | HTTPS ke gerbang RPC (klien RPC sekarang tanpa TLS: TLS lewat paket baru atau plugin HTTP Tauri), output jenis 1, tombol faucet, tombol daftar, label REGTEST besar |
| Paket (usul) | `store.ratuaurion.zevrigo.explorer` | `store.ratuaurion.zevrigo.wallet` |
| Tempat di repo | `apps/explorer/android` (DEC-36) | `apps/wallet` (MOD-15) |

- Satu keystore untuk kedua paket, di luar repo, izin `600`, dicadangkan di luar komputer. Keystore hilang berarti pembaruan
  tidak bisa dipasang di atas versi lama.
- Toolchain Android (SDK, NDK, JDK, target Rust) dipasang di WSL lewat skrip langkah.

### 4.11 Halaman panduan
"Tentang demo" (koin tidak bernilai, data apa yang terlihat, jam operasional, chain bisa direset), "Cara bergabung" (pasang
APK, peringatan aplikasi tak dikenal, cadangan 24 kata, faucet, kirim, menjalankan node dan miner sendiri dengan Docker), dan
"Belajar" (blok, kesulitan dan ASERT, slow start, kematangan coinbase, vesting, wallet panas dan dingin, risiko 51%).

## 5. Hostname dan port

| Hostname | Tujuan route | Isi | Status |
|---|---|---|---|
| `zevrigo.ratuaurion.store` | `localhost:28802` | Explorer, halaman faucet, treasury, panduan, unduh APK | ada |
| `zevrigo-seed.ratuaurion.store` | `localhost:28803` | Seed P2P WebSocket | ada |
| `zevrigo-rpc.ratuaurion.store` | `localhost:28804` | Gerbang RPC wallet | baru |
| `zevrigo-faucet.ratuaurion.store` | `http://node-faucet:28805` (jaringan Docker) | Faucet | baru |
| `zevrigo-daftar.ratuaurion.store` | `localhost:28806` | Pendaftaran Investor Demo | baru |

Semua route tipe HTTP (TLS diputus Cloudflare), satu tingkat subdomain (sertifikat `*.ratuaurion.store`). Testnet nanti memakai
seri 1880x.

## 6. Urutan task

Diubah pencipta 12 November 2026 ("dahulukan jalur orang awam"): peserta dengan laptop Windows memakai wallet desktop
MOD-15 lebih dulu, Android menyusul. Faucet diisi dari kantong Faucet di `G` (Bagian 4.2), jadi upacara genesis dan reset
chain dimajukan ke tahap 1. Tahap 2 tidak butuh reset kedua: vesting dibagi dari kantong Investor Demo yang sudah ada.

**Tahap 1: orang awam bisa memakai wallet di laptop**

| # | Task | Isi | Bergantung |
|---|---|---|---|
| 1 | TSK-M47 | 5 node di compose publik; bagian B: wallet penambang dan treasury dibuat pencipta | TSK-S08 |
| 2 | TSK-M53 | `apps/demo` (MOD-18) mode `rpc`: gerbang RPC di `zevrigo-rpc.ratuaurion.store` (Bagian 4.9) | TSK-S08 |
| 3 | TSK-M60 | HTTPS di klien RPC MOD-08: fitur `tls` `jsonrpsee-http-client` (pilihan pencipta, DEC-10 bagian 14) | TSK-M53 |
| 4 | TSK-M61 | Wallet desktop versi demo: alamat node bawaan gerbang publik, label "REGTEST: koin demo", installer Windows dibangun GitHub Actions dan ditandatangani SignPath Foundation (DEC-07) | TSK-M60 |
| 5 | TSK-M62 | Alat upacara genesis regtest `zevrigo-demo upacara` (MOD-18): `G` (kunci rahasia `1`) ke 8 kantong (Bagian 4.2) | TSK-M47 |
| 6 | TSK-F11 bagian 1 | Peluncuran tertutup (Bagian 4.3): reset, upacara, buka kembali | TSK-M62 |
| 7 | TSK-M54 | Mode `faucet` di node faucet, halaman faucet di explorer, Turnstile; stok diisi dari kantong Faucet | TSK-M53, TSK-F11 bagian 1 |
| 8 | TSK-M59 bagian 1 | Halaman panduan dasar, teks persetujuan, unduh installer + SHA-256 | TSK-M61, TSK-M54 |

**Tahap 2: statistik, vesting, treasury, Android**

| # | Task | Isi | Bergantung |
|---|---|---|---|
| 9 | TSK-M48, TSK-M49 | `get_peer_info` (MOD-06, 08, 11); `/api/v1/network` dan tampilan statistik (MOD-14) | TSK-M47 |
| 10 | TSK-M50, TSK-M51 | Output jenis 1 di MOD-09; CLI jenis 1 dan alat distribusi di MOD-12; wallet desktop menampilkan koin terkunci | TSK-S08 |
| 11 | TSK-M52 | Halaman treasury dan konfigurasi kantong (MOD-14, DEC-35) | TSK-M51 |
| 12 | TSK-M55 | Mode `daftar`: pesan bertanda tangan, antrean, alat operator | TSK-M50, TSK-M53 |
| 13 | TSK-M56 | Toolchain Android di WSL (skrip pemasangan) | — |
| 14 | TSK-M57 | APK Explorer (`apps/explorer/android`) dan CORS | TSK-M49, TSK-M52, TSK-M56 |
| 15 | TSK-M58 | APK Wallet: HTTPS, jenis 1, faucet, daftar | TSK-M50, TSK-M54, TSK-M55, TSK-M56, TSK-M60 |
| 16 | TSK-M59 bagian 2 | Panduan APK, unduh APK + SHA-256 | TSK-M57, TSK-M58 |
| 17 | TSK-F11 bagian 2 | Buka pendaftaran, distribusi vesting dari kantong Investor Demo, pengumuman | semua di atas |

Semua dikerjakan di chain uji yang berjalan; reset hanya sekali di TSK-F11 bagian 1. Paket baru dicatat di DEC-10 pada task
yang memakainya. Pembagian task bisa berubah saat kartunya ditulis.

## 7. Risiko dan mitigasi

| Risiko | Mitigasi |
|---|---|
| `G` dicuri saat peluncuran (kunci `1` publik) | Peluncuran tertutup Bagian 4.3 |
| Chain ditimpa peserta (PoW dev murah) | Diumumkan; ASERT menaikkan kesulitan; reset dan distribusi ulang bila perlu |
| Komputer pencipta mati | Jam operasional diumumkan; sleep Windows mati; Docker/WSL otomatis; `restart: unless-stopped` |
| Disangka penawaran investasi | Nama "Investor Demo", gratis, tanpa harga, label tidak bernilai, persetujuan peserta |
| Privasi peserta (alamat publik) | Persetujuan; alamat khusus demo; tanpa nama dan kontak; IP tidak disimpan |
| Faucet dikuras bot | Batas per pengunjung dan per alamat, Turnstile, stok wallet panas kecil |
| Keystore atau frasa treasury hilang | Cadangan di luar komputer; keystore di luar repo |
| APK palsu | SHA-256 di situs dan di catatan rilis GitHub |
| Slot seed penuh (32) | Pemakai APK lewat gerbang RPC; hanya node peserta yang memakai slot |
| Wallet HP lambat (pemindaian blok) | Dipantau; metode bantu menyusul bila perlu |

## 8. Keputusan

| ID | Keputusan | Status |
|---|---|---|
| DEC-34 | Ekosistem demo regtest menurut dokumen ini | 🟢 12 November 2026 |
| DEC-35 | Pengecualian sempit DEC-28 untuk treasury demo regtest | 🟢 12 November 2026 |
| DEC-36 | Struktur repo: `apps/demo` (MOD-18) dan `apps/explorer/android` | 🟢 12 November 2026 |
| DEC-04 | Untuk demo regtest: Tauri 2 di Android | 🟡 demo saja |
| DEC-05 | Untuk demo regtest: unduh dari situs + SHA-256 | 🟡 demo saja |
| DEC-07 | Untuk installer wallet desktop demo: SignPath Foundation (build GitHub Actions dari repo publik) | 🟡 permohonan belum |
| DEC-10 bagian 13 | MOD-18 memakai axum, jsonrpsee (`client-core`), serde_json, thiserror, tokio, tracing; tanpa paket baru | 🟢 12 November 2026 |
| DEC-10 bagian 14 | HTTPS klien RPC lewat fitur `tls` jsonrpsee-http-client, 11 paket (pilihan pencipta) | 🟢 13 November 2026 |

## 9. Rincian yang diputuskan di task
Peserta memilih kategori sendiri atau diberi kategori yang masih kosong (usul: memilih sendiri, TSK-M55); stok awal wallet
panas faucet (usul 10.000 ZVRG, TSK-M54); cara faucet mengenali alamat namespace seed (TSK-M54); anti-bot di APK (TSK-M58).

## 10. Teks persetujuan peserta (versi 1, disetujui pencipta 12 November 2026)

Ditampilkan sebelum tombol "Daftar Investor Demo" dan "Minta koin" aktif. Nomor versi teks (1) ikut ditandatangani saat
pendaftaran. Mengubah teks berarti versi baru, dan pendaftar lama tidak dianggap menyetujui versi baru.

> **Jaringan demo Zevrigo (regtest)**
> 1. Koin di jaringan ini adalah koin demo untuk belajar. Koin ini tidak bernilai, tidak bisa dijual, dan tidak akan ditukar
>    ke testnet atau mainnet.
> 2. Program "Investor Demo" adalah simulasi cara kerja kunci waktu. Ini bukan penawaran investasi. Saya tidak membayar apa
>    pun dan tidak dijanjikan apa pun.
> 3. Alamat yang saya daftarkan, jumlah koin, dan jadwal bukanya akan tampil publik di explorer dengan label nomor (misalnya
>    "Pre-Seed #3"), tanpa nama. Saya memakai wallet atau alamat khusus untuk demo ini.
> 4. Jaringan demo berjalan di satu komputer dan bisa mati, ditimpa, atau di-reset kapan saja. Koin dan riwayat saya bisa
>    hilang.
> 5. Saya menyimpan sendiri 24 kata pemulihan wallet saya. Tidak ada yang bisa memulihkannya untuk saya.
>
> ☐ Saya sudah membaca dan setuju.
