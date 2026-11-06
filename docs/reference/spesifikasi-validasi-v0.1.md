# Spesifikasi Validasi Zevrigo Coin (ZVRG) v0.1: target, ASERT, chainwork, cap waktu, transaksi, coinbase, dan blok

> Diperbarui: 30 Oktober 2026 · Pemilik: pencipta · Ditulis: Quick · Task: TSK-S02 · Status: 🟢 v0.1
> Lokasi di repo: `docs/reference/spesifikasi-validasi-v0.1.md` · Melengkapi `spesifikasi-format-v0.1.md`

| Status | Ringkasan |
|---|---|
| 🟢 Dikunci | Format target compact (Bagian 3), ASERT bilangan bulat (4), chainwork (5), cap waktu (6), aturan transaksi (7), coinbase (8), urutan validasi blok (9), genesis regtest (10) |
| 🟢 Diputuskan | DEC-17: coinbase wajib mengklaim **tepat** subsidy + total biaya (tanpa pembakaran) |
| ⏳ Menunggu | Cap waktu genesis testnet (saat peluncuran, TSK-F06). Genesis, `pow_limit`, dan target awal mainnet (`TODO(param)`) |
| ➡️ Langkah berikutnya | TSK-M25 (`Network` di MOD-01), TSK-M08 (Bagian 3–6 dan 10), lalu TSK-M09 (Bagian 7–9) |

Legenda: 🟢 dikunci · 🟡 draf · ⏳ menunggu keputusan atau parameter testnet

---

## 1. Prinsip

1. **Sah atau tidak sah adalah fungsi pasti** dari isi blok, leluhurnya, dan parameter jaringan. Satu-satunya pengecualian adalah batas waktu masa depan (V8), yang hanya *menunda* blok, tidak pernah menandainya tidak sah.
2. **Semua angka bilangan bulat.** Pembagian selalu dibulatkan ke bawah (floor, ke arah −∞). Tidak ada float.
3. **Murah dulu, mahal belakangan.** Pemeriksaan disusun supaya blok palsu ditolak sebelum node mengerjakan hal yang mahal (tanda tangan). Urutan ini tidak mengubah hasil: blok sah hanya jika semua aturan terpenuhi.
4. **Semua aturan di MOD-04.** MOD-02 (sighash, Schnorr, ID) dan MOD-03 (hash PoW) hanya menyediakan fungsi. Keputusan sah atau tidak ada di MOD-04.
5. **Varian error hanya untuk diagnosis.** Yang menjadi konsensus adalah hasil sah/tidak sah. Test di MOD-04 tetap memeriksa varian error sesuai urutan di Bagian 9.

---

## 2. Parameter per jaringan

| Parameter | Regtest | Testnet | Mainnet |
|---|---|---|---|
| Fungsi PoW | `regtest_pow_hash` (DEC-09) | RandomX dengan kunci (DEC-01, V3) | RandomX dengan kunci (DEC-01, V3) |
| Salt Argon2 RandomX | — | `RandomHSIS\x01` (11 byte) | `RandomHSIS\x01` (11 byte) |
| `pow_limit` (compact) | `0x207fffff` | `0x1f369d03` (kerja 1_200, sekitar 10 hash/detik) | `TODO(param)` |
| Target genesis (compact) | `0x207fffff` | `0x1e8bcf64` (kerja 120_000, sekitar 1_000 hash/detik) | `TODO(param)` |
| Genesis | Bagian 10 (tetap) | Bagian 10b (cap waktu saat peluncuran) | `TODO(param)` |
| `network_id` | `b933d7c5…6a04c5` (Bagian 10) | block_id genesis | block_id genesis |

- `pow_limit` adalah target terbesar (kesulitan terendah) yang boleh dipakai di jaringan itu. Target genesis wajib ≤ `pow_limit`.
- Regtest memakai aturan yang sama persis dengan mainnet, termasuk ASERT. Yang berbeda hanya fungsi PoW dan parameter di tabel ini.
- Testnet dan mainnet memakai RandomX (DEC-01): kunci blok setinggi `h` adalah `block_id` leluhurnya di `key_height(h)` pada chain blok itu sendiri: 0 (genesis, yaitu `network_id`) sampai `h = 2_112`, sesudahnya kelipatan 2_048 terbesar yang ≤ `h − 65`. Tanpa penghitung RandomX, header testnet dan mainnet ditolak (`PowUnavailable`). Hash PoW regtest **tidak pernah** diterima di testnet dan mainnet (DEC-09), dan ini wajib dibuktikan dengan test.

