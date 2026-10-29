# TSK-M10 · Penyimpanan in-memory: trait ChainStore dan MemoryStore di MOD-05 · 🟡 · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-05 (crates/storage) · **Untuk:** FTR-01 sampai FTR-04
- **Bergantung pada:** TSK-M03, TSK-M22, DEC-03 · **Memblokir:** TSK-M11, TSK-F01
- **Estimasi:** M

## Tujuan
Menyediakan antarmuka penyimpanan chain (`ChainStore`) dan implementasi acuan di memori (`MemoryStore`):
ujung chain, ID blok per tinggi, blok, set UTXO, dan undo data per blok. Dengan undo data, reorg sedalam apa pun
bisa dibatalkan blok demi blok, karena protokol tidak punya batas reorg maupun checkpoint.
`MemoryStore` menjadi pembanding untuk `RedbStore` di TSK-M11 (DEC-03).

## Konteks
- Storage TIDAK memvalidasi aturan konsensus (tanda tangan, jumlah, biaya, kematangan coinbase, PoW, waktu).
  Pemanggil (MOD-04/MOD-11) memvalidasi blok lebih dulu. Storage hanya menjaga integritas datanya sendiri.
- Storage TIDAK menghitung hash. `block_id` dan daftar txid diberikan pemanggil (dihitung dengan MOD-02),
  sehingga MOD-05 tetap hanya bergantung pada MOD-01.
- Tipe yang dipakai dari MOD-01: `Block`, `Header`, `Transaction`, `TxBody`, `Input`, `Output`, `Hash32`,
  `BlockHeight`, `OutPoint`, `UtxoEntry` (TSK-M22). MOD-01 tidak boleh diubah.
- Determinisme: hanya `BTreeMap`, `BTreeSet`, dan `Vec`. Dilarang `HashMap` dan `HashSet`.
- Lint konsensus aktif di crate ini, termasuk di file test. Di test pun dilarang operator `+ - * / %` pada
  bilangan bulat serta indexing/slicing. Pakai metode `checked_*` atau `wrapping_*`, `.get()`, dan iterator.

## Ruang lingkup
- Boleh diubah: `crates/storage/**`; `Cargo.lock` (hanya karena dependensi homeostasis-storage bertambah);
  serta teks status di bagian "Status yang ditulis saat task selesai".
- `crates/storage/Cargo.toml`: di `[dependencies]` tambahkan tepat satu baris `thiserror.workspace = true`.
  Tambahkan bagian baru `[dev-dependencies]` berisi `proptest.workspace = true`. Tanpa crate pihak ketiga baru.
  redb BELUM ditambahkan di task ini (TSK-M11).
- Jangan diubah: crate lain (termasuk crates/primitives), Cargo.toml root, AGENTS.md, docs/system-map.md.
- Di luar lingkup: RedbStore dan pruning (TSK-M11); penyimpanan blok cabang samping dan pemilihan chain
  (MOD-04/MOD-11, TSK-F04); validasi apa pun.

## Struktur file
- `src/lib.rs`: deklarasi modul dan re-export
- `src/store/mod.rs`: trait `ChainStore`, `BlockUndo`, `StoreError` (tanpa logika, jadi tanpa file test)
- `src/memory/mod.rs` dan `src/memory/tests.rs`: `MemoryStore`
- `src/tests.rs`: file lama, tetap ada (berisi `module_id_matches`)

## Persyaratan
1. `src/lib.rs`: tiga baris pertama (doc comment lama) diganti dengan tepat satu baris:
   `//! MOD-05: Penyimpanan blok, set UTXO, dan undo data. Berisi trait ChainStore dan MemoryStore (TSK-M10); RedbStore menyusul (TSK-M11, DEC-03).`
   Setelah blok `#![deny(...)]`, tambahkan tepat:
   ```rust
   pub mod memory;
   pub mod store;

   pub use memory::MemoryStore;
   pub use store::{BlockUndo, ChainStore, StoreError};
   ```
   Baris MODULE_ID dan `#[cfg(test)] mod tests;` tetap ada.
