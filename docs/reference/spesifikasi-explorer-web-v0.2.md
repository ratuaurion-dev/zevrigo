# Spesifikasi Antarmuka Web Explorer Homeostasis Coin (HSIS) v0.2: Next.js, Tailwind CSS, shadcn/ui, dua bahasa

> Diperbarui: 3 November 2026 (DEC-28) · Pemilik: pencipta · Ditulis: Quick · Task: TSK-S07, TSK-M38, TSK-M39 · Status: 🟢 v0.2
> Lokasi di repo: `docs/reference/spesifikasi-explorer-web-v0.2.md` · Menggantikan
> [spesifikasi-explorer-web-v0.1.md](spesifikasi-explorer-web-v0.1.md) (Vite + React tanpa Tailwind)

| Status | Ringkasan |
|---|---|
| 🟢 Dikunci | Next.js (App Router, static export) + React + TypeScript + Tailwind CSS + shadcn/ui (DEC-26, diubah 2 November 2026 atas keinginan pencipta). Tampilan dirancang lewat Google AI Studio, lalu disesuaikan Quick. Dua bahasa (Indonesia bawaan, English), mode terang dan gelap. Jumlah koin selalu `bigint`. Tanpa cookie, analitik, font, atau skrip dari luar |
| 🟢 Hanya data sistem (DEC-28) | Explorer tidak menampilkan "apa dan siapa": tanpa halaman alamat, kode QR, pencarian alamat, halaman alokasi, alamat dan jumlah per penerima di transaksi, jejak koin, daftar txid mempool, angka alokasi genesis, maupun teks galat mentah dari node (keputusan pencipta 3 November 2026) |
| ⚪ Dihapus | Peta node global dan tabel node aktif dari rancangan AI Studio: datanya karangan, karena node dan explorer tidak punya dan tidak mengumpulkan lokasi atau IP rekan (keputusan pencipta 2 November 2026) |
| ⏳ Ditunda | Paket Docker (TSK-M33), perkiraan daya tambang, peringatan konsentrasi tambang |
| ➡️ Langkah berikutnya | TSK-M33 paket Docker sesudah TSK-M38 |

Antarmuka web hanya **menampilkan** isi API explorer (Spesifikasi Explorer v0.1 Bagian 5). Tidak ada logika chain,
pemeriksaan, atau kunci di browser. Siapa pun yang menjalankan node dan explorer sendiri mendapat tampilan yang sama.

---

## 1. Komponen (DEC-26)

| Hal | Nilai |
|---|---|
| Lokasi | `apps/explorer/web/` (bagian MOD-14, bukan crate Rust) |
| Perangkat | Next.js App Router dengan `output: 'export'` dan `trailingSlash: true`: build statis ke `apps/explorer/web/out/` (tidak di-commit). Tanpa server Node.js, route handler, middleware, optimasi gambar, atau `next/font` |
| Tampilan | Tailwind CSS 4, komponen shadcn/ui (`components/ui/`, di atas `radix-ui`), ikon `lucide-react`, notifikasi `sonner` |
| Penyajian | Program `homeostasis-explorer --web-dir apps/explorer/web/out` di origin yang sama dengan API (tanpa CORS), Bagian 7 |
| Pengembangan | `npm run dev:mock` (server Next.js di `http://127.0.0.1:3000/` dengan data Regtest tiruan). Static export tidak bisa meneruskan `/api`, jadi tampilan dengan data sungguhan dilihat lewat build + explorer |
| Dependensi | DEC-10 bagian 7 (Bagian 8) |

---

## 2. Halaman dan jalur

Static export membuat satu `index.html` per halaman; parameter lewat query, sehingga tidak ada rute dinamis.

| Jalur | Isi | API |
|---|---|---|
| `/` | Status (tinggi indeks dan node, chainwork, tanda galat sinkronisasi), 8 kartu ringkasan, pasokan hasil tambang, grafik 20 blok terbaru (selang waktu, ukuran, jumlah transaksi), blok terbaru | `/status`, `/supply`, `/blocks` |
| `/blocks/?before=<h>` | Daftar blok per 20, tautan "lebih lama" dan "lebih baru" | `/blocks?before=` |
| `/block/?id=<tinggi atau block_id>` | Header lengkap, subsidy, biaya, konfirmasi, daftar transaksi, blok sebelum dan sesudah | `/block/` |
| `/tx/?id=<txid>` | Blok, posisi, ukuran, biaya, laju biaya, konfirmasi, jumlah input dan jumlah output. Tanpa alamat, jumlah per penerima, dan jejak koin (DEC-28) | `/tx/` |
| `/supply/` | Pasokan hasil tambang dibanding jadwal rumus, biaya, subsidy berikutnya, rata-rata waktu blok, aturan emisi. Tanpa angka alokasi genesis dan total beredar (DEC-28) | `/supply` |
| `/mempool/` | Jumlah transaksi di mempool (tanpa daftar txid) | `/mempool` |
| `/search/?q=<kata>` | Pencarian tinggi, block_id, atau txid (juga dari kotak pencarian dan Ctrl+K / ⌘K di setiap halaman) → pindah ke halaman yang sesuai. Alamat tidak dicari | `/search/` |

