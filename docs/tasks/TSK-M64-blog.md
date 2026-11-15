# TSK-M64 · Blog resmi Zevrigo · 🟡 kode dan layanan · Prioritas: Sedang
- **Jenis:** task aplikasi · **Modul:** MOD-18 (mode `web`), `apps/blog` (DEC-37) · **Untuk:** FTR-11 (DEC-34)
- **Ditulis:** Quick, 15 November 2026 (pencipta: blog resmi; desain dari `zevrigo-research-portal.zip`, isi ditulis ulang;
  hostname `zevrigo-blog.ratuaurion.store`; nama pena Badak Besi)

## Bahan dan pemeriksaan
Portal buatan Google AI Studio (73 berkas): desainnya dipakai (kertas `#FAF9F6`, aksen teal `#0F766E`/`#2DD4BF`, judul serif,
kartu artikel, huruf awal besar). Yang dibuang: semua isi dan angka (jaminan aset 100%, cadangan $164.892.400, blok teraudit,
BFT 380 ms, Mainnet v2.4, kliring Eropa, penulis dan kantor rekaan), server Express dengan artikel di memori, halaman admin
dengan kata sandi di kode, newsletter yang mengumpulkan email, Google Fonts, Gemini, folder `.aistudio`, bilah metrik palsu.
Dari 4 gambar sampul hanya satu yang dipakai (kisi kubus batu, tanpa teks; diperkecil ke 1200 piksel); tiga lainnya memuat
teks menyesatkan ("Sovereign Ledger", "OPTICS/VLSI LAB", "Central Trust Bank").

## Isi
| Bagian | Isi |
|---|---|
| `apps/blog` | Next.js 16 static export, React 19, Tailwind 4; `package.json` dan `package-lock.json` sama dengan `apps/explorer/web` kecuali nama dan versi (paket yang tidak dipakai tetap tercantum). Huruf sistem, mode gelap dari `prefers-color-scheme`, tanpa skrip sendiri |
| Artikel | `posts/<slug>.md`, front matter `judul`, `ringkasan`, `tanggal`, `kategori` (Pengumuman, Panduan, Protokol, Tokenomik), `penulis`, opsional `sampul` dan `unggulan: ya`. Front matter salah menghentikan build |
| Markdown | Pengurai sendiri (`lib/markdown.ts`): `##`, `###`, paragraf, daftar, kode berpagar, kutipan, tabel GFM, `---`, `kode`, **tebal**, tautan hanya `https://`, `/...`, `#...`. Tanpa HTML mentah |
| Halaman | `/` (artikel unggulan + kartu), `/artikel/<slug>/`, `/unduh/`, `404.html`, `robots.txt` |
| Unduh (Q139) | Keputusan pencipta 15 November 2026: halaman di blog, berkas di GitHub Releases (blog tidak menyajikan berkas besar; aturan Cloudflare untuk berkas besar di paket gratis, upload rumah). Data di `lib/unduhan.ts`: Windows 0.2.0 (`releases/download/wallet-v0.2.0/zevrigo-wallet-0.2.0-x64-setup.exe`, `SHA256SUMS.txt`, `sha256` diisi sesudah rilis terbit), Android "segera" (APK rilis bertanda tangan menyusul) |
| Uji | `lib/blog.spec.ts`: pengurai, tautan aman, front matter, 5 artikel termuat dengan penulis Badak Besi, tautan antar-artikel sah, sampul ada, dan tidak ada frasa klaim investasi atau nama penulis rekaan |
| MOD-18 | Mode `web` (modul `site`, salinan penyajian MOD-14 + `jpg`/`webp`), 8 test baru (6 site, 1 cli, 1 run_web) |
| Docker | Tahap `blog-build` di Dockerfile, layanan `blog` di `compose.publik.yaml` (namespace `node`, `127.0.0.1:28807`) |

## Artikel pertama (Badak Besi)
1. Jaringan demo diluncurkan, dengan upacara genesis di blok 100 (unggulan, 14 November 2026)
2. Panduan: mencoba wallet Zevrigo dan faucet demo
3. Mengapa Zevrigo memakai Proof-of-Work RandomX
4. Ringkasan tokenomik Zevrigo v0.1
5. Kabar wallet Android, APK uji pertama

## Sesudahnya (pencipta)
Route Cloudflare `zevrigo-blog.ratuaurion.store` → HTTP `localhost:28807`. Artikel baru: tambah `posts/<slug>.md`, commit,
bangun ulang image, `up -d --no-deps blog`.
