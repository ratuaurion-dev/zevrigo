# TSK-M54 · Faucet demo (`zevrigo-demo faucet`) · 🟡 layanan, compose, dan halaman explorer · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-18 (`apps/demo`), nanti MOD-14 (halaman explorer) · **Untuk:** FTR-11 tahap 1 (DEC-34)
- **Bergantung pada:** TSK-M53, TSK-M62 · **Memblokir:** TSK-F11 bagian 1 (isi faucet sesudah upacara), TSK-M58
- **Estimasi:** M · **Ditulis:** Quick, 13 November 2026

## Tujuan
Pengunjung demo (orang awam) butuh ZVRG regtest untuk mencoba wallet tanpa menambang. Faucet mengirim 100 ZVRG per pengunjung
per hari dari kantong Faucet treasury (Rancangan Ekosistem Demo v0.1 Bagian 4.7). Pembagian dana tetap jelas sejak awal:
faucet hanya diisi dari kantong Faucet (500.000 ZVRG, contoh demo).

## Bagian A: mode `faucet` (Q131)
| Bagian | Isi |
|---|---|
| Wallet faucet | Frasa 24 kata di `<datadir>/faucet-frasa.txt`, dibuat sekali dari `/dev/urandom` dengan izin `0600`; frasa yang rusak membuat faucet tidak start (file tidak ditimpa). Kunci akun regtest `m/44'/1'/0'` tanpa passphrase, satu alamat `0/0` untuk menerima isi ulang dan kembalian. Wallet panas, hanya regtest |
| Kiriman | `scan_utxos` + `scan_mempool` alamat faucet: UTXO yang belum dibelanjakan mempool ditambah output mempool ke faucet yang belum dibelanjakan kiriman sendiri (dicatat di memori). Pemilihan koin, biaya, kembalian, dan tanda tangan dengan `build_transaction` MOD-09 (`aux_rand` dari `/dev/urandom`), lalu `send_raw_transaction`. Kiriman berikutnya langsung memakai kembalian yang belum masuk blok (berantai lewat mempool, MOD-07) |
| Batas | Urutan: alamat regtest sah dan bukan alamat faucet, batas harian (`--batas-harian`, bawaan 100), pengunjung (hash alamat IP dengan kunci acak per hari, hanya di memori), alamat (sekali per hari UTC), node (jaringan regtest dan genesis regtest), saldo. Batas hanya terpakai bila node menerima transaksi. Batas alamat dan harian disimpan di `<datadir>/faucet-batas.json` (`versi`, `hari`, `jumlah`, `alamat`), tetap berlaku setelah restart; file rusak membuat faucet tidak start |
| Sesudah restart | Kiriman lama yang masih di mempool tidak diketahui; faucet menjawab `busy` sampai mempool tidak memuat transaksi faucet (paling lama satu blok), agar tidak membelanjakan output yang sama dua kali. Selama itu output mempool ke faucet tidak dihitung (saldo `GET` hanya UTXO terkonfirmasi yang belum dibelanjakan mempool) |
| HTTP | `GET /api/v1/faucet`: `network`, `address`, `amount`, `balance` (quanta dan ZVRG), `daily_limit`, `remaining_today`, `synced`, `ready`. `POST` badan `{"address": ...}` paling besar 4 KiB: `txid`, `address`, `amount`, `fee`. Galat `{"error": kode, "message": teks}`: `400` `bad_address`, `own_address`, `bad_request`; `429` `visitor_limit`, `address_limit`, `daily_limit`, `rate_limited` (laju bawaan 2 per detik, `crate::limit`); `503` `empty`, `busy`, `node_unreachable`, `wrong_network`, `rejected` tanpa rincian node; `500` `internal`. `405`, `404`, `413` |
| CORS dan proxy | Kepala CORS hanya untuk origin di `--allow-origin` (bawaan `https://zevrigo.ratuaurion.store`), preflight `OPTIONS`. `CF-Connecting-IP` hanya dipercaya untuk koneksi dari alamat `--trust-proxy-host` (misalnya container `cloudflared`), di-resolve ulang tiap 60 detik |
| Privasi | Log: start, berhenti, txid kiriman, galat node. Tanpa alamat IP dan tanpa alamat penerima. File batas hanya berisi alamat penerima hari itu |
| Dependensi | Fitur `sync` tokio (paket yang sama, tanpa paket baru); Cargo.lock tidak berubah |
| Test | `faucet`: 6 (file frasa, kiriman berantai + batas + file batas, restart + `busy` + hari berikutnya, saldo habis + node salah/mati, HTTP + CORS + galat, laju + alamat pengunjung + kode galat) dengan node tiruan di belakang server MOD-08 yang memeriksa input, tanda tangan BIP340, dan jumlah. `cli`: 2. Workspace 460 → 468 |

## Gerbang (Q131)
`make check-all` paling sedikit 468 test. Latihan di chain sementara di WSL (node `--rpc-port 38801 --p2p-port 38800` tanpa
`--connect`, faucet `--http-port 38805 --batas-harian 3 --trust-proxy-host localhost`, folder sementara, tidak menyentuh
Docker atau chain publik): faucet kosong menjawab `empty`; upacara mengisi kantong Faucet 500.000 ZVRG ke alamat faucet;
3 kiriman berantai di mempool lalu masuk satu blok; `visitor_limit`, `address_limit`, `daily_limit`; restart dengan alamat
sama, `busy` sampai blok berikutnya; kiriman keempat; restart lagi: sisa hari ini 0; CORS hanya untuk explorer; setiap
penerima tepat 100 ZVRG, faucet satu UTXO = 500.000 ZVRG − biaya upacara − biaya 4 kiriman; log dan file batas tanpa alamat IP.

## Bagian B
- Selesai bersama peluncuran (Q133): layanan `faucet` di `compose.publik.yaml` di namespace `node-faucet` (`--datadir /data`,
  `--http-bind 0.0.0.0`, `--trust-proxy-host node`, volume `zvrg-faucet-dompet`). Route `zevrigo-faucet.ratuaurion.store` →
  `http://node-faucet:28805` dibuat pencipta di dasbor Cloudflare. Pengisian (keputusan pencipta 13 November 2026): upacara
  ke kantong Faucet di wallet treasury; faucet diisi ulang bertahap dengan `zevrigo-cli send --dari` (TSK-M63).
- Halaman faucet di explorer (Q134, keputusan pencipta 14 November 2026: tanpa anti-bot dulu, cukup batas harian):
  `/faucet/` di antarmuka web MOD-14, memanggil `https://zevrigo-faucet.ratuaurion.store/api/v1/faucet` (CORS sudah
  mengizinkan origin explorer); program explorer `--faucet-origin` menambahkan origin itu ke `connect-src` halaman HTML.
  Alamat diperiksa bech32m di peramban; galat faucet ditampilkan dengan teks Indonesia dan Inggris.
- Berikutnya: anti-bot (misalnya Cloudflare Turnstile, layanan pihak ketiga yang butuh persetujuan pencipta) bila jatah
  harian sering habis oleh bot. Tombol faucet di APK Wallet: TSK-M58.
