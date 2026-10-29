# MOD-05: Storage (`homeostasis-storage`)

- **ID:** `MOD-05`
- **Path:** `crates/storage`
- **Paket:** `homeostasis-storage`
- **Status:** ⚪ belum dimulai (TSK-M10 menunggu TSK-M22; penyimpanan redb TSK-M11 menunggu TSK-M10)
- **Bergantung pada:** `MOD-01`
- **Dipakai oleh:** `MOD-11`, `MOD-14`, `MOD-17`

## Tujuan

Menyimpan chain aktif secara deterministik dan atomik: blok, set UTXO, dan undo data per blok, sehingga reorg sedalam apa pun bisa dibatalkan blok demi blok (protokol tanpa batas reorg dan tanpa checkpoint).

## Tanggung Jawab

- Trait `ChainStore`: ujung chain, ID blok per tinggi, blok, UTXO, undo data, `apply_block`, dan `undo_tip` (TSK-M10).
- `MemoryStore` berbasis `BTreeMap`: implementasi acuan untuk test dan regtest (TSK-M10).
- `RedbStore` persisten dengan redb 4.x (`DEC-03`): satu blok = satu transaksi tulis, diuji diferensial terhadap `MemoryStore` (TSK-M11).
- Integritas data saja: blok wajib menyambung ke ujung chain, UTXO yang dihabiskan wajib ada, outpoint baru tidak boleh ganda, dan setiap penerapan blok bersifat atomik (gagal berarti tidak ada perubahan).

## Bukan Tanggung Jawab

- Aturan konsensus: tanda tangan, jumlah, biaya, kematangan coinbase, PoW, cap waktu (`MOD-04`).
- Menghitung txid dan block_id. Nilainya diberikan pemanggil (dihitung dengan `MOD-02`), sehingga MOD-05 tetap hanya bergantung pada `MOD-01`.
- Penyimpanan blok cabang samping dan pemilihan chain (`MOD-04`/`MOD-11`, TSK-F04).
- Membuang undo data. Pruning (TSK-M11) hanya boleh membuang badan blok lama, karena reorg tidak dibatasi kedalamannya.

## Task terkait
- [TSK-M10](../tasks/TSK-M10-storage-memory.md) — 🟡 siap (trait ChainStore dan MemoryStore)
- [TSK-M11](../tasks/README.md) — ⚪ penyimpanan redb, kartu menyusul
