# TSK-M22 · Tipe OutPoint dan UtxoEntry di MOD-01 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-01 (crates/primitives) · **Untuk:** FTR-01 sampai FTR-03
- **Bergantung pada:** TSK-M03 · **Memblokir:** TSK-M10, TSK-M09, TSK-M13, TSK-M16
- **Estimasi:** XS

## Tujuan
Menambah dua tipe dasar yang dipakai bersama oleh consensus (MOD-04), storage (MOD-05), mempool (MOD-07), dan
wallet (MOD-09):
- `OutPoint`: rujukan ke satu output (txid + indeks output);
- `UtxoEntry`: output yang belum dibelanjakan, beserta tinggi blok pembuatnya dan tanda coinbase.
Tipe ini ditaruh di MOD-01 karena MOD-04 dan MOD-05 sama-sama memakainya, padahal keduanya tidak saling bergantung.

## Konteks
- `Input` (TSK-M03) sudah punya `prev_txid()` dan `prev_index()`. `OutPoint` berisi pasangan yang sama, tetapi
  sebagai tipe sendiri yang bisa diurutkan dan dijadikan kunci `BTreeMap`.
- Task ini TIDAK membuat encoding biner untuk kedua tipe. UTXO bukan bagian format transaksi atau blok di
  spesifikasi. Layout byte penyimpanan ditentukan di TSK-M11.
- Kematangan coinbase (100 blok) TIDAK diperiksa di sini. Itu aturan konsensus (MOD-04, TSK-M09).
- Lint konsensus aktif di crate ini. Jangan dinonaktifkan.

## Ruang lingkup
- Boleh diubah: `crates/primitives/src/utxo/mod.rs` (baru), `crates/primitives/src/utxo/tests.rs` (baru),
  `crates/primitives/src/lib.rs`, `crates/primitives/src/tests.rs`, serta teks status di bagian
  "Status yang ditulis saat task selesai".
- Jangan diubah: modul lain di crates/primitives (amount, block, encoding, hash, height, params, tx), crate lain,
  semua Cargo.toml, dan Cargo.lock. Tanpa dependensi baru.

## Persyaratan
1. `crates/primitives/src/utxo/mod.rs`:
   ```rust
   #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
   pub struct OutPoint { txid: Hash32, index: u32 }
   ```
   - Urutan field wajib `txid` lalu `index`, sehingga `Ord` hasil derive mengurutkan menurut txid dulu, lalu index.
     Tulis hal ini di doc comment.
   - `pub const fn new(txid: Hash32, index: u32) -> Self`
   - `pub const fn from_input(input: &Input) -> Self` (memakai `input.prev_txid()` dan `input.prev_index()`)
   - `pub const fn txid(&self) -> Hash32` dan `pub const fn index(&self) -> u32`
   ```rust
   #[derive(Clone, Debug, PartialEq, Eq)]
   pub struct UtxoEntry { output: Output, height: BlockHeight, is_coinbase: bool }
   ```
   - `pub const fn new(output: Output, height: BlockHeight, is_coinbase: bool) -> Self`
   - `pub const fn output(&self) -> &Output`, `pub const fn height(&self) -> BlockHeight`,
     `pub const fn is_coinbase(&self) -> bool`
   - Doc comment `UtxoEntry` wajib menyebut bahwa kematangan coinbase diperiksa MOD-04, bukan oleh tipe ini.
   - Semua item publik punya doc comment.
   - Jika ada `const fn` yang tidak bisa dikompilasi sebagai const, hapus `const` di fungsi itu saja dan sebutkan
     di laporan.
2. `crates/primitives/src/lib.rs`, hanya dua sisipan:
   - baris `pub mod utxo;` tepat setelah baris `pub mod tx;`;
   - baris `pub use utxo::{OutPoint, UtxoEntry};` tepat setelah baris `};` yang menutup blok `pub use tx::{`.
   Doc comment crate dan baris lain tidak diubah.
3. Tanpa unwrap, expect, panic, atau indexing di kode non-test.

## Daftar test (nama persis; test zevrigo-primitives bertambah dari 52 menjadi tepat 56)
`src/utxo/tests.rs`:
1. `outpoint_new_and_from_input_preserve_txid_and_index`:
   `OutPoint::new(Hash32::from_bytes([0x11; 32]), 7)` mengembalikan txid `[0x11; 32]` dan index 7;
   `OutPoint::from_input(&Input::new(Hash32::from_bytes([0x22; 32]), 3))` sama dengan
   `OutPoint::new(Hash32::from_bytes([0x22; 32]), 3)`.
