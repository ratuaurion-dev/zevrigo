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

## Menambang di regtest

Dengan node berjalan, di terminal lain:

```bash
cargo run -p homeostasis-miner -- --payout <kunci x-only hex 64> --blocks 10
```

Miner mengambil template dari node, mencari nonce dengan PoW dev regtest, dan mengirim blok. Hadiah coinbase bisa
dibelanjakan setelah 100 blok. `--blocks 0` (bawaan) menambang terus sampai Ctrl-C.
