# Dokumen Referensi Utama (`docs/reference/`)

Folder ini disediakan sebagai tempat penyimpanan dua dokumen sumber kebenaran desain Zevrigo Coin (ZVRG) yang disalin langsung oleh pengguna:

1. **Dokumen Konsep v1.1** — spesifikasi filosofi desain, ekonomi moneter, jadwal pasokan, Proof of Work, ASERT, dan aturan konsensus inti.
2. **Dokumen Aplikasi v0.1** — spesifikasi ekosistem aplikasi pengguna (node, CLI, miner, explorer, dompet personal, dan aplikasi pelaku usaha/merchant).

Kedua dokumen tersebut merupakan sumber kebenaran desain tertinggi. Ringkasan parameter numerik dan aturan kerja agent yang mengikat kode sehari-hari disalin ke `AGENTS.md` di akar repositori; jika ditemukan perbedaan tafsir antara kode/kartu tugas dan kedua dokumen referensi ini, pengerjaan wajib dihentikan untuk diklarifikasi.

## Spesifikasi turunan (ditulis Quick, disetujui pencipta)

| Dokumen | Isi | Task |
|---|---|---|
| [spesifikasi-format-v0.1.md](spesifikasi-format-v0.1.md) | Format transaksi dan blok, encoding kanonis, alamat, sighash | TSK-S01 |
| [spesifikasi-validasi-v0.1.md](spesifikasi-validasi-v0.1.md) | Aturan validasi, ASERT, chainwork, genesis regtest | TSK-S02 |
| [spesifikasi-wallet-v0.1.md](spesifikasi-wallet-v0.1.md) | Frasa BIP39, turunan BIP32, alamat, pemilihan koin, pemindaian saldo | TSK-S03 |
| [spesifikasi-rpc-v0.1.md](spesifikasi-rpc-v0.1.md) | 8 metode JSON-RPC regtest | TSK-S04 |
| [spesifikasi-p2p-v0.1.md](spesifikasi-p2p-v0.1.md) | Transport libp2p (DEC-24): TCP atau WebSocket (DEC-33), 5 protokol, sinkronisasi header lalu blok, siaran, sanksi rekan, keepalive | TSK-S05, TSK-M45 |
| [spesifikasi-explorer-v0.1.md](spesifikasi-explorer-v0.1.md) | Explorer: indeks SQLite tahan reorg (DEC-25), pemeriksaan jumlah koin, API HTTP baca saja, halaman alokasi genesis | TSK-S06 |
| [spesifikasi-explorer-web-v0.2.md](spesifikasi-explorer-web-v0.2.md) | Antarmuka web explorer Next.js + Tailwind + shadcn/ui (DEC-26): halaman, dua bahasa, jumlah `bigint`, CSP dengan hash, penyajian `--web-dir`, dependensi npm | TSK-S07, TSK-M38 |
| [tokenomik-v0.1.md](tokenomik-v0.1.md) | Tokenomik (DEC-30): pasokan tambang 100 jt, alokasi genesis 10 jt, ekor 2 ZVRG, jadwal pasokan, program investor awal dari `G` dengan kunci waktu di chain | DEC-30 |
| [spesifikasi-docker-v0.1.md](spesifikasi-docker-v0.1.md) | Paket Docker regtest: image dasar dikunci digest, isi image, compose node + explorer, opsi `--http-bind`, explorer dan seed publik lewat Cloudflare Tunnel | TSK-M33, TSK-M44, TSK-M45 |
| [paket-mod06-websocket-v0.1.txt](paket-mod06-websocket-v0.1.txt) | 6 paket fitur `websocket` libp2p: lisensi, `unsafe`, checksum (DEC-10 bagian 12) | TSK-M45 |
| [spesifikasi-explorer-web-v0.1.md](spesifikasi-explorer-web-v0.1.md) | ⚪ Diganti v0.2: versi Vite + React tanpa Tailwind | TSK-S07 |
