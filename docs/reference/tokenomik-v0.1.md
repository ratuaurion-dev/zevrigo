# Spesifikasi Tokenomik Zevrigo v0.1

> Status: 🟢 parameter pasokan disetujui pencipta 7 November 2026 (DEC-30). Bagian 6 (program investor dari `G`): kerangka
> disetujui, porsi dan jadwal final menunggu keputusan pencipta. Angka konsensus bersumber dari `AGENTS.md` dan
> `crates/primitives/src/params`; dokumen ini menjelaskan, bukan menggantikan.

## 1. Parameter
| Parameter | Nilai |
|---|---|
| Satuan | 1 ZVRG = 10^18 quanta (`u128`, tanpa float) |
| Waktu blok | 120 detik, 262.800 blok per tahun |
| Pasokan tambang `S` | **100.000.000 ZVRG** |
| Alokasi genesis `G` | **10.000.000 ZVRG** (`S / 10`), satu output di blok 0, di luar `S`, tanpa kunci waktu di konsensus |
| Konstanta peluruhan `D` | 2.274.842 |
| Hadiah ekor `T` | **2 ZVRG per blok** (525.600 ZVRG per tahun), selamanya |
| Slow start | 10.080 blok (±14 hari) |
| Kematangan coinbase | 100 blok (berlaku juga untuk `G`) |
| Biaya transaksi | 100% untuk penambang, tanpa pembakaran |

## 2. Rumus hadiah
- `M` = total hasil tambang sebelum blok yang dihitung (`G` tidak dihitung).
- `base(M) = max((S − M) / D, T)`, pembagian bilangan bulat dibulatkan ke bawah; jika `M ≥ S`, `S − M` dianggap 0.
- Tinggi `1 ≤ h < 10.080`: `subsidy = base(M) × h / 10.080`. Tinggi `h ≥ 10.080`: `subsidy = base(M)`. Genesis: 0.
- Hadiah blok penuh pertama ±43,96 ZVRG, turun mulus setiap blok tanpa *halving* mendadak.
- Vektor uji persis (quanta): kartu `docs/tasks/TSK-M07-consensus-pasokan.md`.

## 3. Jadwal pasokan (dihitung per blok)
| Akhir tahun | Hasil tambang | Total beredar (dengan `G`) | Pertambahan tahun itu | Porsi `G` |
|---|---|---|---|---|
| 1 | 10,71 jt | 20,71 jt | — | 48,3% |
| 2 | 20,45 jt | 30,45 jt | 47,0% | 32,8% |
| 3 | 29,13 jt | 39,13 jt | 28,5% | 25,6% |
| 5 | 43,75 jt | 53,75 jt | 14,7% | 18,6% |
| 6 | 49,89 jt | 59,89 jt | 11,4% | 16,7% |
| 10 | 68,43 jt | 78,43 jt | 5,2% | 12,7% |
| 15 | 82,28 jt | 92,28 jt | 2,4% | 10,8% |
| 20 | 90,06 jt | 100,06 jt | 1,2% | 10,0% |
| 30 | 97,15 jt | 107,15 jt | 0,5% | 9,3% |
| 50 | 107,66 jt | 117,66 jt | 0,4% | 8,5% |

Setengah `S` tertambang dalam ±6 tahun. Hadiah ekor dimulai di blok **7.034.557** (±tahun 26,8).

## 4. Pasokan ekor
Sesudah `(S − M) / D` turun ke 2 ZVRG, setiap blok menerbitkan tepat 2 ZVRG selamanya. Tujuannya: penambang tetap dibayar
sesudah pasokan tambang habis, sehingga keamanan chain tidak hanya bergantung pada biaya transaksi. Akibatnya pasokan total
tidak punya batas keras; pertambahannya tetap 525.600 ZVRG per tahun, sehingga persentasenya terus mengecil (±0,4% di tahun
ke-50).

## 5. Distribusi publik
Publik mendapat ZVRG dengan menambang (PoW RandomX, DEC-01) tanpa izin, atau lewat pasar. Tidak ada koin pra-alokasi selain `G`.

## 6. Alokasi genesis `G` dan program investor awal
### 6.1 Kedudukan `G`
`G` adalah satu output ke kunci pencipta di blok 0. Konsensus tidak mengunci `G` (keputusan pencipta 3 November 2026); output itu
bisa dibelanjakan mulai blok 100. Semua putaran pendanaan awal diambil dari `G`.

