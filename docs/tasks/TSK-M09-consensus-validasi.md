# TSK-M09 · Validasi transaksi dan blok di MOD-04 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-04 (crates/consensus) · **Untuk:** FTR-01 sampai FTR-05
- **Bergantung pada:** TSK-M03, M04, M05, M07, M08, M21, M24, TSK-S02, DEC-09, DEC-13, DEC-17 · **Memblokir:** TSK-F01, TSK-M13
- **Estimasi:** M · **Ditulis:** Quick (kode dan test), 31 Oktober 2026

## Tujuan
Menerapkan Spesifikasi Validasi v0.1 Bagian 7–9 (`docs/reference/spesifikasi-validasi-v0.1.md`): V1–V10, T1–T6, C1.
Termasuk penolakan PoW regtest di testnet dan mainnet beserta test-nya (DEC-09).

## API (modul `validation` dan `chain_params`, di-re-export dari `lib.rs`)
| Tahap | Fungsi | Aturan |
|---|---|---|
| Bebas konteks | `decode_block`, `check_block_context_free(block, params)` | V1, V2–V4 |
| Konteks header | `check_header_context(header, parent, params)`, `check_future_timestamp(header, now)` | V5–V7, V8 (ditunda) |
| Konteks UTXO | `check_block_transactions(block, params, mined_before, utxos)`, `check_transaction(tx, height, network_id, utxos)` | V9 (T1–T6), V10 (C1, DEC-17) |

- `ChainParams` (network, pow_limit, target dan cap waktu genesis, network_id), `ChainParams::regtest()`, `ChainParams::new` untuk testnet/mainnet setelah parameternya ditetapkan.
- `ParentContext { block_id, height, timestamps }`, `BlockSummary { subsidy, fees }`, trait `UtxoLookup` (sudah diimplementasikan untuk `BTreeMap<OutPoint, UtxoEntry>`).
- Error: `BlockError`, `TxError`, `ParamsError`. Tanpa dependensi baru. MOD-04 tidak membaca penyimpanan maupun jam sistem: node mengambil dulu UTXO yang dirujuk blok dari MOD-05 ke `BTreeMap`, dan memberi `now` serta `mined_before`.
- `check_transaction` juga untuk mempool (MOD-07, TSK-M13).

## Vektor blok 1 regtest (dihitung Quick dengan `blake3_ref.py`)
Coinbase tinggi 1, `extra` kosong, satu output 1_910_127_426_536_917 quanta ke kunci genesis regtest; cap waktu genesis + 120; target `0x207fffff`.
txid coinbase `c1161c14…6525e6d2`, merkle root `3f7259fe…d1eaba64`, nonce hasil tambang 0, block_id `a9005667…b3732de9`.
Header yang sama dengan merkle root nol: nonce hasil tambang 3.

## Test (consensus 35 → 56 + 1 ignored, workspace 175 → 196)
- `src/validation/tests.rs` (19): `block1_passes_all_stages`, `regtest_pow_rejected_on_testnet_and_mainnet`, `context_free_rejections_v2_to_v4`,
  `decode_block_rejects_bad_bytes_v1`, `header_context_rejections_v5_and_v7`, `header_context_enforces_asert_target_v6`, `future_timestamp_rule_v8`,
  `spend_genesis_at_maturity`, `immature_coinbase_rejected_t2`, `missing_and_duplicate_inputs_t1`, `double_spend_across_transactions_t1`,
  `chained_spend_in_block_respects_order`, `witness_kind_mismatch_t3`, `outputs_exceed_inputs_t5`, `invalid_signatures_t6`,
  `condition_spend_rules_t4_and_t6`, `coinbase_must_claim_exact_amount_dec17`, `genesis_and_coinbase_are_not_checked_as_regular`,
  `prop_coinbase_must_claim_exact_amount`.
- `src/chain_params/tests.rs` (1): `regtest_chain_params_match_genesis`.
- `src/tests.rs` (1): `validation_reexports_are_accessible`.
