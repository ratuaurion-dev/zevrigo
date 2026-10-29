# MOD-01: Primitives (`homeostasis-primitives`)

- **ID:** `MOD-01`
- **Path:** `crates/primitives`
- **Paket:** `homeostasis-primitives`
- **Status:** 🟡 sedang berjalan (TSK-M01 dan TSK-M03 selesai; TSK-M22 siap dikerjakan)
- **Bergantung pada:** —
- **Dipakai oleh:** `MOD-02`, `MOD-03`, `MOD-04`, `MOD-05`, `MOD-06`, `MOD-07`, `MOD-08`, `MOD-09`, `MOD-10`, `MOD-12`, `MOD-13`, `MOD-14`, `MOD-17`

## Tujuan

Menyediakan tipe data dasar yang deterministik dan bebas luapan diam-diam bagi seluruh crate Homeostasis Coin.

## Tanggung Jawab

- Tipe jumlah koin (`u128` quanta), tinggi blok, dan hash 32 byte.
- Konstanta parameter protokol (sumber kebenaran numerik).
- Struktur data transaksi, blok, dan encoding kanonis.

## Bukan Tanggung Jawab

- Aturan validasi konsensus, perhitungan jadwal pasokan, dan penyesuaian kesulitan ASERT (`MOD-04`).
- Operasi hash kriptografis BLAKE3 dan tanda tangan Schnorr/MuSig2 (`MOD-02`).

## Task terkait
- [TSK-M01](../tasks/TSK-M01-primitives-tipe-dasar.md) — 🟢 selesai (tipe dasar: `Amount`, `BlockHeight`, `Hash32`, parameter chain)
- [TSK-M03](../tasks/TSK-M03-primitives-encoding.md) — 🟢 selesai (struktur transaksi, witness, kondisi, header, blok, dan encoding kanonis)
- [TSK-M22](../tasks/TSK-M22-primitives-utxo.md) — 🟡 siap (tipe OutPoint dan UtxoEntry)
