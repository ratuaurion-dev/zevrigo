# Spesifikasi Paket Docker Zevrigo Coin (ZVRG) v0.1: node, explorer, dan antarmuka web untuk regtest

> Diperbarui: 12 November 2026 · Pemilik: pencipta · Ditulis: Quick · Task: TSK-M33, TSK-M44, TSK-M45, TSK-M46, TSK-M47 · Status: 🟢 v0.1
> Lokasi di repo: `docs/reference/spesifikasi-docker-v0.1.md`

| Status | Ringkasan |
|---|---|
| 🟢 Dikunci | Satu image `zevrigo:regtest` (node, explorer, miner, CLI, antarmuka web) dari image dasar resmi yang dikunci digest, pengguna biasa uid 10001, data di volume `/data`. `compose.yaml` menjalankan node + explorer; explorer hanya terbuka di `127.0.0.1:28802` komputer host. `compose.publik.yaml` membuka explorer dan seed P2P WebSocket regtest ke publik lewat Cloudflare Tunnel, dengan miner berkala satu blok setiap 120 detik (Bagian 7) |
| ⏳ Ditunda | Paket testnet (sesudah genesis testnet, TSK-F06), mainnet, seed testnet, image untuk arsitektur lain (arm64), build yang bisa diulang bit demi bit (TSK-F10) |
| ➡️ Langkah berikutnya | Paket testnet: node `--testnet`, explorer `--testnet`, tunnel ke port 18802. Batas laju per pengunjung di belakang tunnel selesai (TSK-M43) |

Paket Docker hanya membungkus program yang sudah ada. Tidak ada aturan chain, kunci, atau data baru di dalamnya.

---

## 1. Image dasar (DEC-10 bagian 9)

| Tahap | Image | Digest indeks |
|---|---|---|
| Build Rust | `rust:1.99.0-slim-trixie` | `sha256:2752b332db73fdbb7dc576f06c82ed1f312005784ef913d7e04a28f5f55dc581` |
| Build web | `node:24.22.0-trixie-slim` | `sha256:a6155384446670e9719bdc4fd0ebe4d4b29fb32a8d6aa523da3547a6cfbcf22c` |
| Runtime | `debian:trixie-slim` (Debian 13) | `sha256:f6b08b09a05725f9431502d3b5714f5c4576e82c1a84fa1debe0c24a7e0c3ec4` |

- Digest dibaca dari registry Docker Hub dengan `docker buildx imagetools inspect` (Q93, 3 November 2026).
- Rust 1.99.0 sama dengan `rust-toolchain.toml`; Node.js 24.22.0 sama dengan lingkungan WSL pencipta.
- Mengganti image dasar = mengganti digest di `Dockerfile` dan di tabel ini, lewat skrip gerbang.
- Tahap build Rust memasang `cmake`, `make`, dan `g++` dari repositori Debian untuk kode C++ RandomX (DEC-01, DEC-10 bagian 10).
  Versi paket Debian belum dikunci; dikunci bersama compiler C++ di TSK-F10.

## 2. Isi image

| Bagian | Isi |
|---|---|
| Program | `/usr/local/bin/zevrigo-node`, `zevrigo-explorer`, `zevrigo-miner`, `zevrigo-cli` (`cargo build --release --locked`) |
| Antarmuka web | `/usr/share/zevrigo/web` (hasil `npm ci --ignore-scripts` dan `next build`, static export) |
| Pengguna | `zevrigo` (uid dan gid 10001), bukan root |
| Data | Volume `/data`: chain `regtest/chain.redb`, indeks `regtest/explorer.sqlite`, wallet `regtest/wallet.json` |
| Perintah bawaan | `zevrigo-node --regtest --datadir /data` |

