# Spesifikasi RPC Zevrigo Coin (ZVRG) v0.1: 9 metode JSON-RPC untuk node regtest (FTR-01 sampai FTR-03)

> Diperbarui: 13 November 2026 · Pemilik: pencipta · Ditulis: Quick · Task: TSK-S04, TSK-M60 · Status: 🟢 v0.1
> Lokasi di repo: `docs/reference/spesifikasi-rpc-v0.1.md`

| Status | Ringkasan |
|---|---|
| 🟢 Dikunci | Transpor JSON-RPC 2.0 lewat HTTP, aturan encoding nilai, 9 metode (`scan_mempool` ditambah 2 November 2026, TSK-M36), kode error |
| ⏳ Ditunda | Autentikasi untuk testnet/mainnet, metode wallet dan explorer (FTR-05, FTR-07), pesan P2P (TSK-S05) |
| ➡️ Langkah berikutnya | TSK-M12 (tipe dan klien/server di MOD-08), TSK-M26 (iterasi set UTXO di MOD-05), TSK-M13, TSK-M18 |

RPC **bukan konsensus**. Perubahannya tidak memecah chain, cukup lewat versi baru dokumen ini.

---

## 1. Transpor

| Hal | Nilai |
|---|---|
| Protokol | JSON-RPC 2.0 lewat HTTP `POST /`, `Content-Type: application/json` |
| Pustaka | `jsonrpsee` 0.26.1 (DEC-10 bagian 2), server di node, klien di CLI, miner, explorer, dan wallet |
| Klien `https://` | Sejak TSK-M60 (DEC-10 bagian 14): klien MOD-08 menerima `http://` dan `https://` (rustls dengan penyedia `ring`, verifikasi sertifikat lewat sistem operasi), untuk gerbang RPC demo di belakang Cloudflare (MOD-18). Server node tetap HTTP tanpa TLS di `127.0.0.1` |
| Alamat bawaan | `127.0.0.1`, hanya lokal |
| Port bawaan | regtest 28_801 · testnet 18_801 · mainnet 8_801 |
| Ukuran request maksimum | 1 MiB (blok 100_000 byte = 200_000 karakter hex) |
| Autentikasi | Regtest: tanpa autentikasi, karena hanya lokal. Testnet/mainnet: ditunda, wajib ada sebelum TSK-F06 |
| Parameter | Berdasarkan nama (objek JSON). Parameter tidak dikenal ditolak (`-32602`) |

---

## 2. Encoding nilai

| Jenis | Encoding JSON | Contoh |
|---|---|---|
| Hash 32 byte (txid, block_id, payload) | string hex huruf kecil, tepat 64 karakter | `"8b9eb318…364dab"` |
| Data biner (blok, transaksi) | string hex huruf kecil dari encoding kanonis (Spesifikasi Format) | `"0100…"` |
| Jumlah koin | string desimal dalam quanta, tanpa nol di depan, tanpa tanda, tanpa titik | `"1910127426536917"` |
| Target compact | string hex 8 karakter dari `u32` | `"207fffff"` |
| Chainwork | string hex 64 karakter (256-bit big-endian) | `"00…0002"` |
| Tinggi, cap waktu, jumlah, ukuran | angka JSON bilangan bulat | `1`, `1793145720` |
| Nonce (`u64` penuh) | string desimal | `"0"` |
| Jaringan | `"mainnet"`, `"testnet"`, `"regtest"` | |
| Jenis output | angka `0` atau `1` | |

- **Jumlah koin tidak pernah angka JSON.** `u128` quanta melebihi presisi angka di JavaScript (2^53), dan UI TypeScript wajib menyimpannya sebagai `string` atau `bigint` (AGENTS.md).
- Hex huruf besar, panjang salah, atau string jumlah dengan nol di depan ditolak (`-32602`).

---

## 3. Metode

### 3.1 `get_chain_info`
Parameter: tidak ada. Hasil:
| Field | Isi |
|---|---|
| `network` | jaringan node |
| `height`, `best_block_id` | tinggi dan ID ujung chain aktif |
| `chainwork` | chainwork ujung (Spesifikasi Validasi Bagian 5) |
| `next_target` | target wajib blok berikutnya (ASERT) |
| `median_time_past` | MTP ujung; cap waktu blok berikutnya wajib lebih besar |
| `mined_total` | `mined_total_after(height)` |
| `total_supply` | `G + mined_total` |
| `mempool_count` | jumlah transaksi di mempool |

### 3.2 `get_block_hash`
Parameter: `height`. Hasil: `block_id` di chain aktif. Tidak ada → error `1001`.

### 3.3 `get_block`
Parameter: `block_id`. Hasil:
| Field | Isi |
|---|---|
| `block_id`, `height`, `prev_block_id`, `merkle_root`, `timestamp`, `target`, `nonce` | isi header |
| `size` | ukuran encoding blok dalam byte |
| `txids` | txid semua transaksi, urut seperti di blok |
| `hex` | encoding kanonis blok |

Tidak ada di chain aktif → error `1001`.

