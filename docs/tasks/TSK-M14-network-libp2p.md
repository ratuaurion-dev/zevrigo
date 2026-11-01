# TSK-M14 · P2P dasar MOD-06: swarm libp2p 0.57, 5 protokol, codec, kunci rekan, sanksi · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-06 (crates/network) · **Untuk:** FTR-04
- **Bergantung pada:** TSK-M03, TSK-S05, DEC-24 · **Memblokir:** TSK-M28, TSK-F04
- **Estimasi:** L · **Ditulis:** Quick (kode dan test), 1 November 2026

## Tujuan
Lapisan P2P Spesifikasi P2P v0.1 Bagian 1 sampai 4, 6, dan 7 di MOD-06, siap dipakai daemon (TSK-M28). API libp2p
diperiksa Quick dari file .crate resmi (`libp2p` 0.57.1, `-swarm` 0.48.0, `-swarm-derive` 0.36.0, `-request-response`
0.30.0, `-noise` 0.47.0, `-tcp` 0.45.0, `-yamux` 0.48.0, `-identity` 0.3.0, `-connection-limits` 0.7.0,
`-allow-block-list` 0.7.0, `multiaddr` 0.19.0).

## Isi
| Modul | Isi |
|---|---|
| `message` | 6 pesan + `Block` dan `Transaction`, trait `Wire` (`to_wire`, `from_wire`), `MessageError`. Batas: locator 1–64, header ≤ 2_000, ID 1–16, entri ≤ 16, `Ack` 0–4, tanpa sisa byte |
| `frame` | `encode_frame`, `read_frame`, `write_frame`; panjang ≤ 2_097_152 |
| `codec` (internal) | Codec request-response: satu bingkai per permintaan dan per jawaban |
| `protocol` | `ProtocolKind`, `protocol_name` |
| `key` | `load_or_create`, `keypair_from_secret`, `generate_secret` |
| `ban` | `BanList`, `BAN_DURATION` |
| `service` | `Network`, `NetworkConfig`, `NetEvent`, `Reply`, `Offense`, `NetworkError` |

## Keputusan teknis (Quick, tanpa DEC)
- **Pesan rusak dibaca sebagai nilai.** `request_response` 0.30 tidak melaporkan permintaan masuk yang gagal dibaca
  codec (hanya log debug), sehingga pengirimnya tidak bisa disanksi. Codec MOD-06 mengembalikan `Err(MessageError)`
  sebagai isi permintaan atau jawaban; layanan lalu memberi sanksi. Galat I/O biasa tetap galat.
- **Satu behaviour request-response per protokol** (5), digabung dengan `#[derive(NetworkBehaviour)]` (fitur `macros`)
  bersama `connection_limits` dan `allow_block_list` (keduanya selalu ada di `libp2p`, tanpa fitur tambahan). Satu
  behaviour dengan banyak protokol tidak dipakai, karena libp2p menawarkan semua protokolnya untuk setiap permintaan.
- **Koreksi Spesifikasi P2P Bagian 3:** `Status` berukuran 105 byte (1 + 32 + 8 + 32 + 32), bukan 73.
- **Tambahan Spesifikasi P2P Bagian 7:** alamat loopback hanya diblokir per `PeerId`, supaya beberapa node uji di satu
  mesin (TSK-F04) tidak saling memblokir.
- `Status` dikirim otomatis begitu koneksi terbentuk dan dijawab otomatis dengan `Status` lokal
  (`Network::set_local_status`). Versi selain 1, `network_id` lain, atau protokol tidak didukung: putus tanpa sanksi.
- Blokir disimpan di `BanList` (memori) dan di `allow_block_list` (menutup koneksi dan menolak koneksi baru); dicabut
  pada selang sambung ulang setelah 24 jam.
- **Tanpa fitur `tokio` libp2p.** Fitur itu berisi rujukan lemah ke `libp2p-quic`, `-dns`, `-mdns`, `-upnp`; Cargo
  memasukkan paket dari rujukan seperti itu ke Cargo.lock walaupun tidak dikompilasi (Q78 berhenti di J2: Cargo.lock
  415 paket, termasuk QUIC, rustls, aws-lc-sys, hickory). Runtime tokio diaktifkan lewat `libp2p-swarm` dan
  `libp2p-tcp` (fitur `tokio`), dan swarm dibangun langsung dengan `Swarm::new`: TCP, Noise XX, Yamux, batas waktu
  10 detik, sama dengan `SwarmBuilder::with_tcp`.
- `tokio` langsung di MOD-06 dengan fitur `macros` dan `time` (untuk `select!` dan selang sambung ulang).
- Rahasia kunci rekan belum di-zeroize di memori (sama dengan wallet; dicatat sebagai risiko).

## Test (network 1 → 26, workspace 310 → 335)
- `message` (8): ukuran dan isi `Status`, batas locator, 2_000 header, ID `GetBlocks`, entri dan tag `Blocks`, nilai
  `Ack`, `Block`/`Transaction` sama dengan encoding Format v0.1, sisa byte ditolak di setiap pesan.
- `frame` (4): awalan minimal, batas 2 MiB tepat, awalan rusak, aliran terpotong. `codec` (1): pesan rusak sampai
  sebagai nilai, nilai `Err` tidak pernah dikirim.
- `protocol` (2): nama dari 16 hex pertama `network_id`. `key` (3): vektor RFC 8032 TEST 1 → PeerId
  `12D3KooWQK1wnefoLrcVHbbnf5tLzbopUd3K3bFAoJpA7YJgL5pV` (dihitung Quick terpisah), file dibuat sekali dengan izin
  `0600`, panjang salah ditolak. `ban` (2): 24 jam, loopback tidak diblokir per IP.
- `service` (5, TCP sungguhan di 127.0.0.1): pertukaran `status`, `headers`, `blocks`, `block`, `tx`; jaringan lain
  diputus tanpa blokir; jumlah entri `Blocks` salah → sanksi, blokir, putus; sambung ulang setelah diputus.

## Dependensi (DEC-10 bagian 5)
Daftar paket baru di Cargo.lock beserta lisensi, build script, dan jumlah baris `unsafe`:
[reference/paket-mod06-v0.1.txt](../reference/paket-mod06-v0.1.txt), dibuat skrip gerbang Q78.
