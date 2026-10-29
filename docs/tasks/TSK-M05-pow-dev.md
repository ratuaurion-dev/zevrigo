# TSK-M05 · Antarmuka PoW dan PoW dev regtest di MOD-03 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-03 (crates/pow) · **Untuk:** FTR-01, FTR-02
- **Bergantung pada:** TSK-M02, TSK-M03, DEC-09, DEC-14, DEC-15 · **Memblokir:** TSK-M08, TSK-M09, TSK-M20
- **Estimasi:** S

## Tujuan
Menyediakan PoW dev khusus regtest (DEC-09), supaya node regtest dan penambang bisa dibangun tanpa menunggu DEC-01.
Isinya: tipe `Target` 256-bit, hash PoW dev dari header 96 byte, pemeriksaan `hash ≤ target`, dan pencarian nonce.

## Konteks
- Sumber kebenaran: `docs/reference/spesifikasi-format-v0.1.md` Bagian 3 (domain `regtest pow`) dan Bagian 6.1
  (hash PoW regtest dibaca sebagai bilangan 256-bit big-endian dan wajib ≤ target).
  Jika kartu ini dan spesifikasi berbeda, BERHENTI dan laporkan.
- DEC-14: MOD-03 bergantung pada MOD-02. Pakai `homeostasis_crypto::{hash, Domain}` dengan `Domain::RegtestPow`
  (sudah ada sejak TSK-M02). Jangan memanggil crate blake3 langsung.
- DEC-15: task ini menerima target sebagai `Target` 256-bit. Field `target: u32` (compact) di header TIDAK dibaca
  di task ini. Konversinya dikerjakan di TSK-M08.
- Yang dipakai dari MOD-01: `Header` (`new`, `height`, `prev_block_id`, `merkle_root`, `timestamp`, `target`, `nonce`,
  `to_array`) dan `Hash32` (`from_bytes`, `as_bytes`, `ZERO`). Jika ada nama yang berbeda di kode MOD-01, pakai nama
  yang ada di kode. MOD-01 tidak boleh diubah.
- Lint konsensus aktif di crate ini. Jangan dinonaktifkan.

## Ruang lingkup
- Boleh diubah: `crates/pow/**`; `Cargo.lock` (hanya karena dependensi homeostasis-pow bertambah); serta teks status
  yang tertulis persis di bagian "Status yang ditulis saat task selesai".
- `crates/pow/Cargo.toml`: di bagian `[dependencies]` tambahkan tepat dua baris, `homeostasis-crypto.workspace = true`
  dan `thiserror.workspace = true`. Tambahkan bagian baru `[dev-dependencies]` berisi `proptest.workspace = true`.
  Tidak boleh ada crate pihak ketiga baru di Cargo.lock, karena semuanya sudah ada di workspace.
- Jangan diubah: crate lain (termasuk crates/primitives dan crates/crypto), Cargo.toml root, AGENTS.md,
  docs/system-map.md. Jika butuh perubahan di sana, BERHENTI dan tanya.
- Di luar lingkup:
  - konversi compact u32 ke `Target`, ASERT, dan chainwork (TSK-M08);
  - aturan jaringan mana yang boleh memakai PoW dev, beserta test penolakannya di testnet dan mainnet (TSK-M09);
  - RandomX (TSK-M06);
  - loop penambangan multi-utas (TSK-M20).

## Struktur file
- `src/lib.rs`: deklarasi modul dan re-export
- `src/target/mod.rs` dan `src/target/tests.rs`
- `src/regtest/mod.rs` dan `src/regtest/tests.rs`
- `src/tests.rs`: file lama, tetap ada (berisi `module_id_matches`)

## Persyaratan
1. Modul `target`: `pub struct Target([u8; 32])`, berupa bilangan 256-bit dengan urutan byte big-endian
   (byte ke-0 paling signifikan).
   - Derive: `Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash`. Urutan `Ord` hasil derive pada `[u8; 32]`
     sama dengan urutan numerik big-endian. Tulis hal ini di doc comment.
   - `pub const ZERO: Target` (32 byte 0x00) dan `pub const MAX: Target` (32 byte 0xff).
   - `pub const fn from_be_bytes(bytes: [u8; 32]) -> Target` dan `pub const fn to_be_bytes(&self) -> [u8; 32]`.
   - `pub fn is_met_by(&self, pow_hash: &Hash32) -> bool` mengembalikan `true` jika dan hanya jika 32 byte
     `pow_hash.as_bytes()`, dibaca sebagai bilangan big-endian, ≤ `self`. Nilai yang sama persis dengan target
     termasuk lolos. Cukup bandingkan dua array `[u8; 32]`.
