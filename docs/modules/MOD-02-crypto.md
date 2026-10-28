# MOD-02: Crypto (`homeostasis-crypto`)

- **ID:** `MOD-02`
- **Path:** `crates/crypto`
- **Paket:** `homeostasis-crypto`
- **Status:** ⚪ belum dimulai (kerangka R0)
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
- [TSK-M02](../tasks/README.md)
- [TSK-M04](../tasks/README.md)
- [TSK-M21](../tasks/README.md)