### 3.4 `get_block_template`
Parameter: `payout_kind` (`0` atau `1`) dan `payout_payload` (hash 32 byte). Hasil:
| Field | Isi |
|---|---|
| `height`, `prev_block_id`, `target` | tinggi, induk, dan target wajib blok berikutnya |
| `full_target` | target yang sama sebagai hash 32 byte (256-bit big-endian, hasil decode `target`), agar miner tidak menulis ulang aturan decode compact. Ditambahkan 31 Oktober 2026 (TSK-M20); field hasil baru tidak memecah klien lama |
| `pow_key` | kunci RandomX blok berikutnya (hash 32 byte): `block_id` leluhur di tinggi kunci pada chain aktif (Spesifikasi Validasi Bagian 2); regtest 32 byte nol. Ditambahkan 6 November 2026 (TSK-M06 bagian C3), agar miner tidak menulis ulang aturan kunci |
| `min_timestamp` | `median_time_past + 1` |
| `subsidy`, `fees` | subsidy blok dan total biaya transaksi yang dimasukkan |
| `hex` | blok lengkap siap ditambang: coinbase `extra` kosong dengan satu output `subsidy + fees` ke payout, transaksi dari mempool, Merkle root sudah benar, `nonce = 0`, `timestamp = max(jam lokal, min_timestamp)` |

Miner hanya mengubah header: `nonce` (offset 88) dan, jika semua nonce habis, `timestamp` (offset 76) dalam rentang `min_timestamp` sampai jam lokal + 3_600. Miner yang ingin membagi kerja boleh mengisi `extra` coinbase, lalu wajib menghitung ulang Merkle root.

### 3.5 `submit_block`
Parameter: `hex`. Hasil:
| Field | Isi |
|---|---|
| `status` | `"accepted"` (masuk chain aktif atau cabang yang dikenal), `"duplicate"` (sudah dikenal), `"orphan"` (induk belum dikenal), `"deferred"` (gagal V8, diperiksa ulang nanti), `"invalid"` |
| `block_id` | ID blok, jika decode berhasil |
| `reason` | untuk `"invalid"`: nama aturan yang dilanggar (misalnya `"V6"`, `"T2"`), dari error MOD-04 |

Hex yang tidak bisa di-decode → `status = "invalid"`, `reason = "V1"`, tanpa `block_id`.

### 3.6 `send_raw_transaction`
Parameter: `hex`. Hasil: `txid`. Transaksi diperiksa T1–T6 terhadap ujung chain + mempool, dengan tinggi = tinggi ujung + 1. Tidak sah → error `1002` dengan `data.reason` (misalnya `"T6"`). Decode gagal → error `1003`.

### 3.7 `get_mempool`
Parameter: tidak ada. Hasil: `txids`, urut naik bytewise.

### 3.8 `scan_utxos`
Parameter: `payloads`, daftar 1 sampai 64 hash 32 byte. Hasil: daftar UTXO di ujung chain aktif yang `payload`-nya ada di daftar, urut `(txid, index)`:
| Field | Isi |
|---|---|
| `txid`, `index` | outpoint |
| `amount`, `kind`, `payload` | output |
| `height`, `is_coinbase` | asal output |
| `confirmations` | `tinggi ujung − height + 1` |
| `spendable_at` | tinggi blok paling awal yang boleh membelanjakannya: `height + 100` untuk coinbase, `height + 1` untuk lainnya (kunci waktu kondisi tidak terlihat dari output) |

Butuh iterasi set UTXO di MOD-05 (TSK-M26). Output yang sedang dibelanjakan transaksi di mempool tetap ditampilkan;
lihat `scan_mempool`.

### 3.9 `scan_mempool`
Parameter: `payloads`, daftar 1 sampai 64 hash 32 byte (sama dengan `scan_utxos`). Hasil: transaksi tertunda di mempool
yang menyentuh payload itu:
| Field | Isi |
|---|---|
| `spent` | UTXO di ujung chain aktif yang `payload`-nya ada di daftar dan sedang dibelanjakan transaksi di mempool: `txid`, `index` (outpoint), `spent_by` (txid transaksi mempool), urut `(txid, index)` |
| `outputs` | Output transaksi di mempool yang `payload`-nya ada di daftar: `txid`, `index`, `amount`, `kind`, `payload`, urut `(txid, index)` |

Dipakai wallet untuk saldo tertunda dan agar koin yang sedang dibelanjakan tidak dipilih lagi (TSK-M36). Mempool v0.1
tidak menerima transaksi yang membelanjakan output transaksi lain di mempool, jadi output tertunda belum bisa
dibelanjakan. `scan_utxos` dan `scan_mempool` adalah dua panggilan terpisah; jika blok baru masuk di antaranya, wallet
cukup memanggil ulang.

---

## 4. Kode error

| Kode | Arti |
|---|---|
| `-32700`, `-32600`, `-32601`, `-32602`, `-32603` | Standar JSON-RPC: JSON rusak, request tidak sah, metode tidak ada, parameter tidak sah, galat internal |
| `1001` | Tidak ditemukan |
| `1002` | Transaksi tidak sah; `data.reason` = aturan T1–T6 |
| `1003` | Hex atau encoding kanonis tidak sah |

---

## 5. Pemetaan ke fitur

| Fitur | Metode |
|---|---|
| FTR-01 node regtest | `get_chain_info`, `get_block_hash`, `get_block`, `submit_block` |
| FTR-02 menambang | `get_block_template`, `submit_block` |
| FTR-03 kirim dan terima lewat CLI | `scan_utxos`, `scan_mempool`, `send_raw_transaction`, `get_mempool` |

Wallet CLI (TSK-M19) menyusun dan menandatangani transaksi sendiri (MOD-09). Node tidak pernah menerima kunci privat.
