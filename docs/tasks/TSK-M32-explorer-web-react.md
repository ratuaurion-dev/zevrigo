# TSK-M32 · Aplikasi web React explorer · 🟢 selesai · Prioritas: Sedang
> Diganti [TSK-M38](TSK-M38-explorer-web-nextjs.md): Next.js + Tailwind CSS + shadcn/ui (DEC-26 diubah 2 November 2026).
- **Jenis:** task modul · **Modul:** MOD-14 (`apps/explorer/web`, TypeScript) · **Untuk:** antarmuka web explorer (DEC-26)
- **Bergantung pada:** TSK-M31, TSK-S07, DEC-26, DEC-10 bagian 7 · **Memblokir:** TSK-M33
- **Estimasi:** M · **Ditulis:** Quick (kode dan test), 2 November 2026

## Tujuan
Aplikasi web statis sesuai Spesifikasi Explorer Web v0.1 Bagian 1 sampai 6: semua halaman Bagian 2 (termasuk halaman
alokasi pencipta), Bahasa Indonesia dan English, jumlah koin `bigint`, tanpa sumber luar, disajikan program explorer
lewat `--web-dir apps/explorer/web/dist`.

## Isi
| File | Isi |
|---|---|
| `package.json`, `package-lock.json` | Versi persis (DEC-10 bagian 7); skrip `dev`, `typecheck`, `test`, `build`, `check`; Node.js ≥ 22.12 |
| `index.html`, `vite.config.ts`, `tsconfig.json`, `.gitignore` | Titik masuk tanpa skrip inline, build ke `dist/` tanpa polyfill dan tanpa aset inline, TypeScript `strict`, proxy `/api` untuk `npm run dev` |
| `src/api.ts` | Tipe jawaban API (jumlah = `string`), `getJson` (tidak pernah melempar: `http`/`network`/`format`), `createApi`, `browserApi` (tanpa cookie, tanpa referrer) |
| `src/amount.ts`, `src/time.ts` | quanta ↔ `bigint`, format ZVRG per bahasa; waktu UTC dengan aritmetika `bigint` |
| `src/route.ts` | Jalur hash `#/…`, pemeriksaan tinggi, hash, alamat, dan masukan pencarian, `searchHref` |
| `src/i18n/` | Kamus `id` (bawaan) dan `en` (100 kunci), `detectLanguage`, `fill` |
| `src/load.ts`, `src/ui.ts` | `useLoad` (muat dan perbarui berkala), konteks tampilan |
| `src/components/common.tsx` | Jumlah, angka, tautan hash, konfirmasi (tanda belum aman di bawah 6), galat, isian, halaman, tabel blok |
| `src/pages/` | Beranda, blok, transaksi, alamat, pasokan, alokasi, mempool, daftar blok: tampilan murni + pemuat |
| `src/App.tsx`, `src/main.tsx`, `src/style.css` | Kepala halaman (navigasi, pencarian, tombol bahasa), peringatan `index_error`, halaman per jalur |

## Keputusan teknis (Quick, tanpa DEC)
- **Tanpa pustaka tambahan** (router, i18n, state, HTTP, angka, CSS): semuanya kecil dan ditulis sendiri, sehingga paket
  ke browser hanya React.
- **Tampilan murni terpisah dari pemuat data**, sehingga semua halaman diuji dengan `react-dom/server` di Node tanpa
  browser tiruan.
- **Status dimuat sekali di `App`** (setiap 10 detik) untuk peringatan `index_error` di semua halaman dan panel beranda.
- `vite.config.ts` tidak ikut typecheck (butuh tipe Node.js); diperiksa lewat `vite build`.

## Test (vitest, 24 test di 6 file; Rust tetap 360)
`amount` 4, `time` 2, `route` 5, `i18n` 4, `api` 3, `pages` 6. Gerbang juga memeriksa: package-lock (registry,
integrity, paket ke browser), pola terlarang di `src/` (number untuk jumlah, `innerHTML`, gaya inline, URL luar), nama
file `dist/` cocok dengan aturan `--web-dir`, dan program explorer sungguhan menyajikan `dist/` dengan kepala keamanan.
