# Spesifikasi Format Zevrigo Coin (ZVRG) v0.1: alamat, encoding, transaksi, dan blok terdefinisi; sighash v1 dikunci (DEC-13)

> Diperbarui: 30 Oktober 2026 · Pemilik: pencipta · Task: TSK-S01 · Status: 🟢 v0.1
> Lokasi di repo: `docs/reference/spesifikasi-format-v0.1.md`

| Status | Ringkasan |
|---|---|
| 🟢 Dikunci | Encoding biner kanonis (DEC-12), alamat bech32m dengan dua jenis output (DEC-11), struktur transaksi, header, blok, dan Merkle |
| 🟢 Diputuskan | DEC-13 (30 Oktober 2026): sighash v1 satu mode, Bagian 9. Dihitung di MOD-02 (TSK-M24) |
| 🟢 Dilanjutkan | Aturan validasi (format target, ASERT, chainwork, cap waktu, transaksi, coinbase, blok): [spesifikasi-validasi-v0.1.md](spesifikasi-validasi-v0.1.md) (TSK-S02) |
| ➡️ Langkah berikutnya | TSK-M25, TSK-M08, TSK-M09 |

Legenda: 🟢 dikunci · 🟡 draf · ❓ perlu keputusan

---

## 1. Prinsip

1. **Satu nilai, satu encoding.** Setiap struktur hanya punya satu representasi byte yang sah. Encoding lain untuk nilai yang sama wajib ditolak.
2. **Konsensus tidak bergantung pada pustaka serialisasi.** Semua aturan ada di dokumen ini dan ditulis tangan di MOD-01.
3. **Tanpa float.** Semua angka berupa bilangan bulat tak bertanda.
4. **Pemisah domain di setiap hash.** Hash untuk keperluan berbeda tidak pernah bisa tertukar.
5. **Tanda tangan tidak memengaruhi ID transaksi.** txid tetap sama sebelum dan sesudah ditandatangani.

---

## 2. Encoding dasar

| Tipe | Encoding | Ukuran |
|---|---|---|
| `u8` | 1 byte | 1 |
| `u32le`, `u64le` | little-endian, ukuran tetap | 4, 8 |
| `bytes32` | 32 byte apa adanya | 32 |
| `varint<N>` | unsigned LEB128, wajib minimal, untuk nilai `uN` | 1 sampai ⌈N/7⌉ |
| `amount` | `varint<128>` dalam quanta | 1 sampai 19 |
| `vec<T>` | `varint<64>` jumlah elemen, lalu elemen berurutan | — |

### 2.1 Aturan varint (LEB128 minimal)
- Setiap byte memuat 7 bit nilai, dari bit terendah. Bit tertinggi (0x80) bernilai 1 jika masih ada byte berikutnya.
- **Wajib minimal:** byte terakhir tidak boleh `0x00`, kecuali nilainya 0 dan hanya satu byte. Contoh: `80 00` untuk nilai 0 **ditolak**.
- Panjang maksimum: 5 byte untuk u32, 10 byte untuk u64, 19 byte untuk u128. Bit di byte terakhir yang melampaui lebar tipe wajib 0, jika tidak maka ditolak.

**Vektor uji varint (final):**

| Nilai | Byte (hex) | Panjang |
|---|---|---|
| 0 | `00` | 1 |
| 1 | `01` | 1 |
| 127 | `7f` | 1 |
| 128 | `8001` | 2 |
| 255 | `ff01` | 2 |
| 300 | `ac02` | 2 |
| 16 383 | `ff7f` | 2 |
| 16 384 | `808001` | 3 |
| 2³² − 1 | `ffffffff0f` | 5 |
| 10¹⁸ (1 ZVRG) | `808090bbbad6adf00d` | 9 |
| 1,5 × 10¹⁸ (1,5 ZVRG) | `8080d8d8d7c1c4e814` | 9 |
| 2⁶⁴ − 1 | `ffffffffffffffffff01` | 10 |
| 2¹²⁸ − 1 | `ffffffffffffffffffffffffffffffffffff03` | 19 |

