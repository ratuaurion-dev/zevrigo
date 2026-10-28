# MOD-02: Crypto (`homeostasis-crypto`)

- **ID:** `MOD-02`
- **Path:** `crates/crypto`
- **Paket:** `homeostasis-crypto`
- **Status:** 🟡 sedang berjalan (TSK-M02, TSK-M04, TSK-M21 selesai; sisa: MuSig2, sighash menunggu DEC-13)
- **Bergantung pada:** `MOD-01`
- **Dipakai oleh:** `MOD-04`, `MOD-09`, `MOD-17`

## Tujuan

Menyediakan primitif kriptografi deterministik yang dipakai oleh protokol dan dompet Homeostasis Coin.

## Tanggung Jawab

- Fungsi hash umum BLAKE3 dengan pemisahan domain yang jelas.
- Tanda tangan Schnorr BIP340 pada kurva `secp256k1`.
- Agregasi kunci dan multi-tanda tangan MuSig2 (*diisi lebih rinci di R2*).

## Bukan Tanggung Jawab

- Aturan validasi skrip/transaksi tingkat protokol (`MOD-04`).
- Manajemen penyimpanan kunci privat pengguna dan penyusunan UTXO dompet (`MOD-09`).

## Task terkait
- [TSK-M02](../tasks/TSK-M02-crypto-blake3.md) — 🟢 selesai (hash BLAKE3 berpemisah domain dan `merkle_root`)
- [TSK-M04](../tasks/TSK-M04-crypto-schnorr.md) — 🟢 selesai (tanda tangan dan verifikasi Schnorr BIP340 x-only)
- [TSK-M21](../tasks/TSK-M21-crypto-ids.md) — 🟢 selesai (identitas transaksi dan blok: txid, wtxid, block_id, condition_hash, block_merkle_root)