- Transaksi yang belum ada di blok tetapi ada di mempool (`404` dengan `in_mempool: true`) ditampilkan sebagai
  "menunggu di mempool", bukan "tidak ditemukan".
- `503 not_ready` ditampilkan sebagai "indeks belum siap"; `index_error` dari `/status` ditampilkan sebagai peringatan di
  setiap halaman (tanda node yang ditunjuk tidak jujur atau rusak); `last_error` sebagai peringatan sinkronisasi. Keduanya
  dengan pesan umum, tanpa teks galat mentah dari node (DEC-28).
- **Konfirmasi selalu ditampilkan** di setiap blok dan transaksi, karena chain tanpa batas reorg. Di bawah 6 konfirmasi
  diberi tanda "belum aman".
- Status dan halaman diperbarui setiap 10 detik selama halaman terbuka. Tanpa WebSocket.
- Tidak ada halaman atau peta node: API tidak punya data node, dan explorer tidak boleh menampilkan angka karangan.
- Tidak ada halaman alamat dan halaman alokasi (DEC-28). Pengguna memeriksa saldonya sendiri lewat wallet (`balance` di
  CLI membaca node milik pengguna, tidak lewat explorer).

---

## 3. Bahasa dan tema

- Bahasa Indonesia (bawaan) dan English. Kamus di `lib/i18n/id.ts` dan `lib/i18n/en.ts` dengan kunci yang sama; uji
  memastikan kunci dan placeholder `{nama}` di kedua kamus persis sama.
- Pilihan pertama: tersimpan di `localStorage` (`hsis-explorer-lang`), lalu `navigator.language` berawalan `en` →
  English, selain itu Indonesia. Tema terang atau gelap di `hsis-explorer-theme`, bawaan mengikuti sistem. Keduanya hanya
  di browser, tidak pernah dikirim ke server.
- Istilah teknis tidak diterjemahkan: block_id, txid, hash, Merkle root, nonce, quanta, HSIS.

---

## 4. Angka, jumlah koin, dan waktu

- **Jumlah koin** (string desimal quanta dari API, sampai `u128::MAX`) diubah ke `bigint` dan ditampilkan sebagai HSIS
  dengan 18 desimal, nol di belakang dibuang, pemisah ribuan sesuai bahasa: Indonesia `4.380.000 HSIS`, `0,005 HSIS`;
  English `4,380,000 HSIS`, `0.005 HSIS`. Nilai quanta lengkap di tooltip. Persentase (alokasi, jadwal) dihitung dalam
  basis poin `bigint`.
- **Dilarang** di kode aplikasi: `parseFloat`, `toFixed`, `Intl.*`, `dangerouslySetInnerHTML`, `innerHTML`, `eval`,
  `new Function`, dan URL luar (kecuali namespace SVG). Diperiksa skrip gerbang. `number` hanya untuk tinggi,
  konfirmasi, ukuran, posisi, nomor halaman, dan tinggi batang grafik, tidak pernah untuk jumlah koin.
- **Waktu blok** (detik Unix) ditampilkan dalam UTC `YYYY-MM-DD HH:MM:SS UTC`, disusun dari aritmetika bilangan bulat
  (`civil_from_days`), bukan zona waktu browser.

---

## 5. Keamanan dan privasi

- Tanpa cookie, analitik, atau sumber luar (font, CDN, gambar, skrip). Semua aset dari build lokal; telemetri Next.js
  dimatikan (`NEXT_TELEMETRY_DISABLED=1`) di skrip `dev` dan `build`.
- Kebijakan keamanan konten dari server (Bagian 7): `default-src 'none'; script-src 'self'; style-src 'self'
  'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none';
  frame-ancestors 'none'`. Untuk setiap halaman HTML, `script-src` ditambah hash SHA-256 setiap skrip inline halaman itu
  (data halaman `self.__next_f` dari Next.js), sehingga skrip inline lain ditolak browser. Gaya inline diizinkan karena
  React dan radix-ui memakai atribut `style` dan `sonner` menyisipkan elemen `<style>`; gaya tidak bisa menjalankan
  kode.
