# TSK-M08 · Target compact, ASERT, chainwork, cap waktu, dan genesis regtest di MOD-04 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-04 (crates/consensus) · **Untuk:** FTR-01 sampai FTR-04
- **Bergantung pada:** TSK-M05, TSK-M25, TSK-S02, DEC-15 · **Memblokir:** TSK-M09
- **Estimasi:** M · **Ditulis:** Quick (kode dan test), 30 Oktober 2026

## Tujuan
Menerapkan Spesifikasi Validasi v0.1 Bagian 2–6 dan 10 (`docs/reference/spesifikasi-validasi-v0.1.md`), sehingga
TSK-M09 tinggal memakai fungsi siap pakai untuk V2, V6, V7, V8, dan pemilihan chain.

## API (semua di-re-export dari `lib.rs`)
| Modul | Isi |
|---|---|
| `uint` | `U256`: `ZERO`, `ONE`, `MAX`, `from_u128`, `from_be_bytes`, `to_be_bytes`, `is_zero`, `bits`, `checked_add`, `checked_sub`, `checked_div`, urutan numerik. Ditulis sendiri, tanpa crate bignum |
| `difficulty` | `decode_compact`, `encode_compact`, `asert_target`, `block_work`, `chain_work`, `median_time_past`, `timestamp_after_median` (V7), `timestamp_within_future_limit` (V8), `ASERT_HALF_LIFE_SECS`, `DifficultyError` |
| `chain_params` | `pow_limit_compact`, `pow_limit`, `regtest_genesis`, `regtest_network_id`, konstanta `REGTEST_*`. Testnet/mainnet: `TODO(param)` (placeholder `0`, sengaja tidak sah) |

Tanpa dependensi baru. MOD-04 tidak membaca jam sistem: `now` untuk V8 diberikan pemanggil.

## Vektor (dari spesifikasi, dihitung Quick dengan `asert_ref.py`)
K1–K9 compact, A1–A11 ASERT, W1–W5 kerja blok, M1–M4 median waktu, dan genesis regtest (186 byte, txid, wtxid,
merkle root, `network_id` `8b9eb318…364dab`).

## Test (consensus 10 → 35, primitives 56 → 57 lewat TSK-M25, workspace 149 → 175)
- `src/uint/tests.rs` (9): `u256_constants_bits_and_be_bytes`, `u256_ordering_is_numeric`, `u256_add_and_sub_detect_overflow`,
  `u256_division_vectors`, `u256_mul_shift_helpers`, `prop_u256_matches_u128`, `prop_be_bytes_round_trip`,
  `prop_mul_shift_matches_u128`, `prop_division_identity`.
- `src/difficulty/tests.rs` (12): `compact_vectors_k1_to_k9`, `compact_rejects_invalid_encodings`, `asert_vectors_a1_to_a11`,
  `block_work_vectors_w1_to_w5`, `chain_work_sums_and_detects_overflow`, `median_time_past_vectors_m1_to_m4`,
  `timestamp_rules_v7_and_v8`, `prop_compact_encoding_is_canonical`, `prop_encode_then_decode_rounds_down`,
  `prop_asert_on_schedule_keeps_anchor`, `prop_asert_is_monotonic_in_parent_time`, `prop_block_work_is_non_increasing`.
- `src/chain_params/tests.rs` (3): `regtest_genesis_matches_spec_vectors`, `regtest_genesis_key_is_public_key_of_secret_one`,
  `pow_limit_per_network`.
- `src/tests.rs` (1): `difficulty_reexports_are_accessible`.
