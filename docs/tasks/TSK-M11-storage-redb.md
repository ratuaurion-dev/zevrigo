# TSK-M11 · Penyimpanan persisten RedbStore (redb 4.4.0) di MOD-05 · 🟡 · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-05 (crates/storage) · **Untuk:** FTR-01 sampai FTR-05
- **Bergantung pada:** TSK-M10, DEC-03, DEC-10 bagian 3 · **Memblokir:** TSK-F01, TSK-F05, TSK-M23
- **Estimasi:** M

## Tujuan
Membuat `RedbStore`, yaitu implementasi `ChainStore` (TSK-M10) yang menyimpan data di file redb, dan membuktikannya
dengan uji diferensial: urutan operasi acak yang sama dijalankan di `MemoryStore` dan `RedbStore`, lalu hasil dan
isi kedua store wajib sama persis. Pruning TIDAK termasuk task ini (dipindah ke TSK-M23).

## Konteks
- DEC-03: redb 4.4.0, `default-features = false` ditambah fitur `std`. DEC-10 bagian 3: redb diizinkan khusus MOD-05.
  Fakta dari pemeriksaan 29 Okt (Q46): dependensi normal hanya `libc`; ada `unsafe` di dalam redb (54 baris, 6 file),
  sudah dilaporkan di DEC-10. Kode kita tetap tanpa `unsafe`.
- API redb yang dipakai harus sesuai kode redb 4.4.0 di `~/.cargo/registry`, bukan ingatan. Yang sudah terlihat di
  Q46: `Database::create`, `Database::open`, `Database::builder`/`Builder::new`, `Builder::create_with_backend`,
  `Database::begin_write`, `begin_read` (dari trait `ReadableDatabase`), `WriteTransaction::open_table`, `commit`,
  `ReadTransaction::open_table`, `TableDefinition::new`, `Table::insert`/`remove`, trait `ReadableTable`,
  `AccessGuard::value`, `InMemoryBackend`. Key/Value bawaan: `u64`, `&str`, `&[u8]`, `&[u8; N]`.
- `WriteTransaction` yang tidak di-commit dibatalkan oleh redb (drop atau `abort`). Jadi error di mana pun sebelum
  `commit` berarti tidak ada perubahan. Uji diferensial membuktikan perilaku ini.
- Storage TIDAK memvalidasi aturan konsensus dan TIDAK menghitung hash (sama seperti TSK-M10).
- Lint konsensus aktif di crate ini, termasuk di file test: dilarang `+ - * / %` pada bilangan bulat dan
  indexing/slicing. Pakai `checked_*`/`wrapping_*`, `.get()`, iterator. Dilarang `HashMap`/`HashSet`.

## Ruang lingkup
- Boleh diubah: `crates/storage/**`; `Cargo.toml` root (tepat satu baris, lihat di bawah); `Cargo.lock`;
  serta teks status di bagian "Status yang ditulis saat task selesai".
- `Cargo.toml` root, di `[workspace.dependencies]`, tepat setelah baris `k256 = ...`, tambahkan tepat:
  `redb = { version = "4.4.0", default-features = false, features = ["std"] }`
- `crates/storage/Cargo.toml`, di `[dependencies]`, tambahkan tepat satu baris `redb.workspace = true`
  (urutan: homeostasis-primitives, redb, thiserror). Tidak ada dependensi lain, termasuk dev-dependency.
- `Cargo.lock`: blok `[[package]]` baru hanya `redb`, ditambah `libc` jika sebelumnya belum ada.
- Jangan diubah: crate lain, AGENTS.md, docs/system-map.md, `src/memory/**` (MemoryStore dan test-nya tetap persis).
- Di luar lingkup: pruning (TSK-M23), blok cabang samping, validasi, migrasi skema.

