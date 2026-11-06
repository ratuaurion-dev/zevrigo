# TSK-M06 · PoW RandomX untuk testnet dan mainnet · 🟡 bagian C · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-03 (bagian A); MOD-04, MOD-11, MOD-13 (bagian B dan C) · **Untuk:** FTR-06 testnet publik
- **Bergantung pada:** DEC-01, DEC-10 bagian 10, DEC-14, DEC-15 · **Memblokir:** TSK-F06
- **Estimasi:** A S, B L, C M · **Ditulis:** Quick, 5 November 2026

## Tujuan
PoW testnet dan mainnet sesuai Konsep Bagian 5.10 (dikunci): RandomX resmi dengan salt Argon2 sendiri, kunci berganti
setiap 2_048 blok dengan jeda 64 blok. Dikerjakan bertahap, supaya rantai build C++ (cmake, compiler, Docker) terbukti
lebih dulu dengan vektor resmi RandomX.

## Bagian A: pustaka di MOD-03 (🟢 `0285052`, Q99c)
| Bagian | Isi |
|---|---|
| Dependensi | `randomx-rs` 1.6.0 (DEC-10 bagian 10), hanya MOD-03; `opt-level = 3` di profil dev; `Dockerfile` memasang `cmake` dan `g++` |
| `key_height(height)` | Tinggi blok kunci: 0 sampai tinggi 2_112; sesudahnya `(h − 65) & !(2_048 − 1)` (`RANDOMX_KEY_INTERVAL`, `RANDOMX_KEY_LAG` dari MOD-01) |
| `RandomxVerifier` | Mode ringan untuk satu kunci (`block_id` blok kunci, 32 byte): `new`, `key`, `hash(header)` atas 96 byte header, `verify(header, target)` dengan `hash ≤ target` (big-endian, sama dengan PoW regtest). Flag anjuran pustaka (JIT, AES perangkat keras). Tidak `Send` |
| `RandomxError` | `Library`, `HashLength`, `AboveTarget { pow_hash, target }` |
| Salt | Masih resmi (`RandomX\x03`), agar vektor resmi membuktikan build benar |

Test (workspace 391 → 395): `official_vectors_light_mode` (vektor resmi 1a–1f), `key_height_follows_interval_and_lag`,
`header_hash_and_target_rule`, `empty_key_and_input_are_rejected`.

## Bagian B: konsensus dan node (🟢 `7df31b4`, Q100)
| Bagian | Isi |
|---|---|
| MOD-03 | `RandomxService`: satu utas pemilik `RandomxVerifier` untuk `SERVICE_KEYS` = 2 kunci terakhir (aktif dan sebelumnya), handle `Send + Sync`, `hash(key, header)`; `RandomxError::Service` |
| MOD-04 | `RandomxHasher` (trait), `randomx_hasher()` (layanan MOD-03), `randomx_key_height`; `check_header_pow` dan `check_block_pow` (V2–V4 dengan kunci dan penghitung); `BlockError::PowHashFailed`. `check_header_context_free`/`check_block_context_free` = tanpa penghitung (regtest; testnet/mainnet `PowUnavailable`) |
| MOD-11 | `Randomx`, `Node::open_with_pow`; `open_with_limits` memulai layanan untuk testnet/mainnet. Kunci = leluhur di tinggi kunci dari induk (`ChainState::ancestor`: chain aktif, blok cabang, indeks header; paling banyak 2_112 langkah). Testnet/mainnet: V2–V4 sesudah induk dikenal. Presync tahap hitung mencatat ID dua header terakhir di tinggi kelipatan 2_048. `PowHashFailed` → galat internal |

Tanpa dependensi baru: MOD-11 memakai RandomX lewat MOD-04 (tabel dependensi tetap). Test (395 → 402):
MOD-03 `service_matches_verifier_and_keeps_recent_keys`; MOD-04 `key_height_is_the_pow_rule`,
`testnet_and_mainnet_check_randomx_hash_with_key`, `regtest_ignores_key_and_hasher`, `service_hash_decides_testnet_pow`;
MOD-11 `randomx_key_comes_from_the_blocks_own_branch` (penghitung uji BLAKE3(kunci ‖ header), chain A 2_120 blok dan cabang
B dari tinggi 2_040: header lewat indeks header, blok cabang, kunci salah ditolak V3, reorg, presync melewati tinggi 2_048
dan 2_113) dan `testnet_node_checks_real_randomx` (layanan sungguhan). Node testnet sungguhan baru bisa dibuka sesudah
genesis testnet (bagian C).