Contoh yang wajib ditolak: `80` (terpotong), `8000` (tidak minimal), `ffffffff1f` sebagai u32 (melampaui 32 bit).

### 2.2 Aturan decode
- **Setiap byte wajib terbaca habis.** Ada sisa byte di akhir struktur berarti ditolak.
- **Perlindungan alokasi:** sebelum membaca `vec<T>`, jumlah elemen wajib ≤ sisa byte ÷ ukuran minimum T. Jika tidak, langsung ditolak tanpa mengalokasikan memori.
- Data terpotong, nilai di luar rentang, atau versi dan jenis yang tidak dikenal berarti ditolak.

---

## 3. Hash dan pemisah domain

Semua hash memakai **BLAKE3 mode derive_key** dengan string konteks tetap, menghasilkan 32 byte:
`H(konteks, data) = BLAKE3-derive_key(konteks, data)`

| Kegunaan | String konteks |
|---|---|
| ID transaksi (txid) | `zevrigo 2026-10-28 txid` |
| ID transaksi lengkap (wtxid) | `zevrigo 2026-10-28 wtxid` |
| Daun Merkle | `zevrigo 2026-10-28 merkle leaf` |
| Simpul Merkle | `zevrigo 2026-10-28 merkle node` |
| ID blok | `zevrigo 2026-10-28 block id` |
| Hash kondisi output jenis 1 | `zevrigo 2026-10-28 condition` |
| Sighash (lihat Bagian 9) | `zevrigo 2026-10-28 sighash v1` |
| PoW dev regtest (DEC-09) | `zevrigo 2026-10-28 regtest pow` |

- String konteks ditulis persis seperti di tabel (ASCII, huruf kecil, satu spasi).
- **Tampilan hex** memakai urutan byte apa adanya, huruf kecil. Tidak ada pembalikan byte seperti di Bitcoin.

---

## 4. Output

```
Output {
  amount:  amount     // ≥ 1 quanta
  kind:    u8         // 0 atau 1
  payload: bytes32
}
```

| `kind` | Nama | `payload` | Cara membelanjakan |
|---|---|---|---|
| 0 | Kunci tunggal | Kunci publik x-only BIP340 (32 byte). Termasuk kunci gabungan MuSig2 untuk n-dari-n | Satu tanda tangan Schnorr |
| 1 | Kondisi | `H(condition, encode(Condition))` | Ungkapkan kondisi + m tanda tangan, dan kunci waktu terpenuhi |

- Output dengan `amount = 0` ditolak.
- Kunci di `payload` tidak diperiksa saat output dibuat. Kunci yang tidak valid membuat output tidak bisa dibelanjakan (koin hangus).
- Ukuran output: 34 sampai 52 byte. Jumlah koin yang umum (0,01 sampai 100 ZVRG) menghasilkan output 42 byte.

### 4.1 Kondisi (output jenis 1)

```
Condition {
  version:     u8          // = 1
  m:           u8          // 1 ≤ m ≤ n
  keys:        vec<bytes32> // n kunci x-only, 1 ≤ n ≤ 16
  lock_height: u64le       // 0 = tanpa kunci waktu
}
```
Aturan kanonis:
- `keys` wajib **urut naik secara bytewise dan tanpa duplikat**. Satu set kunci hanya punya satu hash kondisi.
- Kunci waktu terpenuhi jika tinggi blok yang memuat transaksi ≥ `lock_height`.
- Kunci tunggal dengan kunci waktu ditulis sebagai m = 1, n = 1.
- Ukuran: kondisi 2-dari-3 = 107 byte.

---

## 5. Transaksi

Setiap transaksi terdiri dari **badan** (masuk ke txid) dan **witness** (tanda tangan, tidak masuk ke txid).

### 5.1 Badan

```
TxBody {
  version: u8        // = 1
  tx_kind: u8        // 0 = biasa, 1 = coinbase
  jika tx_kind = 0:
    inputs:  vec<Input>    // ≥ 1
    outputs: vec<Output>   // ≥ 1
  jika tx_kind = 1:
    height:  u64le         // wajib sama dengan tinggi di header
    extra:   vec<u8>       // 0–32 byte (extra nonce penambang; di genesis: jangkar Bitcoin)
    outputs: vec<Output>   // ≥ 1
}

Input {
  prev_txid:  bytes32
  prev_index: varint<32>
}
```

