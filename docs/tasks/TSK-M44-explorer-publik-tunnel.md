# TSK-M44 · Explorer publik lewat Cloudflare Tunnel (compose) · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** repo (paket Docker, MOD-14 lewat opsi yang sudah ada) · **Untuk:** demo publik sementara (FTR-06)
- **Bergantung pada:** TSK-M33, TSK-M43 · **Memblokir:** paket testnet publik
- **Estimasi:** S · **Ditulis:** Quick, 11 November 2026

## Tujuan
Pencipta memutuskan (9 November 2026) membuka explorer dan seed node ke publik untuk sementara dari komputer sendiri lewat
Cloudflare Tunnel dengan domain `ratuaurion.store`, sebagai bukti hidup untuk calon pendukung VPS testnet. Task ini
membuka explorer (regtest sebagai uji). Seed node dan testnet menyusul.

## Isi
| Bagian | Isi |
|---|---|
| `compose.publik.yaml` (baru) | Dipakai bersama `compose.yaml`; project `zevrigo-publik`; port host node dicabut (`!reset`); explorer `--trust-proxy` di `127.0.0.1` dalam namespace node; layanan `cloudflared` (image resmi dikunci digest, `tunnel --no-autoupdate run`, token dari `${HOME}/.zevrigo-tunnel.env`, berbagi namespace node) |
| Dokumen | Spesifikasi Docker v0.1 Bagian 7 dan DEC-10 bagian 9 (tambahan), README, MOD-14, docs/tasks |
| Tanpa | Perubahan kode, dependensi Rust/npm baru, atau port router yang dibuka |

## Gerbang (Q121)
Token ada (izin `600`, satu baris, isinya tidak dicetak); `docker compose config` sah, node tanpa port host, explorer
dengan `--trust-proxy`; image dibangun ulang dan `zevrigo-explorer --help` memuat `--trust-proxy`; `cloudflared` mencatat
koneksi tunnel; miner di namespace yang sama menambang 5 blok; `https://zevrigo.ratuaurion.store/api/v1/status` menjawab
regtest dengan tinggi indeks paling sedikit 5 dan halaman `/` menjawab `200`; `make check-all` lolos.

## Batas
- Komputer pencipta harus menyala; jika mati, explorer publik ikut mati.
- Regtest dulu: yang tampil adalah chain uji. Testnet menyusul bersama cap waktu genesis testnet (TSK-F06).
- Token tunnel memberi akses menjalankan tunnel; disimpan di luar repo dan diganti di dasbor Cloudflare jika bocor.

## Definition of done
- [x] `make check-all` lolos
- [x] Gerbang Q121 lolos
- [x] Spesifikasi Docker v0.1 Bagian 7, README, MOD-14, docs/tasks
