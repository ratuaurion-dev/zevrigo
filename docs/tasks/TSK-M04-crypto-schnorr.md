# TSK-M04 · Tanda tangan Schnorr BIP340 di MOD-02 · 🟡 · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-02 (crates/crypto) · **Untuk:** FTR-03
- **Bergantung pada:** TSK-M01 · **Memblokir:** TSK-M09, TSK-M15, TSK-M16
- **Estimasi:** M

## Tujuan
Menyediakan tanda tangan dan verifikasi Schnorr BIP340 di kurva secp256k1 dengan kunci publik x-only
32 byte, persis sesuai standar, dan terbukti dengan seluruh vektor uji resmi.

## Konteks
- Baca AGENTS.md dan `docs/reference/spesifikasi-format-v0.1.md` Bagian 4 (output jenis 0 memakai kunci x-only BIP340).
- DEC-10 bagian 1 mengizinkan crate `k256`.
- Lint konsensus aktif di crate ini. Jangan dinonaktifkan.
- Pesan yang ditandatangani selalu 32 byte (hasil sighash). Crate ini TIDAK menghasilkan angka acak sendiri.

## Ruang lingkup
- Boleh diubah: crates/crypto/**; Cargo.lock; di Cargo.toml root HANYA menambah satu baris `k256` di
  `[workspace.dependencies]`; status di docs/tasks/README.md dan docs/modules/MOD-02-crypto.md.
- Dependensi crate ini: `k256 = { workspace = true }` dengan fitur seminimal mungkin untuk BIP340
  (`default-features = false`; tanpa ecdsa dan pkcs8). Tidak ada dependensi RNG.
- Jangan diubah: crate lain, modul `hash` dan `merkle` (TSK-M02), AGENTS.md, .agent/.
- Di luar lingkup: MuSig2, turunan kunci, BIP39, sighash.

## Persyaratan (modul `schnorr`)
- `pub struct PublicKey([u8; 32])` (x-only): `from_bytes(&[u8; 32]) -> Result<Self, SchnorrError>` wajib
  memvalidasi bahwa titik ada di kurva (lift_x); `as_bytes(&self) -> &[u8; 32]`.
- `pub struct SecretKey`: `from_bytes(&[u8; 32]) -> Result<Self, SchnorrError>` menolak 0 dan nilai ≥ orde kurva;
  `public_key(&self) -> PublicKey`.
  - Tidak boleh `Display`. `Debug` mencetak `SecretKey(<rahasia>)` tanpa byte kunci.
  - Memori kunci dibersihkan saat drop. Laporkan mekanismenya (misalnya zeroize di dalam k256).
- `pub struct Signature([u8; 64])`: `from_bytes`, `as_bytes`.
- `pub fn sign(sk: &SecretKey, msg: &[u8; 32], aux_rand: &[u8; 32]) -> Result<Signature, SchnorrError>`.
  `aux_rand` disediakan pemanggil.
- `pub fn verify(pk: &PublicKey, msg: &[u8; 32], sig: &Signature) -> bool`.
- `SchnorrError` memakai thiserror (dependensi `thiserror = { workspace = true }` boleh ditambahkan ke crate ini).
- Jika nama API k256 untuk BIP340 berbeda dari dugaan, pakai API yang benar dan laporkan. Jangan mengarang.

## Vektor uji
- **Seluruh vektor resmi BIP340.** Unduh `test-vectors.csv` dari repo `bitcoin/bips`, folder `bip-0340`.
  Simpan sebagai `crates/crypto/tests/data/bip340-test-vectors.csv` (file data). Catat URL dan SHA-256 file
  di komentar test dan di laporan.
  - Setiap baris diuji: vektor dengan kunci rahasia → `sign` harus menghasilkan tanda tangan persis; setiap baris → `verify` sesuai kolom hasil.
  - Jika `PublicKey::from_bytes` menolak kunci (misalnya kunci bukan di kurva), itu dianggap verifikasi gagal.
  - Vektor dengan panjang pesan selain 32 byte DILEWATI karena API hanya menerima 32 byte. Laporkan jumlah dan nomornya.
- **Jangkar wajib cocok** (sudah diperiksa Quick dengan implementasi referensi BIP340 terpisah):
  - Vektor 0: kunci rahasia `00…03`, aux `00…00`, pesan `00…00`
    → kunci publik `F9308A019258C31049344F85F89D5229B531C845836F99B08601F113BCE036F9`
    → tanda tangan `E907831F80848D1069A5371B402410364BDF1C5F8307B0084C55F1CE2DCA821525F66A4A85EA8B71E482A74F382D2CE5EBEEE8FDB2172F477DF4900D310536C0`
  - Vektor 1: kunci rahasia `B7E151628AED2A6ABF7158809CF4F3C762E7160F38B4DA56A784D9045190CFEF`, aux `00…01`,
    pesan `243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89`
    → kunci publik `DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659`
    → tanda tangan `6896BD60EEAE296DB48A229FF71DFE071BDE413E6D43F917DC8DCF8C78DE33418906D11AC976ABCCB20B091292BFF4EA897EFCB639EA871CFA95F6DE339E4B0A`
- Test lain:
  - Kunci rahasia 0 dan orde kurva ditolak.
  - Proptest: membalik satu bit di pesan, tanda tangan, atau kunci publik membuat `verify` gagal.
  - `sign` deterministik untuk (kunci, pesan, aux) yang sama.

## Laporan dependensi (DEC-10)
Untuk `k256` dan dependensi turunannya: versi terkunci, lisensi, fitur aktif, dan apakah memakai `unsafe` di dalamnya.

## Kriteria penerimaan
- [ ] Semua vektor resmi BIP340 (yang pesannya 32 byte) lolos, termasuk jangkar vektor 0 dan 1
- [ ] Test lain lolos
- [ ] Unit test di `src/schnorr/tests.rs`; file CSV hanya data
- [ ] `make check-all` lolos; `#![forbid(unsafe_code)]` dan lint konsensus tetap aktif
- [ ] Status TSK-M04 di docs/tasks/README.md menjadi 🟢, dan kartu MOD-02 mencatat TSK-M04 selesai

## Verifikasi
```bash
make check-all
sha256sum crates/crypto/tests/data/bip340-test-vectors.csv
git diff --stat HEAD~1
```

## Laporan akhir
Format AGENTS.md. Sertakan daftar API publik, jumlah vektor yang diuji dan dilewati, dan laporan dependensi.
