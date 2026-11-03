# Spesifikasi Explorer Homeostasis Coin (HSIS) v0.1: indexer SQLite tahan reorg, API HTTP baca saja, halaman alokasi genesis

> Diperbarui: 2 November 2026 · Pemilik: pencipta · Ditulis: Quick · Task: TSK-S06 · Status: 🟢 v0.1
> Lokasi di repo: `docs/reference/spesifikasi-explorer-v0.1.md`

| Status | Ringkasan |
|---|---|
| 🟢 Dikunci | axum 0.8 + SQLite (DEC-25). Indexer membaca node lewat RPC v0.1, mundur sedalam apa pun saat reorg, memeriksa sendiri jumlah koin setiap blok. API JSON baca saja: status, blok, transaksi, alamat, pasokan, alokasi genesis, mempool, pencarian |
| ⏳ Ditunda | Perkiraan daya tambang, peringatan konsentrasi tambang, detail transaksi mempool, testnet dan mainnet |
| ➡️ Langkah berikutnya | Antarmuka web: [spesifikasi-explorer-web-v0.2.md](spesifikasi-explorer-web-v0.2.md) (DEC-26). TSK-M29 (indexer), TSK-M30 (API dan program), dan TSK-F05 (uji ujung ke ujung) selesai |

Explorer **bukan konsensus** dan tidak memvalidasi blok. Explorer memercayai node yang ditunjuknya untuk PoW dan tanda
tangan; siapa pun bisa menjalankan node dan explorer sendiri (Dokumen Aplikasi Bagian 7). Jumlah koin diperiksa explorer
sendiri dengan rumus MOD-04 (Bagian 4), sehingga pasokan dan alokasi genesis bisa diperiksa tanpa percaya pada pencipta.

---

## 1. Komponen (DEC-25)

| Hal | Nilai |
|---|---|
| Program | `homeostasis-explorer` (MOD-14, `apps/explorer`): pustaka `homeostasis_explorer` dan biner tipis |
| Sumber data | Node mana pun lewat JSON-RPC v0.1: `get_chain_info`, `get_block_hash`, `get_block`, `get_mempool`. Tanpa metode RPC baru |
| Penyimpanan indeks | SQLite lewat `rusqlite` 0.41.1 (fitur `bundled`, SQLite 3.54.1 ikut dikompilasi; fitur `fallible_uint` untuk konversi `u64` yang diperiksa), satu file `<datadir>/<jaringan>/explorer.sqlite` |
| HTTP | `axum` 0.8.9 (fitur `http1` dan `tokio` saja; JSON disusun dengan `serde_json`, query string dibaca sendiri), hanya `GET`, JSON UTF-8 |
| Dependensi modul | MOD-01 (tipe), MOD-02 (txid, block_id), MOD-04 (subsidy dan jadwal pasokan), MOD-08 (klien RPC dan encoding nilai), MOD-09 (alamat bech32m). Tanpa MOD-05 |
| Jaringan | regtest saja di v0.1; testnet dan mainnet bersama TSK-F06 |

---

## 2. Skema indeks SQLite

Semua tabel `STRICT`. Indeks adalah data turunan: menghapus file berarti indeks dibangun ulang dari genesis.

| Tabel | Kolom | Kunci dan indeks |
|---|---|---|
| `meta` | `key TEXT`, `value BLOB` | kunci `key`. Isi: `schema_version` (u32le, = 1), `network_id` (32 byte), `mined_total` (16 byte) |
| `blocks` | `height INTEGER`, `block_id BLOB`, `prev_block_id BLOB`, `merkle_root BLOB`, `timestamp INTEGER`, `target INTEGER`, `nonce BLOB`, `size INTEGER`, `tx_count INTEGER`, `subsidy BLOB`, `fees BLOB` | kunci `height`; unik `block_id` |
| `txs` | `txid BLOB`, `height INTEGER`, `position INTEGER`, `size INTEGER`, `is_coinbase INTEGER`, `fee BLOB` | kunci `txid`; indeks `(height, position)` |
| `outputs` | `txid BLOB`, `idx INTEGER`, `height INTEGER`, `kind INTEGER`, `payload BLOB`, `amount BLOB`, `spent_txid BLOB NULL`, `spent_input INTEGER NULL`, `spent_height INTEGER NULL` | kunci `(txid, idx)`; indeks `(payload, height)`, `spent_txid`, `spent_height` |
| `mempool` | `txid BLOB` | kunci `txid` |

