# TSK-M19 · CLI wallet regtest: info, wallet-create, wallet-restore, address, balance, send · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-12 (apps/cli) · **Untuk:** FTR-03
- **Bergantung pada:** TSK-M12, TSK-M15, TSK-M16, DEC-21 · **Memblokir:** TSK-F03
- **Estimasi:** M · **Ditulis:** Quick (kode dan test), 1 November 2026

## Tujuan
Perintah baris perintah untuk wallet regtest sesuai Spesifikasi Wallet v0.1 Bagian 5 sampai 7, memakai MOD-09 untuk
frasa, kunci, alamat, pemilihan koin, dan tanda tangan, serta klien RPC MOD-08 untuk node. Pustaka `zevrigo_cli`
ditambah biner tipis, seperti miner.

## Perintah
```
zevrigo-cli [--rpc-url <url>] info
zevrigo-cli --datadir <folder> [--passphrase <teks>] [--rpc-url <url>] <perintah wallet> [opsi]
  wallet-create                      wallet baru 24 kata (entropi 32 byte dari /dev/urandom)
  wallet-restore --mnemonic "<kata>" pulihkan dari 12 sampai 24 kata
  address                            alamat penerimaan berikutnya dan payload hex-nya (tanpa node)
  balance                            saldo lewat scan_utxos (gap limit 20)
  send --to <alamat> --amount <ZVRG> [--fee-rate <quanta per byte>]
```
- Galat argumen: kode keluar 2. Galat menjalankan: kode keluar 1. Opsi boleh sebelum atau sesudah perintah; opsi yang
  tidak berlaku untuk perintah itu ditolak. `--amount` memakai `Amount::from_zvrg_str` dan tidak boleh 0.
- Hanya regtest: node wajib melapor `network = regtest` (`WrongNetwork`), alamat dengan HRP lain ditolak.
- `network_id` = `get_block_hash(0)`. `aux_rand` tanda tangan 32 byte dari `/dev/urandom`.

## File wallet (`<datadir>/regtest/wallet.json`)
JSON berindentasi: `version` 1, `network` `"regtest"`, `mnemonic` (ternormalkan), `next_index { receive, change }`.
`deny_unknown_fields`; versi, jaringan, dan frasa diperiksa saat dibaca. File baru `create_new` dengan izin `0600`
(Unix), tidak pernah menimpa. Pembaruan: tulis `<path>.tmp` (`0600`) lalu `rename`. Passphrase tidak disimpan.

## Keputusan teknis (Quick, tanpa DEC)
- Tanpa dependensi baru di Cargo.lock. Dependensi MOD-12: MOD-01, MOD-08, MOD-09 (DEC-21), `serde` dan `serde_json`
  (file wallet; DEC-10 bagian 2), `thiserror`, `tokio` (`rt`; dev `rt-multi-thread`). Cargo.lock hanya bertambah 5
  rujukan di `zevrigo-cli`.
- Pemindaian (`scan`): per cabang, indeks terpakai = maksimum `next_index` file dan indeks yang punya UTXO; dipindai
  sampai 20 indeks di atasnya, paling banyak 64 payload per `scan_utxos`. UTXO jenis 1 dan payload asing diabaikan.
  `balance` dan `send` menyimpan indeks berikutnya yang lebih tinggi hasil pemindaian, sehingga wallet yang dipulihkan
  tidak memakai ulang alamat setelah `balance` pertama.
- `send`: kembalian ke indeks `branch 1` berikutnya; indeks itu dinaikkan hanya jika transaksi diterima node dan memakai
  output kembalian. Penolakan `1002` dilaporkan dengan nama aturannya (`Rejected { reason }`).
- Keterbatasan v0.1: saldo dihitung dari UTXO di chain, jadi koin yang sudah dikirim tetapi belum ditambang masih
  terhitung, dan `send` kedua sebelum blok berikutnya bisa ditolak mempool (T1, konflik). Ditangani saat wallet punya
  catatan transaksi tertunda.
- `Debug` untuk `Config`, `Action::WalletRestore`, dan `WalletFile` tidak mencetak passphrase atau frasa.

## Test (cli 3 → 19, workspace 289 → 305)
- `src/args/tests.rs`: `parses_help_version_and_info`, `parses_wallet_actions_with_options_in_any_order`,
  `rejects_bad_arguments`, `debug_hides_passphrase_and_mnemonic`.
- `src/wallet_file/tests.rs`: `json_format_is_exact_and_round_trips`, `from_json_rejects_bad_content`,
  `create_load_save_and_never_overwrite` (termasuk izin `0600`).
- `src/scan/tests.rs`: `empty_wallet_scans_gap_limit_on_both_branches`, `utxo_inside_gap_extends_scan`,
  `stored_next_index_and_batches_of_64`, `kind_one_utxo_with_wallet_payload_is_ignored`, `lookup_error_stops_scan`.
- `src/commands/tests.rs` (node tiruan di atas `RpcServer`): `create_restore_and_address_without_node` (W1, W5, W6,
  W8, W9), `balance_and_send_match_vector_t1` (byte transaksi yang diterima node sama persis dengan vektor T1
  TSK-M16), `send_errors_leave_wallet_unchanged`, `run_dispatches_actions`.
- `tests/smoke.rs`: `help_and_no_args_print_placeholder_info` diganti `help_and_no_args_print_usage`.
- Uji dengan node sungguhan (MOD-04 memeriksa T1 sampai T6) di TSK-F03.
