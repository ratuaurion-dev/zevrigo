# TSK-M35 · Badan blok cabang di penyimpanan dan presync header · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-05 (crates/storage), MOD-11 (apps/node) · **Untuk:** persiapan testnet (FTR-06)
- **Bergantung pada:** TSK-M34 · **Memblokir:** TSK-F06
- **Estimasi:** M · **Ditulis:** Quick (kode dan test), 4 November 2026

## Tujuan
Menghapus batas yang tersisa di Spesifikasi P2P v0.1 Bagian 9, tanpa batas reorg dan tanpa checkpoint:
1. Reorg sedalam apa pun tidak boleh menahan badan blok di memori node.
2. Cabang bersaing yang sangat dalam (lebih dari `max_headers` header sebelum terbukti lebih berat) tetap bisa
   disinkronkan lewat P2P.

## Keputusan teknis (Quick, tanpa DEC)
- **Dipecah dua bagian.** A: badan blok cabang di penyimpanan (MOD-05 + MOD-11). B: presync header (MOD-11 + P2P).
- **Badan blok di disk, indeks di memori.** Menyimpan seluruh indeks header di disk tidak menyelesaikan masalah: header
  cabang murah (target ASERT di awal chain) bisa memenuhi disk sama seperti memori. Yang membuat memori besar adalah
  badan blok (sampai 100_000 byte per blok); header dan ringkasan cukup kecil (puluhan byte) untuk tetap di memori.
- **Indeks blok cabang tidak dipulihkan saat node dibuka.** Badan blok cabang lama dibuang (`clear_side_blocks`), sama
  seperti perilaku v0.1: cabang diterima lagi dari rekan. Tanpa migrasi skema redb: tabel `side_blocks` dibuat saat
  dibuka, versi skema tetap 1.
- **Bagian B, presync header:** saat indeks header penuh (`HeadersOutcome::Full`), header rekan itu diperiksa
  V2, V3, V5 sampai V8 tanpa disimpan; yang diingat hanya ujung, chainwork, 11 cap waktu terakhir, dan ID setiap 1_000
  header. Setelah chainwork cabang melebihi chain aktif, header diunduh ulang dari titik cabang, dicocokkan dengan ID yang
  diingat, lalu disimpan sebagai header terbukti (tidak dibuang dan tidak dihitung dalam `max_headers`). Memori presync
  40 byte per 1_000 header (tinggi + ID) ditambah tampungan unduh ulang paling banyak 1_000 header.
- **ID penuh sebagai komitmen**, bukan bit acak: lebih sederhana, dan memorinya tetap kecil.
- **ID yang tidak cocok tidak disanksi**: rekan jujur yang baru reorg di antara dua tahap juga menghasilkan ketidakcocokan.
  Biayanya hanya bandwidth; memori tetap terbatas. Header tidak sah di tahap hitung tetap disanksi.
- **Header terbukti dilepas** (jadi header biasa) begitu tidak ada lagi header terbukti yang lebih berat daripada chain
  aktif, agar header terbukti dari cabang yang kalah tidak menempati memori selamanya.

## Bagian A (selesai)
| Bagian | Isi |
|---|---|
| MOD-05 `ChainStore` | `put_side_block(block_id, &Block)`, `side_block(&id) -> Option<Block>`, `remove_side_block(&id)`, `clear_side_blocks()`; tidak menyentuh chain aktif |
| `MemoryStore` | `BTreeMap<Hash32, Block>` terpisah |
| `RedbStore` | Tabel `side_blocks` (key block_id, value encoding blok); setiap operasi satu write transaction |
| MOD-11 `SideBlock` | Hanya `header`, `chainwork`, `sequence`, `size`; badan blok lewat `store_side`/`side_body`/`drop_side` |
| Reorg | Blok yang dilepas langsung jadi blok cabang di penyimpanan; badan blok cabang dibaca satu per satu saat disambungkan; saat gagal, chain lama disambungkan lagi dari penyimpanan |
| Mempool sesudah reorg | Transaksi dari blok yang dilepas dibaca dari penyimpanan, berurutan, paling banyak sebesar batas mempool |
| `Node::open` | `clear_side_blocks` sebelum ringkasan chain disusun |
| `blocks_by_id` | Badan blok cabang dibaca dari penyimpanan |

### Test (workspace 384 → 388)
- MOD-05 (26 → 28): `side_blocks_are_kept_apart_from_active_chain` (MemoryStore dan RedbStore sama), `side_blocks_persist_until_cleared_and_old_files_open` (file redb dibuka ulang, file skema 1 tanpa tabel `side_blocks`).
- MOD-11 (32 → 34): `side_block_bodies_live_in_store_and_are_cleared_on_reopen` (RedbStore: badan blok cabang di penyimpanan, reorg bolak-balik, `GetBlocks`, dibuka ulang), `trimmed_and_invalid_side_blocks_leave_store` (blok yang dipangkas dan blok tidak sah beserta keturunannya hilang dari penyimpanan, chain lama tidak tertinggal di sana).

## Bagian B (selesai)
| Bagian | Isi |
|---|---|
| `Presync` (publik) | Tahap `Start` → `Counting` (ujung, chainwork, 11 cap waktu, komitmen per `PRESYNC_COMMIT_INTERVAL` = 1_000) → `Redownload` (induk yang diharapkan, tampungan, komitmen tersisa) |
| `Node::presync_headers` | `PresyncOutcome::{Continue{after}, Redownload{fork}, Done{added}, NotHeavier, Mismatch, Deferred, Disconnected, Invalid{rule}}` |
| Indeks header | `HeaderEntry::proven`; `store_header` hanya menghitung dan membuang header biasa; `store_proven`; `prune_headers` melepas header terbukti jika tidak ada yang lebih berat daripada chain aktif |
| P2P | `PeerState::presync`; `HeadersOutcome::Full` → `start_presync`; jawaban `Headers` rekan itu ke presync; `Continue` lanjut jika pesan penuh; `Redownload` minta sesudah titik cabang; `Done` → unduh badan blok; `Disconnected`/`Invalid` disanksi; permintaan gagal menghentikan presync; sinkronisasi header biasa tidak dimulai selama presync |

### Test (workspace 388 → 391, MOD-11 34 → 37)
- `presync_syncs_branch_longer_than_header_index`: batas indeks 3 header, cabang 8 blok baru lebih berat di tinggi 6: tanpa presync macet (`Full`, tidak ada badan blok diminta); dengan presync hitung dua pesan, unduh ulang dua pesan, 6 header terbukti melebihi batas, badan blok diunduh, reorg, sisanya lewat sinkronisasi biasa.
- `presync_stops_on_bad_or_changed_headers`: tidak menyambung, V5, pesan kosong, V8, cabang lain saat unduh ulang, header dilompati, rekan berhenti sebelum ujung; tidak ada header tersimpan.
- `presync_checks_commitment_every_interval`: chain aktif 1_001 blok, cabang 1_003 blok; unduh ulang yang berbeda sesudah tinggi 1_001: komitmen 1_000 cocok (1_000 header terbukti), ujung tidak cocok; header terbukti dilepas karena tidak lebih berat; presync ulang menambah 2 header, badan blok diminta dari yang terendah.

Jalur P2P (`p2p/mod.rs`) diperiksa lewat kompilasi, clippy, dan uji multi-node yang ada; uji ujung ke ujung dengan cabang
lebih dari 100_000 blok tidak dibuat karena butuh menambang ratusan ribu blok.
