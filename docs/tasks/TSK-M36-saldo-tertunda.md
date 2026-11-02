# TSK-M36 · Saldo CLI dan pemilihan koin dengan transaksi tertunda · 🟢 selesai · Prioritas: Sedang
- **Jenis:** task modul · **Modul:** MOD-08 (rpc), MOD-11 (node), MOD-12 (cli) · **Untuk:** FTR-03, persiapan FTR-07
- **Bergantung pada:** TSK-M19 · **Memblokir:** —
- **Estimasi:** S · **Ditulis:** Quick (kode dan test), 2 November 2026

## Tujuan
Sebelumnya `balance` hanya melihat UTXO di chain: sesudah `send`, koin yang sedang dibelanjakan masih terhitung bisa
dipakai, kembalian dan kiriman masuk tidak terlihat sampai masuk blok, dan `send` kedua bisa memilih koin yang sama
(ditolak node dengan T1). Sekarang wallet melihat transaksi tertunda di mempool node.

## Isi
| Modul | Isi |
|---|---|
| MOD-08 | Metode ke-9 `scan_mempool` (Spesifikasi RPC 3.9): `ScanMempoolParams { payloads }`, hasil `MempoolScan { spent: [MempoolSpend { txid, index, spent_by }], outputs: [MempoolOutput { txid, index, amount, kind, payload }] }`; `RpcBackend::scan_mempool`, server, `RpcClient::scan_mempool`, `METHODS` 9 |
| MOD-11 | `Node::scan_mempool`: UTXO payload yang `Mempool::spender`-nya ada, dan output transaksi mempool ke payload, urut `(txid, index)`; `Relay` meneruskan |
| MOD-12 | `scan::wallet_payloads` (kedua cabang sampai indeks terpakai + 20); `Balance` + `pending_spent`, `pending_received`, `after_pending`; `spendable` tanpa UTXO yang sedang dibelanjakan; `send` tidak memilih UTXO itu; keluaran `balance` + 3 baris |

## Keputusan teknis (Quick, tanpa DEC)
- **Metode RPC baru, bukan catatan di file wallet.** Wallet tetap tanpa status (hanya frasa dan indeks), melihat juga
  kiriman masuk dari wallet lain, dan tetap benar jika transaksi dikirim dari salinan wallet lain atau dibuang mempool.
- Hanya output jenis 0 yang dihitung masuk tertunda, sama dengan pemindaian saldo.
- Output tertunda tidak bisa dibelanjakan (mempool v0.1 tanpa rantai transaksi tertunda).

## Test (workspace 366 → 369)
- MOD-11: `scan_mempool_reports_pending_spends_and_outputs` (alokasi genesis dibelanjakan di mempool, payload yang
  diminta saja, kosong setelah masuk blok).
- MOD-12: `balance_and_send_follow_pending_mempool` (saldo, keluaran, `send` memilih koin lain, jenis 1 dan payload lain
  diabaikan), `wallet_payloads_cover_used_indices_and_gap`; test lama diperbarui untuk keluaran baru.
- MOD-08: `round_trip_all_methods` dan `invalid_params_and_unknown_method_are_rejected` mencakup `scan_mempool`.
- MOD-17: `two_cli_wallets_send_and_receive_over_rpc` memeriksa saldo kedua wallet selama transaksi menunggu di mempool
  dan `send` kedua yang ditolak karena koin sedang dibelanjakan.
