# Zevrigo Coin (ZVRG)

Zevrigo Coin (ZVRG) adalah blockchain Layer 1 Proof of Work yang ditulis dari nol dalam Rust dengan fokus utama pada kebenaran konsensus, determinisme penuh, dan ketiadaan hak istimewa. Lapisan dasarnya sengaja minimal: model UTXO, tanpa VM serbaguna, dan hanya mendukung tipe transaksi bawaan (transfer, multi-tanda tangan MuSig2, serta kunci waktu).

Dokumentasi arsitektur, peta modul, keputusan desain, dan glosarium tersedia di [docs/README.md](docs/README.md) (serta aturan pengembangan lengkap di [AGENTS.md](AGENTS.md)).

## Prasyarat

- Linux / WSL2 pada filesystem Linux asli (bukan `/mnt/c`)
- Rust `1.99.0` melalui `rustup` (otomatis dikunci lewat `rust-toolchain.toml` beserta `rustfmt` dan `clippy`)
- Perangkat sistem: `make`, `bash`, `git`, `cmake`, dan compiler C++ `g++` (wajib sejak PoW RandomX, DEC-01; di Ubuntu: `sudo apt-get install -y cmake g++`)

## Verifikasi

Jalankan pemeriksaan format, lint `clippy`, larangan `unsafe`/floating point, dan seluruh test dengan satu perintah:

```bash
make check-all
```

## Menjalankan node regtest

```bash
cargo run -p zevrigo-node -- --regtest --datadir ~/.zevrigo
```

Node membuat chain regtest di `~/.zevrigo/regtest/chain.redb`, lalu mencetak alamat JSON-RPC (bawaan
`http://127.0.0.1:28801`). Delapan metode RPC dijelaskan di
[docs/reference/spesifikasi-rpc-v0.1.md](docs/reference/spesifikasi-rpc-v0.1.md). Hentikan dengan Ctrl-C.

Node juga mendengar P2P di `127.0.0.1:28800` dan mencetak alamatnya sebagai `p2p: /ip4/127.0.0.1/tcp/28800/p2p/<PeerId>`.
Node kedua di folder lain bisa tersambung dan mengejar chain:

```bash
cargo run -p zevrigo-node -- --regtest --datadir ~/.zevrigo-2 --rpc-port 28811 --p2p-port 28810 \
  --connect /ip4/127.0.0.1/tcp/28800/p2p/<PeerId>
```

Blok dan transaksi baru disiarkan antar node. Aturan lengkap di [docs/reference/spesifikasi-p2p-v0.1.md](docs/reference/spesifikasi-p2p-v0.1.md).

Opsi `--testnet` (PoW RandomX, port bawaan RPC 18801 dan P2P 18800) sudah ada, tetapi testnet publik belum diluncurkan:
cap waktu genesis testnet ditetapkan saat peluncuran (TSK-F06). Node, miner, CLI, dan wallet desktop sudah mendukung testnet.

## Menambang di regtest

Dengan node berjalan, di terminal lain:

```bash
cargo run -p zevrigo-miner -- --payout <kunci x-only hex 64> --blocks 10
```

Miner mengambil template dari node, mencari nonce dengan PoW dev regtest, dan mengirim blok. Hadiah coinbase bisa
dibelanjakan setelah 100 blok. `--blocks 0` (bawaan) menambang terus sampai Ctrl-C.

Untuk node `--testnet`, tambahkan `--testnet`: miner memakai RandomX mode cepat (butuh sekitar 2,5 GB RAM; dataset disusun
sekitar satu menit setiap kunci berganti). `--light` memakai mode ringan (256 MB, jauh lebih lambat).

## Wallet CLI regtest

```bash
cargo run -p zevrigo-cli -- --datadir ~/.zevrigo wallet-create
cargo run -p zevrigo-cli -- --datadir ~/.zevrigo address
cargo run -p zevrigo-cli -- --datadir ~/.zevrigo balance
cargo run -p zevrigo-cli -- --datadir ~/.zevrigo send --to <alamat rzvrg1...> --amount 1.5
```

Wallet disimpan di `~/.zevrigo/regtest/wallet.json` (izin `0600`), dikunci password: `wallet-create` meminta
password baru dua kali (minimal 8 karakter, tidak tampil di layar), dan `send` serta `wallet-show-phrase` memintanya
lagi. `address` dan `balance` tidak butuh password. Simpan frasa 24 kata yang dicetak `wallet-create`; password tidak
bisa memulihkan wallet tanpa frasa itu. Wallet lama tanpa password dikunci dengan `wallet-encrypt`. Untuk skrip regtest,
password boleh diberikan lewat variabel lingkungan `ZEVRIGO_WALLET_PASSWORD`. `address` juga mencetak payload hex yang bisa dipakai untuk `--payout`
miner, sehingga hadiah tambang masuk ke wallet. `balance` juga menampilkan transaksi yang masih menunggu di mempool
(`keluar tertunda`, `masuk tertunda`, dan saldo setelah semuanya masuk blok), dan `send` tidak memilih koin yang sedang
dibelanjakan. Aturan lengkap di [docs/reference/spesifikasi-wallet-v0.1.md](docs/reference/spesifikasi-wallet-v0.1.md).

