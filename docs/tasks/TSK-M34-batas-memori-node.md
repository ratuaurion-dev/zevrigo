# TSK-M34 · Batas memori node: blok cabang, indeks header, catatan blok tidak sah · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-11 (apps/node) · **Untuk:** persiapan testnet (FTR-06)
- **Bergantung pada:** TSK-M27, TSK-M28 · **Memblokir:** TSK-F06 · **Lanjutan:** TSK-M35 (disimpan di disk)
- **Estimasi:** S · **Ditulis:** Quick (kode dan test), 2 November 2026

## Tujuan
Data node yang hanya ada di memori (blok cabang, header tanpa badan blok, catatan blok tidak sah) sebelumnya tanpa batas,
sehingga rekan bisa menghabiskan memori node dengan cabang atau header murah. Batasnya kini tetap, tanpa menghalangi
reorg ke chain yang lebih berat (Spesifikasi P2P v0.1 Bagian 9).

## Isi
| Bagian | Isi |
|---|---|
| `NodeLimits` (baru, publik) | `max_side_blocks` 10_000, `max_side_bytes` 64 MiB, `max_headers` 100_000, `max_invalid` 10_000; `NodeLimits::DEFAULT`, `Default` |
| `Node::open_with_limits` (baru) | Seperti `Node::open` dengan batas pilihan; `Node::open` memakai `NodeLimits::DEFAULT` |
| Blok cabang | `trim_side` setelah setiap blok cabang dan reorg: buang daun dengan chainwork paling kecil (seri: yang paling akhir), kecuali blok di jalur menuju header yang lebih berat daripada chain aktif (`protected_side`) |
| Indeks header | `store_header` saat penuh: buang header daun paling ringan yang lebih ringan daripada header baru dan bukan induknya, atau berhenti dengan `HeadersOutcome::Full { added }` (tanpa sanksi; P2P memperlakukannya seperti `Deferred`) |
| Catatan blok tidak sah | `note_invalid` (FIFO lewat `invalid_order`) menggantikan semua `invalid.insert` |
| Keturunan | `descendants`: satu peta induk ke anak untuk `mark_invalid` dan `prune_headers` (dulu berulang per tingkat, kuadratik) |

## Keputusan teknis (Quick, tanpa DEC)
- **Buang daun, bukan blok sembarang**, agar setiap cabang yang tersisa tetap tersambung ke chain aktif (reorg dan
  `parent_info` tidak pernah menemui leluhur yang hilang).
- **Lindungi cabang yang terbukti lebih berat.** Header yang chainwork-nya melebihi chain aktif butuh kerja PoW sungguhan,
  jadi memori tambahan hanya bisa dipaksa dengan kerja melebihi chain aktif. Prinsip tanpa batas reorg tetap terjaga.
- **Batas yang tersisa dicatat**, bukan ditutupi: cabang lebih dari 100_000 blok yang belum lebih berat dalam jendela
  indeks header tidak bisa disinkronkan lewat P2P sampai TSK-M35.

## Test (node 25 → 31, workspace 360 → 366)
`default_limits`, `side_blocks_are_trimmed_leaves_with_least_work_first` (daun paling ringan dan paling akhir dibuang,
blok terbuang dilupakan, reorg tetap jalan), `side_blocks_are_trimmed_by_size`, `heavier_branch_being_synced_is_never_trimmed`
(cabang yang sedang diunduh melebihi batas tetap utuh, cabang sampah dibuang, chain lama dipangkas setelah reorg),
`header_index_is_bounded_and_keeps_heavier_headers` (`Full`, header sampah dibuang untuk header yang lebih berat,
sinkronisasi berlanjut setelah badan blok tiba), `invalid_block_records_are_bounded`.
