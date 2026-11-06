# Spesifikasi Antarmuka Web Explorer Zevrigo Coin (ZVRG) v0.1: React, dua bahasa, tanpa sumber luar

> Diperbarui: 2 November 2026 · Pemilik: pencipta · Ditulis: Quick · Task: TSK-S07 · Status: 🟢 v0.1
> Lokasi di repo: `docs/reference/spesifikasi-explorer-web-v0.1.md`
>
> ⚪ **Diganti** [spesifikasi-explorer-web-v0.2.md](spesifikasi-explorer-web-v0.2.md) (Next.js + Tailwind CSS + shadcn/ui,
> DEC-26 diubah pencipta 2 November 2026, TSK-M38). Dokumen ini disimpan sebagai catatan versi Vite (TSK-M32).

| Status | Ringkasan |
|---|---|
| 🟢 Dikunci | React + TypeScript + Vite (DEC-26), aplikasi statis di `apps/explorer/web` yang membaca API v0.1 di origin yang sama. Bahasa Indonesia (bawaan) dan English. Jumlah koin selalu `bigint`. Tanpa cookie, analitik, font, atau skrip dari luar |
| ⏳ Ditunda | Paket Docker (node + explorer + web), grafik, perkiraan daya tambang, peringatan konsentrasi tambang, mode gelap |
| ➡️ Langkah berikutnya | TSK-M33 paket Docker. TSK-M31 (penyajian `--web-dir`) dan TSK-M32 (aplikasi web) selesai |

Antarmuka web hanya **menampilkan** isi API explorer (Spesifikasi Explorer v0.1 Bagian 5). Tidak ada logika chain,
pemeriksaan, atau kunci di browser. Siapa pun yang menjalankan node dan explorer sendiri mendapat tampilan yang sama.

---

## 1. Komponen (DEC-26)

| Hal | Nilai |
|---|---|
| Lokasi | `apps/explorer/web/` (bagian MOD-14, bukan crate Rust) |
| Perangkat | Vite + React + TypeScript, build statis ke `apps/explorer/web/dist/` (tidak di-commit) |
| Tanpa | Tailwind, shadcn/ui, pustaka router, pustaka i18n, pustaka state, pustaka HTTP, pustaka angka. Semua ditulis sendiri di atas React dan `fetch` bawaan browser |
| Penyajian | Program `zevrigo-explorer --web-dir <folder dist>` di origin yang sama dengan API (tanpa CORS), Bagian 7 |
| Pengembangan | `npm run dev` (server Vite) meneruskan `/api` ke `http://127.0.0.1:28802` |
| Dependensi | DEC-10 bagian 7 (Bagian 8) |

---

## 2. Halaman dan jalur

Jalur memakai hash (`#/…`), sehingga server cukup menyajikan file yang ada tanpa fallback ke `index.html`.

| Jalur | Isi | API |
|---|---|---|
| `#/` | Status (tinggi indeks dan node, galat sinkronisasi), ringkasan pasokan, 20 blok terbaru | `/status`, `/supply`, `/blocks` |
| `#/blocks?before=<h>` | Daftar blok per 20, tautan "lebih lama" | `/blocks?before=` |
| `#/block/<tinggi atau block_id>` | Header lengkap, subsidy, biaya, konfirmasi, daftar transaksi, blok sebelum dan sesudah | `/block/` |
| `#/tx/<txid>` | Blok, posisi, ukuran, biaya, konfirmasi, input dan output (alamat, jumlah, dibelanjakan oleh) | `/tx/` |
| `#/address/<alamat>?page=<n>` | Saldo, diterima, dikirim, jumlah UTXO, riwayat per 25 | `/address/` |
| `#/supply` | Pasokan hasil tambang dibanding jadwal rumus, alokasi genesis, total, biaya, subsidy berikutnya, rata-rata waktu blok | `/supply` |
| `#/allocation?page=<n>` | **Halaman alokasi pencipta**: setiap output genesis, sudah dibelanjakan atau belum, dan riwayat setiap alamat penerima | `/allocation` |
| `#/mempool` | Jumlah dan daftar txid di mempool | `/mempool` |
| Kotak pencarian (setiap halaman) | Tinggi, block_id, txid, atau alamat → pindah ke halaman yang sesuai | `/search/` |

- Transaksi yang belum ada di blok tetapi ada di mempool (`404` dengan `in_mempool: true`) ditampilkan sebagai
  "menunggu di mempool", bukan "tidak ditemukan".
