# TSK-M47 · Lima node jaringan demo regtest · 🟡 bagian A · Prioritas: Tinggi
- **Jenis:** task repo (Docker) · **Modul:** — (`compose.publik.yaml`, tanpa perubahan kode Rust) · **Untuk:** FTR-11 (DEC-34)
- **Bergantung pada:** TSK-M46, TSK-S08 · **Memblokir:** TSK-M48, TSK-M51, TSK-M54
- **Estimasi:** S · **Ditulis:** Quick, 12 November 2026 · **Langkah:** D1 Rancangan Ekosistem Demo v0.1

## Tujuan
Jaringan demo memiliki 5 node operator dengan peran berbeda (seed, penambang, treasury, faucet, pengamat), agar pemula
melihat bahwa jaringan terdiri dari banyak node yang saling menyiarkan blok, bukan satu server.

## Ukuran (Q125, 12 November 2026, tinggi 121)
| Yang diukur | Hasil |
|---|---|
| Node seed | sekitar 10 MiB, CPU di bawah 0,2% |
| Explorer, cloudflared, miner-berkala | sekitar 4 MiB, 19 MiB, 3 MiB |
| Node baru lewat seed publik | sinkron dalam 8 detik, puncak sekitar 2 MiB, diam 2,3 MiB, disk 180 KiB |
| Volume data seed (chain + indeks explorer) | 4,4 MB |
| Mesin (WSL) | 4 CPU, 7,7 GiB |

Batas per node operator: **256 MiB dan 0,5 CPU** (jauh di atas ukuran, untuk ruang tumbuh chain sekitar 720 blok per hari).
Node seed dan explorer belum diberi batas agar tidak dinyalakan ulang; batasnya ditambahkan saat peluncuran (TSK-F11).

## Isi
| Bagian | Isi |
|---|---|
| `node-penambang`, `node-treasury`, `node-faucet`, `node-pengamat` | Image `zevrigo:regtest`, volume sendiri, tanpa port host, `--connect` ke seed publik (`ZVRG_SEED` bisa mengganti), batas 256 MiB dan 0,5 CPU, `restart: unless-stopped` |
| `miner-berkala` | Pindah dari namespace seed ke namespace `node-penambang`, volume penambang baca saja. Menunggu 30 detik agar node sinkron, lalu satu blok setiap 120 detik. Payout dari `/data/payout-penambang`; NUMS selama file itu belum ada |
| Dokumen | Spesifikasi Docker v0.1 Bagian 7, README, docs/tasks |

## Bagian B (pencipta, sesudah Q126)
Dilakukan pencipta sendiri di terminal, bukan oleh agent, karena frasa pemulihan tampil di layar:
1. `wallet-create` di `node-penambang` dan `node-treasury`, password diketik sendiri, 24 kata ditulis di kertas.
2. Satu perintah `address` di `node-penambang` menulis payload ke `/data/payout-penambang`; `miner-berkala` beralih sendiri
   di putaran berikutnya.
3. Gerbang bagian B: log `miner-berkala` mencatat `payout alamat penambang`, dan `balance` wallet penambang menunjukkan hadiah.

## Gerbang (Q126)
Lihat Spesifikasi Docker v0.1 Bagian 7, "Gerbang lima node".

## Batas
- Node operator tersambung hanya ke seed. Jika internet atau tunnel putus, mereka terpisah dari seed dan dari satu sama lain,
  lalu reorg ke chain dengan kerja terbanyak saat tersambung lagi.
- Semua node berjalan di satu komputer: jumlah node tidak menambah keamanan.
- Chain uji sekarang (tinggi di atas 100) tidak dipakai untuk upacara genesis: output `G` (kunci `1` publik) bisa dibelanjakan
  siapa pun. Reset di TSK-F11 menghapus `regtest/chain.redb` dan indeks saja, bukan `wallet.json`, `p2p_key`, atau
  `payout-penambang`.

## Definition of done
- [x] Gerbang Q126 lolos (bagian A)
- [ ] Bagian B: wallet penambang dan treasury dibuat pencipta, miner membayar ke alamat penambang
- [x] Spesifikasi Docker v0.1 Bagian 7, README, docs/tasks