---

## 3. Target compact (`target` di header, `u32le`)

Target adalah bilangan bulat 256-bit `1 ≤ T < 2^256`. Header menyimpannya dalam 4 byte:
`E = bit 24–31` (jumlah byte target), `M = bit 0–23` (tiga byte teratas target). **Tanpa bit tanda** (berbeda dengan Bitcoin).

**Decode** (`u32 → T`), sah hanya jika semua syarat terpenuhi:
- `1 ≤ E ≤ 32` dan `M ≥ 0x010000` (byte teratas mantisa bukan nol).
- Jika `E ≥ 3`: `T = M × 2^(8 × (E − 3))`.
- Jika `E < 3`: `T = M ÷ 2^(8 × (3 − E))`, dan bit yang terbuang wajib nol.

**Encode** (`T → u32`): `n` = jumlah byte `T` tanpa nol di depan (1 sampai 32). Jika `n ≥ 3`, `M = T ÷ 2^(8 × (n − 3))` (dibulatkan ke bawah); jika `n < 3`, `M = T × 2^(8 × (3 − n))`. Hasil `(n << 24) | M`.

Akibatnya setiap target hanya punya satu encoding (`encode(decode(c)) = c` untuk setiap `c` yang sah), dan `decode(encode(T)) ≤ T` dengan selisih kurang dari `T ÷ 2^16`.

**Vektor (encode lalu decode):**

| ID | T | compact | decode(compact) |
|---|---|---|---|
| K1 | `0x01` | `0x01010000` | `0x01` |
| K2 | `0xff` | `0x01ff0000` | `0xff` |
| K3 | `0x0100` | `0x02010000` | `0x0100` |
| K4 | `0x1234` | `0x02123400` | `0x1234` |
| K5 | `0x123456` | `0x03123456` | `0x123456` |
| K6 | `0x12345678` | `0x04123456` | `0x12345600` (dibulatkan ke bawah) |
| K7 | `2^255` | `0x20800000` | `2^255` |
| K8 | `2^256 − 1` | `0x20ffffff` | `0xffffff` diikuti 29 byte nol |
| K9 | `0x7fffff × 2^232` | `0x207fffff` | sama (= `pow_limit` regtest) |

**Wajib ditolak saat decode:** `0x00000000` (E = 0), `0x00ffffff` (E = 0), `0x21010000` (E = 33), `0x0300ffff` dan `0x03000001` (M < 0x010000), `0x01010001` dan `0x02010001` (bit terbuang tidak nol).

---

## 4. ASERT (penyesuaian kesulitan per blok)

Blok acuan adalah genesis (tinggi 0): `A` = decode(target genesis), `t0` = cap waktu genesis. Untuk blok di tinggi `h ≥ 1` dengan induk di tinggi `p = h − 1` dan cap waktu induk `tp`:

```
time_delta = tp − t0                                   // i128
exponent   = floor((time_delta − 120 × p) × 65536 / 86400)   // 86400 = 720 blok × 120 detik
shifts     = floor(exponent / 65536)
frac       = exponent − shifts × 65536                 // 0 ≤ frac ≤ 65535
factor     = 65536 + ((195766423245049 × frac
                     + 971821376 × frac²
                     + 5127 × frac³
                     + 2^47) >> 48)                    // u128; 65536 ≤ factor ≤ 131071
next       = floor(A × factor × 2^(shifts − 16))       // dihitung tepat, tanpa luapan diam-diam
jika next = 0          → next = 1
jika next > pow_limit  → next = pow_limit
target_wajib(h) = encode(next)                         // Bagian 3
```

