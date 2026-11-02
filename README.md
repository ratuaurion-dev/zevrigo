# Homeostasis Coin (HSIS)

Homeostasis Coin (HSIS) adalah blockchain Layer 1 Proof of Work yang ditulis dari nol dalam Rust dengan fokus utama pada kebenaran konsensus, determinisme penuh, dan ketiadaan hak istimewa. Lapisan dasarnya sengaja minimal: model UTXO, tanpa VM serbaguna, dan hanya mendukung tipe transaksi bawaan (transfer, multi-tanda tangan MuSig2, serta kunci waktu).

Dokumentasi arsitektur, peta modul, keputusan desain, dan glosarium tersedia di [docs/README.md](docs/README.md) (serta aturan pengembangan lengkap di [AGENTS.md](AGENTS.md)).

## Prasyarat

- Linux / WSL2 pada filesystem Linux asli (bukan `/mnt/c`)
- Rust `1.99.0` melalui `rustup` (otomatis dikunci lewat `rust-toolchain.toml` beserta `rustfmt` dan `clippy`)
- Perangkat sistem: `make`, `bash`, dan `git` (serta `g++`, `cmake`, `pkg-config` untuk tahapan native berikutnya bila diperlukan)

## Verifikasi

Jalankan pemeriksaan format, lint `clippy`, larangan `unsafe`/floating point, dan seluruh test dengan satu perintah:

```bash
make check-all
```

## Menjalankan node regtest

```bash
cargo run -p homeostasis-node -- --regtest --datadir ~/.homeostasis
```

Node membuat chain regtest di `~/.homeostasis/regtest/chain.redb`, lalu mencetak alamat JSON-RPC (bawaan
`http://127.0.0.1:28801`). Delapan metode RPC dijelaskan di
[docs/reference/spesifikasi-rpc-v0.1.md](docs/reference/spesifikasi-rpc-v0.1.md). Hentikan dengan Ctrl-C.

Node juga mendengar P2P di `127.0.0.1:28800` dan mencetak alamatnya sebagai `p2p: /ip4/127.0.0.1/tcp/28800/p2p/<PeerId>`.
Node kedua di folder lain bisa tersambung dan mengejar chain:

```bash
cargo run -p homeostasis-node -- --regtest --datadir ~/.homeostasis-2 --rpc-port 28811 --p2p-port 28810 \
  --connect /ip4/127.0.0.1/tcp/28800/p2p/<PeerId>
```

Blok dan transaksi baru disiarkan antar node. Aturan lengkap di [docs/reference/spesifikasi-p2p-v0.1.md](docs/reference/spesifikasi-p2p-v0.1.md).

## Menambang di regtest

Dengan node berjalan, di terminal lain:

```bash
cargo run -p homeostasis-miner -- --payout <kunci x-only hex 64> --blocks 10
```

Miner mengambil template dari node, mencari nonce dengan PoW dev regtest, dan mengirim blok. Hadiah coinbase bisa
dibelanjakan setelah 100 blok. `--blocks 0` (bawaan) menambang terus sampai Ctrl-C.

## Wallet CLI regtest

```bash
cargo run -p homeostasis-cli -- --datadir ~/.homeostasis wallet-create
cargo run -p homeostasis-cli -- --datadir ~/.homeostasis address
cargo run -p homeostasis-cli -- --datadir ~/.homeostasis balance
cargo run -p homeostasis-cli -- --datadir ~/.homeostasis send --to <alamat rhsis1...> --amount 1.5
```

Wallet disimpan di `~/.homeostasis/regtest/wallet.json` (izin `0600`, tanpa enkripsi; hanya untuk regtest). Simpan
frasa 24 kata yang dicetak `wallet-create`. `address` juga mencetak payload hex yang bisa dipakai untuk `--payout`
miner, sehingga hadiah tambang masuk ke wallet. `balance` juga menampilkan transaksi yang masih menunggu di mempool
(`keluar tertunda`, `masuk tertunda`, dan saldo setelah semuanya masuk blok), dan `send` tidak memilih koin yang sedang
dibelanjakan. Aturan lengkap di [docs/reference/spesifikasi-wallet-v0.1.md](docs/reference/spesifikasi-wallet-v0.1.md).

## Explorer regtest

Dengan node berjalan, di terminal lain:

```bash
cargo run -p homeostasis-explorer -- --regtest --datadir ~/.homeostasis
```

Explorer membaca chain dari node lewat RPC, menyimpan indeks di `~/.homeostasis/regtest/explorer.sqlite`, memeriksa
sendiri jumlah koin setiap blok, lalu melayani API JSON baca saja di `http://127.0.0.1:28802/api/v1/` (misalnya
`/status`, `/blocks`, `/block/<tinggi>`, `/tx/<txid>`, `/address/<alamat>`, `/supply`, `/allocation`). Jumlah koin
selalu string desimal quanta. Aturan lengkap di [docs/reference/spesifikasi-explorer-v0.1.md](docs/reference/spesifikasi-explorer-v0.1.md).

### Antarmuka web explorer

Butuh Node.js 22.12 atau lebih baru. Build sekali, lalu jalankan explorer dengan `--web-dir`:

```bash
make web-check
cargo run -p homeostasis-explorer -- --regtest --datadir ~/.homeostasis --web-dir apps/explorer/web/dist
```

Buka `http://127.0.0.1:28802/` di browser. Tampilan tersedia dalam Bahasa Indonesia dan English (tombol di kepala
halaman), tanpa cookie, analitik, atau sumber dari luar. Untuk pengembangan, `npm run dev` di `apps/explorer/web`
membuka server Vite di `http://127.0.0.1:5173/` yang meneruskan `/api` ke explorer. Aturan lengkap di
[docs/reference/spesifikasi-explorer-web-v0.1.md](docs/reference/spesifikasi-explorer-web-v0.1.md).
