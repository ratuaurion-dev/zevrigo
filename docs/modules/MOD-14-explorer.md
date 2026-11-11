# MOD-14: Explorer (`zevrigo-explorer`)

- **ID:** `MOD-14`
- **Path:** `apps/explorer`
- **Paket:** `zevrigo-explorer` (bin)
- **Status:** 🟡 indexer (TSK-M29), API dan program (TSK-M30), uji ujung ke ujung (TSK-F05), penyajian web `--web-dir` (TSK-M31), antarmuka web Next.js + Tailwind + shadcn/ui (TSK-M38, menggantikan TSK-M32), tampilan hanya data sistem (TSK-M39, DEC-28), paket Docker regtest (TSK-M33), batas laju API (TSK-M40), batas laju di belakang Cloudflare Tunnel (TSK-M43), dan explorer publik lewat tunnel dengan regtest (TSK-M44) selesai; explorer testnet publik berikutnya
- **Bergantung pada:** `MOD-01`, `MOD-02`, `MOD-04`, `MOD-08` (DEC-25; `MOD-09` tidak dipakai lagi sejak DEC-28); pustaka `axum`, `rusqlite` (DEC-10 bagian 6), `sha2` (bagian 4)
- **Dipakai oleh:** `MOD-17` (hanya dev-dependency, DEC-25)

## Tujuan

Menyediakan indexer chain dan API HTTP JSON baca saja bagi block explorer Zevrigo Coin, sesuai
[spesifikasi-explorer-v0.1.md](../reference/spesifikasi-explorer-v0.1.md), sehingga siapa pun bisa memeriksa blok,
transaksi, alamat, pasokan, dan alokasi genesis dari node miliknya sendiri.

## Tanggung Jawab

- Jaringan regtest atau testnet (`--testnet`, TSK-M06 C2; port HTTP bawaan testnet 18802, RPC node 18801).
- Indexer SQLite (`<datadir>/<jaringan>/explorer.sqlite`) yang membaca node lewat RPC v0.1, mundur sedalam apa pun saat
  reorg, dan memeriksa sendiri txid, block_id, serta jumlah koin setiap blok (I1–I4).
- API `GET /api/v1/…` hanya data sistem (DEC-28): status, blok, ringkasan transaksi, pasokan hasil tambang dibanding
  jadwal rumus, jumlah mempool, pencarian tinggi/block_id/txid. Tanpa alamat, saldo, jejak koin, dan alokasi genesis
  (tetap dicatat di indeks untuk pemeriksaan I1–I4). Jumlah koin selalu `u128` quanta sebagai string desimal.
- Privasi pengunjung: tanpa cookie, analitik, atau log alamat IP dan kata pencarian.
- Batas laju API per klien dan bersama (`--api-rate`, TSK-M40), tanpa menyimpan alamat IP (hash dengan kunci acak per
  proses, hanya di memori).
- Di belakang Cloudflare Tunnel (`--trust-proxy`, TSK-M43): untuk koneksi dari loopback, pengunjung dikenali dari kepala
  `CF-Connecting-IP`; koneksi dari alamat lain tidak bisa memalsukan alamatnya.
- Antarmuka web Next.js + Tailwind CSS + shadcn/ui di `web/` (DEC-26,
  [spesifikasi-explorer-web-v0.2.md](../reference/spesifikasi-explorer-web-v0.2.md)), dua bahasa, static export yang
  disajikan program explorer lewat `--web-dir` dengan kebijakan keamanan konten berisi hash skrip inline setiap halaman.

## Bukan Tanggung Jawab

- Validasi konsensus (`MOD-04`, dijalankan node) maupun penyimpanan kunci pengguna (`MOD-09`).
- Penyimpanan chain node (`MOD-05`): explorer punya indeks sendiri.
- HTTPS dan perlindungan DDoS untuk explorer publik (di sisi Cloudflare Tunnel; atau reverse proxy di VPS nanti).

## Task terkait
- [TSK-S06](../reference/spesifikasi-explorer-v0.1.md) — 🟢 spesifikasi v0.1
- [TSK-M29](../tasks/TSK-M29-explorer-indexer.md) — 🟢 selesai (indexer, pemeriksaan I1–I4, query)
- [TSK-M30](../tasks/TSK-M30-explorer-api.md) — 🟢 selesai (API axum, adaptor RPC, program `zevrigo-explorer`)
- [TSK-F05](../tasks/TSK-F05-explorer-minimal.md) — 🟢 selesai (FTR-05, uji ujung ke ujung di MOD-17)
- [TSK-S07](../reference/spesifikasi-explorer-web-v0.2.md) — 🟢 spesifikasi antarmuka web v0.2 (v0.1 diganti)
- [TSK-M31](../tasks/TSK-M31-explorer-web-dir.md) — 🟢 selesai (opsi `--web-dir`, penyajian file aman, kepala keamanan)
- [TSK-M32](../tasks/TSK-M32-explorer-web-react.md) — 🟢 selesai (aplikasi web React di `web/`, dua bahasa, pemakaian pertama npm; diganti TSK-M38)
- [TSK-M38](../tasks/TSK-M38-explorer-web-nextjs.md) — 🟢 selesai (Next.js + Tailwind + shadcn/ui dari AI Studio, CSP dengan hash di Rust)
- [TSK-M39](../tasks/TSK-M39-explorer-data-sistem.md) — 🟢 selesai (hanya data sistem: tanpa alamat, jejak koin, dan alokasi genesis; DEC-28)
- [TSK-M33](../tasks/TSK-M33-paket-docker.md) — 🟢 selesai (paket Docker regtest: node + explorer + web, opsi `--http-bind`)
- [TSK-M40](../tasks/TSK-M40-batas-laju-api.md) — 🟢 selesai (batas laju API per klien dan bersama, `429 rate_limited`)
- [TSK-M43](../tasks/TSK-M43-explorer-trust-proxy.md) — 🟢 selesai (batas laju per pengunjung di belakang Cloudflare Tunnel, `--trust-proxy`)
- [TSK-M44](../tasks/TSK-M44-explorer-publik-tunnel.md) — 🟢 selesai (explorer publik lewat Cloudflare Tunnel, `compose.publik.yaml`)