### 5.2 Witness

Hanya ada pada transaksi biasa. Witness ditulis langsung setelah badan, **satu witness per input, tanpa penghitung jumlah**, dengan urutan sama seperti input.

```
Witness {
  kind: u8                       // wajib sama dengan kind output yang dibelanjakan (diperiksa di validasi)
  jika kind = 0:
    sig: bytes64                 // tanda tangan BIP340
  jika kind = 1:
    condition: Condition         // diungkapkan; hash-nya wajib sama dengan payload output
    sigs: vec<SigEntry>          // tepat m entri
}

SigEntry {
  key_index: u8                  // < n, urut naik ketat (tanpa duplikat)
  sig:       bytes64
}
```

### 5.3 ID transaksi
- `txid = H(txid, encode(TxBody))`. Tanda tangan tidak memengaruhi txid.
- `wtxid = H(wtxid, encode(TxBody) ‖ encode(semua Witness))`. Untuk coinbase, witness kosong.
- Input merujuk output lewat **txid**. Merkle root di header memakai **wtxid**, sehingga blok tetap mencakup tanda tangan.
- Karena coinbase memuat `height`, tidak mungkin ada dua coinbase dengan txid yang sama.

### 5.4 Contoh byte: transaksi biasa (1 input, 2 output)

Isi: belanjakan output 0 dari txid `1111…11`, kirim 1,5 ZVRG ke kunci contoh BIP340, dan 0,25 ZVRG kembalian ke kunci `2222…22`. Tanda tangan diisi `3333…33` sebagai pengganti.

| Offset | Byte | Arti |
|---|---|---|
| 0x00 | `01` | version = 1 |
| 0x01 | `00` | tx_kind = biasa |
| 0x02 | `01` | jumlah input = 1 |
| 0x03 | `11` × 32 | prev_txid |
| 0x23 | `00` | prev_index = 0 |
| 0x24 | `02` | jumlah output = 2 |
| 0x25 | `8080d8d8d7c1c4e814` | amount = 1 500 000 000 000 000 000 quanta (1,5 ZVRG) |
| 0x2e | `00` | kind = 0 |
| 0x2f | `f9308a01…bce036f9` | payload (kunci contoh BIP340, 32 byte) |
| 0x4f | `8080e4ceceb58bbc03` | amount = 250 000 000 000 000 000 quanta (0,25 ZVRG) |
| 0x58 | `00` | kind = 0 |
| 0x59 | `22` × 32 | payload |
| 0x79 | `00` | witness input 0: kind = 0 |
| 0x7a | `33` × 64 | tanda tangan (pengganti) |

**Ukuran: badan 121 byte + witness 65 byte = 186 byte.**

### 5.5 Contoh byte: coinbase di tinggi 1

```
01                          version = 1
01                          tx_kind = coinbase
0100000000000000            height = 1 (u64le)
00                          extra kosong
01                          jumlah output = 1
d58bf09583a8b203            amount = 1 910 127 426 536 917 quanta (vektor V2; subsidy tinggi 1 sebelum DEC-30)
00                          kind = 0
44 × 32                     payload
```
**Ukuran: 53 byte.**

### 5.6 Ukuran lain

| Bentuk | Ukuran |
|---|---|
| Witness 2-dari-3 (kondisi + 2 tanda tangan) | 239 byte |
| Coinbase genesis (extra 32 byte + 1 output alokasi pencipta) | 89 byte |

---

## 6. Header dan blok

### 6.1 Header (tepat 96 byte)

| Offset | Field | Tipe | Arti |
|---|---|---|---|
| 0 | version | u32le | = 1 |
| 4 | height | u64le | tinggi blok |
| 12 | prev_block_id | bytes32 | ID blok sebelumnya (genesis: nol semua) |
| 44 | merkle_root | bytes32 | Merkle root dari wtxid |
| 76 | timestamp | u64le | detik Unix |
| 84 | target | u32le | target kesulitan (format compact; rincian di Spesifikasi Validasi Bagian 3) |
| 88 | nonce | u64le | nonce penambang |