## Struktur file
- `src/lib.rs`: deklarasi modul dan re-export
- `src/store/mod.rs`: tambah 3 varian `StoreError` (lihat Persyaratan 2); isi lain tidak diubah
- `src/codec/mod.rs` dan `src/codec/tests.rs`: encoding byte khusus penyimpanan (bukan konsensus)
- `src/redb_store/mod.rs` dan `src/redb_store/tests.rs`: `RedbStore`
- `src/tests.rs`: tambah satu test
Nama modul `redb_store` (bukan `redb`) supaya tidak bentrok dengan nama crate `redb`.

## Persyaratan
1. `src/lib.rs`: baris pertama diganti dengan tepat:
   `//! MOD-05: Penyimpanan blok, set UTXO, dan undo data. Berisi trait ChainStore, MemoryStore (TSK-M10), dan RedbStore berbasis redb (TSK-M11, DEC-03).`
   Blok deklarasi modul dan re-export diganti menjadi tepat:
   ```rust
   mod codec;
   pub mod memory;
   pub mod redb_store;
   pub mod store;

   pub use memory::MemoryStore;
   pub use redb_store::RedbStore;
   pub use store::{BlockUndo, ChainStore, StoreError};
   ```
2. `src/store/mod.rs`: tambahkan tiga varian di akhir enum `StoreError` (setelah `CorruptUndo`), dengan doc comment
   untuk varian dan field:
   | Varian | Pesan |
   |---|---|
   | `Backend { message: String }` | `kesalahan backend penyimpanan: {message}` |
   | `CorruptData` | `data penyimpanan rusak atau melebihi batas format` |
   | `UnsupportedSchema { found: u64 }` | `versi skema penyimpanan {found} tidak didukung` |
3. `src/codec/mod.rs` (semua `pub(crate)`):
   - `fn outpoint_key(op: &OutPoint) -> [u8; 36]`: 32 byte txid, lalu index sebagai u32 **big-endian**.
     Urutan byte key sama dengan urutan `Ord` OutPoint.
   - `fn encode_utxo_entry(entry: &UtxoEntry) -> Vec<u8>`: height u64 little-endian (8 byte), lalu satu byte
     is_coinbase (0 atau 1), lalu encoding kanonis `Output` dari MOD-01 (`Encode::encode_to`).
   - `fn decode_utxo_entry(bytes: &[u8]) -> Result<UtxoEntry, StoreError>`: kebalikannya, memakai `Reader`
     (`read_u64_le`, `read_u8`, `Output::decode_from`, lalu `finish`). Byte is_coinbase selain 0/1, byte sisa, atau
     data kurang: `CorruptData`.
   - `fn encode_block_undo(undo: &BlockUndo) -> Result<Vec<u8>, StoreError>`: jumlah spent u32 LE; tiap spent:
     36 byte `outpoint_key` lalu `encode_utxo_entry`; jumlah created u32 LE; tiap created: 36 byte `outpoint_key`.
     Jumlah yang tidak muat u32: `CorruptData`.
   - `fn decode_block_undo(bytes: &[u8]) -> Result<BlockUndo, StoreError>`: kebalikannya, lalu `finish`. Outpoint
     dibaca dengan `read_bytes::<32>()` dan `read_bytes::<4>()` (u32 big-endian). Entri UTXO di dalam undo dibaca
     langsung dari `Reader` yang sama (fungsi bantu internal yang memakai `&mut Reader`). Jangan memakai
     `Vec::with_capacity` dari angka jumlah yang dibaca. Error apa pun: `CorruptData`.
