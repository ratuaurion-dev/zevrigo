# MOD-18: Layanan demo (`apps/demo`)

- **ID:** `MOD-18`
- **Path:** `apps/demo`
- **Paket:** `zevrigo-demo` (bin) *(belum dibuat, diinisialisasi di TSK-M53)*
- **Status:** ⚪ belum dimulai (DEC-34, DEC-36)
- **Bergantung pada:** `MOD-01`, `MOD-02`, `MOD-08`, `MOD-09`
- **Dipakai oleh:** —

## Tujuan

Menyediakan layanan publik jaringan demo regtest untuk pemula (Rancangan Ekosistem Demo v0.1): satu aplikasi dengan tiga mode
yang dijalankan sebagai tiga proses di belakang Cloudflare Tunnel, masing-masing dengan hostname sendiri.

## Tanggung Jawab

- `rpc`: gerbang RPC untuk wallet HP. Meneruskan hanya metode aman ke node (`get_chain_info`, `get_block_hash`, `get_block`,
  `get_mempool`, `scan_utxos`, `scan_mempool`, `send_raw_transaction`), dengan batas laju per pengunjung dan batas ukuran.
- `faucet`: wallet panas berstok kecil di node faucet, 100 ZVRG per pengunjung per hari dan per alamat, tanpa menyimpan IP.
- `daftar`: pendaftaran Investor Demo dengan pesan bertanda tangan Schnorr, antrean siapa cepat dia dapat, alat operator.

## Bukan Tanggung Jawab

- Aturan konsensus atau validasi (`MOD-04`), penyimpanan chain (`MOD-05`), tampilan explorer (`MOD-14`).
- Jaringan testnet dan mainnet. MOD-18 hanya untuk demo regtest.

## Task terkait
- [TSK-M53](../tasks/README.md) — ⚪ inisialisasi dan mode `rpc`
- [TSK-M54](../tasks/README.md) — ⚪ mode `faucet`
- [TSK-M55](../tasks/README.md) — ⚪ mode `daftar`
