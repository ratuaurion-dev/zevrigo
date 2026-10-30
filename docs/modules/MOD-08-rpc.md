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

- Tipe request/response, encoding nilai (jumlah koin sebagai string desimal quanta), server dan klien JSON-RPC lewat HTTP (`jsonrpsee`, DEC-10 bagian 2). Aturan lengkap: [spesifikasi-rpc-v0.1.md](../reference/spesifikasi-rpc-v0.1.md).

## Bukan Tanggung Jawab

- Logika konsensus (`MOD-04`), penyimpanan kunci dompet (`MOD-09`), maupun tampilan antarmuka pengguna (`MOD-12`..`MOD-16`).

## Task terkait
- [TSK-M12](../tasks/README.md) — 🟡 siap
