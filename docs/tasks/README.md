# Task kerja: FTR-01 sampai FTR-03 selesai, cabang chain dan reorg (TSK-M27) menuju FTR-04

> Diperbarui: 13 November 2026 · Status keseluruhan: 🟡

| Status | Ringkasan |
|---|---|
| 🟢 Selesai | TSK-S01, TSK-S02 (aturan validasi), TSK-S03 (wallet v0.1), TSK-S04 (RPC v0.1), TSK-S05 (P2P v0.1), TSK-S06 (explorer v0.1), TSK-S07 (antarmuka web explorer v0.2), TSK-M01, M02, M03, M04, M05, M06, M07, M08, M09, M10, M11, M12, M13, M14, M15, M16, M18, M19, M27, M28, M29, M30, M31, M32, M33, M34, M35, M36, M37, M38, M39, M40, M41, M42, M43, M44, M20, M21, M22, M24, M25, M26, TSK-F01 (node regtest tunggal), TSK-F02 (menambang di regtest), TSK-F03 (kirim dan terima lewat CLI), TSK-F04 (jaringan multi-node dan reorg dalam), TSK-F05 (explorer minimal), TSK-F07 (wallet desktop v0.1: aplikasi dan installer Windows) |
| 🟡 Siap dikerjakan | TSK-M23 (pruning) |
| 🔴 Terblokir | TSK-F06 oleh keputusan pencipta: waktu peluncuran (cap waktu genesis testnet) dan lokasi seed node |
| ❓ Keputusan dibutuhkan | Waktu peluncuran testnet publik dan lokasi seed node (TSK-F06); DEC-07 penerbit rilis (tanda tangan installer) |
| ➡️ Langkah berikutnya | TSK-F06 testnet publik: seed node dan penemuan rekan, perlindungan eclipse minimal (Spesifikasi P2P Bagian 9), node bisa listen di alamat publik, image Docker testnet, HTTPS untuk explorer, panduan menambang testnet. TSK-M06 (PoW RandomX) dan TSK-F07 (wallet desktop v0.1) selesai |

## Tiga jenis task

| Jenis | Prefix | Siapa | Aturan |
|---|---|---|---|
| **Modul** | TSK-M | Agent | Hanya menyentuh satu folder modul (satu crate atau satu aplikasi). Tidak mengubah modul lain |
| **Fitur** | TSK-F | Agent | Menyatukan beberapa modul menjadi kemampuan bagi pengguna. Baru boleh dimulai setelah semua task modul yang dibutuhkan 🟢 |
| **Spesifikasi** | TSK-S | Pencipta + Quick | Menetapkan aturan yang belum tertulis. Hasilnya dokumen di docs/reference/, bukan kode |

Kartu task yang siap dikerjakan disimpan sebagai file `TSK-xxx-*.md` di folder ini. Cara memberi task:
`Kerjakan docs/tasks/<file>. Gunakan mode Planning.`

## Backlog: task spesifikasi (TSK-S)

