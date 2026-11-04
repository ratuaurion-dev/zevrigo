# MOD-05: Storage (`homeostasis-storage`)

- **ID:** `MOD-05`
- **Path:** `crates/storage`
- **Paket:** `homeostasis-storage`
- **Status:** 🟡 sedang berjalan (TSK-M10, TSK-M11, TSK-M26, dan TSK-M35 selesai; sisa: pruning TSK-M23)
- **Bergantung pada:** `MOD-01`
- **Dipakai oleh:** `MOD-11`, `MOD-14`, `MOD-17`

## Tujuan

Menyimpan chain aktif secara deterministik dan atomik: blok, set UTXO, dan undo data per blok, sehingga reorg sedalam apa pun bisa dibatalkan blok demi blok (protokol tanpa batas reorg dan tanpa checkpoint).

## Tanggung Jawab

- Trait `ChainStore`: ujung chain, ID blok per tinggi, blok, UTXO, undo data, `apply_block`, dan `undo_tip` (TSK-M10).
- `MemoryStore` berbasis `BTreeMap`: implementasi acuan untuk test dan regtest (TSK-M10).
- `RedbStore` persisten dengan redb 4.x (`DEC-03`): satu blok = satu transaksi tulis, diuji diferensial terhadap `MemoryStore` (TSK-M11).
- `utxos_by_payloads`: daftar UTXO per payload, urut outpoint, untuk RPC `scan_utxos` (TSK-M26). Memindai seluruh set UTXO tanpa indeks payload.
- Badan blok cabang menurut ID (TSK-M35 bagian A): `put_side_block`, `side_block`, `remove_side_block`, `clear_side_blocks`. Terpisah dari chain aktif (tabel redb `side_blocks`, ditambahkan tanpa menaikkan versi skema; file lama tetap terbuka) dan tidak mengubah ujung, set UTXO, blok chain aktif, maupun undo data.
- Integritas data saja: blok wajib menyambung ke ujung chain, UTXO yang dihabiskan wajib ada, outpoint baru tidak boleh ganda, dan setiap penerapan blok bersifat atomik (gagal berarti tidak ada perubahan).

## Bukan Tanggung Jawab

- Aturan konsensus: tanda tangan, jumlah, biaya, kematangan coinbase, PoW, cap waktu (`MOD-04`).
- Menghitung txid dan block_id. Nilainya diberikan pemanggil (dihitung dengan `MOD-02`), sehingga MOD-05 tetap hanya bergantung pada `MOD-01`.
- Indeks blok cabang, pemilihan chain, dan reorg (`MOD-11`). MOD-05 hanya menyimpan badan blok cabang menurut ID.
- Membuang undo data. Pruning (TSK-M23) hanya boleh membuang badan blok lama, karena reorg tidak dibatasi kedalamannya.

## Task terkait
- [TSK-M10](../tasks/TSK-M10-storage-memory.md) — 🟢 selesai (trait ChainStore dan MemoryStore)
- [TSK-M11](../tasks/TSK-M11-storage-redb.md) — 🟢 selesai (RedbStore dengan redb 4.4.0)
- [TSK-M26](../tasks/TSK-M26-storage-scan-utxo.md) — 🟢 selesai (pencarian UTXO per payload)
- [TSK-M35](../tasks/TSK-M35-cabang-di-disk.md) — 🟢 selesai (badan blok cabang di MOD-05; presync header di MOD-11)
- [TSK-M23](../tasks/README.md) — ⚪ pruning badan blok, kartu menyusul