2. `prop_outpoint_ord_is_txid_then_index`: proptest dengan input
   `ta in any::<[u8; 32]>(), ia in any::<u32>(), tb in any::<[u8; 32]>(), ib in any::<u32>()`. Assert persis:
   `prop_assert_eq!(OutPoint::new(Hash32::from_bytes(ta), ia).cmp(&OutPoint::new(Hash32::from_bytes(tb), ib)), (ta, ia).cmp(&(tb, ib)));`
3. `utxo_entry_getters_return_constructor_values`: output dibuat dengan
   `Output::new(Amount::from_quanta(10), OutputKind::SingleKey, [0x01; 32]).expect("output")`. Dua entri:
   (output, `BlockHeight::new(0)`, true) dan (output, `BlockHeight::new(5)`, false). Ketiga getter mengembalikan
   nilai konstruktor.
`src/tests.rs`:
4. `utxo_reexports_are_accessible`: membuat satu `crate::OutPoint` dan satu `crate::UtxoEntry` lewat root crate.
Test lama di src/tests.rs tidak diubah.

## Status yang ditulis saat task selesai (teks persis, di commit yang sama)
- docs/tasks/README.md, baris pertama: ganti seluruh baris menjadi
  `# Task kerja: TSK-M10 siap dikerjakan, jalur kritis lewat spesifikasi validasi (TSK-S02)`.
- docs/tasks/README.md, baris yang diawali `| TSK-M22 | MOD-01 |`: kolom terakhir
  `🟡 siap ([kartu](TSK-M22-primitives-utxo.md))` diganti menjadi `🟢 selesai ([kartu](TSK-M22-primitives-utxo.md))`.
- docs/tasks/README.md, baris ringkasan `🟢 Selesai`: sisipkan `, TSK-M22 (OutPoint dan UtxoEntry)` tepat setelah
  `TSK-M05 (PoW dev regtest)`.
- docs/tasks/README.md, baris ringkasan `🟡 Siap dikerjakan`: ganti seluruh kolom kedua menjadi
  `TSK-M10 (penyimpanan in-memory, [kartu](TSK-M10-storage-memory.md))`.
- docs/tasks/README.md, baris ringkasan `➡️ Langkah berikutnya`: ganti seluruh kolom kedua menjadi
  `Kerjakan TSK-M10; TSK-S02 (pencipta + Quick)`.
- docs/modules/MOD-01-primitives.md, baris yang diawali `- **Status:**`: ganti seluruh baris menjadi
  `- **Status:** 🟢 selesai (TSK-M01, TSK-M03, dan TSK-M22 selesai)`.
- docs/modules/MOD-01-primitives.md, baris yang diawali `- [TSK-M22]`: ganti seluruh baris menjadi
  `- [TSK-M22](../tasks/TSK-M22-primitives-utxo.md) — 🟢 selesai (tipe OutPoint dan UtxoEntry)`.
- Baris judul kartu ini: ganti `· 🟡 ·` menjadi `· 🟢 selesai ·`.
- docs/system-map.md dan kartu modul lain TIDAK diubah.

## Kriteria penerimaan
- [ ] 4 test baru dengan nama persis lolos; zevrigo-primitives tepat 56 test
- [ ] Cargo.toml dan Cargo.lock tidak berubah
- [ ] `#![forbid(unsafe_code)]` dan lint konsensus tetap aktif; tanpa unwrap/expect/panic di kode non-test
- [ ] `make check-all` lolos tanpa warning
- [ ] Status diperbarui persis seperti bagian di atas
- [ ] Satu commit: `feat(primitives): tipe OutPoint dan UtxoEntry (TSK-M22)`

## Laporan akhir
OUTPUT MENTAH perintah berikut, bernomor, satu blok kode per perintah, tidak diringkas dan tidak dari ingatan.
Setelah L10, boleh ditambah paling banyak 5 butir asumsi, risiko, atau pertanyaan terbuka.
- L1: `git log --oneline -3`
- L2: `git status --short`
- L3: `git show --stat HEAD`
- L4: `git diff HEAD~1 -- Cargo.lock Cargo.toml crates/primitives/Cargo.toml || true` (harus kosong)
- L5: `cat crates/primitives/src/utxo/mod.rs crates/primitives/src/utxo/tests.rs`
- L6: `git show HEAD -- crates/primitives/src/lib.rs crates/primitives/src/tests.rs`
- L7: `cargo test -p zevrigo-primitives 2>&1 | grep -E "^test (utxo::|tests::)|test result"`
- L8: `make check-all > /tmp/m22-check.log 2>&1; echo "exit=$?"; grep -cE "^test .* ok$" /tmp/m22-check.log; grep -E "^(warning|error)|FAILED" /tmp/m22-check.log || echo "(tidak ada warning/error)"`
- L9: `grep -nE "unwrap\(|expect\(|panic!" crates/primitives/src/utxo/mod.rs || echo "(tidak ada)"`
- L10: `git show HEAD -- docs/`
