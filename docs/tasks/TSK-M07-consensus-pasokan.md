# TSK-M07 · Rumus pasokan HSIS di MOD-04 · 🟡 · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-04 (crates/consensus) · **Untuk:** FTR-02
- **Bergantung pada:** TSK-M01 · **Memblokir:** TSK-M09
- **Estimasi:** M

## Tujuan
Mengimplementasikan jadwal penerbitan koin persis sesuai parameter protokol. Fungsi ini menentukan
berapa koin baru yang boleh dibuat di setiap blok. Satu quanta saja berbeda dari implementasi lain
berarti chain terpecah, jadi hasilnya harus cocok dengan vektor uji di bawah sampai digit terakhir.

## Konteks
- Baca AGENTS.md (tabel parameter: S, G, D, T, M, hadiah dasar, slow start, genesis).
- Pakai tipe dan konstanta dari homeostasis-primitives (TSK-M01). Jangan menyalin ulang angka.
- Crate sudah ada (kerangka R0): crates/consensus. Dependensinya ke primitives, crypto, dan pow sudah terpasang; jangan dihapus.
- Ini task modul: hanya crates/consensus yang boleh berubah, ditambah status di docs/.

## Ruang lingkup
- Boleh diubah: crates/consensus/**; docs/tasks/README.md dan docs/modules/MOD-04-consensus.md (status saja).
- Dependensi crate ini: tambahkan `thiserror = { workspace = true }` dan dev-dependency `proptest = { workspace = true }`. Jangan menambah dependensi lain.
- Jangan diubah: crates/primitives (kalau butuh perubahan di sana, berhenti dan tanya), crate lain, AGENTS.md, .agent/.
- Di luar lingkup: validasi blok/transaksi (TSK-M09), biaya transaksi, ASERT (TSK-M08).

## Persyaratan
API di modul `supply`:
1. `pub fn base_reward(mined_so_far: Amount) -> Amount`
   = `max((S − M) / D, T)` dengan `(S − M) = 0` jika `M ≥ S`. Tidak boleh panic untuk input apa pun.
2. `pub fn block_subsidy(height: BlockHeight, mined_so_far: Amount) -> Result<Amount, SupplyError>`
   - height 0 → `Amount::ZERO`
   - 1 ≤ h < 10_080 → `base_reward(M) × h / 10_080` (perkalian dengan checked; overflow → error)
   - h ≥ 10_080 → `base_reward(M)`
3. `pub struct SupplySchedule` yang mengimplementasikan `Iterator<Item = ScheduleEntry>` mulai dari genesis,
   dengan `ScheduleEntry { height, subsidy, mined_total_after }`. Iterasi O(1) per blok, tanpa menghitung ulang dari awal.
4. `pub fn total_supply_after(mined_total: Amount) -> Option<Amount>` = `G + mined_total`.
5. `SupplyError` memakai thiserror.

Definisi: `mined_total_after(h)` = jumlah subsidy untuk tinggi 0 sampai h (inklusif). `M` untuk blok h
adalah `mined_total_after(h − 1)`.

## Vektor uji (wajib cocok persis, dalam quanta)
| h | subsidy(h) | mined_total_after(h) |
|---|---|---|
| — | base_reward(0) = 19254084459492131761 | — |
| 1 | 1910127426536917 | 1910127426536917 |
| 2 | 3820254852907233 | 5730382279444150 |
| 3 | 5730382278861043 | 11460764558305193 |
| 10_079 | 19209580274199481401 | 96923574892844250978088 |
| 10_080 | 19211477731247777097 | 96942786370575498755185 |
| 10_081 | 19211469286055657712 | 96961997839861554412897 |
| 262_800 (tahun 1) | 17191489153883609052 | 4692095621690256899802646 |
| 1_576_800 (tahun 6) | 9648396965421659659 | 21851430998783226318217064 |
| 2_628_000 (tahun 10) | 6078109167266476710 | 29973268063826360852707222 |
| 5_256_000 (tahun 20) | 1914484381499177180 | 39444852435106030283184651 |
| 6_570_000 (tahun 25) | 1074468019114621630 | 41355756096929275015017236 |
| 6_832_800 (tahun 26) | 1000000000000000000 | 41624566977945720492858347 |
| 7_884_000 (tahun 30) | 1000000000000000000 | 42675766977945720492858347 |

- Tinggi pertama dengan subsidy == T (awal pasokan ekor): **6_733_393** (sekitar tahun ke-25,62).
- Baris h ≤ 10_081 sebagai unit test biasa. Baris tahun 1–30 dan awal pasokan ekor sebagai test `#[ignore]`
  (dijalankan lewat `make test-slow`, mode release).
- Vektor cukup ditulis di dalam test crate consensus. File data `tests/vectors/supply_v1.json` dibuat lewat task terpisah, karena folder tests/ di luar ruang lingkup task modul.
- Jika hasil berbeda walau satu digit, JANGAN mengubah angka di tabel. Berhenti dan laporkan perbedaannya
  beserta tinggi pertama yang berbeda.

## Property test (proptest)
- Untuk semua M: `base_reward(M) ≥ T` dan `base_reward(M) ≤ base_reward(0)`
- `base_reward` tidak naik saat M naik (monoton tidak naik)
- Untuk h ≥ 10_080: `subsidy == base_reward(M)`; untuk 1 ≤ h < 10_080: `subsidy ≤ base_reward(M)`
- `block_subsidy` tidak pernah panic dan tidak error untuk M ≤ S + 10^30 dan h ≤ 10^9
- Setelah awal pasokan ekor, `mined_total_after` bertambah tepat T per blok

## Kriteria penerimaan
- [ ] Semua vektor uji cocok persis
- [ ] Semua property test lolos
- [ ] `make check-all` dan `make test-slow` lolos
- [ ] Tanpa float di mana pun, tanpa pengecualian (persentase di pesan assert pun dihitung dengan bilangan bulat, misalnya basis poin)
- [ ] `#![forbid(unsafe_code)]` tetap ada dan `make check-forbidden` lolos
- [ ] Status TSK-M07 di docs/tasks/README.md menjadi 🟢, dan kartu MOD-04 mencatat TSK-M07 selesai

## Verifikasi
```bash
make check-all
make test-slow
git diff --stat HEAD~1
```

## Laporan akhir
Format AGENTS.md. Sertakan waktu eksekusi `make test-slow`.