# TSK-M33 · Paket Docker regtest: node + explorer + web · 🟢 selesai · Prioritas: Sedang
- **Jenis:** task modul · **Modul:** MOD-14 (`--http-bind`), paket repo (`Dockerfile`, `compose.yaml`) · **Untuk:** persiapan testnet publik (FTR-06)
- **Bergantung pada:** TSK-M39, DEC-10 bagian 9 · **Memblokir:** TSK-F06 (bersama DEC-01)
- **Estimasi:** M · **Ditulis:** Quick, 3 November 2026

## Tujuan
Menjalankan node, explorer, dan antarmuka web dengan satu perintah, tanpa memasang Rust atau Node.js, sebagai dasar paket
testnet. Aturan lengkap: [Spesifikasi Docker v0.1](../reference/spesifikasi-docker-v0.1.md).

## Isi
| Bagian | Isi |
|---|---|
| `Dockerfile` | Tiga tahap: Rust 1.99.0 (release, `--locked`), Node.js 24.22.0 (`npm ci --ignore-scripts`, `next build`), Debian 13 slim; image dasar dikunci digest; pengguna `hsis` uid 10001; volume `/data` |
| `.dockerignore` | Konteks hanya kode yang dipakai (tanpa `.git`, `target`, `node_modules`, `docs`, `ai_advisory_hub`) |
| `compose.yaml` | `node` + `explorer` (berbagi namespace jaringan), profil `miner`; port host hanya `127.0.0.1:28802` |
| MOD-14 | Opsi `--http-bind <ip>` (bawaan `127.0.0.1`), `ArgError::InvalidBind`; server mendengar di alamat itu |
| `Makefile` | `make docker-build` |
| Persiapan | Q92 (Docker belum ada), D1 (pencipta memasang Docker Engine 29.9.2 resmi), Q93 (Docker tanpa sudo, digest image dasar) |

## Test (Rust tetap 380)
- `cli::tests::parses_help_version_and_run`: `--http-bind` 0.0.0.0, `::`, `::1`; bawaan 127.0.0.1.
- `cli::tests::rejects_bad_arguments`: alamat tidak sah, nilai kosong, opsi ganda.
- Gerbang Q94: build image, `--version` setiap program, uid 10001, compose node + explorer, halaman dan kepala keamanan,
  `404` alamat/alokasi, 3 blok ditambang di dalam container dan terindeks, RPC tidak terjangkau dari host, pembersihan.

## Definition of done
- [x] `make check-all` lolos; image dibangun; compose diuji ujung ke ujung
- [x] Spesifikasi Docker v0.1; DEC-10 bagian 9; Spesifikasi Explorer v0.1 Bagian 6; MOD-14; README; AGENTS.md