2. `src/store/mod.rs`:
   ```rust
   #[derive(Clone, Debug, PartialEq, Eq)]
   pub struct BlockUndo { spent: Vec<(OutPoint, UtxoEntry)>, created: Vec<OutPoint> }
   // pub fn new(spent: Vec<(OutPoint, UtxoEntry)>, created: Vec<OutPoint>) -> Self
   // pub fn spent(&self) -> &[(OutPoint, UtxoEntry)]
   // pub fn created(&self) -> &[OutPoint]

   pub trait ChainStore {
       fn tip(&self) -> Result<Option<(BlockHeight, Hash32)>, StoreError>;
       fn block_id_at(&self, height: BlockHeight) -> Result<Option<Hash32>, StoreError>;
       fn block(&self, block_id: &Hash32) -> Result<Option<Block>, StoreError>;
       fn utxo(&self, outpoint: &OutPoint) -> Result<Option<UtxoEntry>, StoreError>;
       fn undo_data(&self, block_id: &Hash32) -> Result<Option<BlockUndo>, StoreError>;
       fn apply_block(&mut self, block: &Block, block_id: Hash32, txids: &[Hash32]) -> Result<(), StoreError>;
       fn undo_tip(&mut self) -> Result<(Hash32, Block), StoreError>;
   }
   ```
   - `spent`: entri UTXO yang ada SEBELUM blok dan dihabiskan blok itu, sesuai urutan dihabiskan.
   - `created`: outpoint yang dibuat blok itu dan masih ada setelah blok selesai, sesuai urutan dibuat.
     Output yang dibuat lalu dihabiskan lagi di blok yang sama tidak masuk `spent` maupun `created`.
   - Fungsi baca mengembalikan `Result` supaya RedbStore (TSK-M11) bisa melaporkan error IO tanpa mengubah trait.
   - `block` dan `undo_data` hanya berlaku untuk blok di chain aktif.
   - `StoreError` memakai thiserror, derive `Clone, Debug, PartialEq, Eq`, dengan varian dan pesan persis:
     | Varian | Pesan |
     |---|---|
     | `TxidCountMismatch { txs: usize, txids: usize }` | `jumlah txid ({txids}) tidak sama dengan jumlah transaksi ({txs})` |
     | `UnexpectedHeight { expected: u64, actual: u64 }` | `tinggi blok {actual} tidak menyambung, seharusnya {expected}` |
     | `PrevMismatch { expected: Hash32, actual: Hash32 }` | `prev_block_id tidak sama dengan ID blok ujung chain` |
     | `MissingUtxo { outpoint: OutPoint }` | `UTXO yang dihabiskan tidak ditemukan` |
     | `DuplicateOutpoint { outpoint: OutPoint }` | `outpoint baru sudah ada` |
     | `OutputIndexOverflow` | `indeks output melebihi u32` |
     | `HeightOverflow` | `tinggi blok melebihi u64` |
     | `EmptyChain` | `chain kosong, tidak ada blok untuk di-undo` |
     | `CorruptUndo` | `undo data blok ujung rusak atau tidak ada` |
   - Semua item publik, varian, dan field varian punya doc comment.
