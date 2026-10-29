# MOD-03: Proof of Work (`homeostasis-pow`)

- **ID:** `MOD-03`
- **Path:** `crates/pow`
- **Paket:** `homeostasis-pow`
- **Status:** 🟡 sedang berjalan (TSK-M05 selesai; sisa: PoW mainnet TSK-M06, menunggu `DEC-01`)
- **Bergantung pada:** `MOD-01`, `MOD-02` (`DEC-14`)
- **Dipakai oleh:** `MOD-04`, `MOD-13`, `MOD-17`

## Tujuan

Menyediakan perhitungan dan verifikasi Proof of Work bagi konsensus dan penambang.

## Tanggung Jawab

- Tipe `Target`: target kesulitan sebagai bilangan 256-bit big-endian (`DEC-15`).
- PoW dev khusus regtest (`DEC-09`): hash `H(regtest pow, header 96 byte)` memakai `hash` dari `MOD-02`, pemeriksaan `hash ≤ target`, dan pencarian nonce untuk penambang regtest (TSK-M05).
- PoW testnet dan mainnet: varian RandomX dengan pergantian kunci setiap `2_048` blok dan jeda `64` blok (TSK-M06, menunggu `DEC-01`).

## Bukan Tanggung Jawab

- Konversi target compact `u32` di header ke `Target` 256-bit, perhitungan ASERT, dan chainwork (`MOD-04`, TSK-M08, `DEC-15`).
- Aturan jaringan mana yang boleh memakai PoW dev. Penolakan PoW dev di testnet dan mainnet adalah aturan konsensus (`MOD-04`, TSK-M09, `DEC-09`).
- Loop penambangan multi-utas dan komunikasi ke daemon node (`MOD-13`).

## Task terkait
- [TSK-M05](../tasks/TSK-M05-pow-dev.md) — 🟢 selesai (antarmuka PoW dan PoW dev regtest)
- [TSK-M06](../tasks/README.md) — 🔴 menunggu `DEC-01` (PoW mainnet)
