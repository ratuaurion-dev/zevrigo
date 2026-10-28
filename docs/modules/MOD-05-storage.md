# MOD-05: Storage (`homeostasis-storage`)

- **ID:** `MOD-05`
- **Path:** `crates/storage`
- **Paket:** `homeostasis-storage`
- **Status:** ⚪ belum dimulai (kerangka R0; mesin penyimpanan menunggu `DEC-03`)
- **Bergantung pada:** `MOD-01`
- **Dipakai oleh:** `MOD-11`, `MOD-14`, `MOD-17`

## Tujuan

Menyediakan penyimpanan persisten yang deterministik dan atomik untuk blok, indeks rantai, dan set UTXO.

## Tanggung Jawab

- Penyimpanan serta pembacaan blok, header, dan set UTXO aktif.
- Dukungan pembaruan atomik saat *reorg* serta pemangkasan blok lama (*pruning*) untuk node pruned (*diisi lebih rinci di R2*).

## Bukan Tanggung Jawab

- Penentuan keabsahan blok atau aturan konsensus (`MOD-04`).
- Orkestrasi jaringan dan layanan RPC daemon (`MOD-11`).

## Task terkait
- [TSK-M10](../tasks/README.md)
- [TSK-M11](../tasks/README.md)