3. `src/memory/mod.rs`:
   ```rust
   #[derive(Clone, Debug, Default, PartialEq, Eq)]
   pub struct MemoryStore {
       utxos: BTreeMap<OutPoint, UtxoEntry>,
       blocks: BTreeMap<Hash32, Block>,
       active: Vec<Hash32>,              // active ke-h = block_id di tinggi h
       undo: BTreeMap<Hash32, BlockUndo>,
   }
   impl ChainStore for MemoryStore { ... }
   ```
   `apply_block(block, block_id, txids)`. Semua pemeriksaan dilakukan sebelum ada perubahan. Jika ada error,
   isi store tetap persis sama. Dilarang meng-clone seluruh set UTXO.
   1. Jika `txids.len() != block.txs().len()`: `TxidCountMismatch`.
   2. Store kosong: tinggi header wajib 0, jika tidak `UnexpectedHeight { expected: 0, .. }` (prev tidak dicek).
      Store tidak kosong: tinggi wajib tinggi ujung + 1 (`checked_add`, jika gagal `HeightOverflow`), jika tidak
      `UnexpectedHeight`; lalu `prev_block_id` header wajib sama dengan block_id ujung, jika tidak `PrevMismatch`.
   3. Overlay lokal: `created: BTreeMap<OutPoint, UtxoEntry>`, `created_order: Vec<OutPoint>`,
      `created_all: BTreeSet<OutPoint>`, `spent: Vec<(OutPoint, UtxoEntry)>`, `spent_set: BTreeSet<OutPoint>`.
      Untuk setiap pasangan (transaksi, txid) sesuai urutan:
      a. Jika transaksi biasa (`TxBody::Regular`), untuk setiap input, `op = OutPoint::from_input(input)`:
         - jika `created.remove(&op)` berisi nilai: hapus juga `op` dari `created_order` (dihabiskan di blok sama);
         - selain itu, jika `op` tidak ada di `spent_set` dan ada di `utxos`: masukkan `(op, entri.clone())` ke
           `spent` dan `op` ke `spent_set`;
         - selain itu: `MissingUtxo { outpoint: op }`.
      b. Untuk setiap output ke-j (`u32::try_from(j)`, jika gagal `OutputIndexOverflow`), `op = OutPoint::new(txid, j)`:
         jika `op` ada di `utxos` atau di `created_all`: `DuplicateOutpoint { outpoint: op }`. Jika tidak, masukkan
         `UtxoEntry::new(output.clone(), tinggi header, transaksi.is_coinbase())` ke `created`, dan `op` ke
         `created_order` dan `created_all`.
   4. Penerapan (tanpa langkah yang bisa gagal): hapus semua outpoint `spent` dari `utxos`; masukkan seluruh isi
      `created` ke `utxos`; `blocks.insert(block_id, block.clone())`; `active.push(block_id)`;
      `undo.insert(block_id, BlockUndo::new(spent, created_order))`.
   `undo_tip()`:
   1. Jika `active` kosong: `EmptyChain`.
   2. Ambil (clone) undo data dan blok milik block_id ujung. Jika salah satunya tidak ada: `CorruptUndo`.
   3. Periksa semua outpoint `created` ada di `utxos` dan semua outpoint `spent` tidak ada di `utxos`. Jika tidak:
      `CorruptUndo`, tanpa perubahan.
   4. Penerapan: hapus semua `created` dari `utxos`; masukkan kembali semua `spent`; `active.pop()`; hapus entri
      `undo` dan `blocks` milik block_id itu. Kembalikan `(block_id, blok)`.
   Setelah `apply_block` lalu `undo_tip` yang berhasil, isi `MemoryStore` sama persis dengan sebelum `apply_block`.
4. Tanpa unwrap, expect, panic, atau indexing yang bisa panic di kode non-test.

## Bahan test (dipakai di src/memory/tests.rs)
- `fn h(byte: u8) -> Hash32 { Hash32::from_bytes([byte; 32]) }`
- Output: `Output::new(Amount::from_quanta(q), OutputKind::SingleKey, [p; 32]).expect("output")`.
- Header blok tinggi n: `Header::new(BlockHeight::new(n), prev, Hash32::ZERO, 1_800_000_000, 0x1d00_ffff, 0)`.
- Coinbase: `Transaction::coinbase(BlockHeight::new(n), extra, outputs)`. Extra = `vec![0_u8; 32]` untuk tinggi 0
  (wajib 32 byte), dan `vec![]` untuk tinggi lain.
- Transaksi biasa: `Transaction::regular(inputs, outputs, witnesses)` dengan satu `Witness::single_key([0_u8; 64])`
  per input.