Aturan nilai (larangan float, AGENTS.md):
- Jumlah koin (`amount`, `subsidy`, `fees`, `fee`, `mined_total`) disimpan sebagai **BLOB 16 byte big-endian** dari `u128`
  quanta, bukan `INTEGER` (paling banyak 63 bit) dan bukan `REAL`.
- Fungsi agregat SQL (`SUM`, `TOTAL`, `AVG`) **dilarang** pada kolom jumlah; penjumlahan dilakukan di Rust dengan
  `checked_add`. Kolom `REAL` dilarang.
- Hash 32 byte sebagai BLOB 32 byte. `nonce` (`u64`) sebagai BLOB 8 byte big-endian. Tinggi, cap waktu, ukuran, dan
  posisi sebagai `INTEGER` lewat konversi `i64::try_from` yang diperiksa.
- File baru: `PRAGMA journal_mode = WAL`. File dengan `network_id` lain atau `schema_version` lain ditolak saat start
  (galat, keluar kode 1).

---

## 3. Sinkronisasi dan reorg

Satu langkah sinkronisasi (dijalankan setiap 2 detik, dan bisa dipanggil langsung oleh test):
1. `get_chain_info` → tinggi `N` dan ID ujung node.
2. **Titik cabang.** Jika indeks kosong, indeks genesis (tinggi 0) dulu. Jika tidak, mulai dari `min(tinggi indeks, N)`
   turun satu per satu sampai `get_block_hash(k)` sama dengan `block_id` indeks di tinggi `k`. Genesis berbeda: galat
   jaringan, indexer berhenti. **Tanpa batas kedalaman.**
3. **Mundur.** Jika titik cabang `f` di bawah ujung indeks: dalam **satu transaksi SQLite**, hapus `blocks`, `txs`, dan
   `outputs` dengan `height > f`; kosongkan `spent_txid`, `spent_input`, `spent_height` pada output dengan
   `spent_height > f`; hitung ulang `mined_total` dari subsidy blok yang tersisa.
4. **Maju.** Untuk `k = f + 1` sampai `N`: `get_block_hash(k)`, `get_block`. Jika `prev_block_id` tidak sama dengan ujung
   indeks (node reorg di tengah jalan), hentikan langkah ini; langkah berikutnya mencari titik cabang lagi. Setiap blok
   diperiksa (Bagian 4) lalu ditulis dalam satu transaksi SQLite.
5. `get_mempool` → isi tabel `mempool` diganti.

Node yang tidak bisa dihubungi: langkah gagal, indeks lama tetap dilayani, dicoba lagi 2 detik kemudian.
`confirmations = tinggi ujung indeks − height + 1` di setiap jawaban yang memuat blok atau transaksi.

---

## 4. Pemeriksaan explorer sendiri

| # | Pemeriksaan | Dengan |
|---|---|---|
| I1 | Encoding blok kanonis; `block_id(header)` sama dengan `block_id` dari RPC; tinggi header = `k`; `prev_block_id` = ujung indeks | MOD-01, MOD-02 |
| I2 | `txid` setiap transaksi dihitung ulang dan sama dengan `txids` dari RPC | MOD-02 |
| I3 | Setiap input merujuk output di indeks yang belum dibelanjakan; `fee = Σ input − Σ output` setiap transaksi biasa | indeks |
| I4 | Coinbase = `block_subsidy(height, mined_total)` + total biaya blok (DEC-17), dan `mined_total` mengikuti jadwal pasokan | MOD-04 |

Gagal salah satu: blok tidak ditulis, indexer berhenti dengan `index_error` di `/api/v1/status`, API tetap melayani
indeks terakhir yang sah. Ini tanda node yang ditunjuk tidak jujur atau rusak.

---

## 5. API HTTP v0.1

Awalan `/api/v1`. Hanya `GET` (metode lain `405`). Nilai memakai encoding Spesifikasi RPC v0.1 Bagian 2: hash dan
payload hex huruf kecil 64 karakter, jumlah koin string desimal quanta (tidak pernah angka JSON), target compact hex 8
karakter, nonce string desimal. Alamat bech32m regtest (`rhsis1…`, DEC-11). Jumlah tidak pernah dijumlahkan dengan float.