- `503 not_ready` ditampilkan sebagai "indeks belum siap"; `index_error` dari `/status` ditampilkan sebagai peringatan di
  setiap halaman (tanda node yang ditunjuk tidak jujur atau rusak).
- **Konfirmasi selalu ditampilkan** di setiap blok dan transaksi, karena chain tanpa batas reorg. Di bawah 6 konfirmasi
  diberi tanda "belum aman".
- Halaman `#/` dan status diperbarui setiap 10 detik selama halaman terbuka; halaman lain dimuat saat dibuka (tombol
  "muat ulang"). Tanpa WebSocket.

---

## 3. Bahasa

- Bahasa Indonesia (bawaan) dan English. Kamus di kode (`src/i18n/id.ts`, `src/i18n/en.ts`) dengan kunci yang sama;
  test memastikan kedua kamus punya kunci yang persis sama dan tidak ada teks kosong.
- Pilihan pertama: `navigator.language` berawalan `en` → English, selain itu Indonesia. Tombol ganti bahasa di kepala
  halaman. Pilihan disimpan di `localStorage` (`zvrg-explorer-lang`) di browser saja, tidak pernah dikirim ke server.
- Istilah teknis tidak diterjemahkan: block_id, txid, hash, Merkle root, nonce, quanta, ZVRG.

---

## 4. Angka, jumlah koin, dan waktu

- **Jumlah koin** (string desimal quanta dari API) diubah ke `bigint` dan ditampilkan sebagai ZVRG dengan 18 desimal,
  nol di belakang dibuang, pemisah ribuan sesuai bahasa: Indonesia `4.380.000 ZVRG`, `0,005 ZVRG`; English
  `4,380,000 ZVRG`, `0.005 ZVRG`. Nilai quanta lengkap tersedia di atribut `title`.
- **Dilarang** untuk jumlah koin: tipe `number`, `parseFloat`, `Number(`, `toFixed`, `Math.*`, `Intl.NumberFormat`
  (AGENTS.md, larangan floating point). Diperiksa skrip gerbang dengan pencarian pola di `src/` (kecuali file test).
- Tinggi, konfirmasi, ukuran, dan posisi tetap `number` (bilangan bulat dari API, jauh di bawah 2^53).
- **Waktu blok** (detik Unix) ditampilkan dalam UTC `YYYY-MM-DD HH:MM:SS UTC`, disusun dari aritmetika bilangan bulat,
  bukan zona waktu browser.
- Target compact ditampilkan apa adanya (hex 8 karakter); chainwork hex 64 karakter.

---

## 5. Keamanan dan privasi

- Tanpa cookie, tanpa analitik, tanpa sumber luar (font, CDN, gambar, skrip). Semua aset dari build lokal.
- Kebijakan keamanan konten dari server (Bagian 7): `default-src 'none'; script-src 'self'; style-src 'self';
  img-src 'self' data:; font-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'`.
  Akibatnya: tanpa skrip inline, tanpa atribut `style` inline (gaya hanya lewat kelas CSS), tanpa `eval`.
- `dangerouslySetInnerHTML` dilarang; semua teks dari API dirender sebagai teks oleh React. Diperiksa skrip gerbang.
- Isi dari API (txid, alamat, block_id) dimasukkan ke jalur API setelah diperiksa pola (hex 64 huruf kecil, desimal,
  awalan `rhsis1`/`thsis1`/`hsis1` dan karakter bech32) dan `encodeURIComponent`.

---

## 6. Struktur dan uji

| File | Isi |
|---|---|
| `package.json`, `package-lock.json` | Versi persis (tanpa `^`), skrip `dev`, `typecheck`, `test`, `build`, `check` |
| `index.html`, `vite.config.ts`, `tsconfig.json` | Titik masuk, build, TypeScript `strict` |
| `src/api.ts` | Tipe jawaban API (jumlah sebagai `string`) dan klien `fetch` |
| `src/amount.ts` | quanta ↔ `bigint`, format ZVRG per bahasa |
| `src/route.ts` | Mengurai dan menyusun jalur hash |
| `src/i18n/` | Kamus Indonesia dan English, pemilih bahasa |
| `src/pages/`, `src/components/` | Halaman dan komponen |
| `src/*.test.ts(x)` | Uji vitest di lingkungan Node: format jumlah, jalur, kamus, klien API dengan `fetch` tiruan, dan render halaman dengan `react-dom/server` (tanpa jsdom) |

