# MOD-12: CLI (`zevrigo-cli`)

- **ID:** `MOD-12`
- **Path:** `apps/cli`
- **Paket:** `zevrigo-cli` (bin)
- **Status:** 🟢 TSK-M19 selesai: `info`, `wallet-create`, `wallet-restore`, `address`, `balance`, `send`; regtest dan testnet (`--testnet`, TSK-M06 bagian C3b)
- **Bergantung pada:** `MOD-01`, `MOD-08`, `MOD-09` (DEC-21)
- **Dipakai oleh:** —

## Tujuan

Menyediakan alat baris perintah ringan bagi operator untuk memeriksa status dan berinteraksi dengan daemon `zevrigo-node` melalui JSON-RPC.

## Tanggung Jawab

- Pengiriman perintah administrasi baca/kirim transaksi ke endpoint RPC node dan pemformatan keluaran terminal secara deterministik.
- Perintah wallet regtest dan testnet (buat, pulihkan, alamat, saldo, kirim) dan file wallet sesuai [spesifikasi-wallet-v0.1.md](../reference/spesifikasi-wallet-v0.1.md) Bagian 7, memakai `MOD-09` (DEC-21).
- Jaringan (TSK-M06 bagian C3b): `--regtest` (bawaan) atau `--testnet`; file wallet di `<folder>/<jaringan>/wallet.json` dengan `network` yang wajib cocok (`load_network`), alamat `rzvrg1…` atau `tzvrg1…`, RPC bawaan port 28801 atau 18801, dan `balance`/`send` menolak node jaringan lain (`CliError::WrongNetwork`).

## Bukan Tanggung Jawab

- Validasi konsensus lokal (`MOD-04`).
- Menulis sendiri logika frasa, kunci, alamat, atau tanda tangan; semuanya dari `MOD-09`.

## Task terkait
- [TSK-M19](../tasks/TSK-M19-cli-wallet.md) — 🟢 selesai (pustaka `zevrigo_cli`: `args`, `wallet_file`, `scan`, `commands`)
- [TSK-M36](../tasks/TSK-M36-saldo-tertunda.md) — 🟢 selesai (saldo dan pemilihan koin dengan transaksi tertunda)
- [TSK-M37](../tasks/TSK-M37-enkripsi-wallet.md) — 🟢 selesai (file wallet versi 2 dikunci password, `wallet-encrypt`, `wallet-show-phrase`, DEC-27)