- Genesis G: tinggi 0, prev `Hash32::ZERO`, satu coinbase (txid `h(0x01)`) dengan dua output: 10 quanta payload 0x01
  dan 20 quanta payload 0x02. block_id `h(0xb0)`. Entri E0 dan E1 = output itu, tinggi 0, coinbase true.
- Blok 1: tinggi 1, prev `h(0xb0)`, block_id `h(0xb1)`.

## Daftar test (nama persis; test homeostasis-storage menjadi tepat 12)
`src/memory/tests.rs`:
1. `empty_store_has_no_tip_no_utxo_and_cannot_undo`: `tip()` = `Ok(None)`; `utxo(&OutPoint::new(h(0x01), 0))` =
   `Ok(None)`; `undo_tip()` = `Err(StoreError::EmptyChain)`.
2. `apply_genesis_creates_coinbase_utxos_at_height_zero`: apply G dengan txids `[h(0x01)]`. `tip()` =
   `Some((BlockHeight::new(0), h(0xb0)))`; `block_id_at(0)` = `Some(h(0xb0))`; `utxo` (h(0x01),0) = E0 dan
   (h(0x01),1) = E1; (h(0x01),2) = None; `block(h(0xb0))` = G; `undo_data(h(0xb0))` =
   `BlockUndo::new(vec![], vec![(h(0x01),0), (h(0x01),1)])`.
3. `apply_rejects_txid_count_mismatch_without_changes`: setelah G, blok 1 berisi coinbase saja. txids `[]` →
   `TxidCountMismatch { txs: 1, txids: 0 }`; txids `[h(0x02), h(0x03)]` → `TxidCountMismatch { txs: 1, txids: 2 }`.
   Store sama dengan sebelum percobaan (bandingkan dengan clone).
4. `apply_rejects_unconnected_block_without_changes`: (a) store kosong, blok tinggi 1 →
   `UnexpectedHeight { expected: 0, actual: 1 }`; (b) setelah G, blok tinggi 2 →
   `UnexpectedHeight { expected: 1, actual: 2 }`; (c) setelah G, blok tinggi 1 dengan prev `h(0xee)` →
   `PrevMismatch { expected: h(0xb0), actual: h(0xee) }`. Setiap kasus: store tidak berubah.
5. `apply_spend_then_undo_restores_previous_state_exactly`: setelah G, simpan clone S0. Blok 1: coinbase txid
   `h(0x02)` (satu output 30 quanta payload 0x03), lalu transaksi txid `h(0x03)` yang menghabiskan (h(0x01),0) dan
   membuat dua output (4 dan 5 quanta, payload 0x04). Setelah apply: (h(0x01),0) = None; (h(0x01),1) = E1;
   (h(0x02),0) tinggi 1 coinbase true; (h(0x03),0) dan (h(0x03),1) tinggi 1 coinbase false;
   `undo_data(h(0xb1))` = `BlockUndo::new(vec![((h(0x01),0), E0)], vec![(h(0x02),0), (h(0x03),0), (h(0x03),1)])`.
   `undo_tip()` = `Ok((h(0xb1), blok 1))` dan store == S0. `undo_tip()` lagi = `Ok((h(0xb0), G))` dan store ==
   `MemoryStore::default()`. `undo_tip()` sekali lagi = `Err(EmptyChain)`.
6. `apply_rejects_missing_utxo_and_double_spend_without_changes`: setelah G: (a) blok 1 dengan transaksi yang
   menghabiskan (h(0x99),0) → `MissingUtxo { outpoint: (h(0x99),0) }`; (b) blok 1 dengan dua transaksi (txid
   `h(0x03)` dan `h(0x04)`) yang sama-sama menghabiskan (h(0x01),0) → `MissingUtxo { outpoint: (h(0x01),0) }`.
   Store tidak berubah.
7. `apply_rejects_duplicate_outpoint_without_changes`: setelah G: (a) blok 1 dengan coinbase ber-txid `h(0x01)` →
   `DuplicateOutpoint { outpoint: (h(0x01),0) }`; (b) blok 1 dengan coinbase txid `h(0x02)` dan transaksi biasa
   yang juga ber-txid `h(0x02)` (menghabiskan (h(0x01),0)) → `DuplicateOutpoint { outpoint: (h(0x02),0) }`.
   Store tidak berubah.
