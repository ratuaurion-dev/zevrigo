# TSK-M24 · Pesan tanda tangan sighash v1 di MOD-02 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-02 (crates/crypto) · **Untuk:** FTR-01 sampai FTR-05
- **Bergantung pada:** TSK-M21, DEC-13 · **Memblokir:** TSK-M09, TSK-M16
- **Estimasi:** XS · **Ditulis:** Quick (kode dan test), 30 Oktober 2026

## Tujuan
Satu fungsi `sighash_v1` yang menghasilkan pesan 32 byte untuk tanda tangan BIP340 setiap input, sesuai
`docs/reference/spesifikasi-format-v0.1.md` Bagian 9 (DEC-13). Ditaruh di MOD-02 karena MOD-04 (validasi) dan
MOD-09 (wallet) sama-sama memakainya, dan MOD-02 sudah punya `txid` serta domain `Sighash`.

## API
`pub fn sighash_v1(network_id: &Hash32, tx: &Transaction, input_index: u32, spent_outputs: &[Output]) -> Result<Hash32, SighashError>`
- `SighashError::Coinbase`, `InputIndexOutOfRange { index, inputs }`, `SpentCountMismatch { inputs, spent }`.
- Tanpa dependensi baru. Re-export dari lib.rs.

## Vektor (dihitung Quick dengan blake3_ref.py)
O(q) = output q quanta, kind 0, payload `77` x 32. Transaksi V1 (kartu TSK-M03), input 0.
| ID | network_id | spent | msg |
|---|---|---|---|
| S1 | H6 | O(2 ZVRG) | `f2ee67583afb2b9b35b88f175ae593b4e1800390b5c6b69a6ce5a2f9c11922ad` |
| S2 | H3 | O(2 ZVRG) | `c3635edb315f2813f1361a360321711813580bd92c075d02ec77a256bcce0e00` |
| S3 | H6 | O(2 ZVRG + 1 quanta) | `d9b77ec8b60e72c105623bfcba49c7cf4275a6d8cf81ba5780f183e8c26f6dbf` |

## Test (crypto 24 → 30, workspace 143 → 149)
`src/sighash/tests.rs`: `vectors_sighash_v1_s1_to_s3`, `sighash_rejects_coinbase`,
`sighash_rejects_input_index_out_of_range`, `sighash_rejects_spent_count_mismatch`,
`sighash_does_not_depend_on_witness`. `src/tests.rs`: `sighash_reexport_is_accessible`.
