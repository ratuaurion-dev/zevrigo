# TSK-M20 · Miner CPU dengan PoW dev regtest di MOD-13 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-13 (apps/miner) · **Untuk:** FTR-02
- **Bergantung pada:** TSK-M05, TSK-M12, TSK-M18 · **Memblokir:** TSK-F02
- **Estimasi:** S · **Ditulis:** Quick (kode dan test), 31 Oktober 2026

## Tujuan
Penambang CPU yang mengambil template dari node lewat RPC (MOD-08), mencari nonce dengan PoW dev regtest (MOD-03,
DEC-09), lalu mengirim blok. Hanya regtest; PoW mainnet menunggu DEC-01.

## Pemakaian
`homeostasis-miner --payout <payload hex 64> [--payout-kind 0|1] [--rpc-url <url>] [--blocks <n>]`
- `--payout`: kunci x-only (jenis 0) atau hash kondisi (jenis 1) penerima hadiah. Format alamat bech32m masih
  `TODO(param)`, jadi yang dipakai payload hex.
- `--rpc-url` bawaan `http://127.0.0.1:28801`; `--blocks` bawaan 0 (terus sampai Ctrl-C).
- Galat argumen keluar dengan kode 2, galat saat menambang kode 1.

## Isi
| Item | Isi |
|---|---|
| `cli` | `parse_args` tanpa clap, `MinerConfig`, `ArgError`, `USAGE` |
| `mining::Miner` | `connect(url, kind, payload, attempts)`, `round()`, `run(blocks, on_round)`. Runtime tokio satu thread untuk RPC; pencarian nonce dengan `mine_regtest` di luar runtime |
| `Round` | `Accepted { height, block_id, nonce }`, `Stale { status }` (duplicate/orphan/deferred), `Exhausted` (nonce belum ketemu dalam `ATTEMPTS_PER_TEMPLATE` = 1_000_000; putaran berikutnya mengambil template baru) |
| `MinerError` | `Runtime`, `Rpc`, `BadTemplate`, `Rejected { reason }` (node menyatakan blok tidak sah = bug) |

## Keputusan teknis (Quick)
- **`full_target` di hasil `get_block_template`** (Spesifikasi RPC 3.4): target 256-bit big-endian yang sudah
  di-decode node. Decode compact adalah aturan konsensus V2 di MOD-04, sedangkan miner hanya boleh memakai MOD-01, 03,
  dan 08 dan aturan konsensus tidak boleh ditulis di luar MOD-04. Menambah field hasil tidak memecah klien lama.
  Perubahan ini menyentuh MOD-08 (`BlockTemplate`) dan MOD-11 (mengisi field) di commit yang sama.
- Miner tidak memeriksa ulang blok; node memeriksa V1 sampai V10. Miner hanya memastikan header template konsisten
  dengan field `height` dan `target`.
- Satu thread pencarian. Multi-thread menyusul jika dibutuhkan (regtest di `pow_limit` butuh sekitar 2 percobaan).
- Dependensi aplikasi: `tokio` (fitur `rt`; dev `rt-multi-thread`) dan `thiserror`, tercakup DEC-10 bagian 2.
  Tanpa paket baru di Cargo.lock.

## Test (miner 3 → 9, workspace 240 → 246)
- `src/cli/tests.rs`: `parses_help_version_and_run`, `rejects_bad_arguments`.
- `src/mining/tests.rs` (node tiruan di atas `RpcServer` MOD-08): `mines_chain_of_blocks_over_rpc` (3 blok
  berurutan, PoW lolos, coinbase ke payload penerima), `unreachable_target_exhausts_round`,
  `bad_template_and_rejection_are_errors`, `unreachable_node_is_rpc_error`.
- `tests/smoke.rs`: teks bantuan diganti (`help_and_no_args_print_usage`).
- Node: `mines_blocks_from_template` kini juga memeriksa `full_target` = `pow_limit` regtest.

## Belum di task ini
Uji ujung ke ujung biner miner + biner node (TSK-F02) butuh tempat uji yang memakai dua aplikasi sekaligus;
diputuskan saat TSK-F02.
