# TSK-M46 · Miner berkala untuk demo explorer publik regtest · 🟢 selesai · Prioritas: Sedang
- **Jenis:** task repo (Docker) · **Modul:** — (hanya `compose.publik.yaml`, tanpa perubahan kode) · **Untuk:** demo publik sementara (FTR-06)
- **Bergantung pada:** TSK-M44, TSK-M45 · **Memblokir:** —
- **Estimasi:** XS · **Ditulis:** Quick, 12 November 2026

## Tujuan
Explorer publik `https://zevrigo.ratuaurion.store` hanya menampilkan blok yang ditambang saat gerbang (tinggi 6), sehingga
tampak statis. Pencipta meminta explorer tampak hidup (12 November 2026). Satu layanan Docker menambang satu blok regtest
secara berkala ke seed, sehingga blok baru muncul terus dan juga tersiar ke rekan yang tersambung lewat seed.

## Isi
| Bagian | Isi |
|---|---|
| `compose.publik.yaml` | Layanan `miner-berkala`: image `zevrigo:regtest`, `/bin/sh -c` mengulang `zevrigo-miner --payout <NUMS> --blocks 1` lalu `sleep $ZVRG_JEDA`; bawaan 120 detik (waktu blok target); `network_mode: service:node`; `init: true`; `restart: unless-stopped` |
| Hadiah | Kunci NUMS `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0` yang tidak bisa dibelanjakan (sama dengan gerbang Q121). Tanpa wallet dan tanpa rahasia di container |
| Dokumen | Spesifikasi Docker v0.1 Bagian 7, README (Docker), docs/tasks |

## Gerbang (Q123)
Konfigurasi compose sah; `miner-berkala` di namespace node, tanpa port host, jeda 120; layanan dinyalakan dengan
`up -d --no-deps` (node, explorer, cloudflared tidak dinyalakan ulang); dalam 300 detik tinggi seed naik paling sedikit 2
dan log mencatat `blok diterima`; `https://zevrigo.ratuaurion.store/api/v1/status` mengikuti tinggi baru; kebijakan
restart `unless-stopped`.

## Batas
- Regtest: PoW dev, tanpa nilai. Chain demo bertambah sekitar 720 blok per hari; volume `zevrigo-publik_zvrg-data` ikut
  tumbuh (blok berisi coinbase saja, kecil).
- Hanya blok, belum ada transaksi berkala (blok berisi coinbase saja). Transaksi demo memerlukan wallet berpassword di
  container dan bisa dibuat di task tersendiri.
- Rekan yang tersambung ke seed tetap bisa menambang instan dan mengganti chain demo (Batas TSK-M45).

## Definition of done
- [x] Gerbang Q123 lolos
- [x] Spesifikasi Docker v0.1 Bagian 7, README, docs/tasks