2. Modul `regtest`:
   - `pub fn regtest_pow_hash(header: &Header) -> Hash32`, yaitu `hash(Domain::RegtestPow, &header.to_array())`.
   - `pub fn verify_regtest_pow(header: &Header, target: &Target) -> Result<(), PowError>` mengembalikan `Ok(())` jika
     `target.is_met_by(&regtest_pow_hash(header))`. Jika tidak, mengembalikan
     `Err(PowError::AboveTarget { pow_hash, target })`.
   - `pub fn mine_regtest(header: &Header, target: &Target, max_attempts: u64) -> Option<Header>`:
     - Percobaan ke-i (i = 0, 1, …, max_attempts − 1) memakai nonce `header.nonce() + i`. Semua field lain sama.
     - Kembalikan header pertama yang lolos `verify_regtest_pow`.
     - Kembalikan `None` jika `max_attempts == 0`, jika semua percobaan gagal, atau jika nonce berikutnya
       melewati `u64::MAX`. Pakai `checked_add`, tanpa wrap dan tanpa panic.
   - Doc comment ketiga fungsi wajib menyebut bahwa PoW ini hanya sah di regtest (DEC-09), dan penolakannya
     di testnet dan mainnet dilakukan oleh MOD-04 (TSK-M09).
3. `pub enum PowError` memakai `thiserror`, dengan satu varian: `AboveTarget { pow_hash: Hash32, target: Target }`,
   pesan error `hash PoW regtest di atas target`.
4. Re-export dari lib.rs: `Target`, `PowError`, `regtest_pow_hash`, `verify_regtest_pow`, `mine_regtest`.
   Di doc comment crate (`//!`) di lib.rs, hapus kalimat "Belum diimplementasikan (kerangka R0)." dan ganti judulnya
   menjadi `MOD-03: Proof of Work. Berisi Target 256-bit dan PoW dev regtest (DEC-09); PoW mainnet menunggu DEC-01.`
5. Tanpa unwrap, expect, panic, atau indexing yang bisa panic di kode non-test. Semua item publik punya doc comment
   (lint `missing_docs` dengan `-D warnings`).

## Vektor uji
Nilai di bawah dihitung Quick dengan implementasi BLAKE3 yang terpisah dari kode kita. Agent wajib menghitung ulang
dengan b3sum (lihat bagian Verifikasi) dan menulis perintahnya sebagai komentar di test.

Header dasar = vektor V4 dari kartu TSK-M03 (96 byte, height 1, prev 32×0x11, merkle 32×0x22,
timestamp 1_800_000_000, target compact 0x1d00ffff, nonce 42):
`0100000001000000000000001111111111111111111111111111111111111111111111111111111111111111222222222222222222222222222222222222222222222222222222222222222200d2496b00000000ffff001d2a00000000000000`

Target uji (hex big-endian, 64 karakter):
- T16 = `0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff` (lolos jika karakter hex pertama hash = 0)
- T256 = `00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff` (lolos jika dua karakter hex pertama hash = 00)