8. `output_spent_in_same_block_is_not_in_undo_data`: setelah G, simpan clone S0. Blok 1: coinbase txid `h(0x02)`;
   transaksi A txid `h(0x03)` menghabiskan (h(0x01),0) dan membuat satu output; transaksi B txid `h(0x04)`
   menghabiskan (h(0x03),0) dan membuat satu output. Setelah apply: (h(0x03),0) = None, (h(0x04),0) ada;
   `undo_data(h(0xb1))` = `BlockUndo::new(vec![((h(0x01),0), E0)], vec![(h(0x02),0), (h(0x04),0)])`.
   Setelah `undo_tip()`, store == S0.
9. `prop_apply_chain_then_undo_all_restores_each_snapshot`: proptest dengan input
   `plan in proptest::collection::vec((1_u8..=3, 0_u8..=2, any::<u64>()), 1..=8)`. Setiap elemen = satu blok:
   (jumlah output coinbase, jumlah transaksi biasa, seed). Blok ke-b (tinggi b) memakai txid
   `tx_hash(b, 0)` untuk coinbase dan `tx_hash(b, t)` untuk transaksi biasa ke-t (t = 1, 2), serta block_id
   `blk_hash(b)`:
   ```rust
   fn tx_hash(block: u8, tx: u8) -> Hash32 {
       let mut bytes = [0_u8; 32];
       for (dst, src) in bytes.iter_mut().zip([0x7a, block, tx]) {
           *dst = src;
       }
       Hash32::from_bytes(bytes)
   }
   fn blk_hash(block: u8) -> Hash32 {
       let mut bytes = [0_u8; 32];
       for (dst, src) in bytes.iter_mut().zip([0x6b, block]) {
           *dst = src;
       }
       Hash32::from_bytes(bytes)
   }
   ```
   Test menyimpan daftar `available: Vec<OutPoint>` (UTXO yang masih ada, menurut model test). Setiap transaksi
   biasa menghabiskan satu outpoint dari `available`, yaitu indeks `seed.wrapping_add(t) % available.len()`
   (dihitung dengan `checked_rem`), lalu membuat satu output. Jika `available` kosong, transaksi itu dilewati.
   Output baru masuk `available` segera setelah transaksinya, sehingga pembelanjaan di blok yang sama ikut teruji.
   Sebelum setiap apply, simpan clone store. Semua apply wajib `Ok`. Setelah semua blok, panggil `undo_tip()` satu
   per satu. Setiap undo wajib `Ok`, dan store wajib sama dengan clone yang disimpan sebelum blok itu diterapkan.
   Di akhir, store == `MemoryStore::default()`.
10. `prop_failed_apply_leaves_state_unchanged`: proptest dengan input `n in 1_u8..=4, fault in 0_u8..=4`. Terapkan
    n blok berisi coinbase saja (blok ke-b: txid `tx_hash(b, 0)`, block_id `blk_hash(b)`, satu output). Simpan
    clone. Buat blok tinggi n dengan satu cacat sesuai `fault`: 0 = txids kosong; 1 = tinggi header n+1;
    2 = prev `h(0xee)`; 3 = tambahan transaksi biasa yang menghabiskan (h(0x99),0); 4 = txid coinbase
    `tx_hash(0, 0)`. `apply_block` wajib `Err`, dan store wajib sama dengan clone.
`src/tests.rs`:
11. `module_id_matches` (lama, tidak diubah)
12. `crate_root_reexports_are_accessible`: memakai `crate::{BlockUndo, ChainStore, MemoryStore, StoreError}`;
    `MemoryStore::default().tip()` = `Ok(None)`; `BlockUndo::new(vec![], vec![]).spent()` kosong;
    `StoreError::EmptyChain.to_string()` = `chain kosong, tidak ada blok untuk di-undo`.