Konteks build hanya `Cargo.toml`, `Cargo.lock`, `crates/`, `apps/` (tanpa `node_modules`, `.next`, `out`), `tests/`, dan
`vendor/` (salinan RandomX dengan salt ZVRG, DEC-01) (`.dockerignore`). `rust-toolchain.toml` dan `.cargo/` tidak disalin.

## 3. Compose (`compose.yaml`)

| Layanan | Isi |
|---|---|
| `node` | Node regtest; satu-satunya layanan dengan port di host: `127.0.0.1:28802` (milik explorer) |
| `explorer` | `zevrigo-explorer --http-bind 0.0.0.0 --web-dir /usr/share/zevrigo/web`, berbagi namespace jaringan dengan `node` |
| `miner` | Profil `miner`, payload dari variabel `ZVRG_PAYOUT`, berbagi namespace jaringan dengan `node` |

- Karena berbagi namespace, explorer, miner, dan CLI menjangkau RPC node di `127.0.0.1:28801` di dalam container. RPC
  dan P2P node tidak pernah diterbitkan ke host.
- Explorer di dalam container mendengar di `0.0.0.0:28802` (opsi `--http-bind`, Spesifikasi Explorer v0.1 Bagian 6);
  Docker hanya meneruskannya ke `127.0.0.1:28802` komputer host. Membuka explorer ke internet tetap wajib lewat reverse
  proxy HTTPS (belum dikerjakan). Explorer memakai batas laju bawaan (`--api-rate 10`, TSK-M40); karena semua koneksi
  dari host masuk lewat penerus port Docker, batas itu berlaku bersama untuk semua tab browser di komputer host.
- Data tetap di volume `zevrigo_zvrg-data` sesudah `docker compose down`; `docker compose down -v` menghapusnya.

## 4. Perintah

```bash
docker compose build                                          # atau: make docker-build
docker compose up -d                                          # node + explorer, buka http://127.0.0.1:28802/
docker compose exec node zevrigo-cli --datadir /data wallet-create
docker compose exec node zevrigo-cli --datadir /data address
ZVRG_PAYOUT=<payload hex 64> docker compose --profile miner up -d miner
docker compose logs -f explorer
docker compose down
```

## 5. Gerbang (Q94)

Image dibangun dari repo; setiap program menjawab `--version`; pengguna image uid 10001; antarmuka web ada; compose
menjalankan node + explorer; explorer di host menyajikan halaman dengan kebijakan keamanan konten, `/api/v1/address/…`
dan `/api/v1/allocation` `404`; miner di dalam container menambang 3 blok dan explorer mengindeksnya; RPC node tidak
bisa dijangkau dari host; `docker compose down -v` membersihkan container dan volume uji.

## 6. Keputusan

| ID | Keputusan | Status |
|---|---|---|
| DEC-10 bagian 9 | Image dasar Docker Bagian 1, dikunci digest, hanya untuk paket Docker | 🟢 3 November 2026 (konsekuensi TSK-M33, pilihan pencipta) |
| DEC-10 bagian 9 (tambahan) | Image resmi `cloudflare/cloudflared` 2026.10.7, digest `sha256:4955524aeaa379e154023d5a027c250f7b35041a2e7e950cf13f144bc9b9c5ba` (Q120), hanya di `compose.publik.yaml` | 🟢 11 November 2026 (pilihan pencipta: explorer publik sementara lewat Cloudflare) |

## 7. Explorer dan seed P2P publik lewat Cloudflare Tunnel (`compose.publik.yaml`, TSK-M44, TSK-M45)

Sementara (sebelum VPS), explorer dan seed P2P dibuka ke internet dari komputer pencipta lewat Cloudflare Tunnel dengan
domain `ratuaurion.store`. `cloudflared` hanya membuat koneksi keluar ke Cloudflare; tidak ada port router yang dibuka, dan
alamat IP komputer pencipta tidak terlihat oleh pengunjung. HTTPS dan perlindungan DDoS di sisi Cloudflare.