- Header blok `h` wajib memuat `target = target_wajib(h)` persis (perbandingan `u32`).
- Blok 1 selalu mendapat target genesis (`time_delta = 0`, `p = 0`). Chain yang tepat jadwal mempertahankan target genesis.
- ASERT tidak iteratif: hasilnya hanya bergantung pada genesis dan induk, sehingga galat pembulatan tidak menumpuk.
- Polinomial `factor` sama dengan aserti3-2d Bitcoin Cash (pendekatan `2^frac`, galat < 0,013%). Bedanya: blok acuan genesis itu sendiri (tanpa `+1` di suku tinggi), pembagian floor, dan target 256-bit penuh.
- Implementasi: `A × factor` bisa sampai 273 bit dan `shifts` bisa sangat besar atau sangat kecil. Hasilnya wajib sama dengan rumus di atas yang dihitung dengan bilangan tak terbatas: geser ke kanan yang menghabiskan semua bit memberi 0 (lalu 1), dan geser ke kiri yang melewati `pow_limit` memberi `pow_limit`.

**Vektor** (`t0 = 1_793_145_600`, `pow_limit = 0x207fffff`, anchor = target genesis):

| ID | anchor | `p` | `time_delta` | exponent | target_wajib |
|---|---|---|---|---|---|
| A1 | `0x1e0fffff` | 0 | 0 | 0 | `0x1e0fffff` |
| A2 | `0x1e0fffff` | 1_000 | 120_000 | 0 | `0x1e0fffff` |
| A3 | `0x1e0fffff` | 720 | 172_800 | 65_536 | `0x1e1ffffe` (×2) |
| A4 | `0x1e0fffff` | 720 | 0 | −65_536 | `0x1e07ffff` (÷2) |
| A5 | `0x1e0fffff` | 1 | 43_320 | 32_768 | `0x1e16a01e` (×≈√2) |
| A6 | `0x1e0fffff` | 100 | 11_999 | −1 | `0x1e0ffff7` |
| A7 | `0x207fffff` | 10 | 260_400 | 196_608 | `0x207fffff` (dibatasi `pow_limit`) |
| A8 | `0x1e0fffff` | 1_000_000 | 0 | −91_022_223 | `0x01010000` (dibatasi 1) |
| A9 | `0x1e0fffff` | 12_345 | 1_482_177 | 589 | `0x1e1019ae` |
| A10 | `0x1e0fffff` | 12_345 | 1_480_623 | −590 | `0x1e0fe697` |
| A11 | `0x1d7fffff` | 262_800 | 31_539_600 | 2_730 | `0x1d83c27e` |

Vektor dihitung Quick dengan referensi Python bilangan tak terbatas (`ai_advisory_hub/alat/asert_ref.py`).

---

## 5. Chainwork dan pemilihan chain

- Kerja satu blok: `work(T) = floor(2^256 / (T + 1))`, dengan `T` = decode(target header).
- Chainwork sebuah blok = jumlah `work` semua blok dari genesis sampai blok itu, **termasuk genesis**. Disimpan sebagai bilangan 256-bit; penjumlahan memakai checked (luapan berarti tidak sah; secara fisik mustahil).
- **Pemilihan chain:** node mengikuti chain sah dengan chainwork terbesar. Tanpa batas kedalaman reorg, tanpa checkpoint. Jika chainwork sama, node tetap di ujung yang sudah dipakai (kebijakan node, bukan konsensus).

| ID | target | `work` |
|---|---|---|
| W1 | `0x207fffff` | 2 |
| W2 | `0x1e0fffff` | 1_048_577 (`0x100001`) |
| W3 | `0x01010000` (T = 1) | `2^255` |
| W4 | `0x20ffffff` | 1 |
| W5 | `0x1d7fffff` | 33_554_436 |

Contoh: genesis regtest ditambah `n` blok di `pow_limit` memiliki chainwork `2 × (n + 1)`.

---

## 6. Cap waktu

**V7 (median, sah/tidak sah):** untuk blok `h ≥ 1`, ambil cap waktu `k = min(h, 11)` blok terakhir sebelum `h` (tinggi `h − k` sampai `h − 1`), urutkan naik, `MTP = urutan[k ÷ 2]`. Wajib `timestamp(h) > MTP`.

| ID | Cap waktu sebelum `h` (lama → baru) | MTP |
|---|---|---|
| M1 | `[100]` | 100 |
| M2 | `[100, 90]` | 100 |
| M3 | `[5, 1, 9, 3, 7, 11, 2, 8, 6, 10, 4]` | 6 |
| M4 | `[0, 5, 1, 9, 3, 7, 11, 2, 8, 6, 10, 4]` (hanya 11 terakhir) | 6 |

