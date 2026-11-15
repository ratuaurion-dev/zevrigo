# Blog resmi Zevrigo (`apps/blog`)

Situs statis (DEC-37, TSK-M64): Next.js static export dari artikel Markdown di `posts/`, disajikan program
`zevrigo-demo web` (MOD-18) di `https://zevrigo-blog.ratuaurion.store`. Tanpa kuki, analitik, font, atau skrip dari pihak
lain. Paket npm sama persis dengan `apps/explorer/web` (package-lock identik kecuali nama dan versi).

## Menulis artikel

Buat `posts/<slug>.md` (huruf kecil, angka, tanda hubung):

```markdown
---
judul: Judul artikel
ringkasan: Satu atau dua kalimat untuk kartu dan mesin pencari.
tanggal: 2026-11-15
kategori: Pengumuman
penulis: Badak Besi
sampul: /sampul/kisi-batu.jpg
---
Isi artikel. Didukung: ## dan ### judul, paragraf, daftar - dan 1., blok kode berpagar, kutipan >, tabel, ---,
`kode`, **tebal**, dan [tautan](https://...).
```

`kategori`: Pengumuman, Panduan, Protokol, atau Tokenomik. `sampul` opsional (berkas di `public/sampul/`). `unggulan: ya`
menaruh artikel di atas halaman depan. Tulis hanya fakta yang bisa diperiksa; uji menolak frasa klaim investasi.

## Perintah

```bash
npm ci --ignore-scripts          # paket dari package-lock.json
npm run check                    # typecheck, uji (vitest), build -> out/
cargo run -p zevrigo-demo -- web --web-dir apps/blog/out   # pratinjau di http://127.0.0.1:28807/
```

Dari akar repo: `make blog-check`. Di Docker, blog ikut dibangun di image (`/usr/share/zevrigo/blog`).