| Hal | Isi |
|---|---|
| Pemakaian | Dipakai bersama `compose.yaml` (`-f compose.yaml -f compose.publik.yaml`), project `zevrigo-publik` (volume sendiri `zevrigo-publik_zvrg-data`) |
| `node` | Port host dicabut (`ports: !reset []`); RPC dan P2P TCP tetap hanya di dalam namespace. `--p2p-ws-port 28803` (TSK-M45): P2P WebSocket `/ws` di `127.0.0.1:28803` dalam namespace, hanya dicapai lewat `cloudflared` |
| `explorer` | `--web-dir /usr/share/zevrigo/web --trust-proxy`, mendengar di `127.0.0.1:28802` (bawaan) di dalam namespace node; tidak bisa dijangkau dari host maupun container lain |
| `cloudflared` | Image resmi dikunci digest (Bagian 6), `tunnel --no-autoupdate run`, berbagi namespace jaringan dengan `node`, sehingga sampai ke explorer dan WebSocket P2P lewat loopback |
| `miner-berkala` | TSK-M46, TSK-M47: image `zevrigo:regtest`, `/bin/sh -c`: tunggu 30 detik, lalu ulangi `zevrigo-miner --blocks 1` dan `sleep $ZVRG_JEDA` (bawaan 120 detik = waktu blok target). Berbagi namespace jaringan dengan `node-penambang` (RPC lewat `127.0.0.1:28801` node itu), volume `zvrg-penambang` dipasang baca saja. Hadiah ke payload hex 64 di `/data/payout-penambang` (ditulis operator dari wallet penambang); selama file itu belum ada atau tidak sah, ke kunci NUMS `50929b74…3ac0` yang tidak bisa dibelanjakan. Batas 256 MiB, 0,5 CPU, `init: true`, `restart: unless-stopped` |
| `node-penambang`, `node-treasury`, `node-faucet`, `node-pengamat` | TSK-M47 (DEC-34): node operator jaringan demo, container dan volume sendiri (`zvrg-penambang`, `zvrg-treasury`, `zvrg-faucet`, `zvrg-pengamat`), tanpa port host, `--connect` ke seed publik (alamat Bagian 7 di bawah, bisa diganti `ZVRG_SEED`), lewat internet dan Cloudflare seperti peserta, memakai 4 dari 32 slot masuk seed. Batas 256 MiB dan 0,5 CPU per node (Q125: node seed sekitar 10 MiB, node baru sekitar 2 MiB dan sinkron dalam 8 detik pada tinggi 121; volume data seed 4,4 MB). Wallet operator dibuat pencipta sendiri dengan CLI di container (`wallet-create`, password diketik di terminal), file di `/data/regtest/wallet.json` volume itu |
| Token | `${HOME}/.zevrigo-tunnel.env` di luar repo, izin `600`, satu baris `TUNNEL_TOKEN=` diikuti token dari dasbor Cloudflare Zero Trust (Networks, Tunnels, Install and run, pilihan Docker). Token tidak pernah ditulis ke repo, log skrip, atau chat |
| Public hostname | Diatur di dasbor Cloudflare (tab Routes, Published application), keduanya tipe **HTTP** (bukan HTTPS: TLS diputus Cloudflare): `zevrigo.ratuaurion.store` → `localhost:28802` (explorer) dan `zevrigo-seed.ratuaurion.store` → `localhost:28803` (seed P2P WebSocket). Satu tingkat, tercakup sertifikat `*.ratuaurion.store`; subdomain lain di domain ini milik proyek lain. Testnet nanti `localhost:18802` dan `localhost:18803` |
| Bergabung ke seed | `zevrigo-node --regtest --datadir <folder> --connect /dns4/zevrigo-seed.ratuaurion.store/tcp/443/tls/sni/zevrigo-seed.ratuaurion.store/ws/p2p/<PeerId seed>`. PeerId seed tercetak di log node (`p2p-ws: …`) dan tetap selama volume data sama. Keepalive `status` 60 detik menjaga WebSocket tetap hidup (Cloudflare menutup yang diam sekitar 100 detik) |