Batas yang dicatat: header diperiksa RandomX sambil state node terkunci (mode ringan, beberapa milidetik per header), jadi
sinkronisasi header testnet lebih lambat daripada regtest; hash dihitung berurutan di satu utas.

## Bagian C: salt sendiri, parameter testnet, penambang (paket parameter disetujui pencipta 5 November 2026)
- **C1 (🟢 `6f18f4f`, Q101 hitung, Q102 commit):** `vendor/randomx-rs` = `randomx-rs` 1.6.0 dari crates.io tanpa `Cargo.lock` paket, dua
  perubahan: `RANDOMX_ARGON_SALT` = `"RandomZVRG\x01"` dan `[profile.release]` paket dihapus (vendor/README.md). Dipakai lewat
  `[patch.crates-io]`, `exclude = ["vendor"]`; Cargo.lock hanya kehilangan `source` dan `checksum` randomx-rs. Q101: isi vendor
  dibandingkan `diff -r` dengan sumber asli; RandomX C++ asli dan vendor dibangun cmake terpisah; program uji
  `vektor_rx.cpp` (flag bawaan dan JIT harus sama) menghasilkan vektor resmi 1a–1f persis dari sumber asli, lalu vektor ZVRG
  dari vendor. Test `zvrg_vectors_light_mode` (pengganti `official_vectors_light_mode`) dan vektor header h1.
- **C2:** testnet `pow_limit` `0x1f369d03` (sekitar 10 hash/detik), target genesis `0x1e8bcf64` (sekitar 1_000 hash/detik,
  120_000 hash per blok); genesis testnet berbentuk genesis regtest dengan `extra` = ASCII `zevrigo testnet 1` diisi nol
  sampai 32 byte (tanpa jangkar Bitcoin), alokasi `G` ke kunci NUMS BIP341
  `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0` (tidak bisa dibelanjakan), cap waktu = saat peluncuran
  (`TODO(param)` `TESTNET_GENESIS_TIMESTAMP`, sementara `0`, ditetapkan di TSK-F06; mengubahnya mengubah `network_id`).
  Q103: MOD-04 `ChainParams::testnet`/`for_network`, `testnet_genesis`, `testnet_network_id`, `known_genesis`,
  `ParamsError::Unavailable`; MOD-11 genesis dipilih dari `network_id`, daemon `--testnet`; MOD-14 `--testnet`. Test 402 → 407.
- **C2:** 🟢 `6c649d1` (Q103).
- **C3a (Q104):** MOD-03 `RandomxMiner` + `RandomxMode` (ringan/cepat); MOD-08 field `pow_key` di `BlockTemplate` (Spesifikasi
  RPC 3.4); MOD-11 mengisi `pow_key`; MOD-13 `--testnet`, `--light`, `MinerPow`, `connect_with_pow`, `check_network`,
  `RANDOMX_ATTEMPTS_PER_TEMPLATE` 2_000, VM disimpan per kunci. Test 407 → 410, ditambah test mode cepat `#[ignore]`
  (dataset 2 GiB) yang dijalankan skrip gerbang; skrip juga menambang 2 blok testnet sungguhan (node + miner mode cepat).
  Multi-utas ditunda: `randomx-rs` tidak menandai dataset `Send`/`Sync`.
- **C3a:** 🟢 `495b200` (Q104): dataset mode cepat sekitar 55 detik, 2 blok testnet sungguhan dalam 285 detik.
- **C3b (Q105):** MOD-12 `--regtest`/`--testnet` (`Config.network`, `ArgError::NetworkConflict`, `default_rpc_url(network)`),
  `wallet_path(network, datadir)`, `network_name`, `WalletFile::network`, `load_network` + `WalletFileError::WrongNetwork`,
  fungsi wallet dan `Cli::connect` menerima jaringan, `CliError::WrongNetwork { expected, actual }`. Test 410 → 413.

## Batas yang ditulis jujur
- Varian hanya penghalang kecil: CPU dan server sewaan tetap bisa berpindah (Konsep P13).
- `randomx-rs` tidak memeriksa VM yang gagal dibuat (misalnya JIT ditolak sistem): node berhenti, bukan menerima hash salah.
- Build butuh compiler C++; versinya dikunci di TSK-F10.