- Semua teks dari API dirender sebagai teks oleh React. Isi dari pengguna atau API (txid, alamat, block_id) dimasukkan
  ke jalur API setelah diperiksa pola dan `encodeURIComponent`.
- Mode data tiruan (`lib/mock.ts`) hanya aktif dengan `NEXT_PUBLIC_USE_MOCK=1`; build biasa selalu membaca API. Data
  tiruan hanya mengeluarkan bentuk publik API (tanpa alamat dan alokasi).
- **Hanya data sistem (DEC-28):** transparan untuk sistem (aturan, blok, pasokan hasil tambang), privat untuk individu.
  Explorer tidak menampilkan alamat, saldo, jumlah per penerima, jejak koin, maupun alokasi genesis. Batas: data tetap
  ada di chain; explorer lain yang dijalankan orang lain tetap bisa menampilkannya. Explorer ini hanya memastikan bukan
  penyelenggaranya yang memamerkan.

---

## 6. Struktur dan uji

| File | Isi |
|---|---|
| `package.json`, `package-lock.json` | Versi persis (tanpa `^`), skrip `dev`, `dev:mock`, `typecheck`, `test`, `build`, `check` |
| `next.config.ts`, `tsconfig.json`, `global.d.ts`, `postcss.config.mjs`, `vitest.config.ts`, `components.json` | Static export, TypeScript `strict`, Tailwind, uji, konfigurasi shadcn/ui |
| `app/` | Tata letak, penyedia bahasa, tema, dan tooltip, serta halaman Bagian 2 |
| `components/ui/` | Komponen shadcn/ui: button, card, input, table, skeleton, alert, progress, tooltip, dialog, sheet, dropdown-menu; `primitives.tsx` titik impor halaman |
| `components/explorer/` | Kepala halaman (navigasi, Ctrl+K, bahasa, tema, menu HP), grafik, tampilan jumlah, hash, konfirmasi, dan status API |
| `lib/` | `api.ts` (tipe jawaban API, jumlah sebagai `string`, klien `fetch` yang tidak pernah melempar galat), `amount.ts`, `time.ts`, `route.ts`, `i18n/`, `mock.ts`, `config.ts` |
| `lib/explorer.spec.ts` | 15 uji vitest: jumlah, basis poin, waktu UTC, jalur dan validasi (alamat bukan kunci pencarian), kamus, klien API dengan `fetch` tiruan, jalur API, mode tiruan mati, data tiruan tanpa alamat dan alokasi |

- `make web-check` = `cd apps/explorer/web && npm ci --ignore-scripts && npm run check` (typecheck, uji, build).
  `make check-all` tetap untuk Rust saja; skrip gerbang dan CI menjalankan keduanya.
- Node.js ≥ 22.12. `node_modules/`, `.next/`, `out/`, `next-env.d.ts` (ditulis ulang Next.js), dan `*.tsbuildinfo`
  diabaikan git.

---

## 7. Penyajian oleh program explorer (MOD-14, TSK-M31, TSK-M38)

- Opsi `--web-dir <folder>`: folder hasil build (`apps/explorer/web/out`). Saat start, folder wajib berisi `index.html`
  (jika tidak, explorer keluar dengan kode 1). Tanpa opsi ini explorer hanya melayani API.
- `/api` dan `/api/…` selalu API. Metode selain `GET` dan `HEAD` di jalur web → `405`.
- Jalur halaman: `/`, jalur berakhiran `/`, atau jalur yang segmen terakhirnya tanpa titik → `index.html` di folder itu
  (`/blocks/` dan `/blocks` sama-sama `blocks/index.html`). Halaman yang tidak ada → `404` dengan `404.html` hasil build.
- Jalur file: paling banyak 8 segmen, setiap segmen hanya huruf, angka, `.`, `_`, `-`, tidak kosong, tidak diawali
  titik, dengan ekstensi `html`, `js`, `css`, `json`, `svg`, `png`, `ico`, `woff2`, `txt` (data RSC Next.js
  `__next.*.txt` disajikan sebagai `text/plain`). Selain itu `404 not found` (teks). Query diabaikan.
- Cache: file di `_next/static/` (nama berisi hash isi) `public, max-age=31536000, immutable`; halaman dan file lain
  `no-cache`; API, `404`, dan galat `no-store`.
