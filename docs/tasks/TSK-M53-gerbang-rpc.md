# TSK-M53 · Gerbang RPC untuk wallet peserta (MOD-18 mode `rpc`) · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-18 (`apps/demo`, baru), repo (Docker) · **Untuk:** FTR-11 tahap 1 (DEC-34, DEC-36)
- **Bergantung pada:** TSK-S08 · **Memblokir:** TSK-M54, TSK-M55, TSK-M60
- **Estimasi:** M · **Ditulis:** Quick, 12 November 2026

## Tujuan
Peserta yang tidak menjalankan node sendiri (wallet desktop di laptop, nanti APK Wallet) butuh pintu RPC publik. RPC node
sengaja tidak terbuka, jadi gerbang ini meneruskan hanya metode yang aman, dengan batas laju per pengunjung.

## Isi
| Bagian | Isi |
|---|---|
| MOD-18 baru | `apps/demo`, paket `zevrigo-demo` (pustaka + bin), anggota workspace. Modul `cli` (mode `rpc`; `faucet` dan `daftar` dijawab "belum tersedia"), `gateway`, `limit` |
| `gateway` | axum 0.8, satu permintaan JSON-RPC 2.0 per `POST` di semua jalur. `parse_call`: `-32700` JSON rusak; `-32600` batch, bukan objek, `id` bukan angka atau teks, `jsonrpc` bukan `"2.0"`, `method` bukan teks, `params` bukan objek atau array; `-32601` di luar `ALLOWED_METHODS` (7 metode, tanpa `get_block_template` dan `submit_block`). Diteruskan lewat `RpcClient::raw_request` (MOD-08); galat node diteruskan dengan kode, pesan, dan `data` (misalnya `reason` `T6`); node tidak menjawab `-32603` "node tidak bisa dihubungi". Badan dibaca paling banyak 256 KiB (`413`); selain `POST` `405` dengan `Allow: POST`. Jawaban `no-store` dan `nosniff` |
| `limit` | Salinan pembatas MOD-14 (TSK-M40): `--rate` bawaan 20 (per pengunjung 20/detik kapasitas 80, bersama 400/detik), diperiksa sebelum badan dibaca, `429` dengan `Retry-After: 1`. `--trust-proxy`: `CF-Connecting-IP` hanya dari loopback (pola TSK-M43). Tanpa menyimpan atau mencatat alamat |
| Docker | Image `zevrigo:regtest` memuat `zevrigo-demo`. Layanan `gerbang-rpc` di namespace node seed (`rpc --trust-proxy`, 256 MiB, 0,5 CPU). Public hostname `zevrigo-rpc.ratuaurion.store` → HTTP `localhost:28804` |
| Dependensi | Tanpa paket baru (DEC-10 bagian 13); Cargo.lock hanya bertambah entri `zevrigo-demo` |
| Test | `cli`: 2; `limit`: 3 (sama dengan MOD-14); `gateway`: 7 (subset metode, `parse_call`, node tiruan MOD-08 lewat gerbang termasuk `data.reason` dan metode tambang tidak pernah sampai, HTTP rusak 405/413/-32700/-32600, batas laju 429, node mati -32603, `client_ip`). Workspace 437 → 449 |

## Gerbang (Q127)
Lihat Spesifikasi Docker v0.1 Bagian 7, "Gerbang RPC". Ringkas: Cargo.lock hanya `zevrigo-demo`; `make check-all` 449 test;
image dibangun ulang; `gerbang-rpc` berjalan tanpa menyalakan ulang container lain; `zevrigo-cli --rpc-url
http://127.0.0.1:28804 info` di namespace seed menjawab; dari internet `https://zevrigo-rpc.ratuaurion.store` menjawab
`get_chain_info` (regtest) dan `get_block_hash` 0 (`network_id` regtest), menolak `get_block_template` dan `submit_block`
(`-32601`), batch (`-32600`), `GET` (`405`), dan badan 262.145 byte (batas + 1, `413`).

## Batas
- Wallet MOD-15 dan CLI belum bisa memakai gerbang lewat internet sampai klien RPC mendukung HTTPS (TSK-M60).
- Riwayat wallet lewat gerbang memanggil `get_block` untuk setiap blok baru; dengan batas 20 per detik, riwayat penuh untuk
  ribuan blok butuh beberapa menit sekali (sesudah itu tembolok `history.json`).
- Semua pengunjung lewat Cloudflare; tanpa `CF-Connecting-IP` yang sah, mereka berbagi satu ember.

## Definition of done
- [x] Gerbang Q127 lolos
- [x] MOD-18, AGENTS.md, peta sistem, DEC-10 bagian 13, DEC-07 (pilihan demo), Rancangan Ekosistem Demo v0.1 Bagian 4.9 dan 6,
  Spesifikasi Docker v0.1 Bagian 2 dan 7, README, docs/tasks