Akibatnya setiap blok `h ≥ 1` punya cap waktu lebih besar dari genesis, sehingga `time_delta` di ASERT selalu positif untuk `p ≥ 1`.

**V8 (masa depan, ditunda):** wajib `timestamp ≤ now + 3_600`, dengan `now` = jam lokal node dalam detik Unix (tanpa penyesuaian dari peer). MOD-04 tidak membaca jam sistem: `now` diberikan pemanggil (MOD-11). Blok yang gagal V8 **tidak** ditandai tidak sah dan boleh diperiksa ulang nanti. V8 hanya dipakai saat menerima blok baru, tidak saat memeriksa ulang chain yang sudah tersimpan.

---

## 7. Transaksi biasa

Diperiksa berurutan sesuai posisinya di blok `h`, terhadap **view UTXO**: set UTXO di ujung induk, ditambah output dari transaksi sebelumnya di blok yang sama, dikurangi yang sudah dibelanjakan sebelumnya di blok yang sama.

| # | Aturan |
|---|---|
| T1 | Setiap input merujuk outpoint yang ada di view. Tidak ada → tidak sah. Ini sekaligus menolak pembelanjaan ganda di dalam satu transaksi, di dalam satu blok, dan output dari transaksi yang posisinya sama atau lebih belakang |
| T2 | Kematangan: jika entri berasal dari coinbase, wajib `h ≥ tinggi_entri + 100`. Berlaku juga untuk output genesis (bisa dibelanjakan mulai blok 100) |
| T3 | `kind` witness wajib sama dengan `kind` output yang dibelanjakan |
| T4 | Jenis 1: `condition_hash(kondisi yang diungkapkan) = payload`, dan `h ≥ lock_height` |
| T5 | `Σ input` dan `Σ output` dihitung checked (luapan → tidak sah). Wajib `Σ input ≥ Σ output`. Biaya = selisihnya |
| T6 | Tanda tangan, terakhir karena paling mahal. `msg_i = sighash_v1(network_id, tx, i, output yang dibelanjakan semua input)`. Jenis 0: `payload` wajib kunci x-only sah dan `verify(payload, msg_i, sig)` benar. Jenis 1: untuk setiap `SigEntry`, `keys[key_index]` wajib kunci sah dan `verify(keys[key_index], msg_i, sig)` benar. Kunci yang tidak dirujuk tidak diperiksa |

- Setelah lolos, input dihapus dari view dan setiap output ditambahkan sebagai `UtxoEntry { tinggi h, is_coinbase = false }` dengan outpoint `(txid, indeks)`.
- Tidak ada biaya minimum, batas dust, atau batas jumlah tanda tangan di konsensus. Jumlah tanda tangan sudah dibatasi ukuran blok (paling banyak sekitar 1_536 per blok).
- Urutan transaksi di blok bebas, selama transaksi yang memakai output transaksi lain berada setelahnya.
- Aturan seperti BIP30 Bitcoin tidak diperlukan: coinbase memuat tinggi (txid unik), dan transaksi biasa dengan badan sama pasti membelanjakan input yang sama (T1).

---

## 8. Coinbase dan pasokan

| # | Aturan |
|---|---|
| C1 | Untuk `h ≥ 1`: `Σ output coinbase = block_subsidy(h, M_h) + Σ biaya semua transaksi di blok`, **tepat sama** (DEC-17). Penjumlahan checked |
| C2 | `M_h` = jumlah subsidy tinggi 1 sampai `h − 1` = `mined_total_after(h − 1)` dari `SupplySchedule` (TSK-M07). Karena C1 mewajibkan klaim penuh, `M_h` hanya bergantung pada tinggi (P4) |
| C3 | Output coinbase ditambahkan sebagai `UtxoEntry { tinggi h, is_coinbase = true }` dan terkena T2 |
| C4 | Struktur coinbase (`height` = tinggi header, `extra` 0–32 byte, ≥ 1 output, setiap output ≥ 1 quanta) sudah diperiksa decode (Spesifikasi Format Bagian 5 dan 10). `kind` output coinbase boleh 0 atau 1 |

