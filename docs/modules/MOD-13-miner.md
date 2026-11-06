# MOD-13: Miner (`zevrigo-miner`)

- **ID:** `MOD-13`
- **Path:** `apps/miner`
- **Paket:** `zevrigo-miner` (pustaka + bin)
- **Status:** 🟢 regtest (TSK-M20, PoW dev DEC-09); 🟡 testnet RandomX (TSK-M06 bagian C3, DEC-01)
- **Bergantung pada:** `MOD-01`, `MOD-03`, `MOD-08`
- **Dipakai oleh:** —

## Tujuan

Menyediakan aplikasi penambang CPU mandiri yang mengambil templat blok dari node melalui RPC, mencari solusi Proof of Work, dan mengirimkan blok yang ditemukan.

## Tanggung Jawab

- `zevrigo-miner --payout <hex> [--payout-kind 0|1] [--rpc-url <url>] [--blocks <n>]`: ambil template (`get_block_template`, termasuk `full_target`), cari nonce dengan `mine_regtest` (`MOD-03`), kirim (`submit_block`), ulangi.
- Satu thread pencarian; template diambil ulang setiap 1_000_000 nonce atau setelah blok diterima.
- Testnet (TSK-M06 bagian C3): `--testnet` (RPC bawaan `http://127.0.0.1:18801`) menambang RandomX dengan kunci `pow_key`
  dari template (`RandomxMiner` MOD-03). Bawaan mode cepat (dataset sekitar 2 GiB, disusun sekitar satu menit setiap kunci
  berganti, yaitu setiap 2_048 blok); `--light` mode ringan (256 MiB, jauh lebih lambat). VM disimpan selama kunci sama;
  template diambil ulang setiap 2_000 nonce. Sebelum menambang, miner memeriksa jaringan node (`get_chain_info`).
- Batas: satu utas. Pustaka `randomx-rs` tidak menandai dataset aman dipakai bersama antar utas, jadi beberapa utas mode
  cepat (satu dataset bersama) menunggu perubahan pustaka itu.

## Bukan Tanggung Jawab

- Validasi penuh rantai atau penyimpanan UTXO (`MOD-04`, `MOD-05`).
- Decode target compact (aturan konsensus `MOD-04`); target 256-bit diberikan node lewat `full_target`.

## Task terkait
- [TSK-M20](../tasks/TSK-M20-miner.md) — 🟢 selesai
- [TSK-M06](../tasks/TSK-M06-pow-randomx.md) — 🟡 bagian C3 (miner testnet RandomX)
