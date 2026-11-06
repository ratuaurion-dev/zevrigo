# MOD-17: Integration Tests (`zevrigo-integration-tests`)

- **ID:** `MOD-17`
- **Path:** `tests/integration`
- **Paket:** `zevrigo-integration-tests`
- **Status:** 🟡 sedang berjalan (FTR-02 sampai FTR-05 diuji ujung ke ujung, TSK-F02 sampai TSK-F05)
- **Bergantung pada:** `MOD-01`, `MOD-02`, `MOD-03`, `MOD-04`, `MOD-05`, `MOD-06`, `MOD-07`, `MOD-08`, `MOD-09`, `MOD-10` (semua crate inti); dev-dependency `MOD-11` node dan `MOD-13` miner (DEC-20) serta `MOD-12` CLI (DEC-23) dan `MOD-14` explorer (DEC-25) untuk uji fitur ujung ke ujung
- **Dipakai oleh:** —

## Tujuan

Menjalankan pengujian integrasi lintas crate inti serta memverifikasi kecocokan vektor uji konsensus ujung-ke-ujung.

## Tanggung Jawab

- Pengujian skenario lintas modul (konsensus + penyimpanan + kriptografi + dompet/merchant) dan pembacaan vektor uji dari `tests/vectors/`.
- Uji fitur ujung ke ujung: `tests/ftr02_mining.rs` (FTR-02, node + RPC + miner di satu proses, coinbase sama dengan jadwal pasokan) `tests/ftr03_send_receive.rs` (FTR-03, dua wallet CLI saling mengirim lewat node sungguhan), dan `tests/ftr04_multi_node.rs` (FTR-04, lima node dengan P2P sungguhan: sinkron, terpisah lalu tersambung dengan reorg dalam, node offline dan node baru mengejar), dan `tests/ftr05_explorer.rs` (FTR-05, explorer dengan indeks SQLite mengikuti node sungguhan lewat RPC: alokasi genesis dibelanjakan, reorg, buka ulang, node mati).

## Bukan Tanggung Jawab

- Unit test internal masing-masing modul (yang diletakkan di `src/<modul>/tests.rs` pada tiap crate).

## Task terkait
- [TSK-F02](../tasks/TSK-F02-menambang-regtest.md) — 🟢 selesai
- [TSK-F03](../tasks/TSK-F03-kirim-terima-cli.md) — 🟢 selesai
- [TSK-F04](../tasks/TSK-F04-multi-node.md) — 🟢 selesai
- [TSK-F05](../tasks/TSK-F05-explorer-minimal.md) — 🟢 selesai
