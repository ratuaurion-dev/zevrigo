# MOD-08: RPC (`zevrigo-rpc`)

- **ID:** `MOD-08`
- **Path:** `crates/rpc`
- **Paket:** `zevrigo-rpc`
- **Status:** 🟢 selesai untuk regtest (TSK-M12: tipe, encoding, server, dan klien)
- **Bergantung pada:** `MOD-01`
- **Dipakai oleh:** `MOD-11`, `MOD-12`, `MOD-13`, `MOD-14`, `MOD-15`, `MOD-16`, `MOD-17`

## Tujuan

Menyediakan definisi tipe pesan serta klien dan server JSON-RPC antara daemon node dan aplikasi ekosistem.

## Tanggung Jawab

- Tipe request/response, encoding nilai (jumlah koin sebagai string desimal quanta), server dan klien JSON-RPC lewat HTTP (`jsonrpsee`, DEC-10 bagian 2).
- Trait `RpcBackend` sebagai batas ke node: MOD-08 tidak membaca storage atau mempool sendiri, sehingga tetap hanya bergantung pada `MOD-01`. Aturan lengkap: [spesifikasi-rpc-v0.1.md](../reference/spesifikasi-rpc-v0.1.md).

## Bukan Tanggung Jawab

- Logika konsensus (`MOD-04`), penyimpanan kunci dompet (`MOD-09`), maupun tampilan antarmuka pengguna (`MOD-12`..`MOD-16`).

## Task terkait
- [TSK-M12](../tasks/TSK-M12-rpc.md) — 🟢 selesai
- [TSK-M36](../tasks/TSK-M36-saldo-tertunda.md) — 🟢 selesai (metode ke-9 `scan_mempool`)