| Jalur | Isi jawaban |
|---|---|
| `/status` | `network`, `indexed_height`, `indexed_block_id`, `node_height`, `node_best_block_id`, `chainwork` (ujung node), `mempool_count`, `index_error` (galat yang menghentikan indexer, string atau `null`), `last_error` (galat langkah sinkronisasi terakhir, misalnya node tidak bisa dihubungi, string atau `null`) |
| `/blocks?before=<h>` | `{"blocks": […]}`, paling banyak 20 blok dengan tinggi `< h` (bawaan: semua), terbaru dulu: `height`, `block_id`, `timestamp`, `size`, `tx_count`, `confirmations` |
| `/block/<kunci>` | kunci = tinggi desimal atau `block_id` hex. Header lengkap, `size`, `subsidy`, `fees`, `confirmations`, `txids`, `next_block_id` (atau `null`) |
| `/tx/<txid>` | `txid`, `block_id`, `height`, `position`, `size`, `is_coinbase`, `fee`, `confirmations`, `input_count`, `output_count` (DEC-28: tanpa alamat, payload, jumlah per output, rujukan input, dan pembelanja) |
| `/supply` | `height`, `mined_total` (Σ subsidy di indeks), `scheduled_mined_total` (jadwal MOD-04 di tinggi itu), `matches_schedule`, `fees_total`, `next_subsidy`, `tip_target`, `average_block_seconds` (rata-rata bilangan bulat 100 blok terakhir). DEC-28: tanpa `genesis_allocation` dan `total_supply` |
| `/mempool` | `count` (DEC-28: tanpa daftar txid) |
| `/search/<q>` | `{"kind": "block" \| "tx", "key": …}`: desimal = tinggi; hex 64 = `block_id`, lalu `txid`. Teks lain (termasuk alamat) `400`. `key` untuk blok selalu `block_id` |

**Hanya data sistem (DEC-28, 3 November 2026).** Explorer tidak menyajikan "apa dan siapa": jalur `/address/…` dan
`/allocation` dihapus (`404`), dan jawaban mana pun tidak memuat alamat, payload, saldo, jumlah per penerima, jejak koin
(input, output, pembelanja), maupun alokasi genesis. Indeks tetap menyimpan semua itu untuk pemeriksaan I1–I4 dan bisa
dibaca pustaka (`Shared::read_index`, `Indexer::address`, `Indexer::allocation`) oleh operator dan uji, tidak lewat HTTP.
Batas: data tetap ada di chain; orang yang menjalankan node sendiri tetap bisa membacanya.

Galat: `400 {"error": "bad_request"}` (kunci, hex, atau angka query tidak sah), `404 {"error": "not_found"}` (untuk
`/tx` ditambah `"in_mempool": true` jika txid ada di mempool), `503 {"error": "not_ready"}` sebelum genesis terindeks, `405 {"error": "method_not_allowed"}`, dan
`500 {"error": "internal"}` jika indeks tidak bisa dibaca. Pencarian hex yang tidak ditemukan juga memuat `in_mempool`.

---

## 6. Program dan privasi

`homeostasis-explorer --regtest --datadir <folder> [--rpc-url <url>] [--http-port <port>] [--web-dir <folder>]`

- `--web-dir`: folder hasil build antarmuka web; aturan penyajian di Spesifikasi Explorer Web v0.1 Bagian 7 (TSK-M31).

- RPC bawaan `http://127.0.0.1:28801`. Port HTTP bawaan regtest 28_802 (testnet 18_802, mainnet 8_802), `0` = port bebas.
- Mendengar di `127.0.0.1` (bawaan). `--http-bind <ip>` (IPv4 atau IPv6) memilih alamat lain, hanya untuk container
  (paket Docker, TSK-M33, Spesifikasi Docker v0.1) atau di belakang reverse proxy dengan batas laju. Setelah siap, stdout
  mencetak `http: http://<alamat>:<port>`; log ke stderr.