| ID | Judul | Memblokir | Status |
|---|---|---|---|
| TSK-S01 | Format transaksi UTXO dan blok, encoding kanonis, format alamat, sighash | TSK-M03 | 🟢 v0.1 lengkap di [reference/spesifikasi-format-v0.1.md](../reference/spesifikasi-format-v0.1.md), termasuk sighash (DEC-13) |
| TSK-S02 | Aturan validasi: coinbase, kematangan, biaya, ukuran, chainwork, rumus ASERT bilangan bulat, cap waktu | TSK-M08, TSK-M09 | 🟢 v0.1 di [reference/spesifikasi-validasi-v0.1.md](../reference/spesifikasi-validasi-v0.1.md), termasuk DEC-17 |
| TSK-S03 | Wallet: BIP39, jalur turunan kunci, alamat watch-only, filter blok | TSK-M15 | 🟢 v0.1 di [reference/spesifikasi-wallet-v0.1.md](../reference/spesifikasi-wallet-v0.1.md) (DEC-21, DEC-22, DEC-10 bagian 4); filter blok ditunda |
| TSK-S04 | Daftar metode RPC dan aturan encoding JSON (regtest, FTR-01 sampai FTR-03) | TSK-M12 | 🟢 v0.1 di [reference/spesifikasi-rpc-v0.1.md](../reference/spesifikasi-rpc-v0.1.md) |
| TSK-S05 | Pesan P2P (sinkronisasi header dan blok, gossip transaksi) | TSK-M14 | 🟢 v0.1 di [reference/spesifikasi-p2p-v0.1.md](../reference/spesifikasi-p2p-v0.1.md) (DEC-24, DEC-10 bagian 5) |
| TSK-S06 | Explorer: indeks SQLite tahan reorg, pemeriksaan jumlah koin, API HTTP baca saja, halaman alokasi genesis | TSK-M29, TSK-M30 | 🟢 v0.1 di [reference/spesifikasi-explorer-v0.1.md](../reference/spesifikasi-explorer-v0.1.md) (DEC-25, DEC-10 bagian 6) |
| TSK-S07 | Antarmuka web explorer: halaman, dua bahasa, format jumlah `bigint`, keamanan, penyajian `--web-dir`, dependensi npm | TSK-M31, TSK-M32, TSK-M38 | 🟢 v0.2 di [reference/spesifikasi-explorer-web-v0.2.md](../reference/spesifikasi-explorer-web-v0.2.md) (DEC-26, DEC-10 bagian 7; v0.1 diganti) |
| TSK-S08 | Rancangan ekosistem demo regtest untuk pemula: 5 node, statistik jaringan, upacara genesis, treasury dan vesting dipercepat, faucet, pendaftaran, gerbang RPC, dua APK (DEC-34, DEC-35, DEC-36) | TSK-M47 sampai TSK-M59, TSK-F11 | 🟢 v0.1 di [reference/rancangan-ekosistem-demo-v0.1.md](../reference/rancangan-ekosistem-demo-v0.1.md) |

## Backlog: task modul (TSK-M)

