# TSK-M30 · API HTTP explorer dan program `zevrigo-explorer` · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-14 (apps/explorer) · **Untuk:** FTR-05
- **Bergantung pada:** TSK-M29 · **Memblokir:** TSK-F05
- **Estimasi:** M · **Ditulis:** Quick (kode dan test), 2 November 2026

## Tujuan
API baca saja `GET /api/v1/…` dan program explorer sesuai Spesifikasi Explorer v0.1 Bagian 5 dan 6, di atas indexer
TSK-M29. API axum 0.8.9 dan axum-core 0.5.5 diperiksa Quick dari file .crate resmi, termasuk fitur yang mengaktifkan
setiap impl (`serve` dan `Router` sebagai layanan butuh `tokio` + `http1`; `State`, `Method`, `Uri`, dan
`IntoResponse` untuk tuple selalu ada). Cargo memilih axum-core 0.5.6 (rilis terbaru); bedanya dengan 0.5.5 hanya
atribut `#[diagnostic::do_not_recommend]` untuk pesan galat compiler (diperiksa dari .crate, Q83 berhenti di J2).

## Isi
| Modul | Isi |
|---|---|
| `api` | `respond(index, sync, network, method, path, query) -> ApiResponse { status, body }`: 9 jalur, galat 400/404/405/500/503, `NodeView`, `SyncStatus`, `BLOCKS_PER_PAGE` (20), `ADDRESS_PAGE_SIZE` (25) |
| `rpc_source` | `RpcSource` (`ChainSource` di atas `RpcClient`, `Handle::block_on`; `1001` = tidak ada), `node_view` |
| `cli` | `parse_args` (`--regtest --datadir <folder> [--rpc-url <url>] [--http-port <port>]`, `--help`, `--version`), `Config`, `ArgError`, `default_http_port` (28_802), `default_rpc_url`, `db_path` |
| `server` | `Shared` (indeks + `SyncStatus` di mutex), `Shared::{respond, sync_once}`, `router` (axum, semua jalur ke satu handler di `spawn_blocking`), `init_logging`, `run` (indeks, runtime, sinkronisasi 2 detik, `axum::serve` dengan graceful shutdown, stdout `http: …`), `ExplorerError` |
| `main.rs` | Biner tipis: kode keluar 0, 1 (galat jalan), 2 (argumen) |

## Keputusan teknis (Quick, tanpa DEC)
- **API sebagai fungsi murni.** Router axum hanya punya satu handler cadangan (`fallback`) yang meneruskan metode,
  jalur, dan query ke `api::respond`; seluruh API diuji tanpa jaringan, dan axum hanya butuh fitur `http1` dan
  `tokio`. JSON disusun dengan `serde_json::Value` (tanpa makro `json!`), query string dibaca sendiri, sehingga fitur
  axum `json`, `query`, `form`, `matched-path`, `original-uri`, `tower-log`, dan `tracing` tidak dipakai. Cargo.lock
  hanya bertambah 4 paket (axum, axum-core, matchit, mime).
- **Tambahan Spesifikasi Explorer Bagian 5:** `/status` memuat `last_error` (galat langkah sinkronisasi terakhir,
  misalnya node mati) di samping `index_error` (galat yang menghentikan indexer); `/blocks` dibungkus `{"blocks": […]}`;
  `/search` untuk blok mengembalikan block_id sebagai `key`; galat `405 method_not_allowed` dan `500 internal`.
- **RPC sinkron di thread blocking.** Langkah sinkronisasi memegang mutex indeks selama memanggil node; permintaan HTTP
  menunggu sebentar. Cukup untuk regtest; pembatasan blok per langkah menyusul jika sinkronisasi awal panjang.
- Test node memakai node tiruan di belakang `RpcServer` sungguhan (pola miner TSK-M20), bukan pustaka node, karena
  MOD-14 tidak boleh bergantung pada MOD-11 (DEC-25). Uji dengan node dan miner sungguhan ada di TSK-F05 (MOD-17).

## Test (explorer 10 → 16, workspace 350 → 356)
- `api`: `api_serves_indexed_chain` (9 jalur, encoding nilai, konfirmasi, riwayat alamat per halaman, alokasi genesis
  dibelanjakan), `api_rejects_bad_requests_and_reports_status` (16 kasus galat, `in_mempool`, `not_ready`, ujung node,
  `last_error`, `index_error`).
- `rpc_source`: `rpc_source_feeds_indexer_and_follows_reorg` (RPC sungguhan ke node tiruan, reorg 2 → 3 blok, node mati).
- `server`: `http_server_answers_and_syncs_from_node` (axum di port bebas, HTTP mentah: `503` lalu sinkron, `405`,
  `404`, tanpa `Set-Cookie`, node mati tetap melayani indeks lama).
- `cli`: `parses_help_version_and_run`, `rejects_bad_arguments` (12 kasus). Smoke test bantuan diperbarui.

## Dependensi (DEC-10 bagian 6)
Paket baru di Cargo.lock: [reference/paket-mod14-axum-v0.1.txt](../reference/paket-mod14-axum-v0.1.txt), dibuat
skrip gerbang Q83.
