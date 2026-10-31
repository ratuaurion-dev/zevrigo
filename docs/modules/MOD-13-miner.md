# MOD-13: Miner (`homeostasis-miner`)

- **ID:** `MOD-13`
- **Path:** `apps/miner`
- **Paket:** `homeostasis-miner` (pustaka + bin)
- **Status:** 🟢 selesai untuk regtest (TSK-M20, PoW dev DEC-09); PoW mainnet menunggu `DEC-01`
- **Bergantung pada:** `MOD-01`, `MOD-03`, `MOD-08`
- **Dipakai oleh:** —

## Tujuan

Menyediakan aplikasi penambang CPU mandiri yang mengambil templat blok dari node melalui RPC, mencari solusi Proof of Work, dan mengirimkan blok yang ditemukan.

## Tanggung Jawab

- `homeostasis-miner --payout <hex> [--payout-kind 0|1] [--rpc-url <url>] [--blocks <n>]`: ambil template (`get_block_template`, termasuk `full_target`), cari nonce dengan `mine_regtest` (`MOD-03`), kirim (`submit_block`), ulangi.
- Satu thread pencarian; template diambil ulang setiap 1_000_000 nonce atau setelah blok diterima.

## Bukan Tanggung Jawab

- Validasi penuh rantai atau penyimpanan UTXO (`MOD-04`, `MOD-05`).
- Decode target compact (aturan konsensus `MOD-04`); target 256-bit diberikan node lewat `full_target`.

## Task terkait
- [TSK-M20](../tasks/TSK-M20-miner.md) — 🟢 selesai
