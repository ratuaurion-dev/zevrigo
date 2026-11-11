# MOD-06: Network (`zevrigo-network`)

- **ID:** `MOD-06`
- **Path:** `crates/network`
- **Paket:** `zevrigo-network`
- **Status:** 🟢 P2P dasar selesai (TSK-M14); dipakai daemon sejak TSK-M28; WebSocket, alamat `/dns*`, dan keepalive (TSK-M45)
- **Bergantung pada:** `MOD-01`; pustaka `libp2p` 0.57.1 (tanpa fitur `tokio`; fitur `websocket` sejak TSK-M45), `libp2p-swarm` dan `libp2p-tcp` (fitur `tokio`), dan `tokio` (DEC-10 bagian 2, 5, dan 12, DEC-24, DEC-33)
- **Dipakai oleh:** `MOD-11`, `MOD-17`

## Tujuan

Menangani komunikasi *peer-to-peer* antar node menggunakan `rust-libp2p` 0.57 (DEC-24, DEC-10 bagian 5), sesuai [spesifikasi-p2p-v0.1.md](../reference/spesifikasi-p2p-v0.1.md).

## Tanggung Jawab

- `message`: pesan `Status` (105 byte), `GetHeaders`, `Headers`, `GetBlocks`, `Blocks`, `Ack`, serta `Block` dan `Transaction`, dengan encoding kanonis dan batas Bagian 3 (trait `Wire`).
- `frame`: bingkai `varint<32>(panjang) ‖ isi`, paling banyak 2 MiB; panjang tidak minimal atau aliran terpotong adalah pelanggaran rekan.
- `protocol`: nama `/zvrg/<16 hex network_id>/<nama>/1` untuk `status`, `headers`, `blocks`, `block`, `tx`.
- `key`: kunci rekan Ed25519, 32 byte mentah di `<datadir>/<jaringan>/p2p_key` (izin `0600`), dibuat sekali.
- `ban`: blokir 24 jam per `PeerId` dan per IP (loopback hanya per `PeerId`).
- `transport` (TSK-M45): `DnsTcp`, TCP yang juga mendial `/dns4`, `/dns6`, `/dns` (resolver sistem setiap dial, tanpa `libp2p-dns`), dasar TCP dan WebSocket; `is_websocket`.
- `service::Network`: swarm TCP atau WebSocket (`/ws`, klien juga `/tls/sni/<nama>/ws`) + Noise XX + Yamux, lima protokol request-response, 8 koneksi keluar / 32 masuk / 1 per `PeerId`, koneksi menganggur 300 detik, waktu tunggu jawaban 30 detik, sambung ulang ke `--connect` setiap 10 detik, pertukaran `status` otomatis saat terhubung dan setiap 60 detik (keepalive, DEC-33), sanksi pesan rusak dan jumlah entri `Blocks` yang salah, putus setelah 3 waktu habis berturut-turut. Permintaan masuk lain diteruskan sebagai `NetEvent` beserta `Reply`.

## Bukan Tanggung Jawab

- Validasi aturan konsensus atas blok, header, atau transaksi yang diterima (`MOD-04`, dipanggil MOD-11). Sanksi untuk isi yang tidak sah dijatuhkan MOD-11 lewat `Network::punish`.
- Indeks header, locator, sinkronisasi, pemilihan chain, dan siaran ke semua rekan (`MOD-11`, TSK-M28).
- Kebijakan penerimaan dan pengurutan biaya transaksi di antrean lokal (`MOD-07`).

## Task terkait
- [TSK-M14](../tasks/TSK-M14-network-libp2p.md) — 🟢 selesai
- [TSK-M28](../tasks/TSK-M28-p2p-daemon.md) — 🟢 selesai (MOD-11)
- [TSK-M45](../tasks/TSK-M45-seed-websocket.md) — 🟢 selesai (WebSocket, `/dns*`, keepalive, seed lewat Cloudflare Tunnel)
