# TSK-M18 · Node regtest di MOD-11 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-11 (apps/node) · **Untuk:** FTR-01 sampai FTR-03
- **Bergantung pada:** TSK-M09, M11, M12, M13, M26, DEC-10, DEC-19 · **Memblokir:** TSK-F01, TSK-M19, TSK-M20
- **Estimasi:** M (dua langkah) · **Ditulis:** Quick (kode dan test), 31 Oktober 2026

## Tujuan
Menyatukan MOD-04 (aturan), MOD-05 (penyimpanan), MOD-07 (mempool), dan MOD-08 (RPC) menjadi node regtest satu
chain. Dipecah dua:
- **Bagian A (selesai):** pustaka `homeostasis_node` (`src/lib.rs`, `src/chain/`): `Node<S: ChainStore>` dan
  implementasi `RpcBackend` untuk 8 metode. Diuji tanpa jaringan dengan `MemoryStore` dan `RedbStore`.
- **Bagian B (selesai):** biner daemon (argumen, folder data `RedbStore`, logging, server RPC, berhenti dengan
  Ctrl-C) dan uji integrasi lewat RPC sungguhan, sekaligus menutup TSK-F01.

## Isi bagian A
| Item | Isi |
|---|---|
| `Node::open(store, params, mempool_max_bytes, clock)` | Penyimpanan kosong diisi genesis regtest; penyimpanan berisi wajib bergenesis sama (`WrongGenesis`). Ringkasan chain (ID, cap waktu, chainwork, total tambang per blok) disusun ulang dari penyimpanan |
| `submit_block` | V1 decode → `duplicate` → V2–V4 → `orphan` jika induk bukan ujung → V5–V7 → V8 `deferred` → V9–V10 dengan UTXO yang diambil dari penyimpanan → `apply_block` → mempool diperiksa ulang → `accepted` |
| `get_block_template` | Tinggi ujung + 1, target ASERT (argumen sama dengan V6), `min_timestamp` = MTP + 1, `timestamp = max(jam, min_timestamp)`, transaksi `select_for_block`, coinbase tepat subsidy + biaya (DEC-17), Merkle root benar, nonce 0 |
| `send_raw_transaction` | Decode gagal → 1003; mempool di tinggi ujung + 1; duplikat dianggap berhasil (txid yang sama); tolak → 1002 dengan `mempool_reason` |
| `get_chain_info`, `get_block_hash`, `get_block`, `get_mempool`, `scan_utxos` | Dari ringkasan chain, penyimpanan (`utxos_by_payloads`, TSK-M26), dan mempool |
| `block_rule`, `mempool_reason` | Nama aturan: `V1`–`V10`, `T1`–`T6` untuk transaksi di blok; mempool juga `struktur`, `terlalu_besar`, `mempool_penuh` |
| `Clock` | Jam disuntikkan (detik Unix) agar test deterministik; jam sistem dipasang di bagian B |

## Keputusan teknis (Quick)
- Satu `Mutex` menjaga penyimpanan, mempool, dan ringkasan chain; `RpcBackend` sinkron.
- Satu chain saja sampai TSK-F04: blok yang induknya bukan ujung dilaporkan `orphan` dan tidak disimpan (Spesifikasi
  RPC menyebut `orphan` untuk induk yang belum dikenal; dicatat untuk v0.2). Blok `deferred` (V8) tidak antre;
  pengirim mengirim ulang.
- Ringkasan chain disimpan di memori dan disusun ulang saat dibuka, sehingga skema `RedbStore` tetap versi 1.
- Paket `homeostasis-node` kini berisi pustaka dan biner; biner belum diubah di bagian A.

## Test bagian A (node 3 → 10, workspace 229 → 236)
`src/chain/tests.rs`: `starts_from_regtest_genesis`, `mines_blocks_from_template`, `submit_block_statuses_and_rules`
(V1–V8 dan V10, `orphan`, `deferred`, `duplicate`), `transaction_flow_through_mempool_and_block` (T2 di tinggi 1,
99 blok, T6, `struktur`, duplikat, T1, blok ke-100 memuat transaksi dengan biaya 1_000 quanta),
`reopen_rebuilds_state_and_rejects_wrong_genesis`, `memory_and_redb_nodes_agree`, `rule_names_for_blocks_and_mempool`.

## Isi bagian B
| Item | Isi |
|---|---|
| Argumen | `homeostasis-node --regtest --datadir <folder> [--rpc-port <port>]`, `--help`, `--version`. Tanpa argumen = `--help`. `--testnet`/`--mainnet` ditolak (DEC-01). Galat argumen keluar dengan kode 2, galat saat berjalan kode 1 |
| Folder data | `<datadir>/regtest/chain.redb` (RedbStore). Folder dibuat jika belum ada; isi lama dipakai ulang |
| Server RPC | `127.0.0.1:<port>`, bawaan regtest 28_801, `0` memilih port bebas. Setelah siap, stdout berisi satu baris `rpc: http://<alamat>` |
| Logging | `tracing-subscriber` ke stderr, level INFO, tanpa warna |
| Berhenti | SIGINT (Ctrl-C) atau SIGTERM. Penangkap sinyal dipasang sebelum server berjalan; server dihentikan, lalu penyimpanan ditutup |
| Jam | Jam sistem (detik Unix) untuk V8 dan cap waktu template |

## Keputusan teknis bagian B (Quick, tanpa DEC baru)
- Logika daemon di pustaka (`src/daemon/`), sehingga pembacaan argumen bisa diuji tanpa menjalankan biner.
- Jaringan wajib ditulis (`--regtest`), supaya saat testnet/mainnet tersedia tidak ada perubahan perilaku diam-diam.
- Dependensi aplikasi: `tokio` (fitur `rt-multi-thread`, `macros`, `signal`), `tracing`, `tracing-subscriber` (fitur
  `std`, `fmt` saja). Semuanya tercakup DEC-10 bagian 2.

## Test bagian B (node 10 → 14, workspace 236 → 240)
- `src/daemon/tests.rs`: `parses_help_version_and_run`, `rejects_bad_arguments`, `chain_file_lives_under_network_folder`.
- `tests/regtest_node.rs` (TSK-F01): `daemon_mines_over_rpc_and_restarts`: biner sungguhan di folder sementara, satu
  blok ditambang lewat RPC (`get_block_template` → cari nonce → `submit_block`), hadiah terlihat di `scan_utxos`,
  SIGINT → keluar kode 0, dibuka ulang dengan chain dan UTXO yang sama.
- `tests/smoke.rs`: `help_and_no_args_print_placeholder_info` diganti `help_and_no_args_print_usage` (teks bantuan baru).
