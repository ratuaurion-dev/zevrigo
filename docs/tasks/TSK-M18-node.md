# TSK-M18 · Node regtest di MOD-11 · 🟡 bagian A selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-11 (apps/node) · **Untuk:** FTR-01 sampai FTR-03
- **Bergantung pada:** TSK-M09, M11, M12, M13, M26, DEC-10, DEC-19 · **Memblokir:** TSK-F01, TSK-M19, TSK-M20
- **Estimasi:** M (dua langkah) · **Ditulis:** Quick (kode dan test), 31 Oktober 2026

## Tujuan
Menyatukan MOD-04 (aturan), MOD-05 (penyimpanan), MOD-07 (mempool), dan MOD-08 (RPC) menjadi node regtest satu
chain. Dipecah dua:
- **Bagian A (selesai):** pustaka `homeostasis_node` (`src/lib.rs`, `src/chain/`): `Node<S: ChainStore>` dan
  implementasi `RpcBackend` untuk 8 metode. Diuji tanpa jaringan dengan `MemoryStore` dan `RedbStore`.
- **Bagian B (berikutnya):** biner daemon (argumen, folder data `RedbStore`, logging, server RPC, berhenti dengan
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
