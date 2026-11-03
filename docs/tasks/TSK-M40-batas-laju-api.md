# TSK-M40 · Batas laju API explorer · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-14 (`limit`, `server`, `cli`, `web/`) · **Untuk:** explorer publik (FTR-06)
- **Bergantung pada:** TSK-M33 · **Memblokir:** explorer testnet publik (bersama reverse proxy HTTPS dan DEC-01)
- **Estimasi:** S · **Ditulis:** Quick, 3 November 2026

## Tujuan
Spesifikasi Explorer v0.1 Bagian 7 mewajibkan batas laju sebelum explorer dibuka ke publik. Explorer sekarang membatasi
permintaan API sendiri, tanpa paket baru dan tanpa menyimpan alamat IP pengunjung.

## Isi
| Bagian | Isi |
|---|---|
| `limit` (baru) | `Rate`, `Limits::from_rate`, `RateLimiter::allow(client, now_ms)`: ember token per klien dan bersama, bilangan bulat dalam mili-permintaan; kunci klien = hash 64-bit alamat IP dengan `RandomState` per proses; paling banyak `MAX_CLIENTS` = 4.096 ember, klien diam dibuang, sesudah itu ember limpahan |
| `cli` | `--api-rate <n>` (bawaan `DEFAULT_API_RATE` = 10; `0` = tanpa batas), `ArgError::InvalidRate` |
| `server` | `router(shared, web_dir, limits)`; hanya jalur API dibatasi; `429 {"error":"rate_limited"}`, `Retry-After: 1`, `no-store`, kepala keamanan; program memakai `into_make_service_with_connect_info::<SocketAddr>()` |
| Web | `ApiStateView` menampilkan "terlalu banyak permintaan" untuk 429, teks dua bahasa |
| Bawaan | Klien 10/detik (kapasitas 40), semua klien 200/detik (kapasitas 400); satu halaman explorer memakai sekitar 4 permintaan per 10 detik |

## Test (Rust 380 → 384; vitest 15 → 16)
- `limit`: `bucket_refills_with_integer_math`, `clients_are_separate_and_global_is_shared`, `bounded_clients_without_storing_addresses`.
- `server`: `http_server_limits_api_requests_per_client` (429, `Retry-After`, kepala keamanan).
- `cli`: `--api-rate` sah dan tidak sah (uji yang sudah ada diperluas).
- vitest: `maps 429 rate_limited`.
- Gerbang Q95: program explorer sungguhan dengan `--api-rate 1` menjawab 429 sesudah kapasitas habis, file web tetap 200; bawaan tidak membatasi 30 permintaan beruntun; image Docker dibangun ulang.

## Batas
Di belakang reverse proxy atau penerus port Docker semua klien tampak dari satu alamat, sehingga batas per klien menjadi
batas bersama. Batas per klien yang sebenarnya untuk explorer publik dipasang di reverse proxy HTTPS (menyusul).

## Definition of done
- [x] `make check-all` dan `make web-check` lolos; image Docker dibangun ulang
- [x] Spesifikasi Explorer v0.1 Bagian 6–7, Spesifikasi Explorer Web v0.2, Spesifikasi Docker v0.1, MOD-14, README
