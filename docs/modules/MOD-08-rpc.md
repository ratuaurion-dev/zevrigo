# MOD-08: RPC (`homeostasis-rpc`)

- **ID:** `MOD-08`
- **Path:** `crates/rpc`
- **Paket:** `homeostasis-rpc`
- **Status:** ⚪ belum dimulai (kerangka R0)
- **Bergantung pada:** `MOD-01`
- **Dipakai oleh:** `MOD-11`, `MOD-12`, `MOD-13`, `MOD-14`, `MOD-15`, `MOD-16`, `MOD-17`

## Tujuan

Menyediakan definisi tipe pesan serta klien dan server JSON-RPC antara daemon node dan aplikasi ekosistem.

## Tanggung Jawab

- Skema request/response JSON-RPC yang aman terhadap presisi `u128` quanta serta implementasi transpor klien/server (*diisi lebih rinci di R2*).

## Bukan Tanggung Jawab

- Logika konsensus (`MOD-04`), penyimpanan kunci dompet (`MOD-09`), maupun tampilan antarmuka pengguna (`MOD-12`..`MOD-16`).

## Task terkait
- [TSK-M12](../tasks/README.md)
