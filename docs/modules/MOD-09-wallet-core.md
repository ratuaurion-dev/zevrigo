# MOD-09: Wallet Core (`homeostasis-wallet-core`)

- **ID:** `MOD-09`
- **Path:** `crates/wallet-core`
- **Paket:** `homeostasis-wallet-core`
- **Status:** ⚪ belum dimulai (kerangka R0)
- **Bergantung pada:** `MOD-01`, `MOD-02`
- **Dipakai oleh:** `MOD-10`, `MOD-12` (DEC-21), `MOD-15`, `MOD-17`

## Tujuan

Menyediakan pustaka inti pengelolaan kunci, pemilihan UTXO, penyusunan, dan penandatanganan transaksi bagi dompet HSIS.

## Tanggung Jawab

- Derivasi serta pengelolaan kunci publik/privat dan mode *watch-only*.
- Pemilihan koin (*coin selection*), perhitungan biaya transaksi, penyusunan transaksi (transfer, MuSig2, kunci waktu), dan pembuatan tanda tangan.
- Aturan lengkap: [spesifikasi-wallet-v0.1.md](../reference/spesifikasi-wallet-v0.1.md) (frasa BIP39 DEC-22, turunan BIP32 `m/44'/1'/0'/k/i`, alamat bech32m DEC-11, pemilihan koin). Dependensi eksternal: `sha2`, `hmac`, `pbkdf2` (DEC-10 bagian 4).

## Bukan Tanggung Jawab

- Antarmuka grafis pengguna (`MOD-15`) maupun komunikasi jaringan langsung ke node tanpa melalui lapisan aplikasi/RPC (`MOD-08`).

## Task terkait
- [TSK-M15](../tasks/README.md)
- [TSK-M16](../tasks/README.md)