**DEC-17 (alasan):** AGENTS.md menetapkan biaya seluruhnya untuk penambang tanpa pembakaran, dan Prinsip #3 menetapkan pasokan terprediksi. Aturan Bitcoin (`≤`) membuat koin yang tidak diklaim hangus dan membuat jumlah koin beredar bergantung pada perilaku penambang. Dengan `=`, total koin setelah blok `h` selalu `G + mined_total_after(h)`. Ini mengoreksi Spesifikasi Format Bagian 11 yang masih menulis `≤`.

Genesis tidak divalidasi dengan aturan ini. Node memuat genesis jaringannya sebagai konstanta dan hanya menerima genesis dengan `block_id` yang sama (Bagian 10). Output genesis: tepat satu, `amount = G`, `is_coinbase = true`.

---

## 9. Urutan validasi blok

**Tahap 1, bebas konteks** (cukup blok dan parameter jaringan):

| # | Aturan |
|---|---|
| V1 | Decode kanonis `Block::from_bytes` lolos, termasuk ukuran ≤ 100_000 byte dan semua batas struktur Spesifikasi Format Bagian 10 |
| V2 | `target` header sah menurut decode Bagian 3 dan ≤ `pow_limit` jaringan |
| V3 | PoW: hash PoW jaringan atas 96 byte header ≤ decode(`target`). Regtest: `regtest_pow_hash`. Testnet/mainnet: hash RandomX dengan kunci Bagian 2 (`check_header_pow`); hash yang gagal dihitung (`PowHashFailed`) bukan tanda blok tidak sah |
| V4 | `merkle_root` header = `block_merkle_root(block)` |

**Tahap 2, konteks header** (induk dan leluhur sudah dikenal dan sah):

| # | Aturan |
|---|---|
| V5 | `prev_block_id = block_id(induk)` dan `height = height(induk) + 1` |
| V6 | `target = target_wajib(h)` (Bagian 4) |
| V7 | `timestamp > MTP` (Bagian 6) |
| V8 | `timestamp ≤ now + 3_600` (Bagian 6, ditunda, bukan tidak sah) |

**Tahap 3, konteks UTXO** (induk adalah ujung view):

| # | Aturan |
|---|---|
| V9 | Setiap transaksi biasa lolos T1–T6 (Bagian 7), berurutan. Pemeriksaan tanda tangan (T6) boleh dikumpulkan dan dijalankan setelah T1–T5 semua transaksi |
| V10 | Coinbase lolos C1 (Bagian 8) |

Blok yang induknya belum dikenal bukan tidak sah; penanganannya (orphan) urusan node. Blok yang lolos V1–V10 diterapkan ke penyimpanan (`ChainStore::apply_block`). Reorg memakai `undo_tip` lalu menerapkan blok chain baru, dan setiap blok chain baru tetap diperiksa V1–V10.

---

## 10. Genesis regtest (tetap)

| Field | Nilai |
|---|---|
| Header | version 1, height 0, `prev_block_id` nol, `timestamp = 1_793_145_600` (28 Oktober 2026 00.00 UTC), `target = 0x207fffff`, `nonce = 0` |
| Coinbase | height 0, `extra` = 32 byte nol, satu output `amount = G` (4_380_000 ZVRG), `kind = 0`, `payload` = kunci x-only dari kunci rahasia `1`: `79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798` |

Kunci rahasia `1` sengaja publik: koin regtest tidak bernilai, dan test bisa membelanjakan alokasi genesis. `network_id` regtest berbeda dari jaringan lain, sehingga tanda tangannya tidak berlaku di tempat lain (DEC-13). Genesis tidak diperiksa PoW-nya.

| Hasil | Nilai |
|---|---|
| Coinbase (89 byte) | `0101000000000000000020` + `00`×32 + `01808080cccfadc9ccc680fe1c00` + payload |
| txid | `d95c2ed4614a2eacece319bf129078c55a74b63fe555b94eeb0b3d1320814875` |
| wtxid | `f0ea325014f4a3f4609f890c2379a1bed228d137959d795806fc136ea5bda614` |
| merkle_root | `66255b2a4e2a87346a7ce7f57078c1325e444a50738f75ef8a6e6246c5c088d6` |
| Header (96 byte) | `01000000` `0000000000000000` `00`×32 `66255b2a…c5c088d6` `003be16a00000000` `ffff7f20` `0000000000000000` |
| block_id = `network_id` regtest | `b933d7c548dc78459ac7376499682cdb9b21ef93f732842ccc89c5d9a16a04c5` |
| Ukuran blok | 186 byte |

