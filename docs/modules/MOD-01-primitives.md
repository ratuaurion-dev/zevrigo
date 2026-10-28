# MOD-01: Primitives (`homeostasis-primitives`)

- **ID:** `MOD-01`
- **Path:** `crates/primitives`
- **Paket:** `homeostasis-primitives`
- **Status:** 🟡 sedang berjalan (`TSK-M01` selesai; menunggu `TSK-M03`)
- **Bergantung pada:** —
- **Dipakai oleh:** `MOD-02`, `MOD-03`, `MOD-04`, `MOD-05`, `MOD-06`, `MOD-07`, `MOD-08`, `MOD-09`, `MOD-10`, `MOD-12`, `MOD-13`, `MOD-14`, `MOD-17`

## Tujuan

Menyediakan tipe data dasar yang deterministik dan bebas luapan diam-diam bagi seluruh crate Homeostasis Coin.

## Tanggung Jawab

- Tipe jumlah koin (`u128` quanta), tinggi blok, dan hash 32 byte.
- Konstanta parameter protokol (sumber kebenaran numerik).
- Struktur data transaksi, blok, dan encoding kanonis (*diisi lebih rinci di R2*).

## Bukan Tanggung Jawab

- Aturan validasi konsensus, perhitungan jadwal pasokan, dan penyesuaian kesulitan ASERT (`MOD-04`).
- Operasi hash kriptografis BLAKE3 dan tanda tangan Schnorr/MuSig2 (`MOD-02`).

## Task terkait
- [TSK-M01](../tasks/TSK-M01-primitives-tipe-dasar.md) — 🟢 selesai (tipe dasar: `Amount`, `BlockHeight`, `Hash32`, parameter chain)
- [TSK-M03](../tasks/README.md) — ⚪ belum dimulai
