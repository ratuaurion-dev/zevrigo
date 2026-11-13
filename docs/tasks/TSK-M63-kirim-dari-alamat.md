# TSK-M63 · Kirim dari satu alamat (`zevrigo-cli send --dari`) · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-12 (`apps/cli`) · **Untuk:** FTR-11 tahap 1 (DEC-34)
- **Bergantung pada:** TSK-M54, TSK-M62 · **Memblokir:** TSK-F11 bagian 1 (isi ulang faucet), nanti TSK-M51 (distribusi)
- **Estimasi:** S · **Ditulis:** Quick, 13 November 2026

## Tujuan
Kedelapan kantong treasury demo ada di satu wallet (node treasury). `send` biasa memperlakukan semua UTXO wallet sebagai satu
saldo: koin terbesar dipilih dulu (kantong Pencipta) dan kembalian pindah ke alamat kembalian baru. Isi ulang faucet dari
kantong Faucet (keputusan pencipta 13 November 2026: upacara ke kantong Faucet di treasury, faucet diisi bertahap) akan
mencampur kantong. Dengan `--dari`, satu kantong bisa dibelanjakan tanpa menyentuh kantong lain.

## Isi
| Bagian | Isi |
|---|---|
| `args` | Opsi `--dari <alamat>`, hanya untuk `send`, sekali saja. `Action::Send { from: Option<String> }` |
| `commands` | `Cli::send_from(.., from: Option<&str>, ..)`; `Cli::send` = `send_from` tanpa sumber (MOD-15 dan test FTR-03 tidak berubah). Dengan sumber: alamat diurai untuk jaringan CLI (`CliError::SourceAddress`), wajib alamat jenis 0 wallet ini sampai batas pemindaian (`wallet_payloads`, gap limit 20; `CliError::SourceNotOwned`), hanya UTXO alamat itu yang menjadi koin (UTXO yang sedang dibelanjakan mempool tetap tidak dipilih), kembalian ke payload alamat itu, indeks kembalian file wallet tidak dinaikkan. `InsufficientFunds` melaporkan saldo alamat itu saja |
| Batasan | Seperti `send` biasa, output mempool tidak dipakai sebagai koin: kiriman kedua dari kantong yang sama menunggu kiriman pertama masuk blok |
| Test | `args`: 1 (`parses_send_from`). `commands`: 1 (`send_from_spends_only_that_address_and_returns_change_to_it`: satu input dari W6, kembalian ke W6, indeks tetap, saldo kurang, alamat kembalian belum matang, alamat luar, jaringan lain). Workspace 468 → 470 |

## Gerbang (Q132)
`make check-all` paling sedikit 470 test. Latihan di chain sementara di WSL (node `--rpc-port 38801 --p2p-port 38800` tanpa
`--connect`, folder sementara, tidak menyentuh Docker atau chain publik): upacara ke 8 alamat wallet latihan "treasury"; `send
--dari` kantong Faucet 10.000 ZVRG ke wallet lain; kiriman kedua sebelum blok ditolak saldo kurang; sesudah blok, kiriman
kedua 5.000 ZVRG; kantong Faucet tetap satu UTXO = 500.000 − biaya upacara − 15.000 − kedua biaya kirim; 7 kantong lain
utuh (satu UTXO, jumlah persis); `--dari` kantong Pre-Seed 2.000.000 ditolak (saldo 1.000.000); `--dari` alamat wallet lain
ditolak; indeks kembalian file wallet treasury tetap 0.
