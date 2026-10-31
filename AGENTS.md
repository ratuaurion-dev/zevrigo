# AGENTS.md: Homeostasis Coin (HSIS)

## Tentang proyek
Homeostasis Coin adalah blockchain Layer 1 Proof of Work yang ditulis dari nol dalam Rust.
Lapisan dasarnya sengaja minimal: model UTXO, tanpa VM serbaguna, dan hanya tipe transaksi
bawaan (transfer, multi-tanda tangan, kunci waktu).

Sebagian besar kode di repo ini adalah KODE KONSENSUS. Bug di sini bisa memecah jaringan,
mencetak koin di luar jadwal, atau membuat dana pengguna hilang. Kebenaran dan determinisme
selalu lebih penting daripada kecepatan menulis atau kerapian.

Prinsip yang tidak bisa ditawar:
- Tidak ada kunci admin, tombol pembekuan, atau hak istimewa untuk pihak mana pun, termasuk pembuat proyek.
- Aturan konsensus hanya ditulis di satu tempat (crate `consensus`).
- Siapa pun harus bisa membangun ulang program dan mendapat hasil identik (reproducible build).

## Pembagian peran (DEC-16)
| Peran | Tugas |
|---|---|
| Pencipta | Keputusan pencipta: arah proyek, parameter koin, nama, lisensi, prioritas. Meneruskan satu baris perintah ke agent |
| Quick (penasihat) | Rancangan, spesifikasi, keputusan teknis, kartu task, kode, test, dan dokumen. Menulis langsung di repo, lalu menulis skrip pemeriksaan di `ai_advisory_hub/langkah/` |
| Agent (pelaksana) | Menjalankan skrip langkah persis: format, lint, test, dan commit. Tidak mengedit file dan tidak memperbaiki error sendiri |

Untuk agent, bagian "Format laporan akhir" di bawah diganti oleh skrip langkah: skrip menulis output lengkap ke folder
langkah, dan agent hanya menulis baris terakhirnya.

## Parameter protokol (sumber kebenaran)
JANGAN mengubah angka di tabel ini. Kalau menurut Anda ada yang salah, berhenti dan laporkan.

| Parameter | Nilai |
|---|---|
| Satuan | 1 HSIS = 10^18 quanta. Semua jumlah disimpan sebagai `u128` dalam quanta |
| Waktu blok target | 120 detik |
| Blok per tahun (konstanta) | 262_800 |
| Pasokan utama `S` (hasil tambang) | 43_800_000 HSIS = 43_800_000 × 10^18 quanta |
| Alokasi genesis pencipta `G` | 4_380_000 HSIS, hanya di blok genesis, DI LUAR `S`, tanpa kunci waktu |
| Konstanta paruh `D` | 2_274_842 |
| Hadiah ekor `T` | tepat 1 HSIS = 10^18 quanta per blok |
| `M` | total quanta hasil tambang yang sudah keluar SEBELUM blok yang sedang dihitung (`G` tidak dihitung) |
| Hadiah dasar | `base(M) = max((S − M) / D, T)`, pembagian bilangan bulat dibulatkan ke bawah. Jika `M ≥ S`, `(S − M)` dianggap 0 |
| Slow start | untuk tinggi `h` dengan `1 ≤ h < 10_080`: `subsidy = base(M) × h / 10_080` (bilangan bulat). Untuk `h ≥ 10_080`: `subsidy = base(M)` |
| Genesis (`h = 0`) | subsidy tambang 0. Hanya berisi alokasi `G` |
| Biaya transaksi | seluruhnya untuk penambang, tanpa pembakaran |
| Kematangan coinbase | 100 blok |
| Ukuran blok maksimum | 100_000 byte |
| Hash umum | BLAKE3 |
| Tanda tangan | Schnorr BIP340 di kurva secp256k1. MuSig2 untuk multi-tanda tangan |
| PoW | varian RandomX (hanya salt Argon2 yang diganti). Kunci diganti setiap 2_048 blok, jeda 64 blok |
| Penyesuaian kesulitan | ASERT per blok, waktu paruh 720 blok |
| Cap waktu blok | harus > median 11 blok terakhir dan ≤ waktu lokal node + 3_600 detik |
| Pemilihan chain | chain sah dengan kerja kumulatif terbanyak. TANPA batas kedalaman reorg. Tanpa checkpoint |
| Jaringan P2P | rust-libp2p (bukan bagian konsensus) |