| ID | Uji | Harapan |
|---|---|---|
| P1 | `regtest_pow_hash(V4)` | `0364aacccf064eac20c82cf0bbb69da7bdfe2c85208267a85707aeaeb6475c17` |
| P2 | `verify_regtest_pow(V4, Target = P1)` | `Ok(())` (nilai sama dengan target lolos) |
| P3 | `verify_regtest_pow(V4, Target = 0364aacccf064eac20c82cf0bbb69da7bdfe2c85208267a85707aeaeb6475c16)` (P1 − 1) | `Err(PowError::AboveTarget { .. })` |
| P4 | `verify_regtest_pow(V4, Target::MAX)` dan `verify_regtest_pow(V4, Target::ZERO)` | `Ok(())` dan `Err(AboveTarget)` |
| P5 | `mine_regtest(V4 dengan nonce 0, T16, 1_000)` | `Some`, nonce 3, `regtest_pow_hash` = `0d9248920ecaf57c4531d401a45d35ce21170b6e483ad891d8aeb7753139446c` |
| P6 | `mine_regtest(V4, T256, 1_000)` | `Some`, nonce 367, `regtest_pow_hash` = `004d9338d441ce96b3cc233909cd1f80d1277612f53bea921482550e37d3d8d4` |
| P7 | `mine_regtest(V4, T256, 325)` dan `mine_regtest(V4, T256, 326)` | `None` dan `Some` dengan nonce 367 |
| P8 | `mine_regtest(V4 dengan nonce u64::MAX, Target::ZERO, 10)` | `None`, tanpa panic |
| P9 | `mine_regtest(V4 dengan nonce u64::MAX, Target::MAX, 1)` | `Some`, header sama persis (nonce u64::MAX) |
| P10 | `regtest_pow_hash(V4)` dibanding `homeostasis_crypto::block_id(V4)` (H6 = `1de077cc1a2c8ad7c055df207ce40cca9643f961d0df21c1c67c7f121d107cfa`) | berbeda |
| P11 | `mine_regtest(V4, Target::MAX, 0)` | `None` |

Catatan P5 sampai P7: hasil `mine_regtest` juga dicek field lainnya (height, prev_block_id, merkle_root, timestamp,
target) sama dengan V4, dan hanya nonce yang berubah.

## Daftar test (nama persis; jumlah test homeostasis-pow menjadi tepat 12)
`src/target/tests.rs`:
1. `zero_and_max_constants_and_is_met_by_extremes`: `Target::MAX` dipenuhi oleh hash 32×0xff; `Target::ZERO`
   dipenuhi oleh `Hash32::ZERO` tetapi tidak oleh hash yang byte terakhirnya 0x01.
2. `prop_be_bytes_round_trip`: proptest, `Target::from_be_bytes(b).to_be_bytes() == b`.
3. `prop_ord_matches_numeric_big_endian`: proptest dengan dua `u128` acak `a` dan `b`. Isi 16 byte terakhir target
   dengan `a.to_be_bytes()` dan `b.to_be_bytes()` (16 byte pertama nol). Urutan target harus sama dengan urutan
   `a` dan `b`, dan `is_met_by` dengan hash berisi byte yang sama harus `true` tepat saat nilai hash ≤ target.

`src/regtest/tests.rs`:
4. `vector_p1_regtest_pow_hash_of_v4`
5. `vectors_p2_to_p4_verify_boundaries`
6. `vectors_p5_to_p7_mine_regtest_first_nonce_and_attempt_limit`
7. `vectors_p8_p9_p11_nonce_overflow_and_zero_attempts`
8. `vector_p10_regtest_pow_hash_differs_from_block_id`
9. `prop_verify_is_monotonic_in_target`: proptest. Jika `verify_regtest_pow(h, t)` lolos, maka juga lolos untuk
   setiap target `t2 ≥ t`. Header acak dibuat dari field acak lewat `Header::new`.
10. `prop_mine_regtest_result_is_valid_and_only_nonce_changes`: proptest dengan `max_attempts` dalam rentang 0..=64
    dan target acak. Jika hasilnya `Some(h2)`, maka `verify_regtest_pow(&h2, &t)` lolos, nonce h2 berada di
    `[nonce awal, nonce awal + max_attempts)`, dan field lain sama dengan header awal.

`src/tests.rs`:
11. `module_id_matches` (lama, tidak diubah)
12. `crate_root_reexports_are_accessible`: memanggil kelima item re-export lewat `homeostasis_pow::...` atau `crate::...`.

## Status yang ditulis saat task selesai (teks persis, di commit yang sama)
- docs/tasks/README.md, baris yang diawali `| TSK-M05 | MOD-03 |`: kolom terakhir
  `🟡 siap ([kartu](TSK-M05-pow-dev.md))` diganti menjadi `🟢 selesai ([kartu](TSK-M05-pow-dev.md))`.
- docs/tasks/README.md, baris ringkasan `🟢 Selesai`: sisipkan `, TSK-M05 (PoW dev regtest)` tepat setelah
  `TSK-M21 (ID transaksi dan blok)`.
- docs/tasks/README.md, baris ringkasan `🟡 Siap dikerjakan`: ganti seluruh kolom kedua menjadi
  `TSK-M10 (penyimpanan in-memory) kartunya menyusul`.
