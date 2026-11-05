# MOD-03: Proof of Work (`homeostasis-pow`)

- **ID:** `MOD-03`
- **Path:** `crates/pow`
- **Paket:** `homeostasis-pow`
- **Status:** 🟡 sedang berjalan (TSK-M05 selesai; TSK-M06 bagian A selesai, bagian B: layanan RandomX)
- **Bergantung pada:** `MOD-01`, `MOD-02` (`DEC-14`); pustaka luar `randomx-rs` 1.6.0 (`DEC-10` bagian 10)
- **Dipakai oleh:** `MOD-04`, `MOD-13`, `MOD-17`

## Tujuan

Menyediakan perhitungan dan verifikasi Proof of Work bagi konsensus dan penambang.

## Tanggung Jawab

- Tipe `Target`: target kesulitan sebagai bilangan 256-bit big-endian (`DEC-15`).
- PoW dev khusus regtest (`DEC-09`): hash `H(regtest pow, header 96 byte)` memakai `hash` dari `MOD-02`, pemeriksaan `hash ≤ target`, dan pencarian nonce untuk penambang regtest (TSK-M05).
- PoW testnet dan mainnet (`DEC-01`, TSK-M06): varian RandomX (Konsep Bagian 5.10) lewat `randomx-rs`. `key_height`: kunci berganti setiap `2_048` blok dengan jeda `64` blok, sebelumnya kunci genesis. `RandomxVerifier`: mode ringan (cache 256 MiB) untuk satu kunci, `hash` atas 96 byte header, `verify` dengan `hash ≤ target`. `RandomxService` (bagian B): satu utas pemilik cache untuk dua kunci terakhir, handle `Send` dan `Sync`, karena cache dan VM pustaka tidak boleh berpindah utas. Salt Argon2 masih resmi sampai TSK-M06 bagian C; penambangan mode cepat juga di bagian C.

## Bukan Tanggung Jawab

- Konversi target compact `u32` di header ke `Target` 256-bit, perhitungan ASERT, dan chainwork (`MOD-04`, TSK-M08, `DEC-15`).
- Aturan jaringan mana yang boleh memakai PoW dev. Penolakan PoW dev di testnet dan mainnet adalah aturan konsensus (`MOD-04`, TSK-M09, `DEC-09`).
- Loop penambangan multi-utas dan komunikasi ke daemon node (`MOD-13`).

## Task terkait
- [TSK-M05](../tasks/TSK-M05-pow-dev.md) — 🟢 selesai (antarmuka PoW dan PoW dev regtest)
- [TSK-M06](../tasks/TSK-M06-pow-randomx.md) — 🟡 bagian A (pustaka RandomX), lalu B (konsensus dan node) dan C (salt sendiri, penambang)