| ID | Modul | Judul | Bergantung pada | Status |
|---|---|---|---|---|
| TSK-M01 | MOD-01 | Tipe dasar: Amount, BlockHeight, Hash32, parameter chain | — | 🟢 selesai ([kartu](TSK-M01-primitives-tipe-dasar.md)) |
| TSK-M02 | MOD-02 | Hash BLAKE3 dengan pemisah domain dan Merkle root (crate blake3) | TSK-M01 | 🟢 selesai ([kartu](TSK-M02-crypto-blake3.md)) |
| TSK-M03 | MOD-01 | Struktur transaksi dan blok, encoding kanonis (tanpa hashing) | TSK-M01, TSK-S01 | 🟢 selesai ([kartu](TSK-M03-primitives-encoding.md)) |
| TSK-M04 | MOD-02 | Schnorr BIP340 (crate k256): kunci, tanda tangan, verifikasi, vektor uji resmi | TSK-M01 | 🟢 selesai ([kartu](TSK-M04-crypto-schnorr.md)) |
| TSK-M05 | MOD-03 | Antarmuka PoW dan PoW dev untuk regtest | TSK-M02, TSK-M03, DEC-14, DEC-15 | 🟢 selesai ([kartu](TSK-M05-pow-dev.md)) |
| TSK-M06 | MOD-03 (bagian B dan C juga MOD-04, 11, 13) | PoW RandomX testnet dan mainnet | DEC-01 | 🟢 selesai: A, B, C1, C2, C3a, C3b ([kartu](TSK-M06-pow-randomx.md)) |
| TSK-M07 | MOD-04 | Rumus pasokan dan vektor uji | TSK-M01 | 🟢 selesai ([kartu](TSK-M07-consensus-pasokan.md)) |
| TSK-M08 | MOD-04 | Target compact ke `Target` 256-bit (DEC-15), ASERT, median waktu, chainwork, parameter per jaringan, genesis regtest (Spesifikasi Validasi Bagian 2–6, 10) | TSK-M01, TSK-M05, TSK-M25, TSK-S02 | 🟢 selesai ([kartu](TSK-M08-consensus-target-asert.md)) |
| TSK-M09 | MOD-04 | Validasi transaksi dan blok (Spesifikasi Validasi Bagian 7–9), termasuk penolakan PoW dev di testnet dan mainnet beserta test-nya (DEC-09) | TSK-M03, M04, M05, M07, M08, M21, M24, TSK-S02 | 🟢 selesai ([kartu](TSK-M09-consensus-validasi.md)) |
| TSK-M10 | MOD-05 | Penyimpanan in-memory: trait ChainStore, MemoryStore, set UTXO, undo data | TSK-M03, TSK-M22 | 🟢 selesai ([kartu](TSK-M10-storage-memory.md)) |
| TSK-M11 | MOD-05 | Penyimpanan persisten RedbStore (redb 4.4.0, DEC-03), uji diferensial terhadap MemoryStore | TSK-M10 | 🟢 selesai ([kartu](TSK-M11-storage-redb.md)) |
| TSK-M12 | MOD-08 | Tipe dan server/klien JSON-RPC (Spesifikasi RPC v0.1) | TSK-M03, TSK-S04, DEC-10 | 🟢 selesai ([kartu](TSK-M12-rpc.md)) |
| TSK-M13 | MOD-07 | Mempool | TSK-M09, DEC-18 | 🟢 selesai ([kartu](TSK-M13-mempool.md)) |
| TSK-M14 | MOD-06 | P2P dasar: swarm libp2p 0.57, 5 protokol, codec, kunci rekan, sanksi (Spesifikasi P2P Bagian 1 sampai 7) | TSK-M03, TSK-S05, DEC-24 | 🟢 selesai ([kartu](TSK-M14-network-libp2p.md)) |
| TSK-M15 | MOD-09 (+ MOD-02) | BIP39, turunan BIP32, kunci akun publik, alamat bech32m | TSK-M04, TSK-S03, DEC-10 bagian 4 | 🟢 selesai ([kartu](TSK-M15-wallet-frasa-turunan-alamat.md)) |
| TSK-M16 | MOD-09 | Penyusunan dan penandatanganan transaksi, pemilihan koin | TSK-M03, TSK-M15, TSK-M24 | 🟢 selesai ([kartu](TSK-M16-wallet-kirim.md)) |
| TSK-M17 | MOD-10 | Tagihan watch-only dan status pembayaran | TSK-M16 | ⚪ |
| TSK-M18 | MOD-11 | Node: logika chain dan backend RPC (A), lalu konfigurasi, folder data, logging, shutdown (B) | TSK-M09, M11, M12, M13, M26, DEC-10, DEC-19 | 🟢 selesai ([kartu](TSK-M18-node.md)) |
| TSK-M19 | MOD-12 | CLI: perintah node dan wallet regtest | TSK-M12, TSK-M16, DEC-21 | 🟢 selesai ([kartu](TSK-M19-cli-wallet.md)) |
| TSK-M20 | MOD-13 | Miner: loop penambangan dengan PoW dev | TSK-M05, TSK-M12, TSK-M18 | 🟢 selesai ([kartu](TSK-M20-miner.md)) |
| TSK-M21 | MOD-02 | ID transaksi dan blok: txid, wtxid, block_id, hash kondisi, Merkle root blok | TSK-M02, TSK-M03 | 🟢 selesai ([kartu](TSK-M21-crypto-ids.md)) |
| TSK-M22 | MOD-01 | Tipe OutPoint dan UtxoEntry (dipakai bersama MOD-04, MOD-05, MOD-07, MOD-09) | TSK-M03 | 🟢 selesai ([kartu](TSK-M22-primitives-utxo.md)) |
| TSK-M23 | MOD-05 | Pruning badan blok lama (undo data tetap disimpan), termasuk perilaku undo_tip untuk blok yang badannya sudah dibuang | TSK-M11 | ⚪ |
| TSK-M24 | MOD-02 | Pesan tanda tangan sighash v1 (DEC-13) | TSK-M21 | 🟢 selesai ([kartu](TSK-M24-crypto-sighash.md)) |
| TSK-M26 | MOD-05 | Iterasi set UTXO berdasarkan payload untuk `scan_utxos` (Spesifikasi RPC 3.8), di MemoryStore dan RedbStore | TSK-M11, TSK-S04 | 🟢 selesai ([kartu](TSK-M26-storage-scan-utxo.md)) |
| TSK-M27 | MOD-11 | Blok cabang, pemilihan chain berdasarkan chainwork, reorg tanpa batas kedalaman (Spesifikasi P2P Bagian 5) | TSK-M11, TSK-M18, TSK-S05 | 🟢 selesai ([kartu](TSK-M27-cabang-reorg.md)) |
| TSK-M28 | MOD-11 | Indeks header, sinkronisasi header lalu blok, siaran blok dan transaksi, opsi `--p2p-port` dan `--connect` (Spesifikasi P2P Bagian 4 sampai 8) | TSK-M14, TSK-M27 | 🟢 selesai ([kartu](TSK-M28-p2p-daemon.md)) |
| TSK-M29 | MOD-14 | Indexer SQLite: skema, sinkronisasi lewat RPC, reorg tanpa batas, pemeriksaan I1–I4, query (Spesifikasi Explorer Bagian 2 sampai 4) | TSK-S06, DEC-25 | 🟢 selesai ([kartu](TSK-M29-explorer-indexer.md)) |
| TSK-M30 | MOD-14 | API HTTP axum `GET /api/v1/…`, program `zevrigo-explorer` (opsi, sinyal, stdout `http:`) (Spesifikasi Explorer Bagian 5 dan 6) | TSK-M29 | 🟢 selesai ([kartu](TSK-M30-explorer-api.md)) |
| TSK-M31 | MOD-14 | Opsi `--web-dir`, penyajian file web yang aman, kepala keamanan (Spesifikasi Explorer Web Bagian 7) | TSK-M30, TSK-S07 | 🟢 selesai ([kartu](TSK-M31-explorer-web-dir.md)) |
| TSK-M32 | MOD-14 (`web/`) | Aplikasi web React: halaman, dua bahasa, format jumlah, uji vitest, `make web-check`, pemakaian pertama npm (Spesifikasi Explorer Web Bagian 1 sampai 6, 8) | TSK-M31, DEC-26 | 🟢 selesai ([kartu](TSK-M32-explorer-web-react.md)) |
| TSK-M33 | MOD-14, repo | Paket Docker regtest: node + explorer + web, image dasar dikunci digest, compose, opsi `--http-bind` (Spesifikasi Docker v0.1) | TSK-M39, DEC-10 bagian 9 | 🟢 selesai ([kartu](TSK-M33-paket-docker.md)) |
| TSK-M34 | MOD-11 | Batas memori blok cabang, indeks header, dan catatan blok tidak sah (Spesifikasi P2P Bagian 9) | TSK-M28 | 🟢 selesai ([kartu](TSK-M34-batas-memori-node.md)) |
| TSK-M35 | MOD-05, MOD-11 | A: badan blok cabang di penyimpanan; B: presync header, menghapus batas sinkronisasi cabang dalam | TSK-M34 | 🟢 selesai ([kartu](TSK-M35-cabang-di-disk.md)) |
| TSK-M37 | MOD-09, MOD-12 | Enkripsi file wallet dengan password (Argon2id + XChaCha20-Poly1305), penghapusan rahasia dari memori, `wallet-encrypt`, `wallet-show-phrase` (Spesifikasi Wallet Bagian 7) | TSK-M19, DEC-27 | 🟢 selesai ([kartu](TSK-M37-enkripsi-wallet.md)) |
| TSK-M38 | MOD-14 (`web/`, `src/web`) | Antarmuka web Next.js + Tailwind + shadcn/ui hasil Google AI Studio (tanpa peta node karangan), penyajian static export dan CSP dengan hash skrip inline (Spesifikasi Explorer Web v0.2) | TSK-M32, DEC-26 | 🟢 selesai ([kartu](TSK-M38-explorer-web-nextjs.md)) |
| TSK-M39 | MOD-14, MOD-17 | Explorer hanya data sistem: tanpa alamat, saldo, jejak koin, dan alokasi genesis di API dan web; `Shared::read_index`; QR dihapus (DEC-28) | TSK-M38, DEC-28 | 🟢 selesai ([kartu](TSK-M39-explorer-data-sistem.md)) |
| TSK-M40 | MOD-14 (`limit`, `server`, `cli`, `web/`) | Batas laju API per klien dan bersama (`--api-rate`), `429 rate_limited` dengan `Retry-After`, tanpa menyimpan alamat IP; pesan 429 di web (Spesifikasi Explorer v0.1 Bagian 6) | TSK-M33 | 🟢 selesai ([kartu](TSK-M40-batas-laju-api.md)) |
| TSK-M41 | MOD-12, MOD-17 | Riwayat transaksi wallet dari blok (`get_block`) dengan tembolok `history.json`, entri tertunda dari mempool, perintah `history`; `os_random` lewat `getrandom` untuk Windows (Spesifikasi Wallet Bagian 6, DEC-31) | TSK-M36, DEC-10 bagian 11 | 🟢 selesai ([kartu](TSK-M41-riwayat-wallet.md)) |
| TSK-M42 | MOD-15 (`apps/wallet`) | Wallet desktop Tauri 2 di atas pustaka CLI: 10 perintah IPC, antarmuka React tanpa simulasi (wallet baru dengan periksa 3 kata, pulihkan, ringkasan, terima + QR, kirim, riwayat, pengaturan), dua bahasa (DEC-31) | TSK-M41, DEC-10 bagian 11 | 🟢 selesai ([kartu](TSK-M42-wallet-desktop.md)) |
| TSK-M43 | MOD-14 (`server`, `cli`) | Batas laju explorer di belakang Cloudflare Tunnel: `--trust-proxy`, kepala `CF-Connecting-IP` hanya untuk koneksi dari loopback (Spesifikasi Explorer v0.1 Bagian 6) | TSK-M40 | 🟢 selesai ([kartu](TSK-M43-explorer-trust-proxy.md)) |
| TSK-M44 | repo (Docker) | Explorer publik lewat Cloudflare Tunnel: `compose.publik.yaml` (cloudflared dikunci digest, token di luar repo, port host dicabut), uji regtest di `zevrigo.ratuaurion.store` (Spesifikasi Docker v0.1 Bagian 7) | TSK-M33, TSK-M43 | 🟢 selesai ([kartu](TSK-M44-explorer-publik-tunnel.md)) |
| TSK-M45 | MOD-06 (`transport`, `service`), MOD-11 (`daemon`, `p2p`), repo (Docker) | Seed P2P regtest lewat Cloudflare Tunnel: transport WebSocket libp2p, alamat `/dns*` lewat resolver sistem, keepalive `status` 60 detik, `--p2p-ws-port`, public hostname `zevrigo-seed.ratuaurion.store` (DEC-33, DEC-10 bagian 12, Spesifikasi P2P v0.1 Bagian 1, 4, 8) | TSK-M14, TSK-M28, TSK-M44 | 🟢 selesai ([kartu](TSK-M45-seed-websocket.md)) |
| TSK-M46 | repo (Docker) | Miner berkala untuk demo explorer publik regtest: layanan `miner-berkala` di `compose.publik.yaml`, satu blok setiap 120 detik ke kunci NUMS (Spesifikasi Docker v0.1 Bagian 7) | TSK-M44, TSK-M45 | 🟢 selesai ([kartu](TSK-M46-miner-berkala-publik.md)) |
| TSK-M47 | repo (Docker) | Demo D1: 5 node di `compose.publik.yaml` (seed, penambang, treasury, faucet, pengamat) lewat seed publik, batas RAM dari hasil ukur, daftar PeerId operator, `miner-berkala` ke node penambang dengan alamat wallet tetap (DEC-34) | TSK-M46, TSK-S08 | 🟢 selesai: 5 node berjalan ([kartu](TSK-M47-lima-node-demo.md)); wallet penambang dan treasury dibuat pencipta (13 November 2026) |
| TSK-M48 | MOD-06, MOD-08, MOD-11 | Demo D2: metode RPC `get_peer_info` (PeerId, arah, waktu tersambung, `status` terakhir, tinggi dan ujung rekan) | TSK-M47 | ⚪ |
| TSK-M49 | MOD-14 | Demo D2: `/api/v1/network` dan tampilan statistik jaringan (node terhubung, aktif, operator/peserta, alamat penambang 720 blok) | TSK-M48 | ⚪ |
| TSK-M50 | MOD-09 | Demo D3: output jenis 1 (kunci waktu, m = 1, n = 1): membuat, memindai, saldo terkunci, membelanjakan; pesan pendaftaran bertanda tangan | TSK-S08 | ⚪ |
| TSK-M51 | MOD-12 | Demo D3: CLI output jenis 1 dan alat distribusi (daftar penerima menjadi transaksi dari kantong `G`, tokenomik v0.1 Bagian 7) | TSK-M50 | ⚪ |
| TSK-M52 | MOD-14 | Demo D4: halaman treasury dan konfigurasi kantong (DEC-35) | TSK-M51 | ⚪ |
| TSK-M53 | MOD-18 | Demo tahap 1 langkah 1: inisialisasi `apps/demo` (DEC-36) dan mode `rpc` (gerbang RPC aman untuk wallet peserta), layanan `gerbang-rpc` dan hostname `zevrigo-rpc.ratuaurion.store` | TSK-S08 | 🟢 selesai ([kartu](TSK-M53-gerbang-rpc.md)) |
| TSK-M54 | MOD-18, MOD-14 | Demo tahap 1: mode `faucet` di node faucet (jaringan Docker), halaman faucet di explorer, Turnstile; stok dari kantong Faucet | TSK-M53, TSK-F11 bagian 1 | 🟡 mode `faucet` dan layanan compose ([kartu](TSK-M54-faucet.md)); berikutnya: halaman explorer, anti-bot |
| TSK-M55 | MOD-18 | Demo D7: mode `daftar` (pesan bertanda tangan, antrean siapa cepat dia dapat, alat operator) | TSK-M50, TSK-M53 | ⚪ |
| TSK-M56 | repo (alat) | Demo D8: toolchain Android di WSL (SDK, NDK, JDK, target Rust) lewat skrip langkah | — | 🟢 di `~/android` WSL (14 November 2026): JDK Temurin 21.0.13+9, cmdline-tools 22.0, platform-tools, platform 36, build-tools 37.0.0, NDK 30.0.16248370, target `aarch64-linux-android`; lisensi SDK disetujui pencipta (skrip Q135, Q135c) |
| TSK-M57 | MOD-14 (`apps/explorer/android`) | Demo D9: APK Explorer Tauri 2 (antarmuka dibundel, alamat API bisa diatur) dan CORS API explorer (DEC-36) | TSK-M49, TSK-M52, TSK-M56 | ⚪ |
| TSK-M58 | MOD-15 | Demo D10: APK Wallet Tauri 2 (HTTPS ke gerbang RPC, output jenis 1, faucet, daftar, label REGTEST) | TSK-M50, TSK-M53, TSK-M54, TSK-M55, TSK-M56 | 🟡 bagian 1: kode Android (pustaka, folder data, TLS akar sistem, menu bawah) ([kartu](TSK-M58-apk-wallet.md)); bagian 2: APK uji |
| TSK-M59 | MOD-14 | Demo D11: halaman panduan, teks persetujuan, unduh APK + SHA-256 | TSK-M57, TSK-M58 | ⚪ |
| TSK-M60 | MOD-08 | Demo tahap 1: HTTPS di klien RPC lewat fitur `tls` `jsonrpsee-http-client` (pilihan pencipta 12 November 2026, DEC-10 bagian 14, 11 paket) | TSK-M53 | 🟢 selesai ([kartu](TSK-M60-tls-klien-rpc.md)) |
| TSK-M61 | MOD-15, repo (CI) | Demo tahap 1: wallet desktop versi demo 0.2.0 (regtest bawaan, node bawaan gerbang publik, pita REGTEST, privasi di installer dan Pengaturan), workflow rilis GitHub Actions (rilis draf + SHA256SUMS), Code signing policy; tanda tangan SignPath menyusul (DEC-07) | TSK-M60 | 🟢 selesai ([kartu](TSK-M61-wallet-demo.md)) |
| TSK-M62 | MOD-18 | Demo tahap 1: alat upacara genesis regtest `zevrigo-demo upacara`, membelanjakan `G` (kunci rahasia `1`) ke kantong treasury dalam satu transaksi (Rancangan Ekosistem Demo v0.1 Bagian 4.2, 4.3) | TSK-M47, TSK-M53 | 🟢 selesai ([kartu](TSK-M62-alat-upacara-genesis.md)) |
| TSK-M63 | MOD-12 | Demo tahap 1: `zevrigo-cli send --dari <alamat>`, kirim hanya dari satu kantong treasury dengan kembalian ke kantong itu (isi ulang faucet bertahap, keputusan pencipta 13 November 2026) | TSK-M54, TSK-M62 | 🟢 selesai ([kartu](TSK-M63-kirim-dari-alamat.md)) |
| TSK-M36 | MOD-08, MOD-11, MOD-12 | Metode RPC `scan_mempool`; saldo CLI dan pemilihan koin dengan transaksi tertunda (Spesifikasi RPC 3.9, Wallet Bagian 6) | TSK-M19 | 🟢 selesai ([kartu](TSK-M36-saldo-tertunda.md)) |
| TSK-M25 | MOD-01 | `enum Network { Mainnet, Testnet, Regtest }` di `params` (Spesifikasi Validasi Bagian 11) | TSK-S02 | 🟢 selesai ([kartu](TSK-M25-primitives-network.md)) |

