# Spesifikasi Paket Docker Zevrigo Coin (ZVRG) v0.1: node, explorer, dan antarmuka web untuk regtest

> Diperbarui: 3 November 2026 · Pemilik: pencipta · Ditulis: Quick · Task: TSK-M33 · Status: 🟢 v0.1
> Lokasi di repo: `docs/reference/spesifikasi-docker-v0.1.md`

| Status | Ringkasan |
|---|---|
| 🟢 Dikunci | Satu image `zevrigo:regtest` (node, explorer, miner, CLI, antarmuka web) dari image dasar resmi yang dikunci digest, pengguna biasa uid 10001, data di volume `/data`. `compose.yaml` menjalankan node + explorer; explorer hanya terbuka di `127.0.0.1:28802` komputer host |
| ⏳ Ditunda | Testnet dan mainnet (DEC-01), reverse proxy HTTPS untuk explorer publik, image untuk arsitektur lain (arm64), build yang bisa diulang bit demi bit (TSK-F10) |
| ➡️ Langkah berikutnya | Paket testnet sesudah DEC-01. Batas laju API explorer selesai (TSK-M40, bawaan `--api-rate 10`) |

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
| Pengguna | `hsis` (uid dan gid 10001), bukan root |
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
