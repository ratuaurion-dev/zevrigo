# TSK-M38 · Antarmuka web explorer Next.js + Tailwind + shadcn/ui · 🟢 selesai · Prioritas: Sedang
- **Jenis:** task modul · **Modul:** MOD-14 (`apps/explorer/web` TypeScript, `apps/explorer/src/web` Rust) · **Untuk:** antarmuka web explorer (DEC-26)
- **Bergantung pada:** TSK-M31, TSK-M32, DEC-26 (diubah 2 November 2026), DEC-10 bagian 7 · **Memblokir:** TSK-M33
- **Estimasi:** M · **Ditulis:** Google AI Studio (rancangan tampilan), Quick (penyesuaian, Rust, dokumen), 2 November 2026

## Tujuan
Pencipta meminta tampilan explorer semaksimal mungkin dengan Next.js, Tailwind CSS, dan shadcn/ui lewat Google AI Studio,
menggantikan aplikasi Vite TSK-M32. Rancangan AI Studio dipasang ke repo dengan aturan Spesifikasi Explorer Web v0.2.

## Penyesuaian Quick atas zip AI Studio
| Hal | Perubahan |
|---|---|
| Peta node global, tabel node aktif, `d3-geo` | Dihapus (keputusan pencipta): data kota, latensi, dan jumlah node karangan; explorer tidak punya data node |
| Sisa bawaan AI Studio | Dibuang: `@google/genai`, express, dotenv, vite + plugin, tsx, esbuild, `src/`, `index.html`, `vite.config.ts`, `metadata.json`, `.env.example`, `bun.lock`, `.aistudio/`, `cmdk`, `motion`, uji `node:test` ganda |
| Komponen UI | Komponen buatan sendiri yang mirip shadcn diganti komponen shadcn/ui asli (`components/ui/*.tsx` di atas `radix-ui`, `components.json`); `primitives.tsx` tinggal titik impor |
| Versi paket | `^` diganti versi persis terbaru (Next.js 16.4.4, React 19.3.1, TypeScript 7.0.2, Tailwind 4.3.3, vitest 5.0.5) |
| Data tiruan | `USE_MOCK` hanya dengan `NEXT_PUBLIC_USE_MOCK=1` (`npm run dev:mock`); contoh di palet Ctrl+K hanya di mode itu |
| Kepala halaman HP | Nama jaringan pindah ke menu di layar sempit agar tombol tidak terpotong di lebar 390 px |
| Lain-lain | Tautan GitHub palsu di kaki halaman dihapus; telemetri Next.js mati; uji jalur API dan mode tiruan ditambah |

## Rust (MOD-14)
| Isi | Rincian |
|---|---|
| `web::serve` | Halaman static export (`/x/` dan `/x` → `x/index.html`), `404.html` untuk halaman yang tidak ada, sampai 8 segmen, ekstensi `json` ditambah, cache `immutable` untuk `_next/static/` |
| `web::page_policy` | CSP halaman HTML: hash SHA-256 (base64) setiap skrip inline ditambah ke `script-src`; `style-src 'self' 'unsafe-inline'` |
| `WebResponse` | Kolom baru `content_security_policy` |
| `server` | `HEAD` dilayani seperti `GET` di jalur web (prefetch Next.js); kepala CSP per jawaban |
| Dependensi | `sha2` 0.10.9 (sudah di workspace dan Cargo.lock, DEC-10 bagian 4), tanpa paket baru |

## Test (Rust 378 → 380; vitest 14 di 1 file)
- `web`: `serves_files_from_web_dir`, `rejects_unsafe_or_unknown_paths` (diperbarui), `serves_not_found_page_for_missing_pages`,
  `page_policy_hashes_inline_scripts` (vektor base64 RFC 4648, contoh hash CSP Level 3) — baru.
- `server`: `http_server_serves_web_files_and_api` diperbarui (`_next/static`, `HEAD`, hash CSP halaman).
- Gerbang Q90: explorer sungguhan menyajikan `out/`: setiap halaman 200 dengan hash semua skrip inline di CSP, semua
  file `out/` lolos aturan jalur.

## Definition of done
- [x] `make check-all` dan `make web-check` lolos; laporan paket `docs/reference/paket-mod14-web-v0.2.txt`
- [x] DEC-26 dan DEC-10 bagian 7 diperbarui; Spesifikasi Explorer Web v0.2; MOD-14, AGENTS.md, README diperbarui