## Backlog: task fitur (TSK-F)

| ID | Fitur | Bergantung pada | Status |
|---|---|---|---|
| TSK-F01 | FTR-01 Node regtest tunggal | TSK-M09, M10, M12, M18 | 🟢 selesai (uji `apps/node/tests/regtest_node.rs`, [kartu TSK-M18](TSK-M18-node.md)) |
| TSK-F02 | FTR-02 Menambang di regtest | TSK-F01, TSK-M20, DEC-20 | 🟢 selesai ([kartu](TSK-F02-menambang-regtest.md)) |
| TSK-F03 | FTR-03 Kirim dan terima lewat CLI | TSK-F02, TSK-M13, M16, M19, DEC-23 | 🟢 selesai ([kartu](TSK-F03-kirim-terima-cli.md)) |
| TSK-F04 | FTR-04 Jaringan multi-node dan uji reorg | TSK-F02, TSK-M14, TSK-M27, TSK-M28 | 🟢 selesai ([kartu](TSK-F04-multi-node.md)) |
| TSK-F05 | FTR-05 Explorer minimal | TSK-F01, TSK-M29, TSK-M30, DEC-25 | 🟢 selesai ([kartu](TSK-F05-explorer-minimal.md)) |
| TSK-F06 | FTR-06 Testnet publik | TSK-F03, F04, F05, TSK-M06 | 🔴 keputusan pencipta: waktu peluncuran dan lokasi seed node |
| TSK-F07 | FTR-07 Wallet desktop | TSK-F03, TSK-M41, TSK-M42, DEC-31 | 🟢 v0.1: installer Windows NSIS, uji pencipta dengan node regtest (8 November 2026) |
| TSK-F08 | FTR-08 Wallet mobile | TSK-F07, DEC-04, DEC-05 | ⚪ dipecah nanti (demo regtest lebih dulu: APK Wallet di TSK-M58, DEC-34) |
| TSK-F09 | FTR-09 Aplikasi pelaku usaha | TSK-M17, TSK-F07, DEC-06 | ⚪ dipecah nanti |
| TSK-F10 | FTR-10 Reproducible build dan rilis | TSK-F06, DEC-07 | ⚪ dipecah nanti |
| TSK-F11 | FTR-11 Demo publik regtest untuk pemula: bagian 1 peluncuran tertutup (reset, upacara genesis, tahap 1); bagian 2 pendaftaran, distribusi vesting, pengumuman (tahap 2) | TSK-M47 sampai TSK-M62, DEC-34 | 🟡 bagian 1 selesai: peluncuran tertutup ([kartu](TSK-F11-peluncuran-demo.md)) |