Nilai berikut baru ditetapkan saat testnet. JANGAN dikarang. Tulis sebagai konstanta
bertanda `// TODO(param): <nama>` dengan nilai placeholder yang jelas tidak valid:
salt RandomX, kesulitan awal, alamat penerima alokasi genesis, hash blok Bitcoin jangkar
beserta tingginya, magic bytes/ID jaringan, dan nomor SLIP-44 mainnet untuk jalur turunan wallet.
Format alamat sudah ditetapkan (bech32m, DEC-11, Spesifikasi Format Bagian 8).

## Struktur repo dan peta modul
Struktur ini TETAP. Jangan menambah, menghapus, atau mengganti nama crate, aplikasi, atau folder
tingkat atas tanpa izin. Perubahan struktur dicatat sebagai keputusan (DEC) di `docs/decisions/`.

```
homeostasis_lab/
├── AGENTS.md  .agent/        # instruksi agent (jangan diubah agent)
├── ai_advisory_hub/          # ruang kerja Quick: status, antrean, langkah (lokal, tidak di-commit, DEC-16)
├── Cargo.toml                # workspace: members, dependencies, lints
├── Cargo.lock  rust-toolchain.toml  clippy.toml  Makefile
├── crates/                   # MOD-01..MOD-10: library inti
├── apps/                     # MOD-11..MOD-16: aplikasi
├── tests/
│   ├── integration/          # MOD-17: uji lintas crate
│   └── vectors/              # vektor uji konsensus (data)
├── scripts/                  # skrip pemeriksaan (check-forbidden)
├── docs/                     # README, system-map, modules, features, tasks, decisions, glossary, reference
└── .github/workflows/        # CI (asumsi GitHub)
```

| ID | Path | Paket | Fungsi | Bergantung pada |
|---|---|---|---|---|
| MOD-01 | crates/primitives | homeostasis-primitives | Tipe dasar: jumlah (u128 quanta), tinggi blok, hash, parameter chain, struktur blok dan transaksi | — |
| MOD-02 | crates/crypto | homeostasis-crypto | BLAKE3, Schnorr BIP340, MuSig2 | MOD-01 |
| MOD-03 | crates/pow | homeostasis-pow | Proof of Work: antarmuka dan PoW dev regtest (DEC-09); PoW mainnet menunggu DEC-01 | MOD-01, 02 (DEC-14) |
| MOD-04 | crates/consensus | homeostasis-consensus | Satu-satunya tempat aturan konsensus: pasokan, validasi, ASERT, cap waktu | MOD-01, 02, 03 |
| MOD-05 | crates/storage | homeostasis-storage | Penyimpanan blok, set UTXO, pruning | MOD-01 |
| MOD-06 | crates/network | homeostasis-network | P2P (rust-libp2p) | MOD-01 |
| MOD-07 | crates/mempool | homeostasis-mempool | Antrean transaksi | MOD-01, 04 |
| MOD-08 | crates/rpc | homeostasis-rpc | Tipe dan klien/server JSON-RPC | MOD-01 |
| MOD-09 | crates/wallet-core | homeostasis-wallet-core | Kunci, penyusunan dan penandatanganan transaksi | MOD-01, 02 |
| MOD-10 | crates/merchant-core | homeostasis-merchant-core | Tagihan watch-only, status pembayaran | MOD-01, 09 |
| MOD-11 | apps/node | homeostasis-node (bin) | Daemon node | MOD-01, 02, 04, 05, 06, 07, 08 (DEC-19) |
| MOD-12 | apps/cli | homeostasis-cli (bin) | Alat baris perintah ke node | MOD-01, 08, 09 (DEC-21) |
| MOD-13 | apps/miner | homeostasis-miner (bin) | Penambang CPU | MOD-01, 03, 08 |
| MOD-14 | apps/explorer | homeostasis-explorer (bin) | Indexer dan API explorer (web menyusul) | MOD-01, 05, 08 |
| MOD-15 | apps/wallet | (belum dibuat) | Wallet Tauri desktop dan mobile | MOD-08, 09 |
| MOD-16 | apps/merchant | (belum dibuat) | Aplikasi pelaku usaha, Tauri | MOD-08, 10 |
| MOD-17 | tests/integration | homeostasis-integration-tests | Uji integrasi lintas crate | semua crate inti; dev: MOD-11, MOD-13 (DEC-20) |

