# TSK-M28 · P2P di daemon: indeks header, sinkronisasi header lalu blok, siaran, opsi daemon · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-11 (apps/node), ditambah satu fungsi MOD-04 · **Untuk:** FTR-04
- **Bergantung pada:** TSK-M14, TSK-M27 · **Memblokir:** TSK-F04
- **Estimasi:** L · **Ditulis:** Quick (kode dan test), 1 November 2026

## Tujuan
Daemon regtest saling tersambung lewat MOD-06 dan mengikuti chain dengan chainwork terbanyak (Spesifikasi P2P v0.1
Bagian 4 sampai 8): pertukaran `status`, sinkronisasi header lalu blok, siaran blok dan transaksi, sanksi, serta opsi
`--p2p-port` dan `--connect`.

## Isi
| Bagian | Isi |
|---|---|
| MOD-04 `validation` | `check_header_context_free(header, params)`: V2 dan V3 untuk header tanpa badan. `check_block_context_free` kini memanggilnya lalu memeriksa V4 (perilaku sama) |
| MOD-11 `chain` | Indeks header di memori (`headers`, ikut dibuang `mark_invalid`); `receive_block` → `BlockReceipt { result, new_tip }`; `receive_transaction` → `TxOutcome` (`Added`, `Duplicate`, `Rejected`); `tip_summary`; `transaction_result`. RPC `submit_block` dan `send_raw_transaction` memakai keduanya (hasil RPC tidak berubah) |
| MOD-11 `chain::sync` | `locator_heights`, `locator`, `headers_after` (jawaban `GetHeaders`), `blocks_by_id` (jawaban `GetBlocks`, chain aktif dan cabang), `accept_headers` → `HeadersOutcome` (`Accepted`, `Disconnected`, `Invalid`, `Deferred`), `missing_blocks` (jalur header-saja menuju header terberat, 16 sekali) |
| MOD-11 `p2p` | `Relay` (backend RPC yang mengumumkan ujung baru dan transaksi baru), `P2pService::run` (event MOD-06, pengumuman, percobaan ulang 10 detik), `block_ack`, `tx_ack`, `local_status` |
| MOD-11 `daemon` | `--p2p-port <port>` (bawaan regtest 28_800; 0 = bebas), `--connect <multiaddr>` berulang, `p2p_key_path` (`<datadir>/regtest/p2p_key`), `p2p_listen_address` (`/ip4/127.0.0.1/tcp/<port>`), `default_p2p_port`. Stdout: `rpc: …` lalu `p2p: /ip4/127.0.0.1/tcp/<port>/p2p/<PeerId>` |

## Alur
1. Terhubung: MOD-06 menukar `status`. Jika chainwork rekan lebih besar, node mengirim `GetHeaders` (locator chain
   aktif) ke rekan itu; paling banyak satu permintaan header per rekan.
2. `Headers`: `accept_headers`. Tidak menyambung atau tidak sah → sanksi. 2_000 header → minta lagi dengan locator yang
   diawali header terakhir. Selain itu → minta badan blok.
3. Badan blok: satu `GetBlocks` berjalan sekali waktu, 16 ID dari `missing_blocks`, ke rekan dengan chainwork terbesar
   yang belum gagal memberi ID pertama. Setiap blok lewat `receive_block`. Blok tidak sah atau blok lain dari yang
   diminta → sanksi. `Entry 00` → sisa batch dilewati, ID itu diminta dari rekan lain.
4. Siaran: blok yang menjadi ujung chain aktif baru (RPC, `block` dari rekan, atau akhir batch `GetBlocks`) dikirim ke
   rekan yang belum diketahui punya blok itu, kecuali pengirimnya; transaksi baru di mempool ke semua rekan kecuali
   pengirimnya. `Status` lokal diperbarui setiap ujung chain berubah.
5. `block` dari rekan: `Ack` 0 diterima atau duplikat, 1 induk belum dikenal (lalu node meminta header ke pengirim),
   2 ditunda V8, 3 tidak sah (pengirim disanksi). `tx`: 0 diterima atau duplikat, 4 kebijakan mempool, 3 tidak sah
   (disanksi).
6. Setiap 10 detik: daftar gagal per rekan dikosongkan, sinkronisasi header dengan rekan terberat dicoba lagi, badan
   blok yang masih kurang diminta lagi.

## Keputusan teknis (Quick, tanpa DEC)
- **Satu fungsi baru di MOD-04.** V2 dan V3 untuk header saja wajib di MOD-04 (aturan konsensus tidak ditulis di
  MOD-11). Hanya pemecahan `check_block_context_free`, tanpa aturan baru; preseden TSK-M15 (MOD-02) dan TSK-M20 (MOD-08).
- **`Ack` transaksi:** input tidak ada (`MissingUtxo`), coinbase belum matang, dan kunci waktu kondisi bergantung pada
  view chain penerima atau waktu, jadi dijawab `4` tanpa sanksi, sama seperti kebijakan antrean. Pelanggaran T1 sampai
  T6 lain dan struktur dijawab `3` dengan sanksi (Spesifikasi P2P Bagian 6 dilengkapi).
- **Indeks header di memori**, seperti blok cabang (TSK-M27): hilang saat node dibuka ulang, tanpa batas jumlah di v0.1.
- **Ujung chain rekan** hanya dari `status` (saat terhubung). Pembaruan dari `block` dan `Headers` (Spesifikasi P2P
  Bagian 4 langkah 3) belum dipakai; selisihnya ditutup siaran blok dan `Ack 1`. Dicatat untuk TSK-F04.
- **Layanan P2P berjalan di task utama daemon** bersama penantian sinyal (`tokio::select!`), bukan `tokio::spawn`.
- Daemon hanya mendengar di 127.0.0.1 (regtest, Spesifikasi P2P Bagian 1).
- Tanpa dependensi baru: tokio di MOD-11 ditambah fitur `sync` dan `time`; Cargo.lock tidak berubah.

## Test (node 18 → 25, workspace 335 → 342)
- `chain/tests.rs`: `locator_heights_follow_spacing` (0, 5, 9, 30, 2^53, u64::MAX), `headers_then_blocks_sync_two_nodes`
  (20 blok: locator, `headers_after` dengan `stop` dan `max`, `accept_headers`, `missing_blocks` 16 + 4,
  `blocks_by_id`, `receive_block` dengan `new_tip`), `bad_headers_are_rejected_or_deferred` (tidak menyambung, V3, V5,
  V8 lalu diterima).
- `daemon/tests.rs`: `parses_p2p_options` (port, `--connect` berulang, path kunci, alamat dengar, galat).
- `p2p/tests.rs`: `acks_follow_spec_table`, `relay_announces_new_tips_only`.
- `tests/regtest_node.rs`: `two_daemons_sync_and_relay_over_p2p` (biner sungguhan: node A menambang 100 blok, node B
  `--connect` mengejar lewat header dan `GetBlocks`, transaksi lewat RPC B sampai ke mempool A, blok 101 dari A sampai
  ke B, blok 102 dari B sampai ke A); uji lama kini juga memeriksa baris `p2p:` dan PeerId yang sama setelah dibuka ulang.
