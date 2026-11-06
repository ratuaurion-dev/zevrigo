# Antarmuka web explorer Zevrigo Coin (ZVRG)

Bagian MOD-14 (DEC-26). Next.js (App Router, static export) + React + TypeScript + Tailwind CSS + shadcn/ui, dua
bahasa (Indonesia bawaan, English), mode terang dan gelap. Tampilan dirancang lewat Google AI Studio, lalu disesuaikan
Quick: tanpa peta node karangan, paket bawaan AI Studio dibuang, versi paket dikunci, komponen shadcn/ui asli.
Aturan lengkap: `docs/reference/spesifikasi-explorer-web-v0.2.md` (diubah DEC-28).

Antarmuka ini hanya **menampilkan** isi API explorer di origin yang sama (`/api/v1`). Tanpa cookie, analitik, font,
CDN, atau permintaan ke luar. Jumlah koin selalu `bigint`; waktu blok UTC dari aritmetika bilangan bulat.

**Hanya data sistem (DEC-28):** tidak ada halaman alamat, kode QR, pencarian alamat, halaman alokasi, alamat atau jumlah
per penerima di transaksi, maupun angka alokasi genesis. Pengguna memeriksa saldonya sendiri lewat wallet.

## Perintah

```bash
npm ci --ignore-scripts          # paket persis dari package-lock.json, tanpa skrip pasang
npm run check                    # typecheck (tsc), uji (vitest), build (next build -> out/)
npm run dev:mock                 # pratinjau dengan data Regtest tiruan, http://127.0.0.1:3000/
```

Dari akar repo: `make web-check` (sama dengan dua perintah pertama). Hasil build disajikan program explorer:

```bash
cargo run -p zevrigo-explorer -- --regtest --datadir ~/.zevrigo --web-dir apps/explorer/web/out
```

`npm run dev` tanpa data tiruan tidak punya API (static export tidak bisa meneruskan `/api`); pakai `dev:mock` untuk
melihat tampilan, atau build lalu buka lewat explorer.

## Struktur

| Folder | Isi |
|---|---|
| `app/` | Halaman: `/`, `/blocks/`, `/block/?id=`, `/tx/?id=`, `/supply/`, `/mempool/`, `/search/?q=` |
| `components/ui/` | Komponen shadcn/ui (radix-ui + Tailwind); `primitives.tsx` titik impor untuk halaman |
| `components/explorer/` | Kepala halaman, pencarian Ctrl+K, grafik 20 blok, tampilan jumlah, hash, dan status |
| `lib/` | Klien API dan tipe, jumlah `bigint`, waktu UTC, jalur, kamus `i18n/`, data tiruan `mock.ts`, uji `explorer.spec.ts` |
| `hooks/` | Pembaruan berkala 10 detik |