- Setiap jawaban (API maupun web) membawa `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, dan
  kebijakan keamanan konten Bagian 5 (halaman HTML dengan hash skrip inline-nya, dihitung saat disajikan).

---

## 8. Dependensi npm (DEC-10 bagian 7, diubah 2 November 2026)

| Paket | Versi | Lisensi | Peran |
|---|---|---|---|
| `next` | 16.4.4 | MIT | Kerangka, build static export (membawa SWC native per platform; `sharp` opsional tidak dipakai karena `images.unoptimized`) |
| `react`, `react-dom` | 19.3.1 | MIT | UI |
| `radix-ui` | 1.8.1 | MIT | Dasar komponen shadcn/ui (dialog, menu, tooltip, progres) |
| `class-variance-authority` | 0.7.1 | Apache-2.0 | Varian komponen shadcn/ui |
| `clsx`, `tailwind-merge` | 2.1.1, 3.7.0 | MIT | Penggabung kelas `cn()` shadcn/ui |
| `lucide-react` | 1.66.0 | ISC | Ikon |
| `sonner` | 2.0.8 | MIT | Notifikasi "tersalin" |
| `tailwindcss`, `@tailwindcss/postcss` | 4.3.3 | MIT | CSS (alat build; membawa lightningcss dan oxide native) |
| `postcss` | 8.5.32 | MIT | Alat build CSS |
| `typescript` | 7.0.2 | Apache-2.0 | Pemeriksaan tipe |
| `vitest` | 5.0.5 | MIT | Uji |
| `@types/react`, `@types/react-dom`, `@types/node` | 19.3.0, 19.3.0, 26.6.6 | MIT | Tipe |

`qrcode` 1.5.4 dan `@types/qrcode` 1.5.6 dipakai di TSK-M38, lalu dihapus bersama halaman alamat (TSK-M39, DEC-28).

- Versi persis di `package.json`, pohon lengkap di `package-lock.json` (di-commit). Paket dipasang dengan
  `npm ci --ignore-scripts`: skrip pasang paket tidak pernah dijalankan.
- Gerbang: semua paket dari `registry.npmjs.org` dengan `integrity` sha512; daftar paket, lisensi, dan skrip pasang di
  `docs/reference/paket-mod14-web-v0.2.txt`.
- **Lisensi.** Paket wajib permisif (daftar Q78/Q82/Q83 ditambah BlueOak-1.0.0). Pengecualian tercatat, semuanya tidak
  ikut ke berkas yang dikirim ke browser: alat build dan uji (`dev`) boleh MPL-2.0 dan CC-BY-4.0; `caniuse-lite`
  (CC-BY-4.0, data browser untuk build Next.js); `lightningcss*` (MPL-2.0, pengolah CSS); `@img/sharp-*` (biner opsional
  libvips, LGPL-3.0-or-later, tidak pernah dimuat karena optimasi gambar mati). Lisensi lain menghentikan gerbang.

---

## 9. Task turunan

| Isi | Modul | Task |
|---|---|---|
| Opsi `--web-dir`, penyajian file aman, kepala keamanan untuk API dan web | MOD-14 (Rust) | TSK-M31 🟢 |
| Aplikasi web React (Vite), diganti TSK-M38 | MOD-14 (`web/`) | TSK-M32 🟢 |
| Aplikasi Next.js + Tailwind + shadcn/ui hasil AI Studio, penyajian static export dan CSP dengan hash di Rust | MOD-14 | TSK-M38 |
| Hanya data sistem: halaman alamat, alokasi, dan QR dihapus; ringkasan transaksi; banner tanpa galat mentah | MOD-14 | TSK-M39 (DEC-28) |
| Paket Docker: node + explorer + web | MOD-14 | TSK-M33 (sebelum TSK-F06) |

---

## 10. Keputusan

| ID | Keputusan | Status |
|---|---|---|
| DEC-26 | Antarmuka web explorer memakai Next.js (static export) + React + TypeScript + Tailwind CSS + shadcn/ui, dirancang lewat Google AI Studio, dua bahasa (Indonesia bawaan, English) | 🟢 diubah pencipta 2 November 2026 (sebelumnya Vite + React tanpa Tailwind) |
| DEC-10 bagian 7 | Dependensi npm Bagian 8, hanya di `apps/explorer/web` | 🟢 diubah 2 November 2026 (konsekuensi DEC-26); `qrcode` dihapus 3 November 2026 (DEC-28) |
| DEC-28 | Explorer hanya menampilkan data sistem, tanpa "apa dan siapa" (alamat, saldo, jumlah per penerima, jejak koin, alokasi genesis) | 🟢 diputuskan pencipta 3 November 2026 |
