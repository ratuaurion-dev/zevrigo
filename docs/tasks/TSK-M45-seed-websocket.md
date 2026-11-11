# TSK-M45 · Seed P2P lewat WebSocket di belakang Cloudflare Tunnel · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-06 (`transport`, `service`), MOD-11 (`daemon`, `p2p`), repo (Docker) · **Untuk:** demo publik sementara (FTR-06)
- **Bergantung pada:** TSK-M14, TSK-M28, TSK-M44, DEC-33, DEC-10 bagian 12 · **Memblokir:** paket testnet publik, panduan bergabung
- **Estimasi:** M · **Ditulis:** Quick, 11 November 2026

## Tujuan
Membuka seed node regtest ke publik dari komputer pencipta lewat Cloudflare Tunnel, agar node lain di internet bisa
tersambung tanpa membuka port router. Cloudflare Tunnel hanya meneruskan HTTP dan WebSocket ke peserta tanpa cloudflared,
jadi P2P perlu transport WebSocket (DEC-33). Pencipta memilih regtest dulu (11 November 2026).

## Isi
| Bagian | Isi |
|---|---|
| MOD-06 `transport` (baru) | `DnsTcp`: TCP libp2p yang juga mendial `/dns4`, `/dns6`, `/dns` (nama di-resolve `tokio::net::lookup_host` setiap dial, tanpa `libp2p-dns`); `is_websocket` |
| MOD-06 `service` | Transport `DnsTcp` atau WebSocket di atas `DnsTcp` (`or_transport`), lalu Noise + Yamux seperti sebelumnya. Keepalive: `status` ke setiap rekan setiap `KEEPALIVE_INTERVAL` (60 detik), `NetworkConfig::keepalive_interval` |
| MOD-11 `daemon` | `--p2p-ws-port <port>` (bawaan mati; `0` = port bebas), `p2p_ws_listen_address` (`/ip4/127.0.0.1/tcp/<port>/ws`), `ArgError::InvalidP2pWsPort`; `--connect` menerima `/dns4/<nama>/tcp/443/tls/sni/<nama>/ws/p2p/<PeerId>` |
| MOD-11 `p2p` | Alamat WebSocket pertama dicetak sebagai `p2p-ws: ...`; baris `p2p: ...` tetap alamat TCP |
| `compose.publik.yaml` | Node `--p2p-ws-port 28803`; public hostname kedua `zevrigo-seed.ratuaurion.store` → HTTP `localhost:28803` |
| Dependensi | Fitur `websocket` libp2p (+6 paket, DEC-10 bagian 12, reference/paket-mod06-websocket-v0.1.txt); fitur `net` dan `rt` tokio di MOD-06 (tanpa paket baru) |
| Test | MOD-06: `dns_target_reads_name_port_and_family`, `websocket_addresses_are_recognized`, `websocket_and_dns_addresses_connect` (dua node lewat `/dns4/localhost/.../ws` dan `/dns4/localhost/tcp/...`), `keepalive_sends_status_periodically`; MOD-11: `parses_websocket_options`. Workspace 432 → 437 |

## Gerbang (Q122)
Cargo.lock bertambah tepat 6 paket dengan checksum sama dengan file .crate yang diperiksa Quick, 0 hilang; `make check-all`
437 test; image dibangun ulang; node publik mencetak `p2p-ws:`; dari node di container lain (jaringan Docker biasa, lewat
internet) `--connect /dns4/zevrigo-seed.ratuaurion.store/tcp/443/tls/sni/zevrigo-seed.ratuaurion.store/ws/p2p/<PeerId>`
tersambung, mengejar tinggi seed, menerima blok baru lewat siaran, dan masih tersambung sesudah 150 detik (keepalive
melewati batas diam WebSocket Cloudflare).

## Batas
- Regtest: PoW dev, siapa pun yang tersambung bisa menambang instan dan mengganti chain demo. Hanya bukti P2P.
- Komputer pencipta harus menyala; jika mati, seed ikut mati. Node lain menyambung ulang setiap 10 detik.
- Semua rekan yang masuk lewat tunnel tampak dari 127.0.0.1: batas 32 koneksi masuk berlaku untuk semua peserta bersama,
  dan blokir hanya per `PeerId`.
- Tanpa penemuan rekan otomatis: peserta hanya tersambung ke seed (dan alamat lain yang mereka tulis sendiri).

## Definition of done
- [x] `make check-all` lolos (437 test)
- [x] Gerbang Q122 lolos
- [x] Spesifikasi P2P v0.1 Bagian 1, 4, 7, 8, 10, 11; Spesifikasi Docker v0.1 Bagian 7; DEC-33, DEC-10 bagian 12; MOD-06,
  MOD-11; README; docs/tasks