- **Batas laju API (TSK-M40):** `--api-rate <n>` (bawaan 10; `0` = tanpa batas). Setiap klien `n` permintaan per detik
  dengan kapasitas `4n`; semua klien bersama `20n` per detik dengan kapasitas `40n` (ember token, bilangan bulat dalam
  mili-permintaan). Hanya jalur `/api` dan `/api/…`; file antarmuka web tidak dibatasi. Di atas batas: `429
  {"error": "rate_limited"}` dengan `Retry-After: 1`, `no-store`, dan kepala keamanan yang sama. Klien dikenali dari
  alamat IP koneksi, tetapi alamat itu tidak disimpan: kunci ember adalah hash 64-bit dengan kunci acak per proses, hanya
  di memori, paling banyak 4.096 ember (klien yang diam dibuang lebih dulu, sesudah itu klien baru berbagi satu ember
  limpahan). Di belakang reverse proxy atau penerus port Docker semua klien tampak dari satu alamat, sehingga batas per
  klien menjadi batas bersama; batas per klien yang sebenarnya dipasang di reverse proxy.
- Berhenti bersih dengan SIGINT atau SIGTERM. Akses SQLite lewat satu koneksi yang dijaga mutex di thread blocking.
- **Privasi pengunjung:** tanpa cookie, tanpa analitik, tanpa log alamat IP, jalur, atau kata pencarian. Log hanya untuk
  start, kemajuan indeks, reorg, dan galat.

---

## 7. Batas v0.1 dan risiko yang diketahui

- Batas laju ada di explorer (`--api-rate`, Bagian 6), tetapi explorer bawaannya tetap hanya mendengar di `127.0.0.1` (di
  Docker hanya diteruskan ke `127.0.0.1` host). Membuka explorer ke publik (testnet) wajib lewat reverse
  proxy dengan batas laju, atau batas laju di MOD-14, sebelum TSK-F06.
- Pencarian titik cabang linear (satu panggilan RPC per tinggi); cukup untuk regtest, pencarian biner menyusul.
- SQLite (sekitar 271 ribu baris C) dan `rusqlite` penuh `unsafe` (FFI); kode proyek tetap tanpa `unsafe` (DEC-10 bagian
  6). Reproducible build (TSK-F10) wajib mengunci compiler C dan melarang `LIBSQLITE3_SYS_USE_PKG_CONFIG`.
- PostgreSQL (Dokumen Aplikasi Bagian 9.2) ditinjau ulang sebelum mainnet (DEC-25).

---

## 8. Task turunan

| Isi | Modul | Task |
|---|---|---|
| Skema SQLite, sinkronisasi dan reorg, pemeriksaan I1–I4, query (pustaka) | MOD-14 | TSK-M29 |
| API HTTP axum, program `homeostasis-explorer`, opsi, sinyal | MOD-14 | TSK-M30 |
| Uji ujung ke ujung: node + miner + explorer, reorg, alokasi genesis dibelanjakan | MOD-17 | TSK-F05 |
| API dan antarmuka hanya data sistem (tanpa alamat, jejak koin, dan alokasi genesis) | MOD-14, MOD-17 | TSK-M39 (DEC-28) |
| Antarmuka web Next.js + Tailwind + shadcn/ui (spesifikasi terpisah, DEC-26) | MOD-14 | TSK-S07, TSK-M31, TSK-M32, TSK-M38 |
| Paket Docker regtest, opsi `--http-bind` (Spesifikasi Docker v0.1) | MOD-14 | TSK-M33 🟢 |
| Batas laju API per klien dan bersama, `429 rate_limited`, tanpa menyimpan alamat IP | MOD-14 | TSK-M40 🟢 |

---

## 9. Keputusan

| ID | Keputusan | Status |
|---|---|---|
| DEC-25 | Explorer v0.1: axum 0.8 + SQLite (`rusqlite` 0.41, `bundled`), indeks `explorer.sqlite`; peta dependensi MOD-14 = MOD-01, 02, 04, 08, 09; MOD-17 boleh memakai pustaka MOD-14 hanya sebagai dev-dependency | 🟢 diputuskan pencipta 2 November 2026 |
| DEC-10 bagian 6 | `axum` 0.8.9 dan `rusqlite` 0.41.1, hanya di MOD-14 | 🟢 2 November 2026 (konsekuensi DEC-25) |
