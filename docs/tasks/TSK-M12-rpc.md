# TSK-M12 · Tipe pesan, encoding nilai, dan server/klien JSON-RPC di MOD-08 · 🟡 bagian A selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-08 (crates/rpc) · **Untuk:** FTR-01 sampai FTR-03
- **Bergantung pada:** TSK-M03, TSK-S04, DEC-10 bagian 2 · **Memblokir:** TSK-M18, TSK-M19, TSK-M20, TSK-F01
- **Estimasi:** M (dua langkah) · **Ditulis:** Quick (kode dan test), 31 Oktober 2026

## Tujuan
Menerapkan [Spesifikasi RPC v0.1](../reference/spesifikasi-rpc-v0.1.md) di MOD-08. Dipecah dua agar perubahan
Cargo.lock pertama (serde_json) terpisah dari pemakaian jsonrpsee dan tokio.
- **Bagian A (selesai):** encoding nilai (Bagian 2), parameter dan hasil 8 metode (Bagian 3), kode error (Bagian 4),
  parameter transpor (Bagian 1). Tanpa jaringan.
- **Bagian B (berikutnya):** server dan klien `jsonrpsee` 0.26 lewat HTTP, pemetaan `ValueError` ke `-32602`, batas
  ukuran request, dan uji server-klien lokal.

## Isi bagian A
| Modul | Isi |
|---|---|
| `value` | `Hex32` (hex huruf kecil 64 karakter: hash, payload, chainwork), `HexBytes`, `Quanta` (jumlah koin sebagai string desimal quanta), `DecimalU64` (nonce), `CompactHex` (8 karakter), `NetworkName`, `KindNumber` (0 atau 1), `ValueError`, `decode_hex`/`encode_hex` |
| `types` | Nama 8 metode (`METHODS`), parameter dengan `deny_unknown_fields`, `PayloadList` (1 sampai 64), hasil `ChainInfo`, `BlockInfo::from_block`, `BlockTemplate`, `SubmitBlockResult`/`SubmitStatus`, `SendRawTransactionResult`, `MempoolInfo`, `UtxoInfo::from_entry` (`confirmations`, `spendable_at`) |
| `error` | Kode `-32700` sampai `-32603`, `1001` sampai `1003`, `RejectData { reason }` |
| `transport` | `DEFAULT_HOST` 127.0.0.1, `MAX_REQUEST_BYTES` 1 MiB, `default_port` (regtest 28_801, testnet 18_801, mainnet 8_801) |

## Keputusan teknis (Quick, tanpa DEC)
- Setiap nilai berencoding khusus adalah tipe pembungkus dengan `serde(try_from = "String")`, sehingga nilai yang
  salah (hex huruf besar, panjang salah, jumlah koin berupa angka JSON atau bernol depan) ditolak saat decode.
- Parameter menolak field tak dikenal; hasil tidak, agar klien lama tetap bisa membaca versi berikutnya.
- MOD-08 hanya bergantung pada MOD-01: chainwork dikirim sebagai 32 byte big-endian (`Hex32`), konversi dari U256
  dilakukan node (MOD-11). txid dihitung pemanggil (MOD-02) lalu diberikan ke `BlockInfo::from_block`.
- Dependensi crate: `serde`, `serde_json`, `thiserror`; dev: `proptest`. Cargo.lock menambah tepat 3 paket:
  `serde_json` 1.0.154, `itoa`, `zmij` (dependensi bawaan serde_json). serde 1.0.229 sudah ada sebelumnya.

## Test bagian A (rpc 1 → 18, workspace 199 → 216)
- `value/tests.rs`: `hex32_accepts_lowercase_and_rejects_bad_input`, `hex_bytes_rules`,
  `quanta_string_canonical_rules`, `compact_hex_and_decimal_u64_rules`, `network_and_output_kind_encoding`,
  `prop_quanta_round_trip`, `prop_hex32_round_trip`, `prop_hex_bytes_round_trip`.
- `types/tests.rs`: `params_reject_unknown_fields_and_wrong_types`, `scan_utxos_params_accept_1_to_64_payloads`,
  `chain_info_json_vector`, `block_info_from_block_matches_header`, `utxo_info_confirmations_and_spendable_at`,
  `submit_block_result_omits_absent_fields`.
- `tests.rs`: `error_codes_match_spec`, `transport_defaults_match_spec`, `crate_root_reexports_are_accessible`.
