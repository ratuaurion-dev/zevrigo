# TSK-F02 · FTR-02 Menambang di regtest · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task fitur · **Fitur:** FTR-02 · **Modul:** MOD-01, 03, 04, 08, 11, 13 (diuji di MOD-17)
- **Bergantung pada:** TSK-F01, TSK-M20, DEC-20 · **Memblokir:** TSK-F03, TSK-F04
- **Estimasi:** XS · **Ditulis:** Quick (uji), 31 Oktober 2026

## Kriteria selesai (docs/features/README.md)
Miner mengambil template blok, menemukan blok, node menerimanya, dan coinbase sama persis dengan `block_subsidy`
(TSK-M07) ditambah biaya transaksi.

## Bukti
`tests/integration/tests/ftr02_mining.rs` · `miner_and_node_follow_supply_schedule`, di `make check-all`:
- `Node<RedbStore>` (di memori) + `RpcServer` + `Miner` di satu proses, lewat RPC HTTP sungguhan. Jam node disuntikkan
  (maju 120 detik per blok), sehingga target tetap `pow_limit` dan hasil deterministik.
- 99 blok ditambang, lalu transaksi yang membelanjakan alokasi genesis (biaya 1_000 quanta) masuk mempool, lalu blok 100.
- Diperiksa untuk setiap tinggi 1 sampai 100: coinbase tepat `SupplySchedule[h].subsidy` (+ biaya di blok 100), satu
  output ke payload penerima, cap waktu `T0 + 120h`, target `0x207fffff`.
- `get_chain_info`: tinggi 100, `mined_total` = jadwal pasokan setelah blok 100, `total_supply = G + mined_total`,
  chainwork 202, mempool kosong. `scan_utxos`: 100 UTXO coinbase berjumlah `mined_total + biaya`; penerima transaksi
  punya satu UTXO; alokasi genesis sudah dibelanjakan.

## Keputusan
- **DEC-20:** MOD-17 memakai pustaka MOD-11 (node) dan MOD-13 (miner) hanya sebagai dev-dependency. Crate inti tetap
  tidak boleh bergantung pada aplikasi. Keduanya dirujuk lewat `path` di `tests/integration/Cargo.toml`, bukan lewat
  `[workspace.dependencies]`, agar daftar dependensi workspace tetap hanya pustaka inti.
- Uji dua biner sekaligus (proses terpisah) belum dibuat; biner node sudah diuji di `apps/node/tests/regtest_node.rs`,
  dan biner miner hanya membungkus pustaka yang diuji di sini.

## Test (MOD-17 1 → 2, workspace 246 → 247)
`tests/ftr02_mining.rs`: `miner_and_node_follow_supply_schedule`. Uji lama `core_crates_expose_expected_module_ids` tetap.