4. `src/redb_store/mod.rs`:
   - Tabel (const, nama string persis):
     ```rust
     const META: TableDefinition<&str, u64> = TableDefinition::new("meta");
     const UTXOS: TableDefinition<&[u8; 36], &[u8]> = TableDefinition::new("utxos");
     const BLOCKS: TableDefinition<&[u8; 32], &[u8]> = TableDefinition::new("blocks");
     const ACTIVE: TableDefinition<u64, &[u8; 32]> = TableDefinition::new("active");
     const UNDO: TableDefinition<&[u8; 32], &[u8]> = TableDefinition::new("undo");
     const SCHEMA_KEY: &str = "schema";
     const SCHEMA_VERSION: u64 = 1;
     ```
     UTXOS: key `outpoint_key`, value `encode_utxo_entry`. BLOCKS: key block_id, value encoding kanonis blok MOD-01.
     ACTIVE: key tinggi, value block_id. UNDO: key block_id, value `encode_block_undo`.
   - `pub struct RedbStore { db: redb::Database }`, dengan konstruktor:
     - `pub fn create(path: impl AsRef<Path>) -> Result<Self, StoreError>` memakai `Database::create`
     - `pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError>` memakai `Database::open`
     - `pub fn in_memory() -> Result<Self, StoreError>` memakai `InMemoryBackend` lewat `Builder::create_with_backend`
     Ketiganya lalu menjalankan satu write transaction inisialisasi: buka kelima tabel (supaya tabel ada); baca
     META[SCHEMA_KEY]. Jika tidak ada, tulis SCHEMA_VERSION. Jika ada dan bukan SCHEMA_VERSION, kembalikan
     `UnsupportedSchema { found }` tanpa commit. Lalu commit.
   - Semua error redb diubah menjadi `StoreError::Backend { message: e.to_string() }` lewat satu fungsi bantu.
     Decode blok (`Block::decode_from` lalu `finish`), UTXO, atau undo yang gagal menjadi `CorruptData`.
   - `impl ChainStore for RedbStore`:
     - Fungsi baca memakai satu read transaction. `tip` = entri terakhir ACTIVE (tinggi, block_id), atau `None`.
     - `apply_block`: satu write transaction. Urutan pemeriksaan dan error **sama persis** dengan `MemoryStore`
       (Persyaratan 3 kartu TSK-M10, langkah 1–3, termasuk overlay `created`/`created_order`/`created_all`/`spent`/
       `spent_set`). UTXO yang sudah ada dibaca dari tabel UTXOS di transaksi yang sama. Jika semua lolos: hapus
       spent dari UTXOS, masukkan created, tulis BLOCKS, ACTIVE, dan UNDO, lalu commit.
     - `undo_tip`: satu write transaction dengan urutan dan error sama persis dengan `MemoryStore::undo_tip`
       (EmptyChain, CorruptUndo, cek created ada dan spent tidak ada), lalu kembalikan UTXO, hapus ACTIVE[tinggi],
       UNDO[id], BLOCKS[id], commit, dan kembalikan `(block_id, blok)`.
   - Semua item publik punya doc comment. Tanpa unwrap/expect/panic/indexing di kode non-test.

## Vektor encoding (dihitung Quick)
Bahan: `E = UtxoEntry::new(Output::new(Amount::from_quanta(10), OutputKind::SingleKey, [0x01; 32]), BlockHeight::new(5), true)`.
- K1 `outpoint_key(&OutPoint::new(h(0x11), 0x0102_0304))` = `111111111111111111111111111111111111111111111111111111111111111101020304`
- U1 `encode_utxo_entry(&E)` (43 byte) = `0500000000000000010a000101010101010101010101010101010101010101010101010101010101010101`
- B1 `encode_block_undo(&BlockUndo::new(vec![(OutPoint::new(h(0x11), 1), E)], vec![OutPoint::new(h(0x22), 0)]))` (123 byte) =
  `010000001111111111111111111111111111111111111111111111111111111111111111000000010500000000000000010a00010101010101010101010101010101010101010101010101010101010101010101000000222222222222222222222222222222222222222222222222222222222222222200000000`
Data rusak (semua wajib `Err(StoreError::CorruptData)`):
- C1 `decode_utxo_entry` dari `` (kosong)
- C2 `decode_utxo_entry` dari `0500000000000000020a000101010101010101010101010101010101010101010101010101010101010101` (byte coinbase = 02)
- C3 `decode_utxo_entry` dari U1 ditambah `00`
- C4 `decode_utxo_entry` dari U1 tanpa byte terakhir (42 byte)
- C5 `decode_block_undo` dari B1 dengan 4 byte pertama `02000000` (jumlah spent 2, isi hanya 1)

