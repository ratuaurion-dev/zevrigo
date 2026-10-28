# TSK-M03 · Struktur transaksi dan blok, encoding kanonis di MOD-01 · 🟡 · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-01 (crates/primitives) · **Untuk:** FTR-01 sampai FTR-05
- **Bergantung pada:** TSK-M01, TSK-S01 · **Memblokir:** TSK-M09, TSK-M10, TSK-M12, TSK-M14, TSK-M16, TSK-M21
- **Estimasi:** L

## Tujuan
Membuat tipe data transaksi, witness, kondisi, header, dan blok beserta encoding biner kanonisnya,
persis sesuai Spesifikasi Format v0.1. Setiap nilai hanya punya satu encoding yang sah. Encoding lain
untuk nilai yang sama, data terpotong, sisa byte, atau klaim jumlah elemen yang mustahil wajib ditolak.
Task ini TIDAK menghitung hash apa pun (txid, wtxid, block_id, dan hash kondisi ada di TSK-M21, MOD-02).

## Konteks
- Sumber kebenaran: `docs/reference/spesifikasi-format-v0.1.md` Bagian 2 (encoding dasar), 4 (output dan kondisi),
  5 (transaksi dan witness), 6.1 dan 6.2 (header dan blok), 7 (genesis), dan 10 (batas struktur).
  Jika kartu ini dan spesifikasi berbeda, BERHENTI dan laporkan.
- Pakai tipe dari TSK-M01: `Amount`, `BlockHeight`, `Hash32`, dan `params::MAX_BLOCK_SIZE`.
- MOD-01 TIDAK boleh bergantung pada MOD-02. Kunci publik dan tanda tangan disimpan sebagai byte mentah
  (`[u8; 32]` dan `[u8; 64]`), tanpa validasi kriptografi. Validasi kriptografi ada di TSK-M09.
- Lint konsensus aktif (arithmetic_side_effects, float_arithmetic, indexing_slicing). Jangan dinonaktifkan.
  Akses slice memakai `get`, `split_at_checked`, atau iterator. Konversi angka memakai `try_from`.

