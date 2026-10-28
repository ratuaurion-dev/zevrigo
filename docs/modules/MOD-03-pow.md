# MOD-03: Proof of Work (`homeostasis-pow`)

- **ID:** `MOD-03`
- **Path:** `crates/pow`
- **Paket:** `homeostasis-pow`
- **Status:** ⚪ belum dimulai (kerangka R0; menunggu `DEC-01`)
- **Bergantung pada:** `MOD-01`
- **Dipakai oleh:** `MOD-04`, `MOD-13`, `MOD-17`

## Tujuan

Menyediakan perhitungan dan verifikasi Proof of Work bagi konsensus dan penambang.

## Tanggung Jawab

- Evaluasi fungsi Proof of Work serta pengelolaan pergantian kunci setiap `2_048` blok dengan jeda `64` blok (*strategi teknis menunggu `DEC-01`, diisi di R2*).

## Bukan Tanggung Jawab

- Perhitungan target kesulitan ASERT dan aturan pemilihan rantai (`MOD-04`).
- Loop penambangan multi-utas dan komunikasi ke daemon node (`MOD-13`).

## Task terkait
- [TSK-M05](../tasks/README.md)
- [TSK-M06](../tasks/README.md)