## Daftar test (nama persis; test homeostasis-storage menjadi tepat 23)
Helper test yang wajib dipakai, ditulis persis di file test yang membutuhkannya:
```rust
fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn from_hex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks(2)
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).expect("utf8"), 16).expect("hex"))
        .collect()
}
```
Di `src/redb_store/tests.rs`, salin persis (byte demi byte) helper `h`, `tx_hash`, `blk_hash`, `make_output`,
`make_header`, `make_coinbase`, `make_regular`, dan `make_genesis` dari `src/memory/tests.rs`. File itu sendiri
tidak diubah. Di `src/codec/tests.rs`, salin `h` saja.

`src/codec/tests.rs`:
1. `vectors_outpoint_key_utxo_entry_and_block_undo`: K1, U1, dan B1 dibandingkan lewat `to_hex`; lalu
   `decode_utxo_entry(&from_hex(U1))` = `Ok(E)` dan `decode_block_undo(&from_hex(B1))` = BlockUndo yang sama.
2. `prop_outpoint_key_order_matches_outpoint_ord`: input `ta in any::<[u8; 32]>(), ia in any::<u32>(),
   tb in any::<[u8; 32]>(), ib in any::<u32>()`. Assert:
   `prop_assert_eq!(outpoint_key(&a).cmp(&outpoint_key(&b)), a.cmp(&b));` dengan a, b = OutPoint dari input.
3. `prop_utxo_entry_round_trip`: input `q in 1_u128..=u128::MAX, cond in any::<bool>(), p in any::<[u8; 32]>(),
   height in any::<u64>(), cb in any::<bool>()`. Kind = `OutputKind::ConditionHash` jika cond, selain itu
   `OutputKind::SingleKey`. `decode_utxo_entry(&encode_utxo_entry(&e))` = `Ok(e)`.
4. `prop_block_undo_round_trip`: input `spent in proptest::collection::vec((any::<[u8; 32]>(), any::<u32>(),
   1_u128..=1_000_000, any::<u64>(), any::<bool>()), 0..=8)` dan `created in proptest::collection::vec(
   (any::<[u8; 32]>(), any::<u32>()), 0..=8)`. Output dengan kind SingleKey dan payload `[0x01; 32]`.
   Round-trip encode lalu decode = `Ok(undo)`.
5. `decode_rejects_corrupt_bytes`: C1 sampai C5.
`src/redb_store/tests.rs`:
6. `in_memory_store_starts_empty`: `RedbStore::in_memory()` berhasil; `tip()` = `Ok(None)`; `utxo` (h(0x01),0) =
   `Ok(None)`; `undo_tip()` = `Err(StoreError::EmptyChain)`.
7. `scripted_chain_matches_memory_store_step_by_step`: skenario test 5 dan test 8 kartu TSK-M10 (G, lalu blok 1
   berisi coinbase txid h(0x02), transaksi A txid h(0x03) yang menghabiskan (h(0x01),0), dan transaksi B txid
   h(0x04) yang menghabiskan (h(0x03),0)) dijalankan di MemoryStore dan RedbStore::in_memory(). Hasil setiap
   apply/undo wajib sama, dan `assert_same_state` dipanggil setelah setiap langkah, termasuk setelah undo kedua dan
   undo ketiga (`Err(EmptyChain)`).
8. `reopen_file_preserves_state`: path `std::env::temp_dir().join("homeostasis-tsk-m11-reopen.redb")`; hapus file
   lama jika ada (`let _ = std::fs::remove_file(&path);`). `create` → apply G dan blok 1 dari test 7 → drop.
   `open` → `tip()` = `Some((BlockHeight::new(1), h(0xb1)))`, UTXO (h(0x04),0) ada, (h(0x01),0) tidak ada, dan
   `undo_data(h(0xb1))` sama dengan hasil MemoryStore → `undo_tip()` = `Ok((h(0xb1), blok 1))` → drop. `open` lagi →
   `tip()` = `Some((BlockHeight::new(0), h(0xb0)))` → drop → hapus file.