### 6.2 Kategori penerima dari `G`
| Kategori | Tahap | Keterangan |
|---|---|---|
| Pre-Seed / Angel / Early Backers | Sebelum testnet publik atau bersamaan | Masuk paling awal, risiko tertinggi, kunci paling panjang |
| Seed / Strategic Partners | Sebelum mainnet | Mitra yang mendanai produk inti, membawa jaringan atau keahlian |
| Private Sale | Sesaat sebelum mainnet | Peserta sebelum koin diperdagangkan |
| Genesis / Founding Contributors | Peluncuran mainnet | Komunitas awal, penyedia likuiditas sejak hari pertama |
| Likuiditas pasar | Sesudah mainnet | Kantong terpisah untuk pasar |
| Operasional dan ekosistem | Jangka panjang | Server, domain, audit, perawatan |
| Pencipta | — | Bagian milik pencipta |

Porsi tiap kategori ditetapkan pencipta dan dicatat di versi berikutnya dokumen ini.

### 6.3 Jadwal kunci acuan
Satu bulan = **21.900 blok** (262.800 / 12). TGE (*Token Generation Event*) = blok 0 mainnet; distribusi dari `G` mulai blok 100.

| Kategori | Cair saat distribusi | Cliff | Vesting (linear bulanan) |
|---|---|---|---|
| Pre-Seed / Angel | 0–5% | 6–12 bulan | 18–24 bulan |
| Seed / Strategic | 5–10% | 6 bulan | 12–18 bulan |
| Private Sale | 10–25% | 0–3 bulan | 6–12 bulan |
| Genesis Contributors | 25–50% | tanpa cliff | 3–6 bulan |

### 6.4 Mekanisme di chain (tanpa perubahan konsensus)
Kunci investor dijamin aturan konsensus yang sudah ada, bukan janji. Untuk setiap penerima, transaksi distribusi dari `G` berisi:
1. Bagian cair → output jenis 0 biasa ke alamat penerima.
2. Setiap tahap vesting → satu output **jenis 1** (kondisi m = 1, n = 1 dengan kunci penerima) dengan
   `lock_height = B + cliff + k × 21.900`, `B` = tinggi blok distribusi, `k = 1 … jumlah bulan vesting`.

Node menolak transaksi yang membelanjakan output jenis 1 sebelum `lock_height` (Spesifikasi Validasi T4). Akibatnya:
- penerima tidak bisa menjual lebih cepat dari jadwal;
- sesudah dikirim, koin milik penerima; pencipta tidak bisa menariknya kembali;
- siapa pun bisa memeriksa jumlah dan tinggi buka setiap output terkunci di chain.

Contoh ukuran: cliff 12 bulan + vesting 24 bulan = 1 output cair + 24 output terkunci, ±1–1,5 KB per penerima (batas blok
100.000 byte).

### 6.5 Investor yang membayar sebelum mainnet
Perjanjian tertulis (bentuk lazim: SAFT, *Simple Agreement for Future Tokens*) mencatat jumlah ZVRG dan jadwal kunci. Alamat
mainnet penerima dikumpulkan sesudah wallet mainnet tersedia. Koin testnet tidak dikonversi ke mainnet.

### 6.6 Yang tidak dicantumkan di dokumen publik
Harga atau diskon tiap putaran, target dana, dan identitas investor tidak ditulis di repo. Di chain hanya terlihat jumlah, alamat,
dan tinggi buka (prinsip DEC-28: transparan untuk sistem, privat untuk individu).

## 7. Pekerjaan teknis terkait (tanpa perubahan konsensus)
| Pekerjaan | Modul |
|---|---|
| Membuat dan membelanjakan output jenis 1 dengan kunci waktu (alamat `…1p…`) | MOD-09 wallet-core, MOD-12 CLI |
| Alat jadwal distribusi: daftar penerima → transaksi distribusi dari `G` | MOD-12 CLI |
| Tampilan jumlah output terkunci per tinggi buka (data sistem, DEC-28) | MOD-14 explorer |

## 8. Catatan hukum
Penawaran aset kripto kepada investor di Indonesia diawasi OJK. Bentuk perjanjian dan penawaran diperiksa ahli hukum sebelum
ditandatangani.
