# TSK-M62 · Alat upacara genesis regtest (`zevrigo-demo upacara`) · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-18 (`apps/demo`) · **Untuk:** FTR-11 tahap 1 (DEC-34)
- **Bergantung pada:** TSK-M47, TSK-M53 · **Memblokir:** TSK-F11 bagian 1, TSK-M54
- **Estimasi:** S · **Ditulis:** Quick, 13 November 2026

## Tujuan
Pembagian alokasi genesis `G` ke 8 kantong (Rancangan Ekosistem Demo v0.1 Bagian 4.2) butuh satu transaksi yang
membelanjakan output `G` regtest. Kunci `G` regtest adalah kunci rahasia `1` (bukan wallet 24 kata), jadi CLI tidak bisa
membelanjakannya; sebelumnya hanya kode uji MOD-17 yang bisa.

## Isi
| Bagian | Isi |
|---|---|
| `upacara` (baru) | `GENESIS_PAYLOAD`, `REGTEST_NETWORK_ID` (Spesifikasi Validasi Bagian 10). `genesis_from_scan`: output kunci `1` wajib tinggi 0, indeks 0, coinbase, jenis 0, jumlah `G`. `build_ceremony` (fungsi murni): 1 sampai 16 kantong, alamat tidak ganda, jumlah tepat `G`, biaya = tarif × ukuran dengan jumlah penuh (mengurangi jumlah tidak pernah memperbesar encoding), diambil dari kantong terakhir; sighash v1 + BIP340 kunci `1`, `aux_rand` nol, diverifikasi ulang. `run`: jaringan node regtest, `get_block_hash(0)` = `REGTEST_NETWORK_ID`, `scan_utxos` kunci `1`, matang bila `tinggi + 1 ≥ spendable_at`, kirim hanya dengan `--kirim` dan txid node wajib sama. `format_report` |
| `cli` | Mode `upacara`: `--node-url`, `--kantong <alamat>=<ZVRG>` (berulang, alamat regtest jenis 0, jumlah > 0), `--fee-rate`, `--kirim`. Galat `InvalidPocket`, `MissingPockets`, `InvalidFeeRate` |
| Dependensi | MOD-01, MOD-02, MOD-09 menjadi dependensi MOD-18 (sudah diizinkan DEC-36). Tanpa paket baru dari registry; Cargo.lock hanya rujukan `zevrigo-demo` |
| Test | `upacara`: 6 (konstanta = spesifikasi, transaksi + tanda tangan + biaya, kantong tidak sah, pilihan output genesis, node tiruan MOD-08: rencana vs kirim, node salah/belum matang/sudah dibelanjakan). `cli`: 2. Workspace 452 → 460 |

## Gerbang (Q130)
`make check-all` paling sedikit 460 test; latihan di chain sementara di WSL (node `--rpc-port 38801 --p2p-port 38800` tanpa
`--connect`, folder sementara, tidak menyentuh Docker atau chain publik): wallet latihan membuat 8 alamat; di tinggi 98 upacara
ditolak "belum matang"; di tinggi 99 rencana tanpa `--kirim` tidak mengubah mempool; dengan `--kirim` txid sama dengan
rencana; sesudah 1 blok, saldo wallet latihan = 10.000.000 ZVRG − biaya dengan 8 UTXO dan setiap kantong menerima jumlahnya;
upacara kedua ditolak karena `G` sudah dibelanjakan.

## Batas
- Hanya regtest. Siapa pun yang tersambung saat `G` matang bisa membelanjakan `G` lebih dulu (kunci `1` publik): upacara
  sungguhan wajib dalam peluncuran tertutup (Rancangan Bagian 4.3, TSK-F11 bagian 1).
- Image Docker `zevrigo:regtest` baru memuat alat ini setelah dibangun ulang (TSK-F11 bagian 1).
- Label kantong (Pre-Seed, Faucet, …) tidak ada di transaksi; dicatat di konfigurasi halaman treasury (TSK-M52).

## Definition of done
- [x] Gerbang Q130 lolos
- [x] MOD-18, DEC-36, AGENTS.md, peta sistem, Rancangan Ekosistem Demo v0.1 Bagian 4.3 dan 6, docs/tasks