9. `open_rejects_unknown_schema_version`: path `homeostasis-tsk-m11-schema.redb` di temp_dir, hapus file lama.
   `create` → drop. Buka langsung dengan `redb::Database::open`, tulis META[SCHEMA_KEY] = 2, commit, drop.
   `RedbStore::open` = `Err(StoreError::UnsupportedSchema { found: 2 })`. Hapus file.
10. `prop_differential_against_memory_store`: `#![proptest_config(ProptestConfig::with_cases(64))]`. Input
    `ops in proptest::collection::vec((0_u8..=2, 1_u8..=3, 0_u8..=2, any::<u64>()), 1..=24)`, tiap elemen =
    (op, jumlah output coinbase, jumlah transaksi biasa, seed). Model test: `available: Vec<OutPoint>`,
    `history: Vec<Vec<OutPoint>>` (isi `available` sebelum tiap blok yang berhasil diterapkan), dan `seen:
    Vec<OutPoint>` (semua outpoint yang pernah dibuat, ditambah (h(0x99),0)). Tinggi berikutnya `n =
    history.len()`, `b = u8::try_from(n)`, prev = `blk_hash(b − 1)` atau `Hash32::ZERO` untuk n = 0.
    - op 0 (blok sah): dibangun persis seperti test 9 kartu TSK-M10 (coinbase `tx_hash(b,0)`, transaksi biasa ke-t
      `tx_hash(b,t)` menghabiskan `available` indeks `seed.wrapping_add(t) % len` dengan `checked_rem`, output baru
      langsung masuk `available`), block_id `blk_hash(b)`. Simpan `available` lama ke `history` sebelum mengubahnya.
      Kedua apply wajib `Ok`.
    - op 1 (blok cacat), `fault = seed % 4` (checked_rem): 0 = blok sah dengan txids kosong; 1 = header dan
      coinbase bertinggi n+1; 2 = prev `h(0xee)` (jika n = 0, pakai fault 1); 3 = blok sah ditambah transaksi biasa
      txid `tx_hash(b,1)` yang menghabiskan (h(0x99),0). Kedua hasil wajib `Err` dan sama. Model tidak berubah.
    - op 2 (undo): kedua hasil wajib sama. Jika `Ok`, `available = history.pop()`.
    Setelah setiap op: `assert_same_state`. Di akhir: undo berulang di kedua store sampai keduanya
    `Err(EmptyChain)`, dengan hasil sama dan `assert_same_state` di setiap langkah.
    `assert_same_state(mem, rdb, outpoints, max_height)` (fungsi bantu test): `tip` sama; `block_id_at` sama untuk
    tinggi 0..=max_height; `utxo` sama untuk setiap outpoint; `block` dan `undo_data` sama untuk `blk_hash(0)` sampai
    `blk_hash(25)`. Di test 10, max_height = 25.
`src/tests.rs`:
11. `redb_store_reexport_is_accessible`: `crate::RedbStore::in_memory()` berhasil dan `tip()` = `Ok(None)`.
Test lama (12 test TSK-M10) tidak diubah.

## Status yang ditulis saat task selesai (teks persis, di commit yang sama)
- docs/tasks/README.md, baris pertama: ganti seluruh baris menjadi
  `# Task kerja: TSK-S02 dan DEC-13 berikutnya, jalur kritis menuju node regtest (TSK-F01)`.
- docs/tasks/README.md, baris yang diawali `| TSK-M11 | MOD-05 |`: kolom terakhir
  `🟡 siap ([kartu](TSK-M11-storage-redb.md))` diganti menjadi `🟢 selesai ([kartu](TSK-M11-storage-redb.md))`.