---

## 10b. Genesis testnet (disetujui pencipta 5 November 2026)

Bentuk sama dengan genesis regtest (Bagian 10). Testnet tidak memakai jangkar blok Bitcoin (khusus mainnet, Konsep L2):
koin testnet tidak bernilai dan testnet di-reset sebelum mainnet (Konsep L6).

| Field | Nilai |
|---|---|
| Header | version 1, height 0, `prev_block_id` nol, `timestamp` = saat peluncuran testnet publik (`TODO(param)`, sementara `0`), `target = 0x1e8bcf64`, `nonce = 0` |
| Coinbase | height 0, `extra` = ASCII `homeostasis testnet 1` (21 byte) diisi nol sampai 32 byte, satu output `amount = G`, `kind = 0`, `payload` = titik NUMS `H` BIP341 `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0` (kunci rahasianya tidak diketahui siapa pun: alokasi testnet tidak bisa dibelanjakan) |

Dengan cap waktu sementara `0`, ASERT memberi `pow_limit` untuk blok sesudah genesis bercap waktu sekarang. Mengubah cap
waktu tidak mengubah txid dan `merkle_root`, tetapi `block_id` dan `network_id` testnet berubah.

| Hasil (cap waktu `0`) | Nilai |
|---|---|
| txid | `0d93648849e31f2ab8ae61804be580cd0feac5a5e2772a52789ccaec649cbfa1` |
| wtxid | `0b3046d7091720edd3932035215833057ad5355cf56c4e5ac17a46d2850d67b3` |
| merkle_root | `541b5bde4435fb223bf451d5ab73911f7cf521ba25f71e4afe3971769df6c6fd` |
| block_id = `network_id` testnet | `11280b26cf1c372a39b5f387d30556def7dd7e633abfe84902f8295d653fe34b` |
| Ukuran blok | 186 byte |

Node memilih genesis dari `network_id` parameternya (`known_genesis`, MOD-04): regtest atau testnet.

---

## 11. Lokasi kode dan task turunan

| Isi | Modul | Task |
|---|---|---|
| `enum Network { Mainnet, Testnet, Regtest }` | MOD-01 `params` | TSK-M25 |
| Aritmetika 256-bit internal, target compact, ASERT, `work`, chainwork, MTP, V8, parameter per jaringan (Bagian 2), genesis regtest (Bagian 10) | MOD-04 | TSK-M08 |
| V1–V10, T1–T6, C1–C4, penolakan PoW regtest di testnet/mainnet | MOD-04 | TSK-M09 |
| Hash PoW regtest, `Target` 256-bit | MOD-03 (ada) | TSK-M05 🟢 |
| `sighash_v1`, `verify`, ID | MOD-02 (ada) | TSK-M21, M24 🟢 |

Tanpa dependensi baru. Aritmetika 256-bit ditulis sendiri di MOD-04 (tanpa crate bignum). Test wajib: semua vektor K, A, W, M, dan genesis di dokumen ini, ditambah property test (`proptest`): `encode(decode(c)) = c` untuk compact sah; `decode(encode(T)) ≤ T`; ASERT tidak turun jika `tp` naik; ASERT tepat jadwal = anchor; `work` tidak naik jika target naik.

---

## 12. Bukan konsensus (kebijakan node, MOD-07 dan MOD-11)

Biaya minimum dan relay, batas dust, ukuran transaksi untuk relay, penanganan orphan, pilihan saat chainwork sama, dan sumber jam lokal. Semua bisa diubah tanpa fork.

---

## 13. Keputusan

| ID | Keputusan | Status |
|---|---|---|
| DEC-17 | Coinbase wajib mengklaim tepat subsidy + total biaya (Bagian 8). Turunan teknis dari "biaya tanpa pembakaran" dan Prinsip #3 | 🟢 diputuskan Quick 30 Oktober 2026 |