Untuk node `--testnet`, tambahkan `--testnet` ke setiap perintah: wallet disimpan terpisah di
`~/.zevrigo/testnet/wallet.json`, alamatnya `tzvrg1...`, dan RPC bawaan `http://127.0.0.1:18801`.

## Explorer regtest

Dengan node berjalan, di terminal lain:

```bash
cargo run -p zevrigo-explorer -- --regtest --datadir ~/.zevrigo
```

Explorer membaca chain dari node lewat RPC, menyimpan indeks di `~/.zevrigo/regtest/explorer.sqlite`, memeriksa
sendiri jumlah koin setiap blok, lalu melayani API JSON baca saja di `http://127.0.0.1:28802/api/v1/` (misalnya
`/status`, `/blocks`, `/block/<tinggi>`, `/tx/<txid>`, `/supply`, `/mempool`). Jumlah koin selalu string desimal quanta.
Explorer hanya menyajikan data sistem (DEC-28): tanpa alamat, saldo, jumlah per penerima, jejak koin, dan alokasi genesis.
Saldo sendiri diperiksa lewat `balance` di CLI. API dibatasi `--api-rate` (bawaan 10 permintaan per detik per klien;
`0` = tanpa batas); di atas batas explorer menjawab `429`. Di belakang Cloudflare Tunnel, `--trust-proxy` membuat batas
berlaku per pengunjung (kepala `CF-Connecting-IP`, hanya dari loopback). Aturan lengkap di [docs/reference/spesifikasi-explorer-v0.1.md](docs/reference/spesifikasi-explorer-v0.1.md).

### Antarmuka web explorer

Butuh Node.js 22.12 atau lebih baru. Build sekali, lalu jalankan explorer dengan `--web-dir`:

```bash
make web-check
cargo run -p zevrigo-explorer -- --regtest --datadir ~/.zevrigo --web-dir apps/explorer/web/out
```

Buka `http://127.0.0.1:28802/` di browser. Tampilan (Next.js, Tailwind CSS, shadcn/ui) tersedia dalam Bahasa Indonesia
dan English, mode terang dan gelap, pencarian Ctrl+K, tanpa cookie, analitik, atau sumber dari luar. Untuk melihat
tampilan tanpa node, `npm run dev:mock` di `apps/explorer/web` membuka `http://127.0.0.1:3000/` dengan data tiruan.
Aturan lengkap di [docs/reference/spesifikasi-explorer-web-v0.2.md](docs/reference/spesifikasi-explorer-web-v0.2.md).

## Docker (regtest)

Tanpa memasang Rust atau Node.js: butuh Docker Engine dengan plugin compose. Dari akar repo:

```bash
docker compose build
docker compose up -d
```

Buka `http://127.0.0.1:28802/`. Node dan explorer berjalan di container dengan data di volume `zevrigo_zvrg-data`;
RPC node tidak terbuka ke host, dan explorer hanya bisa dibuka dari komputer ini. Wallet dan miner di dalam container:

```bash
docker compose exec node zevrigo-cli --datadir /data wallet-create
docker compose exec node zevrigo-cli --datadir /data address
ZVRG_PAYOUT=<payload dari perintah address> docker compose --profile miner up -d miner
docker compose down
```

Aturan lengkap di [docs/reference/spesifikasi-docker-v0.1.md](docs/reference/spesifikasi-docker-v0.1.md).

### Explorer publik lewat Cloudflare Tunnel

`compose.publik.yaml` membuka explorer ke internet lewat Cloudflare Tunnel tanpa membuka port router. Token tunnel
disimpan di luar repo, di `~/.zevrigo-tunnel.env` (izin `600`, satu baris `TUNNEL_TOKEN=` diikuti token dari dasbor
Cloudflare Zero Trust). Public hostname tunnel diarahkan ke `localhost:28802`.

```bash
docker compose -f compose.yaml -f compose.publik.yaml build node
docker compose -f compose.yaml -f compose.publik.yaml up -d node explorer cloudflared
docker compose -f compose.yaml -f compose.publik.yaml down
```

RPC dan P2P node tidak terbuka, explorer hanya bisa dicapai lewat tunnel, dan batas laju berlaku per pengunjung
(`--trust-proxy`). Saat ini regtest (uji); testnet menyusul. Rincian di Spesifikasi Docker v0.1 Bagian 7.

## Lisensi

Dilisensikan dengan salah satu dari berikut, sesuai pilihan Anda (DEC-02):

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

Kontribusi yang dikirim untuk dimasukkan ke proyek ini, kecuali dinyatakan lain, dilisensikan ganda seperti di atas
tanpa syarat tambahan.