- `make web-check` = `cd apps/explorer/web && npm ci --ignore-scripts && npm run check` (typecheck, test, build).
  `make check-all` tetap untuk Rust saja; skrip gerbang dan CI menjalankan keduanya.
- Node.js ≥ 22.12 (dibutuhkan vitest 5 dan Vite 8). `node_modules/` dan `dist/` diabaikan git.

---

## 7. Penyajian oleh program explorer (MOD-14, TSK-M31)

- Opsi `--web-dir <folder>`: folder hasil build (`apps/explorer/web/dist`). Saat start, folder wajib berisi `index.html`
  (jika tidak, explorer keluar dengan kode 1). Tanpa opsi ini explorer hanya melayani API seperti TSK-M30.
- `/api` dan `/api/…` selalu API. Jalur lain: `/` = `index.html`; file lain hanya jika jalurnya paling banyak 4 segmen,
  setiap segmen hanya huruf, angka, `.`, `_`, `-` dan tidak diawali titik, dengan ekstensi `html`, `js`, `css`, `svg`,
  `png`, `ico`, `woff2`, `txt`. Selain itu `404 not found` (teks). Metode selain `GET` → `405`.
- Cache: file di `assets/` (nama berisi hash isi) `public, max-age=31536000, immutable`; `index.html` dan file lain
  `no-cache`; API dan galat `no-store`.
- Setiap jawaban (API maupun web) membawa `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, dan
  kebijakan keamanan konten Bagian 5.

---

## 8. Dependensi npm (DEC-10 bagian 7)

| Paket | Versi saat spesifikasi ditulis | Lisensi | Peran |
|---|---|---|---|
| `react`, `react-dom` | 19.3.1 | MIT | UI (satu-satunya dependensi yang ikut ke browser) |
| `vite` | 8.3.5 | MIT | Build dan server pengembangan (membawa rolldown dan lightningcss, biner native per platform) |
| `@vitejs/plugin-react` | 6.1.2 | MIT | JSX untuk Vite |
| `typescript` | 7.0.2 | Apache-2.0 | Pemeriksaan tipe |
| `vitest` | 5.0.5 | MIT | Uji |
| `@types/react`, `@types/react-dom` | 19.3.0 | MIT | Tipe |

- Versi persis dikunci saat pemakaian pertama (TSK-M32) di `package.json` dan `package-lock.json` (di-commit).
- Gerbang pemakaian pertama: semua paket dari `registry.npmjs.org` dengan `integrity` sha512; paket yang ikut ke
  browser hanya `react`, `react-dom`, dan `scheduler`, semuanya MIT; daftar paket, lisensi, dan ada tidaknya skrip
  pasang ditulis ke `docs/reference/paket-mod14-web-v0.1.txt`.
- **Lisensi.** Paket yang ikut ke browser wajib permisif (daftar Q78/Q82/Q83 ditambah BlueOak-1.0.0). Alat build dan uji
  (`dev` di package-lock) boleh juga MPL-2.0 (misalnya `lightningcss`, pengolah CSS Vite) dan CC-BY-4.0 (data), karena
  tidak ikut ke berkas build dan tidak diubah. Lisensi lain menghentikan gerbang.
- Paket dipasang dengan `npm ci --ignore-scripts`: skrip pasang paket tidak pernah dijalankan.
- Hanya `react` dan `react-dom` yang ikut ke berkas build; lainnya hanya alat build dan uji.

---

## 9. Task turunan

| Isi | Modul | Task |
|---|---|---|
| Opsi `--web-dir`, penyajian file aman, kepala keamanan untuk API dan web | MOD-14 (Rust) | TSK-M31 |
| Aplikasi web React: halaman Bagian 2, dua bahasa, format jumlah, uji, `make web-check`, pemakaian pertama npm | MOD-14 (`web/`) | TSK-M32 |
| Paket Docker: node + explorer + web | MOD-14 | TSK-M33 (sebelum TSK-F06) |

---

## 10. Keputusan

| ID | Keputusan | Status |
|---|---|---|
| DEC-26 | Antarmuka web explorer memakai React (Vite + TypeScript) sesuai Dokumen Aplikasi, dua bahasa (Indonesia bawaan, English) | 🟢 diputuskan pencipta 2 November 2026 |
| DEC-10 bagian 7 | Dependensi npm Bagian 8, hanya di `apps/explorer/web` | 🟢 2 November 2026 (konsekuensi DEC-26) |
