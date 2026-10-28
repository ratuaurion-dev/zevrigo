# TSK-M02 · Hash BLAKE3 dengan pemisah domain dan Merkle root di MOD-02 · 🟡 · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-02 (crates/crypto) · **Untuk:** FTR-01 sampai FTR-05
- **Bergantung pada:** TSK-M01 · **Memblokir:** TSK-M05, TSK-M21
- **Estimasi:** S

## Tujuan
Menjadikan MOD-02 satu-satunya tempat hash BLAKE3 dihitung di proyek. Setiap hash diberi pemisah
domain, sehingga hash untuk kegunaan berbeda (txid, Merkle, ID blok, dan lainnya) tidak pernah bisa
tertukar. Termasuk perhitungan Merkle root dari daftar hash.

## Konteks
- Baca AGENTS.md dan `docs/reference/spesifikasi-format-v0.1.md` Bagian 3 (hash dan pemisah domain)
  serta Bagian 6.3 (Merkle). Jika file spesifikasi itu tidak ada di repo, BERHENTI dan laporkan.
- DEC-10 bagian 1 mengizinkan crate `blake3`.
- Lint konsensus aktif di crate ini (arithmetic_side_effects, float_arithmetic, indexing_slicing). Jangan dinonaktifkan.

## Ruang lingkup
- Boleh diubah: crates/crypto/**; Cargo.lock; di Cargo.toml root HANYA menambah satu baris `blake3` di
  `[workspace.dependencies]` (versi stabil 1.x terbaru); status di docs/tasks/README.md dan docs/modules/MOD-02-crypto.md.
- Dependensi crate ini: `blake3 = { workspace = true }`; dev-dependency `proptest = { workspace = true }`. Tidak ada yang lain.
- Jangan diubah: crate lain, AGENTS.md, .agent/, Makefile, rust-toolchain.toml.
- Di luar lingkup: encoding transaksi dan blok (TSK-M03), txid/wtxid/block_id (TSK-M21), PoW (TSK-M05), Schnorr (TSK-M04).

## Persyaratan

### 1. Modul `hash`
- `pub enum Domain { Txid, Wtxid, MerkleLeaf, MerkleNode, BlockId, Condition, Sighash, RegtestPow }`
  (derive `Clone, Copy, Debug, PartialEq, Eq`).
- `pub const fn context(self) -> &'static str`, mengembalikan string konteks persis seperti tabel ini:

  | Domain | String konteks |
  |---|---|
  | Txid | `homeostasis 2026-10-28 txid` |
  | Wtxid | `homeostasis 2026-10-28 wtxid` |
  | MerkleLeaf | `homeostasis 2026-10-28 merkle leaf` |
  | MerkleNode | `homeostasis 2026-10-28 merkle node` |
  | BlockId | `homeostasis 2026-10-28 block id` |
  | Condition | `homeostasis 2026-10-28 condition` |
  | Sighash | `homeostasis 2026-10-28 sighash v1` |
  | RegtestPow | `homeostasis 2026-10-28 regtest pow` |

- `pub fn hash(domain: Domain, data: &[u8]) -> Hash32`, memakai BLAKE3 mode derive_key
  (konteks = `domain.context()`, material = `data`), keluaran 32 byte.
- `pub struct DomainHasher` untuk input bertahap: `new(domain) -> Self`, `update(&mut self, data: &[u8]) -> &mut Self`,
  `finalize(&self) -> Hash32`. Hasilnya wajib sama dengan `hash()` atas gabungan semua potongan.

### 2. Modul `merkle`
- `pub fn merkle_root(items: &[Hash32]) -> Option<Hash32>`:
  - `None` jika `items` kosong.
  - Daun: `hash(MerkleLeaf, item)`. Simpul: `hash(MerkleNode, kiri ‖ kanan)` (64 byte).
  - Jika jumlah simpul di satu tingkat ganjil, simpul terakhir dinaikkan apa adanya ke tingkat berikutnya. TIDAK digandakan.
  - Satu item: root = daunnya.
  - Iteratif (tanpa rekursi), tanpa indexing (pakai iterator/`chunks`), tanpa unwrap.
- Re-export `Domain`, `hash`, `DomainHasher`, `merkle_root` dari `lib.rs`.

## Vektor uji (wajib dari alat independen, bukan dari kode kita)
Hitung nilai harapan dengan `b3sum` (sudah dipasang pengguna), lalu tulis hasilnya sebagai konstanta di test,
dengan perintah pembuatnya sebagai komentar di atas setiap konstanta.
- **Domain:** untuk ke-8 domain, `hash(domain, b"")` dan `hash(domain, b"abc")`:
  `printf '' | b3sum --derive-key "<konteks>" --no-names` dan `printf 'abc' | b3sum --derive-key "<konteks>" --no-names`.
- **Merkle:** item ke-i (i mulai dari 1) = 32 byte yang semuanya bernilai i (`0x01…01`, `0x02…02`, dan seterusnya).
  Vektor untuk 1, 2, 3, dan 5 item. Hitung daun dan simpul di shell dengan b3sum, mengubah hex ke biner
  dengan `xxd -r -p` (atau alat lain yang tersedia tanpa sudo; laporkan alat yang dipakai).
  Cantumkan langkah perhitungan di komentar test.
- Jika b3sum tidak tersedia, BERHENTI dan laporkan. Jangan menghitung nilai harapan dengan kode proyek sendiri.

## Test lain
- Ke-8 string konteks berbeda satu sama lain dan persis sama dengan tabel di atas.
- Domain berbeda menghasilkan hash berbeda untuk data yang sama.
- Proptest: `DomainHasher` dengan data dipotong di titik acak memberi hasil sama dengan `hash()`.
- Merkle: urutan item berpengaruh; `merkle_root([a, b, c]) != merkle_root([a, b, c, c])`
  (bukti tidak ada penggandaan daun; celah ini pernah terjadi di Bitcoin, CVE-2012-2459).

## Laporan dependensi (DEC-10)
Untuk `blake3`: versi terkunci, lisensi, fitur aktif, apakah memakai `unsafe` di dalamnya, dan apakah build
mengompilasi kode C/assembly. Jika build gagal karena compiler C, coba fitur `pure` lalu laporkan.

## Kriteria penerimaan
- [ ] Semua vektor dari b3sum cocok
- [ ] Semua test lain lolos
- [ ] Unit test di `src/<modul>/tests.rs`, terpisah dari kode
- [ ] `make check-all` lolos; `#![forbid(unsafe_code)]` dan lint konsensus tetap aktif
- [ ] Status TSK-M02 di docs/tasks/README.md menjadi 🟢, dan kartu MOD-02 mencatat TSK-M02 selesai

## Verifikasi
```bash
make check-all
git diff --stat HEAD~1
```

## Laporan akhir
Format AGENTS.md. Sertakan daftar API publik, output perintah b3sum yang dipakai, dan laporan dependensi.