Aturan arah dependensi:
- Crate di `crates/` tidak boleh bergantung pada apa pun di `apps/` atau `tests/`.
- Dependensi antar modul hanya boleh mengikuti kolom "Bergantung pada". Dependensi baru antar modul wajib minta izin dan dicatat sebagai DEC.
- MOD-15 dan MOD-16 baru diinisialisasi sebagai proyek Tauri lewat task tersendiri (butuh Node.js dan dependensi Tauri).

## Perintah
```bash
make fmt         # cargo fmt --all
make fmt-check   # cargo fmt --all -- --check
make lint        # cargo clippy --workspace --all-targets -- -D warnings
make test        # cargo test --workspace
make test-slow   # cargo test --workspace --release -- --ignored
make check-forbidden  # gagal jika ada f32/f64 atau kata kunci unsafe di kode proyek
make check-all   # fmt-check + lint + check-forbidden + test
```

## Larangan mutlak: `unsafe` dan floating point
Berlaku untuk SEMUA kode milik proyek ini: setiap crate di `crates/`, setiap aplikasi di `apps/`,
`tests/`, build script, contoh, dan benchmark. Tidak ada pengecualian, termasuk untuk test.

1. **Tanpa `unsafe`.** Dilarang menulis blok, fungsi, trait, atau `impl` yang `unsafe`.
   - `[workspace.lints.rust] unsafe_code = "forbid"`, dan setiap `lib.rs`/`main.rs` diawali `#![forbid(unsafe_code)]`.
   - `forbid` tidak boleh diturunkan dengan `#[allow]` di mana pun.
   - Jika suatu tugas tampaknya butuh `unsafe` (misalnya FFI ke C/C++), BERHENTI dan laporkan. Jangan mencari jalan pintas.
2. **Tanpa floating point.** Dilarang memakai tipe `f32`/`f64`, literal desimal (`1.5`, `2e3`),
   konversi `as f32`/`as f64`, atau fungsi matematika float.
   - Persentase, rasio, dan statistik dihitung dengan bilangan bulat, misalnya basis poin
     (1/10_000) atau pasangan pembilang/penyebut.
   - Jumlah koin selalu `u128` quanta. Ditampilkan sebagai teks lewat `Amount::to_hsis_string`.
   - Lint wajib di `[workspace.lints.clippy]`: `float_arithmetic`, `float_cmp`, `float_cmp_const`,
     `lossy_float_literal`, `cast_precision_loss`, `imprecise_flops`, `suboptimal_flops`, semuanya `"deny"`.
   - UI berbasis TypeScript (tugas berikutnya): jumlah koin tidak boleh disimpan di tipe `number`.
     Pakai `string` atau `bigint`.
3. **Pemeriksaan otomatis (`make check-forbidden`):** gagal jika `grep -rnwE 'f32|f64'` menemukan
   kecocokan di file `*.rs` milik proyek, atau jika kata kunci `unsafe` muncul di luar komentar.
   Target ini termasuk di `make check-all`.
4. **Catatan dependensi pihak ketiga:** larangan ini mengikat kode yang kita tulis. Pustaka standar
   Rust dan banyak dependensi berisi `unsafe` atau float di dalamnya. Menambah dependensi tetap wajib
   minta izin dulu, dan jangan memilih dependensi yang memaksa kode kita memakai float atau `unsafe`.

## Aturan wajib untuk crate konsensus (`primitives`, `crypto`, `pow`, `consensus`, `storage`)
1. **Determinisme.** Selain larangan float di atas, dilarang memakai waktu sistem, angka acak, atau urutan
   iterasi `HashMap`/`HashSet` yang memengaruhi hasil. Gunakan `BTreeMap` atau `Vec` terurut.