- `block_id = H(block id, header)`. Dipakai untuk identifikasi dan sebagai `prev_block_id`.
- Hash PoW dihitung terpisah oleh MOD-03 dari 96 byte header:
  - **Regtest (DEC-09):** `H(regtest pow, header)`, dibaca sebagai bilangan 256-bit big-endian, wajib ≤ target. Ditolak di testnet dan mainnet.
  - **Testnet dan mainnet:** menunggu DEC-01.
- Pertumbuhan header: 96 byte × 262 800 blok ≈ 25,2 MB per tahun.

### 6.2 Blok

```
Block {
  header:  Header (96 byte)
  txs:     vec<Transaction>   // ≥ 1; tx pertama wajib coinbase, sisanya wajib transaksi biasa
}
```
- Ukuran **seluruh encoding blok** ≤ 100 000 byte.
- Kapasitas: sekitar **536 transaksi biasa** per blok (≈ 4,47 transaksi per detik), naik dari perkiraan awal ±400.

### 6.3 Merkle tree
- Daun: `leaf_i = H(merkle leaf, wtxid_i)`.
- Simpul: `node = H(merkle node, kiri ‖ kanan)`.
- Jika jumlah simpul di suatu tingkat ganjil, simpul terakhir **dinaikkan apa adanya ke tingkat berikutnya, tidak digandakan**. Ini menghindari celah yang pernah terjadi di Bitcoin (CVE-2012-2459).
- Blok dengan satu transaksi: root = daun transaksi itu.

---

## 7. Genesis

- Header: height 0, `prev_block_id` = 32 byte nol, `target` = kesulitan awal (dari testnet, Dokumen Konsep 5.12).
- Coinbase genesis:
  - `height = 0`
  - `extra` = **tepat 32 byte hash blok Bitcoin jangkar (L2)**, dengan urutan byte sama seperti hex yang ditampilkan block explorer Bitcoin
  - satu output: `amount` = 10 000 000 ZVRG (`808080d084a9808a96aa9642`, 12 byte, DEC-30), `kind = 0`, `payload` = kunci alokasi pencipta (TODO(param))
- Output genesis **masuk ke set UTXO dan bisa dibelanjakan** setelah kematangan coinbase (100 blok). Ini berbeda dengan Bitcoin.
- **ID jaringan** = `block_id` genesis. Testnet yang di-reset otomatis mendapat ID baru.

---

## 8. Alamat (bech32m, BIP350)

`alamat = bech32m(HRP, [kind] ‖ convertbits(payload, 8→5))`

| Jaringan | HRP | Contoh jenis 0 (`…1q…`) | Panjang |
|---|---|---|---|
| Mainnet | `zvrg` | `zvrg1qlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmush8h94g` | 64 |
| Testnet | `tzvrg` | `tzvrg1qlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmusgxrxjr` | 65 |
| Regtest | `rzvrg` | `rzvrg1qlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmusf2k5zq` | 65 |

Contoh jenis 1 (`…1p…`), payload sama: `zvrg1plycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmusgv8qgk`

- Payload contoh: kunci publik vektor uji BIP340 #0 (`f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9`).
- Konstanta checksum bech32m diverifikasi dengan vektor resmi BIP350 (`a1lqfn3a`).
- Aturan decode: huruf kecil saja (huruf besar semua juga boleh, tapi tidak boleh campuran), HRP wajib sesuai jaringan node, `kind` wajib 0 atau 1, payload tepat 32 byte, dan bit sisa (padding) wajib nol.
- Alamat bukan bagian konsensus. Chain hanya menyimpan `kind` dan `payload`.

---

## 9. Sighash v1 🟢 (DEC-13, diputuskan 30 Oktober 2026)