- docs/tasks/README.md, baris ringkasan `➡️ Langkah berikutnya`: ganti seluruh kolom kedua menjadi
  `Kartu TSK-M10 (pencipta + Quick); TSK-S02 (pencipta + Quick)`.
- docs/modules/MOD-03-pow.md, baris yang diawali `- **Status:**`: ganti seluruh baris menjadi
  `- **Status:** 🟡 sedang berjalan (TSK-M05 selesai; sisa: PoW mainnet TSK-M06, menunggu `DEC-01`)`.
- docs/modules/MOD-03-pow.md, baris yang diawali `- [TSK-M05]`: ganti seluruh baris menjadi
  `- [TSK-M05](../tasks/TSK-M05-pow-dev.md) — 🟢 selesai (antarmuka PoW dan PoW dev regtest)`.
- Baris judul kartu ini: ganti `· 🟡 ·` menjadi `· 🟢 selesai ·`.
- Status modul lain dan docs/system-map.md TIDAK diubah.

## Kriteria penerimaan
- [ ] P1 sampai P11 lolos, dan nilainya sama dengan hasil b3sum
- [ ] 12 test homeostasis-pow lolos dengan nama persis seperti daftar di atas
- [ ] Cargo.toml crate pow persis seperti di Ruang lingkup; tidak ada crate pihak ketiga baru di Cargo.lock
- [ ] `#![forbid(unsafe_code)]` dan lint konsensus tetap aktif; tanpa unwrap/expect/panic di kode non-test;
      tidak ada pemanggilan `blake3::` di crates/pow
- [ ] `make check-all` lolos tanpa warning
- [ ] Status diperbarui persis seperti bagian "Status yang ditulis saat task selesai"
- [ ] Satu commit: `feat(pow): Target 256-bit dan PoW dev regtest (TSK-M05)`

## Verifikasi
```bash
V4=0100000001000000000000001111111111111111111111111111111111111111111111111111111111111111222222222222222222222222222222222222222222222222222222222222222200d2496b00000000ffff001d2a00000000000000
PFX=${V4:0:176}
printf '%s' "$V4" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 regtest pow" --no-names
for n in $(seq 0 400); do
  NH=$(printf '%02x%02x000000000000' $((n % 256)) $((n / 256)))
  printf '%s %s\n' "$n" "$(printf '%s' "$PFX$NH" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 regtest pow" --no-names)"
done > /tmp/m05-scan.txt
grep -E '^42 ' /tmp/m05-scan.txt
grep -m1 -E '^[0-9]+ 0' /tmp/m05-scan.txt
awk '$1 >= 42' /tmp/m05-scan.txt | grep -m1 -E '^[0-9]+ 00'
```
Harapan: baris pertama = P1; `42 <P1>`; `3 0d9248920ecaf57c4531d401a45d35ce21170b6e483ad891d8aeb7753139446c`; `367 004d9338d441ce96b3cc233909cd1f80d1277612f53bea921482550e37d3d8d4`.

## Laporan akhir
Laporan berisi OUTPUT MENTAH perintah berikut, bernomor, satu blok kode per perintah. Output tidak diringkas
dan tidak ditulis dari ingatan. Setelah L11, boleh ditambah paling banyak 5 butir asumsi, risiko, atau
pertanyaan terbuka.
- L1: `git log --oneline -3`
- L2: `git status --short`
- L3: `git show --stat HEAD`
- L4: `cat crates/pow/Cargo.toml`
- L5: `git diff HEAD~1 -- Cargo.lock`
- L6: `cat crates/pow/src/lib.rs crates/pow/src/target/mod.rs crates/pow/src/regtest/mod.rs`
- L7: `cargo test -p homeostasis-pow 2>&1 | grep -E "^test |test result"`
- L8: `make check-all > /tmp/m05-check.log 2>&1; echo "exit=$?"; grep -cE "^test .* ok$" /tmp/m05-check.log; grep -E "^(warning|error)|FAILED" /tmp/m05-check.log || echo "(tidak ada warning/error)"`
- L9: semua perintah dan output di bagian Verifikasi
- L10: `grep -rnE "unwrap\(|expect\(|panic!|blake3::" crates/pow/src/lib.rs crates/pow/src/target/mod.rs crates/pow/src/regtest/mod.rs || echo "(tidak ada)"`
- L11: `git show HEAD -- docs/`