- docs/tasks/README.md, baris ringkasan `🟢 Selesai`: sisipkan `, TSK-M11 (penyimpanan redb)` tepat setelah
  `TSK-M10 (penyimpanan in-memory)`.
- docs/tasks/README.md, baris ringkasan `🟡 Siap dikerjakan`: ganti seluruh kolom kedua menjadi
  `— (task modul berikutnya menunggu TSK-S02 dan DEC-13)`.
- docs/tasks/README.md, baris ringkasan `➡️ Langkah berikutnya`: ganti seluruh kolom kedua menjadi
  `TSK-S02 dan DEC-13 (pencipta + Quick)`.
- docs/modules/MOD-05-storage.md, baris yang diawali `- **Status:**`: ganti seluruh baris menjadi
  `- **Status:** 🟡 sedang berjalan (TSK-M10 dan TSK-M11 selesai; sisa: pruning TSK-M23)`.
- docs/modules/MOD-05-storage.md, baris yang diawali `- [TSK-M11]`: ganti seluruh baris menjadi
  `- [TSK-M11](../tasks/TSK-M11-storage-redb.md) — 🟢 selesai (RedbStore dengan redb 4.4.0)`.
- Baris judul kartu ini: ganti `· 🟡 ·` menjadi `· 🟢 selesai ·`.
- docs/system-map.md dan kartu modul lain TIDAK diubah.

## Kriteria penerimaan
- [ ] K1, U1, B1, dan C1–C5 sesuai; 23 test homeostasis-storage lolos dengan nama persis
- [ ] Uji diferensial lolos; tidak ada file `homeostasis-tsk-m11-*` tersisa di temp_dir
- [ ] Cargo.toml root dan crate persis seperti Ruang lingkup; Cargo.lock hanya menambah redb (dan libc jika belum ada)
- [ ] Tanpa HashMap/HashSet; tanpa unwrap/expect/panic di kode non-test; lint konsensus tetap aktif
- [ ] `make check-all` lolos tanpa warning
- [ ] Status diperbarui persis seperti bagian di atas
- [ ] Satu commit: `feat(storage): RedbStore berbasis redb 4.4.0 dengan uji diferensial (TSK-M11)`

## Laporan akhir
OUTPUT MENTAH perintah berikut, bernomor, satu blok kode per perintah, tidak diringkas dan tidak dari ingatan.
Jika kode akhir berbeda dari rencana yang disetujui selain karena rustfmt, sebutkan setiap perbedaannya.
- L1: `git log --oneline -3`
- L2: `git status --short`
- L3: `git show --stat HEAD`
- L4: `git diff HEAD~1 -- Cargo.toml crates/storage/Cargo.toml`
- L5: `git diff HEAD~1 -- Cargo.lock`
- L6: `cargo tree -p homeostasis-storage -e normal`
- L7: `git show HEAD -- crates/storage/src/lib.rs crates/storage/src/store/mod.rs crates/storage/src/tests.rs`
- L8: `cat crates/storage/src/codec/mod.rs crates/storage/src/redb_store/mod.rs`
- L9: `cat crates/storage/src/codec/tests.rs crates/storage/src/redb_store/tests.rs`
- L10: `cargo test -p homeostasis-storage 2>&1 | grep -E "^test |test result"`
- L11: `make check-all > /tmp/m11-check.log 2>&1; echo "exit=$?"; grep -cE "^test .* ok$" /tmp/m11-check.log; grep -E "^(warning|error)|FAILED" /tmp/m11-check.log || echo "(tidak ada warning/error)"`
- L12: `grep -rnE "unwrap\(|expect\(|panic!|HashMap|HashSet|unsafe" crates/storage/src/lib.rs crates/storage/src/store/mod.rs crates/storage/src/codec/mod.rs crates/storage/src/redb_store/mod.rs || echo "(tidak ada)"`
- L13: `ls "${TMPDIR:-/tmp}"/homeostasis-tsk-m11-* 2>/dev/null || echo "(tidak ada file sisa)"`
- L14: `git show HEAD -- docs/`
