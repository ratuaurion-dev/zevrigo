# MOD-17: Integration Tests (`homeostasis-integration-tests`)

- **ID:** `MOD-17`
- **Path:** `tests/integration`
- **Paket:** `homeostasis-integration-tests`
- **Status:** 🟡 sedang berjalan (FTR-02 diuji ujung ke ujung, TSK-F02)
- **Bergantung pada:** `MOD-01`, `MOD-02`, `MOD-03`, `MOD-04`, `MOD-05`, `MOD-06`, `MOD-07`, `MOD-08`, `MOD-09`, `MOD-10` (semua crate inti); dev-dependency `MOD-11` node dan `MOD-13` miner untuk uji fitur ujung ke ujung (DEC-20)
- **Dipakai oleh:** —

## Tujuan

Menjalankan pengujian integrasi lintas crate inti serta memverifikasi kecocokan vektor uji konsensus ujung-ke-ujung.

## Tanggung Jawab

- Pengujian skenario lintas modul (konsensus + penyimpanan + kriptografi + dompet/merchant) dan pembacaan vektor uji dari `tests/vectors/`.
- Uji fitur ujung ke ujung: `tests/ftr02_mining.rs` (FTR-02, node + RPC + miner di satu proses, coinbase sama dengan jadwal pasokan).

## Bukan Tanggung Jawab

- Unit test internal masing-masing modul (yang diletakkan di `src/<modul>/tests.rs` pada tiap crate).

## Task terkait
- [TSK-F02](../tasks/TSK-F02-menambang-regtest.md) — 🟢 selesai
- [TSK-F04](../tasks/README.md)
