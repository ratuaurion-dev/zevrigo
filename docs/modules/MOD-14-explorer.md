# MOD-14: Explorer (`homeostasis-explorer`)

- **ID:** `MOD-14`
- **Path:** `apps/explorer`
- **Paket:** `homeostasis-explorer` (bin)
- **Status:** 🟡 indexer (TSK-M29), API dan program (TSK-M30), uji ujung ke ujung (TSK-F05), penyajian web `--web-dir` (TSK-M31), dan antarmuka web Next.js + Tailwind + shadcn/ui (TSK-M38, menggantikan TSK-M32) selesai; paket Docker (TSK-M33) berikutnya
- **Bergantung pada:** `MOD-01`, `MOD-02`, `MOD-04`, `MOD-08`, `MOD-09` (DEC-25); pustaka `axum` dan `rusqlite` (DEC-10 bagian 6)
- **Dipakai oleh:** `MOD-17` (hanya dev-dependency, DEC-25)

## Tujuan

Menyediakan indexer chain dan API HTTP JSON baca saja bagi block explorer Homeostasis Coin, sesuai
[spesifikasi-explorer-v0.1.md](../reference/spesifikasi-explorer-v0.1.md), sehingga siapa pun bisa memeriksa blok,
transaksi, alamat, pasokan, dan alokasi genesis dari node miliknya sendiri.

## Tanggung Jawab

- Indexer SQLite (`<datadir>/<jaringan>/explorer.sqlite`) yang membaca node lewat RPC v0.1, mundur sedalam apa pun saat
  reorg, dan memeriksa sendiri txid, block_id, serta jumlah koin setiap blok (I1–I4).
- API `GET /api/v1/…`: status, blok, transaksi, alamat, pasokan dibanding jadwal rumus, halaman alokasi genesis, mempool,
  pencarian. Jumlah koin selalu `u128` quanta sebagai string desimal.
- Privasi pengunjung: tanpa cookie, analitik, atau log alamat IP dan kata pencarian.
- Antarmuka web Next.js + Tailwind CSS + shadcn/ui di `web/` (DEC-26,
  [spesifikasi-explorer-web-v0.2.md](../reference/spesifikasi-explorer-web-v0.2.md)), dua bahasa, static export yang
  disajikan program explorer lewat `--web-dir` dengan kebijakan keamanan konten berisi hash skrip inline setiap halaman.

## Bukan Tanggung Jawab

- Validasi konsensus (`MOD-04`, dijalankan node) maupun penyimpanan kunci pengguna (`MOD-09`).
- Penyimpanan chain node (`MOD-05`): explorer punya indeks sendiri.
- Paket Docker (TSK-M33, menyusul sebelum testnet publik).

## Task terkait
- [TSK-S06](../reference/spesifikasi-explorer-v0.1.md) — 🟢 spesifikasi v0.1
- [TSK-M29](../tasks/TSK-M29-explorer-indexer.md) — 🟢 selesai (indexer, pemeriksaan I1–I4, query)
- [TSK-M30](../tasks/TSK-M30-explorer-api.md) — 🟢 selesai (API axum, adaptor RPC, program `homeostasis-explorer`)
- [TSK-F05](../tasks/TSK-F05-explorer-minimal.md) — 🟢 selesai (FTR-05, uji ujung ke ujung di MOD-17)
- [TSK-S07](../reference/spesifikasi-explorer-web-v0.2.md) — 🟢 spesifikasi antarmuka web v0.2 (v0.1 diganti)
- [TSK-M31](../tasks/TSK-M31-explorer-web-dir.md) — 🟢 selesai (opsi `--web-dir`, penyajian file aman, kepala keamanan)
- [TSK-M32](../tasks/TSK-M32-explorer-web-react.md) — 🟢 selesai (aplikasi web React di `web/`, dua bahasa, pemakaian pertama npm; diganti TSK-M38)
- [TSK-M38](../tasks/TSK-M38-explorer-web-nextjs.md) — 🟢 selesai (Next.js + Tailwind + shadcn/ui dari AI Studio, CSP dengan hash di Rust)
