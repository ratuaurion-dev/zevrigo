# MOD-18: Layanan demo (`apps/demo`)

- **ID:** `MOD-18`
- **Path:** `apps/demo`
- **Paket:** `zevrigo-demo` (pustaka + bin)
- **Status:** 🟡 mode `rpc` (gerbang RPC, TSK-M53); mode `faucet` dan `daftar` belum (DEC-34, DEC-36)
- **Bergantung pada:** `MOD-01`, `MOD-02`, `MOD-08`, `MOD-09` (DEC-36; mode `rpc` hanya memakai `MOD-08`)
- **Dipakai oleh:** —

## Tujuan

Menyediakan layanan publik jaringan demo regtest untuk pemula (Rancangan Ekosistem Demo v0.1): satu aplikasi dengan tiga mode
yang dijalankan sebagai tiga proses di belakang Cloudflare Tunnel, masing-masing dengan hostname sendiri.

## Tanggung Jawab

- `zevrigo-demo rpc [--node-url <url>] [--http-bind <ip>] [--http-port <port>] [--rate <n>] [--trust-proxy]` (TSK-M53):
  gerbang RPC untuk wallet peserta di `127.0.0.1:28804` (bawaan). Satu permintaan JSON-RPC 2.0 per `POST`; hanya
  `get_chain_info`, `get_block_hash`, `get_block`, `get_mempool`, `scan_utxos`, `scan_mempool`, dan `send_raw_transaction`
  yang diteruskan ke node lewat `RpcClient` (MOD-08), dan hasil atau galat node (kode, pesan, `data`) dikembalikan apa adanya
  dengan `id` permintaan. Metode lain (termasuk `get_block_template` dan `submit_block`) `-32601`; batch `-32600`; badan di
  atas 256 KiB `413`; selain `POST` `405`; node tidak menjawab `-32603` tanpa rincian.
- Batas laju per pengunjung (bawaan 20 per detik, kapasitas 80) dan bersama (400 per detik), diperiksa sebelum badan dibaca;
  `429 {"error":"rate_limited"}` dengan `Retry-After: 1`. Modul `limit` adalah salinan pembatas MOD-14 (TSK-M40) karena MOD-18
  tidak bergantung pada MOD-14. Dengan `--trust-proxy`, alamat pengunjung dari `CF-Connecting-IP` hanya untuk koneksi dari
  loopback (pola TSK-M43). Alamat IP tidak disimpan dan tidak dicatat; log tanpa metode atau parameter.
- `faucet` (TSK-M54) dan `daftar` (TSK-M55): belum tersedia (`mode ... belum tersedia`).

## Bukan Tanggung Jawab

- Aturan konsensus atau validasi (`MOD-04`), penyimpanan chain (`MOD-05`), tampilan explorer (`MOD-14`).
- HTTPS: TLS diputus Cloudflare; gerbang hanya HTTP di loopback.
- Jaringan testnet dan mainnet. MOD-18 hanya untuk demo regtest.

## Task terkait
- [TSK-M53](../tasks/TSK-M53-gerbang-rpc.md) — 🟢 inisialisasi dan mode `rpc`
- [TSK-M54](../tasks/README.md) — ⚪ mode `faucet`
- [TSK-M55](../tasks/README.md) — ⚪ mode `daftar`
