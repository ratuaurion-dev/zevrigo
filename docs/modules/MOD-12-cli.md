# MOD-12: CLI (`homeostasis-cli`)

- **ID:** `MOD-12`
- **Path:** `apps/cli`
- **Paket:** `homeostasis-cli` (bin)
- **Status:** ⚪ belum dimulai (kerangka R0)
- **Bergantung pada:** `MOD-01`, `MOD-08`, `MOD-09` (DEC-21)
- **Dipakai oleh:** —

## Tujuan

Menyediakan alat baris perintah ringan bagi operator untuk memeriksa status dan berinteraksi dengan daemon `homeostasis-node` melalui JSON-RPC.

## Tanggung Jawab

- Pengiriman perintah administrasi baca/kirim transaksi ke endpoint RPC node dan pemformatan keluaran terminal secara deterministik.
- Perintah wallet regtest (buat, pulihkan, alamat, saldo, kirim) dan file wallet sesuai [spesifikasi-wallet-v0.1.md](../reference/spesifikasi-wallet-v0.1.md) Bagian 7, memakai `MOD-09` (DEC-21).

## Bukan Tanggung Jawab

- Validasi konsensus lokal (`MOD-04`).
- Menulis sendiri logika frasa, kunci, alamat, atau tanda tangan; semuanya dari `MOD-09`.

## Task terkait
- [TSK-M19](../tasks/README.md)
