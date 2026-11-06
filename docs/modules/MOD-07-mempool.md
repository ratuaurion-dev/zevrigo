# MOD-07: Mempool (`zevrigo-mempool`)

- **ID:** `MOD-07`
- **Path:** `crates/mempool`
- **Paket:** `zevrigo-mempool`
- **Status:** 🟢 selesai untuk regtest (TSK-M13)
- **Bergantung pada:** `MOD-01`, `MOD-04` (test juga memakai `MOD-02` sebagai dev-dependency, DEC-18)
- **Dipakai oleh:** `MOD-11`, `MOD-17`

## Tujuan

Mengelola antrean transaksi belum terkonfirmasi di memori sebelum dimasukkan ke dalam kandidat blok.

## Tanggung Jawab

- Pemeriksaan transaksi dengan `check_transaction` (`MOD-04`, T1 sampai T6) terhadap set UTXO ujung chain ditambah output mempool, dengan tinggi = ujung + 1.
- Pencegahan pembelanjaan ganda di antrean (tanpa penggantian transaksi), batas total byte, dan penolakan transaksi yang lebih besar dari blok.
- Pemeriksaan ulang setelah blok baru atau reorg (`revalidate`), dan pemilihan transaksi untuk kandidat blok sesuai urutan kedatangan (`select_for_block`).
- Nama aturan untuk error RPC `1002` (`rule_name`, `MempoolError::rule`).

## Bukan Tanggung Jawab

- Mendefinisikan ulang aturan konsensus di luar `MOD-04`.
- Menghitung txid (diberikan pemanggil, dihitung dengan `MOD-02`).
- Penyimpanan blok permanen di disk (`MOD-05`).

## Task terkait
- [TSK-M13](../tasks/TSK-M13-mempool.md) — 🟢 selesai
