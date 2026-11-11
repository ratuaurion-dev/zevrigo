# TSK-M43 · Batas laju explorer di belakang Cloudflare Tunnel · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-14 (`server`, `cli`) · **Untuk:** explorer publik sementara (FTR-06)
- **Bergantung pada:** TSK-M40 · **Memblokir:** paket Docker testnet dengan `cloudflared`
- **Estimasi:** S · **Ditulis:** Quick, 9 November 2026

## Tujuan
Pencipta memutuskan (9 November 2026) membuka explorer dan seed node ke publik untuk sementara dari PC lewat Cloudflare
Tunnel dengan domain `ratuaurion.store`, sebelum pindah ke VPS. Lewat tunnel, semua pengunjung tersambung ke explorer dari
`cloudflared` di loopback, sehingga batas laju per klien TSK-M40 menjadi satu ember untuk seluruh dunia. Opsi
`--trust-proxy` mengembalikan batas per pengunjung tanpa paket baru dan tanpa menyimpan alamat IP.

## Isi
| Bagian | Isi |
|---|---|
| `server` | `CLIENT_IP_HEADER` = `cf-connecting-ip`; `client_ip(connect, headers, trust_proxy)`: kepala dipakai hanya jika opsi aktif dan koneksi dari loopback (`to_canonical().is_loopback()`), nilainya tepat satu alamat IP (spasi di tepi dibuang); selain itu alamat koneksi. `router_with_proxy(shared, web_dir, limits, trust_proxy)`; `router` tetap (tanpa proxy). `handle` membaca `HeaderMap`. Log start memuat `trust_proxy` |
| `cli` | `--trust-proxy` (tanpa nilai, bawaan mati), `Config::trust_proxy`, `ArgError::Duplicate("--trust-proxy")`, teks bantuan |
| Privasi | Sama dengan TSK-M40: alamat hanya menjadi kunci hash ember di memori, tidak disimpan dan tidak dicatat |

## Test (Rust 429 → 432)
- `server`: `client_ip_uses_proxy_header_only_from_loopback_with_option` (loopback IPv4, `::1`, IPv4-mapped; kepala IPv6;
  tanpa opsi, koneksi bukan loopback, tanpa kepala, dan kepala tidak sah memakai alamat koneksi),
  `http_server_limits_each_visitor_behind_trusted_proxy` (server sungguhan di 127.0.0.1: pengunjung A habis kapasitas 2
  lalu `429`, pengunjung B dan pengunjung IPv6 tetap `200`).
- `cli`: `parses_trust_proxy` (aktif, bawaan mati, disebut dua kali ditolak, teks bantuan).

## Batas
- Aman hanya jika di mesin atau container explorer, satu-satunya yang tersambung ke explorer lewat loopback adalah
  `cloudflared` (paket Docker: `cloudflared` berbagi namespace jaringan dengan node dan explorer; miner tidak memakai HTTP
  explorer). Proses lokal lain bisa mengisi kepala itu sendiri.
- Tanpa tunnel (VPS dengan reverse proxy lain), opsi ini hanya benar jika proxy itu mengisi `CF-Connecting-IP` dan
  tersambung dari loopback.

## Definition of done
- [x] `make check-all` lolos
- [x] Spesifikasi Explorer v0.1 Bagian 6–8, MOD-14, README, docs/tasks
