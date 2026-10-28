# Homeostasis Coin (HSIS)

Homeostasis Coin (HSIS) adalah blockchain Layer 1 Proof of Work yang ditulis dari nol dalam Rust. Lapisan dasarnya dirancang minimal dan deterministik: model UTXO, tanpa VM serbaguna, dan hanya mendukung tipe transaksi bawaan (transfer, multi-tanda tangan MuSig2, serta kunci waktu).

Lihat `AGENTS.md` untuk parameter protokol dan aturan pengembangan kode konsensus (termasuk larangan mutlak `unsafe` dan floating point).

## Prasyarat

- Linux / WSL2 (di filesystem Linux, bukan `/mnt/c`)
- Rust `1.99.0` melalui `rustup` (dikunci otomatis lewat `rust-toolchain.toml` beserta `rustfmt` dan `clippy`)
- Toolchain sistem: `make`, `git`, `g++` (`build-essential`), `cmake`, dan `pkg-config`

Pada Ubuntu/Debian, prasyarat sistem dapat dipasang dengan:

```bash
sudo apt update && sudo apt install -y build-essential cmake pkg-config git
```

## Perintah Pemeriksaan

Jalankan seluruh pemeriksaan format, lint (`clippy`), larangan `unsafe`/float, dan unit test dengan satu perintah:

```bash
make check-all
```

Target `Makefile` lainnya:

- `make fmt` — merapikan format seluruh kode (`cargo fmt --all`)
- `make fmt-check` — memeriksa format kode tanpa mengubah file
- `make lint` — menjalankan `clippy` pada seluruh workspace dan target (`-D warnings`)
- `make check-forbidden` — memindai file `*.rs` proyek dan gagal jika ada `f32`/`f64` atau kata kunci `unsafe` di luar komentar
- `make test` — menjalankan seluruh test standar
- `make test-slow` — menjalankan test lambat/simulasi (`--release -- --ignored`)
