# TSK-F03 · FTR-03 Kirim dan terima lewat CLI di regtest · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task fitur · **Fitur:** FTR-03 · **Modul:** MOD-01, 02, 04, 07, 08, 09, 11, 12, 13 (diuji di MOD-17)
- **Bergantung pada:** TSK-F02, TSK-M13, TSK-M16, TSK-M19, DEC-23 · **Memblokir:** TSK-F06, TSK-F07
- **Estimasi:** S · **Ditulis:** Quick (uji), 1 November 2026

## Kriteria selesai (docs/features/README.md)
Wallet CLI membuat alamat, menerima coinbase yang sudah matang (100 blok), mengirim ke wallet lain, dan transaksi
terkonfirmasi.

## Bukti
`tests/integration/tests/ftr03_send_receive.rs` · `two_cli_wallets_send_and_receive_over_rpc`, di `make check-all`:
- `Node<RedbStore>` (di memori) + `RpcServer` + `Miner` + pustaka CLI (`Cli`, `wallet_create`, `wallet_restore`,
  `next_address`) di satu proses, lewat RPC HTTP sungguhan. Jam node maju 120 detik per blok (target tetap `pow_limit`).
- Wallet A dan B dibuat dengan `wallet_create`. 99 blok ke alamat A: saldo 99 coinbase, semuanya belum matang, dan
  `send` ditolak `InsufficientFunds`.
- Dua blok lagi (tinggi 101): coinbase tinggi 1 dan 2 matang. A mengirim 0,01 ZVRG ke alamat B: dua input,
  kembalian ke cabang 1, `Σ input = jumlah + biaya + kembalian`, biaya ≥ 1 quanta × ukuran; transaksi masuk mempool.
- Blok 102 (ke kunci lain) memuat transaksi itu; coinbase = subsidy + biaya yang dihitung CLI (DEC-17). Saldo B
  0,01 ZVRG, bisa dipakai.
- B mengirim 0,001 ZVRG ke alamat A yang baru (satu input bukan coinbase, kembalian), ditambang di blok 103.
- Saldo akhir A dan B tepat (total, bisa dipakai, belum matang, jumlah UTXO). Wallet yang dipulihkan dari frasa A
  dengan `wallet_restore` melihat saldo yang sama persis, termasuk kembalian di cabang 1.
- `get_chain_info`: tinggi 103, `mined_total` sama dengan jadwal pasokan, mempool kosong.

## Keputusan
- **DEC-23 (disetujui pencipta, 1 November 2026):** MOD-17 memakai pustaka MOD-12 (CLI) hanya sebagai dev-dependency,
  sama seperti DEC-20 untuk node dan miner. Dirujuk lewat `path`, bukan `[workspace.dependencies]`. Crate inti tetap
  tidak boleh bergantung pada aplikasi. Alternatif yang ditolak: uji tiga biner lewat skrip gerbang (tidak berulang
  di check-all).
- Biner CLI sendiri hanya membungkus pustaka yang diuji di sini; argumennya diuji di `apps/cli`.

## Test (MOD-17 2 → 3, workspace 305 → 306)
`tests/ftr03_send_receive.rs`: `two_cli_wallets_send_and_receive_over_rpc`. Cargo.lock +1 rujukan
(`zevrigo-integration-tests → zevrigo-cli`), tetap 203 paket.