## Jalur kritis menuju testnet

```mermaid
graph LR
  TSKM01[TSK-M01 Tipe dasar] --> TSKM02[TSK-M02 BLAKE3]
  TSKM01 --> TSKM07[TSK-M07 Pasokan]
  TSKS01[TSK-S01 Spesifikasi format] --> TSKM03[TSK-M03 Transaksi dan blok]
  TSKM02 --> TSKM21[TSK-M21 ID transaksi dan blok]
  TSKM03 --> TSKM21
  TSKM21 --> TSKM09[TSK-M09 Validasi]
  TSKM05[TSK-M05 PoW dev] --> TSKM09
  TSKS02[TSK-S02 Spesifikasi validasi] --> TSKM09
  TSKS02 --> TSKM08[TSK-M08 ASERT dan chainwork]
  TSKM08 --> TSKM09
  TSKM07 --> TSKM09
  TSKM09 --> TSKF01[TSK-F01 Node regtest]
  TSKF01 --> TSKF02[TSK-F02 Menambang]
  TSKF02 --> TSKF03[TSK-F03 Kirim dan terima]
  TSKF02 --> TSKF04[TSK-F04 Multi-node]
  TSKF01 --> TSKF05[TSK-F05 Explorer]
  TSKF03 --> TSKF06[TSK-F06 Testnet publik]
  TSKF04 --> TSKF06
  TSKF05 --> TSKF06
  DEC01[DEC-01 Strategi PoW] --> TSKM06[TSK-M06 PoW RandomX]
  TSKM06 --> TSKF06
```

**Bisa dikerjakan paralel:** TSK-M07 bersamaan dengan TSK-S01; TSK-M02 dan TSK-M04 setelah TSK-M01; TSK-M10, M12, M14 setelah TSK-M03.

## Titik rawan
- 🟢 **TSK-S01** lengkap, termasuk sighash v1 (DEC-13, TSK-M24).
- 🟢 **TSK-S02** lengkap (aturan validasi v0.1, DEC-17). Vektor K, A, W, M, dan genesis regtest siap dipakai TSK-M08 dan TSK-M09.
- 🟢 **DEC-01** diputuskan 5 November 2026: varian RandomX lewat randomx-rs, selesai di TSK-M06. Testnet publik menunggu TSK-F06 (cap waktu genesis testnet, seed node). Regtest tetap memakai PoW dev (DEC-09).
- 🟡 **MOD-04 consensus** dipakai node, mempool, miner, dan semua fitur. Setiap perubahannya berdampak luas.

## Aturan pembaruan
Setiap task yang selesai wajib memperbarui status di tabel ini dan di kartu modul terkait sebagai bagian dari commit task itu.
