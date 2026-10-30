# TSK-M25 · Tipe `Network` di MOD-01 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-01 (crates/primitives) · **Untuk:** FTR-01 sampai FTR-08
- **Bergantung pada:** TSK-S02 · **Memblokir:** TSK-M08, TSK-M09, TSK-M15
- **Estimasi:** XS · **Ditulis:** Quick (kode dan test), 30 Oktober 2026

## Tujuan
Satu tipe jaringan yang dipakai bersama MOD-04 (parameter per jaringan, Spesifikasi Validasi Bagian 2) dan MOD-09
(awalan alamat, Spesifikasi Format Bagian 8).

## API
`pub enum Network { Mainnet, Testnet, Regtest }` di `params`, dengan `Network::ALL` dan `hrp()`
(`hsis`, `thsis`, `rhsis`). Re-export dari `lib.rs`.

## Test (primitives 56 → 57)
`src/params/tests.rs`: `network_variants_and_hrp_match_spec`.