2. **Aritmetika aman.** Semua operasi pada jumlah dan tinggi blok memakai `checked_*`, atau
   `saturating_*` jika memang disengaja dan dijelaskan di komentar. Di `lib.rs` crate konsensus aktifkan:
   `#![deny(clippy::arithmetic_side_effects, clippy::float_arithmetic, clippy::indexing_slicing)]`
3. **Tanpa panic di kode produksi.** Dilarang `unwrap`, `expect`, `panic!`, atau indexing yang bisa
   panic. Kembalikan `Result` dengan tipe error dari `thiserror`.
4. **Tanpa `unsafe`**, tanpa pengecualian (lihat larangan mutlak di atas). Di crate `pow`, hanya PoW dev regtest (TSK-M05, DEC-09) yang boleh dikerjakan. PoW mainnet menunggu DEC-01; jangan mulai mengerjakannya.
5. **Encoding kanonis.** Satu nilai hanya boleh punya satu representasi byte.
6. Jangan menulis aturan konsensus di luar crate `consensus`.

## Aturan test
- Unit test diletakkan di `src/<modul>/tests.rs` dan dimuat dengan `#[cfg(test)] mod tests;`. Kode dan test tetap di file terpisah.
- Integration test diletakkan di `<crate>/tests/`.
- Test lambat (misalnya simulasi bertahun-tahun) diberi `#[ignore]` dan dijalankan lewat `make test-slow`.
- Setiap rumus konsensus wajib punya test nilai pasti (vektor uji) dan property test (`proptest`).
- DILARANG menonaktifkan, melewati, atau melemahkan test supaya lolos. Kalau test benar tapi kode tidak lolos, perbaiki kodenya. Kalau test-nya yang Anda curigai salah, berhenti dan laporkan.

## Dependensi
- Menambah dependensi baru wajib minta izin dulu, kecuali yang tercantum di prompt tugas.
- Sudah diizinkan: `thiserror` dan `proptest` (khusus dev-dependency).
- Semua versi diatur di `[workspace.dependencies]`. File `Cargo.lock` wajib di-commit.

## Git
- Format commit mengikuti Conventional Commits (`feat:`, `fix:`, `test:`, `chore:`, `docs:`).
- Satu tugas sama dengan satu commit, atau beberapa commit kecil yang saling berkaitan.
- Jangan `git push`, jangan force, dan jangan mengubah riwayat commit.
- Jangan membuat, menghapus, atau mengubah repo remote maupun pengaturan `git remote`.
- Jangan memakai GitHub CLI (`gh`), akun plugin, token, atau akun online milik pengguna dalam bentuk apa pun tanpa izin tertulis di prompt tugas.
- Jangan mengubah identitas commit atau konfigurasi Git (`git config`, baik lokal maupun global).
  Jika identitas belum diset, BERHENTI dan minta pengguna mengaturnya.

Alasan: akun, remote, dan identitas commit adalah keputusan pengguna, bukan agent.
Identitas repo dan commit yang dipakai (`ratuaurion-dev`) sudah diputuskan pengguna (DEC-08).
Jangan menggantinya, dan jangan menambah akun atau remote lain.

## Berhenti dan tanya bila
- Ada parameter protokol yang tampak bertentangan atau belum ditentukan.
- Tugas butuh dependensi baru, `sudo`, atau perubahan di luar ruang lingkup.
- Ada keraguan apakah suatu perilaku termasuk aturan konsensus.
- Perlu menjalankan perintah destruktif (menghapus file di luar yang dibuat tugas ini, reset, force).

## Definition of done
- `make check-all` lolos tanpa warning.
- `make test-slow` lolos, jika tugas menambah test lambat.
- Tidak ada `TODO` baru selain `TODO(param)` yang memang disengaja.
- Laporan akhir sudah ditulis dengan format di bawah.

## Format laporan akhir
1. Ringkasan perubahan dan alasannya
2. Daftar file yang dibuat atau diubah
3. Output lengkap `make check-all` (dan `make test-slow` jika relevan)
4. Asumsi yang diambil, risiko, dan pertanyaan terbuka