## Ruang lingkup
- Boleh diubah: crates/primitives/**; status TSK-M03 di docs/tasks/README.md dan docs/modules/MOD-01-primitives.md.
- Tanpa dependensi baru. Hanya `thiserror` (sudah ada) dan dev-dependency `proptest` (sudah ada).
- Jangan diubah: modul `amount`, `height`, `hash`, `params` dari TSK-M01 (kecuali menambah re-export di lib.rs),
  crate lain, AGENTS.md, .agent/, Cargo.toml root.
- Di luar lingkup: hashing (TSK-M21), validasi yang butuh UTXO atau state (TSK-M09: kecocokan jenis witness dengan
  output yang dibelanjakan, jumlah input ≥ output, kematangan coinbase, tanda tangan), Merkle root, PoW, dan sighash.

## Persyaratan

### 1. Modul `encoding`
- `pub struct Reader<'a>` di atas `&'a [u8]`: membaca u8, u32le, u64le, array byte ukuran tetap, varint, dan
  jumlah elemen. Menyediakan `remaining()` dan `finish(self) -> Result<(), DecodeError>` (gagal jika ada sisa byte).
- Varint LEB128 tanpa tanda, **wajib minimal**, untuk u32, u64, dan u128 (Spesifikasi 2.1): panjang maksimum
  5, 10, dan 19 byte; bit yang melampaui lebar tipe wajib 0; byte terakhir tidak boleh 0x00 kecuali nilainya 0.
- Pembacaan jumlah elemen `vec<T>`: varint<64>, lalu **sebelum mengalokasikan apa pun** periksa
  `jumlah ≤ remaining() / ukuran_minimum(T)`. Ukuran minimum (Spesifikasi 10): Input 33, Output 34, Witness 65,
  SigEntry 65, kunci 32, Transaksi 46, byte 1.
- Penulisan varint dan field ukuran tetap ke `Vec<u8>`.
- `pub trait Encode { fn encode_to(&self, out: &mut Vec<u8>); fn to_bytes(&self) -> Vec<u8>; }` (to_bytes boleh default).
- `pub trait Decode: Sized { fn decode_from(r: &mut Reader<'_>) -> Result<Self, DecodeError>;
  fn from_bytes(bytes: &[u8]) -> Result<Self, DecodeError>; }`. `from_bytes` wajib memanggil `finish`.
- `pub enum DecodeError` (thiserror) dengan varian yang membedakan setidaknya: data terpotong, varint tidak minimal,
  varint melampaui lebar tipe, sisa byte, jumlah elemen mustahil (pelindung alokasi), versi tidak dikenal, tx_kind
  tidak dikenal, jenis output tidak dikenal, jenis witness tidak dikenal, amount nol, input kosong, output kosong,
  extra coinbase terlalu panjang, kondisi tidak valid, daftar tanda tangan witness tidak valid, blok terlalu besar,
  blok kosong, posisi coinbase salah, tinggi coinbase tidak sama dengan header, dan extra genesis tidak tepat 32 byte.
  Error validasi pada konstruktor boleh memakai enum yang sama atau enum terpisah.

### 2. Modul `tx`
Struktur persis seperti Spesifikasi Bagian 4 dan 5:
- `Output` { amount ≥ 1 quanta, kind 0 atau 1, payload `[u8; 32]` }
- `Condition` { version = 1 (tidak disimpan, ditulis saat encode), m, keys, lock_height }:
  1 ≤ m ≤ n ≤ 16; kunci urut naik bytewise dan tanpa duplikat.
- `Input` { prev_txid: `Hash32`, prev_index: u32 (varint<32>) }
- Badan transaksi: versi 1; jenis biasa (≥ 1 input, ≥ 1 output) atau coinbase (height u64le, extra 0–32 byte, ≥ 1 output).
- Witness: jenis 0 (tanda tangan 64 byte) atau jenis 1 (kondisi + tepat m SigEntry dengan `key_index` naik ketat dan < n).
- Transaksi = badan + witness. Transaksi biasa: tepat satu witness per input, tanpa penghitung jumlah.
  Coinbase: tanpa witness.
- Sediakan encoding **badan saja** (bahan txid) dan encoding **lengkap** (bahan wtxid) secara terpisah.

Semua tipe dibuat lewat konstruktor yang memvalidasi aturan di atas, dengan field privat dan getter.
Tujuannya: setiap nilai yang bisa dibuat lewat API publik selalu bisa di-encode dan di-decode ulang
menjadi nilai yang sama.

### 3. Modul `block`
- `Header` tepat 96 byte, urutan field persis seperti Spesifikasi 6.1. Versi 1 ditulis saat encode; versi lain
  ditolak saat decode. `pub const HEADER_SIZE: usize = 96;`. Encode ke `[u8; 96]`.
- `Block` { header, transaksi }:
  - ≥ 1 transaksi; transaksi pertama wajib coinbase; transaksi lain wajib transaksi biasa.
  - height coinbase wajib sama dengan height header.
  - Jika height header 0 (genesis): extra coinbase wajib tepat 32 byte.
  - Ukuran seluruh encoding ≤ `MAX_BLOCK_SIZE`. Saat decode, input lebih dari `MAX_BLOCK_SIZE` byte langsung
    ditolak sebelum dibaca.

## Vektor uji (dihitung Quick dengan implementasi terpisah; wajib cocok byte per byte)
Untuk setiap vektor: `to_bytes(nilai) == hex` dan `from_bytes(hex)` menghasilkan nilai yang sama.
`Q` = 10^18 quanta = 1 HSIS. `xx×32` berarti 32 byte bernilai 0xxx.

| ID | Isi | Ukuran | Hex |
|---|---|---|---|
| V1 | Transaksi biasa: 1 input (prev_txid 11×32, index 0), output 1,5 HSIS jenis 0 ke kunci `f9308a01…bce036f9`, output 0,25 HSIS jenis 0 ke 22×32, witness jenis 0 dengan tanda tangan 33×64 | 186 | `010001111111111111111111111111111111111111111111111111111111111111111100028080d8d8d7c1c4e81400f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f98080e4ceceb58bbc030022222222222222222222222222222222222222222222222222222222222222220033333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333` |
| V1b | Encoding badan saja dari V1 | 121 | 121 byte pertama V1 |
| V2 | Coinbase tinggi 1, extra kosong, 1 output 1910127426536917 quanta jenis 0 ke 44×32 | 53 | `010101000000000000000001d58bf09583a8b203004444444444444444444444444444444444444444444444444444444444444444` |
| V3 | Coinbase genesis: tinggi 0, extra ab×32, 1 output 4380000 HSIS jenis 0 ke 55×32 | 89 | `0101000000000000000020abababababababababababababababababababababababababababababababab01808080cccfadc9ccc680fe1c005555555555555555555555555555555555555555555555555555555555555555` |
| V4 | Header: tinggi 1, prev 11×32, merkle 22×32, timestamp 1800000000, target 0x1d00ffff, nonce 42 | 96 | `0100000001000000000000001111111111111111111111111111111111111111111111111111111111111111222222222222222222222222222222222222222222222222222222222222222200d2496b00000000ffff001d2a00000000000000` |
| V5 | Kondisi 2-dari-3, kunci 01×32, 02×32, 03×32, lock_height 0 | 107 | `0102030101010101010101010101010101010101010101010101010101010101010101020202020202020202020202020202020202020202020202020202020202020203030303030303030303030303030303030303030303030303030303030303030000000000000000` |
| V6 | Witness jenis 1: kondisi V5, SigEntry (indeks 0, 44×64) dan (indeks 2, 66×64) | 239 | `01` + V5 + `02` + `00` + 44×64 + `02` + 66×64 |
| V7 | Transaksi biasa: 1 input (prev_txid 99×32, index 5), 1 output 1 HSIS jenis 1 ke 77×32, witness V6 | 318 | `01000199999999999999999999999999999999999999999999999999999999999999990501808090bbbad6adf00d017777777777777777777777777777777777777777777777777777777777777777` + V6 |
| V8 | Blok: header V4, transaksi [V2, V1] | 336 | V4 + `02` + V2 + V1 |

Vektor varint: semua baris tabel di Spesifikasi Bagian 2.1 (13 nilai), untuk u32, u64, dan u128 sesuai rentangnya.

## Vektor penolakan (wajib ditolak dengan varian error yang tepat)
| ID | Input | Diharapkan |
|---|---|---|
| R1 | varint u64 `80` | data terpotong |
| R2 | varint u64 `8000` | varint tidak minimal |
| R3 | varint u32 `ffffffff1f` | varint melampaui lebar |
| R4 | varint u64 `ffffffffffffffffff02` | varint melampaui lebar |
| R5 | varint u128 `ffffffffffffffffffffffffffffffffffff04` | varint melampaui lebar |
| R6 | V1 + `00` | sisa byte |
| R7 | V1 tanpa byte terakhir | data terpotong |
| R8 | V1 dengan byte pertama `02` | versi tidak dikenal |
| R9 | V1 dengan byte kedua `02` | tx_kind tidak dikenal |
| R10 | `0100ffffffff0f` (klaim 2^32 − 1 input) | jumlah elemen mustahil, tanpa alokasi |
| R11 | `01000001808090bbbad6adf00d002222222222222222222222222222222222222222222222222222222222222222` (0 input) | input kosong |
| R12 | V2 dengan amount `00` | amount nol |
| R13 | V2 dengan kind output `02` | jenis output tidak dikenal |
| R14 | Coinbase dengan extra 33 byte | extra terlalu panjang |
| R15 | Kondisi V5 dengan kunci 01 dan 02 ditukar; m = 0; m = 4; n = 17; kunci kembar | kondisi tidak valid |
| R16 | Witness V6 dengan urutan indeks 2 lalu 0; hanya 1 SigEntry; indeks 3 | daftar tanda tangan tidak valid |
| R17 | V7 dengan byte jenis witness `02` | jenis witness tidak dikenal |
| R18 | V4 dengan versi `02000000` | versi tidak dikenal |
| R19 | Blok dengan transaksi [V1, V2] dan [V2, V2] | posisi coinbase salah |
| R20 | V8 dengan height header 2 | tinggi coinbase tidak sama |
| R21 | Blok genesis (height 0) dengan coinbase tinggi 0 dan extra kosong | extra genesis tidak tepat 32 |
| R22 | Blok tanpa transaksi (`V4 + 00`) | blok kosong |
| R23 | Input 100 001 byte (isi apa saja) ke `Block::from_bytes` | blok terlalu besar, sebelum membaca |

## Property test (proptest)
- Round-trip: untuk nilai acak yang valid dari setiap tipe, `from_bytes(to_bytes(x)) == x`.
- Kanonis: untuk byte acak yang berhasil di-decode, `to_bytes(decode(b)) == b`.
- Byte acak (sampai 2048 byte) ke `from_bytes` setiap tipe tidak pernah panic.
- Setiap prefiks yang lebih pendek dari encoding valid ditolak.
- Varint: round-trip untuk seluruh rentang u32, u64, dan u128; panjang encoding sesuai rumus; encoding tidak minimal ditolak.

## Kriteria penerimaan
- [ ] Semua vektor V1–V8 dan vektor varint cocok byte per byte
- [ ] Semua vektor R1–R23 ditolak dengan varian error yang tepat
- [ ] Semua property test lolos
- [ ] Tanpa dependensi baru; `#![forbid(unsafe_code)]` dan lint konsensus tetap aktif; tanpa unwrap/expect/panic di kode non-test
- [ ] Unit test di `src/<modul>/tests.rs`, terpisah dari kode
- [ ] `make check-all` lolos
- [ ] Status TSK-M03 di docs/tasks/README.md menjadi 🟢, dan kartu MOD-01 mencatat TSK-M03 selesai

## Verifikasi
```bash
make check-all
git diff --stat HEAD~1
```

## Laporan akhir
Format AGENTS.md. Sertakan daftar API publik dan tabel hasil V1–V8 serta R1–R23.
