# MOD-14: Explorer (`homeostasis-explorer`)

- **ID:** `MOD-14`
- **Path:** `apps/explorer`
- **Paket:** `homeostasis-explorer` (bin)
- **Status:** 🟡 indexer SQLite selesai (TSK-M29); API dan program menunggu TSK-M30
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

## Bukan Tanggung Jawab

- Validasi konsensus (`MOD-04`, dijalankan node) maupun penyimpanan kunci pengguna (`MOD-09`).
- Penyimpanan chain node (`MOD-05`): explorer punya indeks sendiri.
- Antarmuka web dan paket Docker (menyusul sebelum testnet publik).

## Task terkait
- [TSK-S06](../reference/spesifikasi-explorer-v0.1.md) — 🟢 spesifikasi v0.1
- [TSK-M29](../tasks/TSK-M29-explorer-indexer.md) — 🟢 selesai (indexer, pemeriksaan I1–I4, query)
- TSK-M30 (API dan program) — 🟡 siap ([backlog](../tasks/README.md))
- TSK-F05 (FTR-05, uji ujung ke ujung) — ⚪