Keamanan `--trust-proxy` (Spesifikasi Explorer v0.1 Bagian 6): yang tersambung ke explorer lewat loopback hanya
`cloudflared` dan proses di namespace yang sama (node dan miner, yang tidak memakai HTTP explorer). Siapa pun yang memegang
token bisa menjalankan tunnel ini; jika token bocor, token diganti di dasbor Cloudflare.

Seed regtest (pilihan pencipta, DEC-33) hanya bukti P2P: dengan PoW dev siapa pun yang tersambung bisa menambang instan.
Semua peserta masuk lewat `cloudflared`, jadi tampak dari 127.0.0.1: batas 32 koneksi masuk dibagi bersama dan blokir
hanya per `PeerId`.

```bash
docker compose -f compose.yaml -f compose.publik.yaml build node
docker compose -f compose.yaml -f compose.publik.yaml up -d
docker compose -f compose.yaml -f compose.publik.yaml logs -f cloudflared
docker compose -f compose.yaml -f compose.publik.yaml logs -f miner-berkala
docker compose -f compose.yaml -f compose.publik.yaml logs --no-log-prefix node | grep '^p2p-ws:'
docker compose -f compose.yaml -f compose.publik.yaml logs --no-log-prefix node-penambang | grep '^p2p:'   # PeerId node operator
ZVRG_JEDA=60 docker compose -f compose.yaml -f compose.publik.yaml up -d --no-deps miner-berkala   # ganti jeda
docker compose -f compose.yaml -f compose.publik.yaml stop miner-berkala                           # hentikan miner saja
docker compose -f compose.yaml -f compose.publik.yaml down
```

Gerbang (Q121): token ada dengan izin `600` (isinya tidak dibaca ke log), konfigurasi compose sah tanpa port host, image
dibangun ulang dengan `--trust-proxy`, `cloudflared` mencatat koneksi tunnel, miner di dalam namespace menambang 5 blok,
lalu dari luar `https://zevrigo.ratuaurion.store/api/v1/status` menjawab `network` regtest dengan tinggi indeks paling
sedikit 5 dan halaman `/` menjawab `200`.

Gerbang seed (Q122): node mencetak `p2p-ws:`; node di container lain (jaringan Docker biasa, lewat internet) tersambung
ke `zevrigo-seed.ratuaurion.store` dengan alamat di atas, mengejar tinggi seed, menerima blok baru lewat siaran, dan masih
tersambung sesudah 150 detik; explorer publik tetap menjawab.

Gerbang miner berkala (Q123): konfigurasi compose sah dengan `miner-berkala` di namespace node tanpa port host; layanan
dinyalakan tanpa menyalakan ulang node, explorer, atau cloudflared; dalam 300 detik tinggi seed naik paling sedikit 2 blok
dan log mencatat `blok diterima`; explorer publik `https://zevrigo.ratuaurion.store/api/v1/status` mengikuti tinggi baru;
kebijakan restart `unless-stopped`.

Gerbang lima node (Q126): konfigurasi compose sah; 4 node operator tanpa port host dengan batas 256 MiB dan 0,5 CPU dan
`--connect` ke seed publik; `miner-berkala` di namespace `node-penambang` dengan volume baca saja; node seed, explorer, dan
cloudflared tidak dinyalakan ulang; dalam 180 detik setiap node operator mencatat `rekan terhubung` dan mencapai tinggi seed,
dan seed mencatat paling sedikit 4 rekan masuk baru; dalam 330 detik tinggi seed naik paling sedikit 2 blok dari
`miner-berkala` yang baru (payout NUMS selama `/data/payout-penambang` belum ada) dan semua node operator mengikuti; explorer
publik mengikuti tinggi baru.
