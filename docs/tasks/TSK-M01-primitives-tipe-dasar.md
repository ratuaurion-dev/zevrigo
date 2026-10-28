# TSK-M01 · Tipe dasar dan parameter chain di MOD-01 · 🟡 · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-01 (crates/primitives) · **Untuk:** FTR-01 sampai FTR-05
- **Bergantung pada:** — · **Memblokir:** TSK-M02, TSK-M03, TSK-M04, TSK-M07, TSK-M08
- **Estimasi:** M

## Tujuan
Menyediakan tipe dasar yang dipakai semua crate lain: jumlah koin (Amount) yang tidak bisa
meluap diam-diam, tinggi blok, hash 32 byte, dan semua parameter protokol sebagai konstanta
yang diperiksa saat kompilasi.

## Konteks
- Baca AGENTS.md (terutama tabel parameter, "Larangan mutlak", dan aturan crate konsensus).
- Crate sudah ada (kerangka R0): crates/primitives. Lint konsensus sudah aktif di lib.rs; pertahankan.
- Ini task modul: hanya crates/primitives yang boleh berubah, ditambah status di docs/.

## Ruang lingkup
- Boleh diubah: crates/primitives/**; docs/tasks/README.md dan docs/modules/MOD-01-primitives.md (status saja).
- Dependensi crate ini: `thiserror = { workspace = true }`, dan `proptest = { workspace = true }` sebagai dev-dependency.
  Keduanya sudah ada di workspace. Jangan menambah dependensi lain.
- Jangan diubah: crate lain, Cargo.toml root, AGENTS.md, .agent/, Makefile, rust-toolchain.toml.
- Di luar lingkup: hashing BLAKE3 (TSK-M02), struktur transaksi dan blok (TSK-M03), rumus pasokan (TSK-M07).

## Persyaratan

### 1. Modul `amount`: `pub struct Amount(u128)` dalam quanta
- `pub const QUANTA_PER_HSIS: u128 = 1_000_000_000_000_000_000;` (10^18)
- `Amount::ZERO`, `Amount::from_quanta(u128)`, `Amount::as_quanta(self) -> u128`. Keduanya `const fn`
  supaya bisa dipakai di pemeriksaan saat kompilasi.
- `checked_add`, `checked_sub` → `Option<Amount>`; `saturating_sub` → `Amount`
- `checked_sum<I: IntoIterator<Item = Amount>>(iter) -> Option<Amount>`
- Derive: `Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default`
- TIDAK ada implementasi operator `Add`/`Sub` (agar tidak ada aritmetika tanpa pemeriksaan).
- Format teks (bukan bagian konsensus, tetapi harus deterministik):
  - `Amount::to_hsis_string(&self) -> String`: bagian bulat, lalu titik dan pecahan maksimal
    18 digit tanpa nol di belakang. Tanpa titik jika pecahan nol.
    Contoh: 0 → "0"; 10^18 → "1"; 1_500_000_000_000_000_000 → "1.5"; 1 → "0.000000000000000001".
  - `Amount::from_hsis_str(&str) -> Result<Amount, AmountParseError>` dengan aturan ketat:
    hanya digit ASCII dan paling banyak satu titik; bagian bulat wajib ada dan tanpa nol di depan
    (kecuali "0"); jika ada titik, harus diikuti 1–18 digit; tanpa tanda +/−, spasi, eksponen,
    pemisah ribuan, atau titik di akhir.
    Ditolak: "", ".5", "5.", "01", "1.0000000000000000001", "1e3", " 1", "1,5", "-1".
    Diterima: "0", "1", "1.5", "0.000000000000000001", "12.30" (hasil format ulang "12.3").
  - Contoh angka di atas ditulis sebagai string di test, sehingga tidak melanggar larangan literal float.
  - Kegagalan dilaporkan dengan enum error `thiserror` yang spesifik (format tidak valid, terlalu banyak desimal, overflow).
  - `impl Display` memakai `to_hsis_string()` diikuti " HSIS".

### 2. Modul `height`: `pub struct BlockHeight(u64)`
- `GENESIS = BlockHeight(0)`, `new`, `get`, `checked_add(u64)`, `checked_sub(u64)`, `next() -> Option<Self>`.

### 3. Modul `hash`: `pub struct Hash32([u8; 32])`
- `from_bytes`, `as_bytes`, `ZERO`; `Display`/`Debug` dalam hex huruf kecil (implementasi sendiri, tanpa dependensi);
  `from_hex(&str) -> Result<Hash32, HashParseError>` (tepat 64 karakter hex, huruf besar/kecil diterima).

### 4. Modul `params`: semua parameter dari tabel AGENTS.md sebagai `pub const`
- Minimal: `BLOCK_TIME_SECS`, `BLOCKS_PER_YEAR`, `MAIN_SUPPLY: Amount`, `GENESIS_ALLOCATION: Amount`,
  `HALF_LIFE_DIVISOR: u128` (= 2_274_842), `TAIL_REWARD: Amount`, `SLOW_START_BLOCKS: u64`,
  `COINBASE_MATURITY: u64`, `MAX_BLOCK_SIZE: usize`, `RANDOMX_KEY_INTERVAL: u64`, `RANDOMX_KEY_LAG: u64`,
  `ASERT_HALF_LIFE_BLOCKS: u64`, `MEDIAN_TIME_SPAN: usize`, `MAX_FUTURE_BLOCK_TIME_SECS: u64`.
- Setiap konstanta punya doc comment yang menjelaskan maknanya.
- Pemeriksaan saat kompilasi (`const _: () = assert!(...)`):
  - `TAIL_REWARD == MAIN_SUPPLY × 6 / 1000 / BLOCKS_PER_YEAR` (hadiah ekor tepat 0,6% S per tahun)
  - `GENESIS_ALLOCATION == MAIN_SUPPLY / 10`
  - `TAIL_REWARD == 1 HSIS`
  - Jika lint konsensus menandai aritmetika di dalam const assert, laporkan dan usulkan solusinya. Jangan menonaktifkan lint.
- Parameter testnet (lihat AGENTS.md) BELUM dibuat di task ini.

## Kriteria penerimaan
- [ ] Semua contoh diterima/ditolak di atas menjadi unit test dan lolos
- [ ] Property test: untuk setiap `u128` acak, `from_hsis_str(to_hsis_string(x)) == x`
- [ ] Property test: `checked_add` mengembalikan `None` tepat saat penjumlahan u128 meluap
- [ ] Test: `Amount::from_quanta(u128::MAX)` bisa diformat dan diparse ulang
- [ ] Test hex: round-trip `Hash32` dan penolakan panjang/karakter salah
- [ ] Unit test di `src/<modul>/tests.rs` atau `src/tests.rs`, terpisah dari kode
- [ ] Tidak ada `unwrap`/`expect`/`panic` di kode non-test; lint konsensus tetap aktif
- [ ] `make check-all` lolos
- [ ] Status TSK-M01 di docs/tasks/README.md menjadi 🟢, dan kartu MOD-01 mencatat TSK-M01 selesai

## Verifikasi
```bash
make check-all
git diff --stat HEAD~1
```

## Laporan akhir
Format AGENTS.md. Sertakan daftar API publik yang dibuat (signature) dan versi thiserror/proptest yang terpakai.