Satu mode. Tanda tangan untuk input ke-`input_index` adalah BIP340 atas pesan 32 byte berikut:
```
msg = H(sighash v1,
        network_id            // block_id genesis, 32 byte
      ‖ txid                  // 32 byte (badan saja, tanpa witness)
      ‖ input_index           // u32le
      ‖ encode(Output) dari output yang dibelanjakan input 0, 1, …, n−1, berurutan)
```
- `encode(Output)` adalah encoding kanonis Bagian 4: `amount ‖ kind ‖ payload`.
- Coinbase tidak punya input, jadi tidak punya sighash.
- Output jenis 1: setiap tanda tangan dari m kunci menandatangani `msg` yang sama untuk input itu.
- Dihitung di MOD-02 (`sighash_v1`, TSK-M24), karena MOD-04 (validasi) dan MOD-09 (wallet) sama-sama memakainya.
- **Melindungi dari:** pemutaran ulang lintas jaringan (network_id), wallet yang dibohongi soal jumlah input atau biaya (output semua input ikut ditandatangani), dan tanda tangan yang dipindah ke input lain (input_index).
- **Tidak mendukung:** mode parsial seperti SIGHASH_NONE, SINGLE, atau ANYONECANPAY. Jika dibutuhkan, ditambahkan lewat transaksi `version = 2` dengan domain baru.
- Vektor uji: S1–S3 di `crates/crypto/src/sighash/tests.rs`.

---

## 10. Ringkasan batas struktur (tingkat decode)

| Aturan | Nilai |
|---|---|
| `version` transaksi | hanya 1 |
| `tx_kind` | 0 atau 1; coinbase hanya sebagai transaksi pertama di blok |
| Input dan output transaksi biasa | masing-masing ≥ 1 |
| `prev_index` | ≤ 2³² − 1 |
| `amount` | ≥ 1 quanta |
| `kind` output dan witness | 0 atau 1 |
| Kondisi | `version` = 1, 1 ≤ m ≤ n ≤ 16, kunci urut naik ketat |
| `sigs` witness jenis 1 | tepat m entri, `key_index` urut naik ketat dan < n |
| `extra` coinbase | 0–32 byte; genesis tepat 32 |
| Ukuran blok | ≤ 100 000 byte (seluruh encoding) |
| Ukuran minimum elemen untuk pemeriksaan alokasi | Input 33 · Output 34 · Witness 65 · SigEntry 65 · kunci 32 · Transaksi 46 (coinbase terkecil) |

---

## 11. Aturan validasi (TSK-S02)

Ditulis di [spesifikasi-validasi-v0.1.md](spesifikasi-validasi-v0.1.md). Koreksi: jumlah output coinbase wajib **tepat** subsidy + total biaya (DEC-17), bukan ≤. Genesis: tepat G.

---

## 12. Vektor uji

| Vektor | Status |
|---|---|
| Varint (Bagian 2.1) | 🟢 final |
| Alamat bech32m (Bagian 8) | 🟢 final |
| Susunan byte transaksi, coinbase, dan header (Bagian 5–6) | 🟢 final |
| Hash (txid, wtxid, Merkle, block_id) | ⚪ dihasilkan di TSK-M03, lalu dicocokkan dengan implementasi kedua yang independen (misalnya skrip Python dengan paket `blake3` di WSL) |

---

## 13. Keputusan

| ID | Keputusan | Status |
|---|---|---|
| DEC-11 | Alamat bech32m dengan HRP per jaringan; dua jenis output: 0 = kunci tunggal/MuSig2, 1 = kondisi m-dari-n dengan kunci waktu | 🟢 diputuskan 28 Oktober 2026 |
| DEC-12 | Encoding biner tulis sendiri, kanonis, varint LEB128 minimal, setiap byte wajib terbaca habis | 🟢 diputuskan 28 Oktober 2026 |
| DEC-13 | Sighash v1 satu mode (Bagian 9) | 🟢 diputuskan 30 Oktober 2026 |

**Koreksi atas Dokumen Aplikasi v0.1:** MuSig2 hanya untuk n-dari-n. Multi-tanda tangan m-dari-n memakai output jenis 1 (DEC-11).
