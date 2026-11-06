# TSK-M31 · Penyajian antarmuka web di program explorer (`--web-dir`) · 🟢 selesai · Prioritas: Sedang
- **Jenis:** task modul · **Modul:** MOD-14 (apps/explorer, Rust) · **Untuk:** antarmuka web explorer (DEC-26)
- **Bergantung pada:** TSK-M30, TSK-S07 · **Memblokir:** TSK-M32
- **Estimasi:** S · **Ditulis:** Quick (kode dan test), 2 November 2026

## Tujuan
Program `zevrigo-explorer` menyajikan hasil build antarmuka web (`apps/explorer/web/dist`) di origin yang sama dengan
API, sesuai Spesifikasi Explorer Web v0.1 Bagian 7, tanpa dependensi baru.

## Isi
| Bagian | Isi |
|---|---|
| `web` (baru) | `serve(dir, path) -> WebResponse { status, content_type, cache_control, body }` fungsi murni; `content_type`; `INDEX_FILE`; `CONTENT_SECURITY_POLICY`. Jalur aman saja (≤ 4 segmen, `[A-Za-z0-9._-]`, tidak diawali titik, 8 ekstensi), tanpa fallback |
| `cli` | Opsi `--web-dir <folder>` (`Config::web_dir`), galat nilai kosong dan duplikat |
| `server` | `router(shared, web_dir)`: `/api` dan `/api/…` ke API, jalur lain ke `web::serve` di `spawn_blocking` (atau `404` JSON tanpa `--web-dir`); `405` untuk metode selain `GET` di jalur web; kepala `nosniff`, `no-referrer`, CSP, dan `Cache-Control` di setiap jawaban; `run` menolak folder tanpa `index.html` (`ExplorerError::WebDir`) |

## Keputusan teknis (Quick, tanpa DEC)
- **File dibaca sendiri, bukan `tower-http`.** `ServeDir` membawa paket baru; penyajian ini cukup puluhan baris dan
  daftar jalur yang boleh dibaca dibuat sempit.
- **Tanpa fallback ke `index.html`.** Antarmuka web memakai jalur hash, jadi jalur tak dikenal selalu `404`.
- **CSP juga di jawaban API.** Tidak berpengaruh pada JSON, dan satu fungsi `reply` untuk semua jawaban lebih sederhana.

## Test (explorer 16 → 19, workspace 357 → 360)
- `web`: `serves_files_from_web_dir`, `rejects_unsafe_or_unknown_paths` (16 jalur: `..`, `.`, file tersembunyi, segmen
  kosong, ekstensi lain, folder, `%`, garis miring terbalik, terlalu dalam, huruf besar).
- `server`: `http_server_serves_web_files_and_api` (axum di port bebas dengan folder web sementara: `index.html`, aset,
  `404`, `405`, kepala keamanan, API tetap dilayani).
- `cli`: kasus `--web-dir` di `parses_help_version_and_run` dan `rejects_bad_arguments`.
