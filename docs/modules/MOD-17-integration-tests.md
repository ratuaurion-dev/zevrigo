# MOD-17: Integration Tests (`homeostasis-integration-tests`)

- **ID:** `MOD-17`
- **Path:** `tests/integration`
- **Paket:** `homeostasis-integration-tests`
- **Status:** ⚪ belum dimulai (kerangka R0)
- **Bergantung pada:** `MOD-01`, `MOD-02`, `MOD-03`, `MOD-04`, `MOD-05`, `MOD-06`, `MOD-07`, `MOD-08`, `MOD-09`, `MOD-10` (semua crate inti)
- **Dipakai oleh:** —

## Tujuan

Menjalankan pengujian integrasi lintas crate inti serta memverifikasi kecocokan vektor uji konsensus ujung-ke-ujung.

## Tanggung Jawab

- Pengujian skenario lintas modul (konsensus + penyimpanan + kriptografi + dompet/merchant) dan pembacaan vektor uji dari `tests/vectors/` (*diisi lebih rinci di R2*).

## Bukan Tanggung Jawab

- Unit test internal masing-masing modul (yang diletakkan di `src/<modul>/tests.rs` pada tiap crate).
