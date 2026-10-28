# MOD-09: Wallet Core (`homeostasis-wallet-core`)

- **ID:** `MOD-09`
- **Path:** `crates/wallet-core`
- **Paket:** `homeostasis-wallet-core`
- **Status:** ⚪ belum dimulai (kerangka R0)
- **Bergantung pada:** `MOD-01`, `MOD-02`
- **Dipakai oleh:** `MOD-10`, `MOD-15`, `MOD-17`

## Tujuan

Menyediakan pustaka inti pengelolaan kunci, pemilihan UTXO, penyusunan, dan penandatanganan transaksi bagi dompet HSIS.

## Tanggung Jawab

- Derivasi serta pengelolaan kunci publik/privat dan mode *watch-only*.
- Pemilihan koin (*coin selection*), perhitungan biaya transaksi, penyusunan transaksi (transfer, MuSig2, kunci waktu), dan pembuatan tanda tangan (*diisi lebih rinci di R2*).

## Bukan Tanggung Jawab

- Antarmuka grafis pengguna (`MOD-15`) maupun komunikasi jaringan langsung ke node tanpa melalui lapisan aplikasi/RPC (`MOD-08`).