## Status yang ditulis saat task selesai (teks persis, di commit yang sama)
- docs/tasks/README.md, baris pertama: ganti seluruh baris menjadi
  `# Task kerja: TSK-M11 dan TSK-S02 berikutnya, jalur kritis lewat spesifikasi validasi (TSK-S02)`.
- docs/tasks/README.md, baris yang diawali `| TSK-M10 | MOD-05 |`: kolom terakhir
  `🟡 siap ([kartu](TSK-M10-storage-memory.md))` diganti menjadi `🟢 selesai ([kartu](TSK-M10-storage-memory.md))`.
- docs/tasks/README.md, baris ringkasan `🟢 Selesai`: sisipkan `, TSK-M10 (penyimpanan in-memory)` tepat setelah
  `TSK-M22 (OutPoint dan UtxoEntry)`.
- docs/tasks/README.md, baris ringkasan `🟡 Siap dikerjakan`: ganti seluruh kolom kedua menjadi
  `TSK-M11 (penyimpanan redb) kartunya menyusul`.
- docs/tasks/README.md, baris ringkasan `➡️ Langkah berikutnya`: ganti seluruh kolom kedua menjadi
  `Kartu TSK-M11 (pencipta + Quick); TSK-S02 (pencipta + Quick)`.
- docs/modules/MOD-05-storage.md, baris yang diawali `- **Status:**`: ganti seluruh baris menjadi
  `- **Status:** 🟡 sedang berjalan (TSK-M10 selesai; sisa: penyimpanan redb TSK-M11)`.
- docs/modules/MOD-05-storage.md, baris yang diawali `- [TSK-M10]`: ganti seluruh baris menjadi
  `- [TSK-M10](../tasks/TSK-M10-storage-memory.md) — 🟢 selesai (trait ChainStore dan MemoryStore)`.
- Baris judul kartu ini: ganti `· 🟡 ·` menjadi `· 🟢 selesai ·`.
- docs/system-map.md dan kartu modul lain TIDAK diubah.

## Kriteria penerimaan
- [ ] 12 test homeostasis-storage lolos dengan nama persis seperti daftar di atas
- [ ] Cargo.toml crate storage persis seperti di Ruang lingkup; tidak ada crate pihak ketiga baru di Cargo.lock
- [ ] Tanpa HashMap/HashSet; tanpa unwrap/expect/panic di kode non-test; lint konsensus tetap aktif
- [ ] `make check-all` lolos tanpa warning
- [ ] Status diperbarui persis seperti bagian di atas
- [ ] Satu commit: `feat(storage): trait ChainStore dan MemoryStore dengan undo data (TSK-M10)`

## Laporan akhir
OUTPUT MENTAH perintah berikut, bernomor, satu blok kode per perintah, tidak diringkas dan tidak dari ingatan.
Setelah L11, boleh ditambah paling banyak 5 butir asumsi, risiko, atau pertanyaan terbuka.
- L1: `git log --oneline -3`
- L2: `git status --short`
- L3: `git show --stat HEAD`
- L4: `cat crates/storage/Cargo.toml`
- L5: `git diff HEAD~1 -- Cargo.lock`
- L6: `cat crates/storage/src/lib.rs crates/storage/src/store/mod.rs crates/storage/src/memory/mod.rs`
- L7: `cat crates/storage/src/memory/tests.rs crates/storage/src/tests.rs`
- L8: `cargo test -p homeostasis-storage 2>&1 | grep -E "^test |test result"`
- L9: `make check-all > /tmp/m10-check.log 2>&1; echo "exit=$?"; grep -cE "^test .* ok$" /tmp/m10-check.log; grep -E "^(warning|error)|FAILED" /tmp/m10-check.log || echo "(tidak ada warning/error)"`
- L10: `grep -rnE "unwrap\(|expect\(|panic!|HashMap|HashSet" crates/storage/src/lib.rs crates/storage/src/store/mod.rs crates/storage/src/memory/mod.rs || echo "(tidak ada)"`
- L11: `git show HEAD -- docs/`
