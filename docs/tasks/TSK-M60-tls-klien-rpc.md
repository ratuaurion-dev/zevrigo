# TSK-M60 · HTTPS di klien RPC MOD-08 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-08 (`client`) · **Untuk:** FTR-11 tahap 1 (DEC-34)
- **Bergantung pada:** TSK-M53 · **Memblokir:** TSK-M61, TSK-M58
- **Estimasi:** S · **Ditulis:** Quick, 13 November 2026

## Tujuan
Wallet desktop (MOD-15) dan CLI (MOD-12) peserta harus bisa memakai gerbang RPC publik `https://zevrigo-rpc.ratuaurion.store`
(TSK-M53). Klien RPC sebelumnya hanya menerima `http://`.

## Isi
| Bagian | Isi |
|---|---|
| MOD-08 `Cargo.toml` | `jsonrpsee-http-client` dengan fitur `tls` (pilihan pencipta 12 November 2026: Cara A) |
| MOD-08 `client` | Dokumentasi `RpcClient::new`: `http://` dan `https://`; skema lain ditolak. Tanpa perubahan API |
| Dependensi | 11 paket baru (DEC-10 bagian 14, reference/paket-mod08-tls-v0.1.txt). Paket lama yang berubah: jsonrpsee-http-client (rujukan TLS) dan rustls (rujukan log). Tanpa aws-lc-rs |
| Test | `client::tests`: `new_accepts_http_and_https_urls`, `https_to_closed_port_is_transport_error`. Workspace 449 → 451 |

## Gerbang (Q128)
Cargo.lock bertambah tepat 11 paket dengan checksum sama dengan file .crate yang diperiksa Quick, 0 hilang, paket lama yang
berubah hanya jsonrpsee-http-client dan rustls; `make check-all` 451 test; dari WSL `zevrigo-cli --rpc-url
https://zevrigo-rpc.ratuaurion.store info` menjawab jaringan regtest dengan tinggi yang sama dengan node seed (selisih paling
banyak 1). Informasi: `https://expired.badssl.com/` ditolak karena sertifikat.

## Batas
- Android: `rustls-platform-verifier` wajib diinisialisasi lewat JNI dan file `.aar`-nya dibundel ke APK sebelum koneksi
  `https://` pertama (TSK-M58). Tanpa itu koneksi gagal.
- Linux tanpa paket `ca-certificates` (misalnya image Docker runtime) tidak punya akar sertifikat; koneksi `https://` dari
  container gagal. Container memakai `http://127.0.0.1` saja.
- Server RPC node tetap HTTP tanpa TLS di `127.0.0.1`; TLS gerbang diputus Cloudflare.

## Definition of done
- [x] Gerbang Q128 lolos
- [x] DEC-10 bagian 14, paket-mod08-tls-v0.1.txt, Spesifikasi RPC v0.1 Bagian 1, MOD-08, Rancangan Ekosistem Demo v0.1,
  docs/tasks
