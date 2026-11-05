# TSK-M06 · PoW RandomX untuk testnet dan mainnet · 🟡 bagian A · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-03 (bagian A); MOD-04, MOD-11, MOD-13 (bagian B dan C) · **Untuk:** FTR-06 testnet publik
- **Bergantung pada:** DEC-01, DEC-10 bagian 10, DEC-14, DEC-15 · **Memblokir:** TSK-F06
- **Estimasi:** A S, B L, C M · **Ditulis:** Quick, 5 November 2026

## Tujuan
PoW testnet dan mainnet sesuai Konsep Bagian 5.10 (dikunci): RandomX resmi dengan salt Argon2 sendiri, kunci berganti
setiap 2_048 blok dengan jeda 64 blok. Dikerjakan bertahap, supaya rantai build C++ (cmake, compiler, Docker) terbukti
lebih dulu dengan vektor resmi RandomX.

## Bagian A: pustaka di MOD-03
| Bagian | Isi |
|---|---|
| Dependensi | `randomx-rs` 1.6.0 (DEC-10 bagian 10), hanya MOD-03; `opt-level = 3` di profil dev; `Dockerfile` memasang `cmake` dan `g++` |
| `key_height(height)` | Tinggi blok kunci: 0 sampai tinggi 2_112; sesudahnya `(h − 65) & !(2_048 − 1)` (`RANDOMX_KEY_INTERVAL`, `RANDOMX_KEY_LAG` dari MOD-01) |
| `RandomxVerifier` | Mode ringan untuk satu kunci (`block_id` blok kunci, 32 byte): `new`, `key`, `hash(header)` atas 96 byte header, `verify(header, target)` dengan `hash ≤ target` (big-endian, sama dengan PoW regtest). Flag anjuran pustaka (JIT, AES perangkat keras). Tidak `Send` |
| `RandomxError` | `Library`, `HashLength`, `AboveTarget { pow_hash, target }` |
| Salt | Masih resmi (`RandomX\x03`), agar vektor resmi membuktikan build benar |

Test (workspace 391 → 395): `official_vectors_light_mode` (vektor resmi 1a–1f), `key_height_follows_interval_and_lag`,
`header_hash_and_target_rule`, `empty_key_and_input_are_rejected`.

## Bagian B: konsensus dan node
- MOD-04: V3 testnet dan mainnet memakai RandomX dengan kunci `block_id` di `key_height(h)` (sebelum pergantian pertama:
  `block_id` genesis), diberikan pemanggil. PoW regtest tetap ditolak di testnet dan mainnet, dan RandomX tidak dipakai di
  regtest (DEC-09).
- MOD-11: satu utas pemeriksa yang memiliki `RandomxVerifier` (tipe tidak `Send`); cache untuk kunci aktif dan kunci
  sebelumnya (reorg melewati batas kunci); sinkronisasi header dan presync membawa `block_id` di tinggi kunci.
- Uji: chain testnet buatan dengan target mudah, pergantian kunci di tinggi 2_113, reorg melewati batas kunci.

## Bagian C: salt sendiri, penambang, parameter testnet
- Kode `randomx-rs` dan RandomX disalin ke repo (Konsep Bagian 5.12); satu baris `RANDOMX_ARGON_SALT` diganti nilai chain
  ini (`TODO(param)`, diusulkan bersama parameter testnet). Vektor baru diuji silang dengan program uji resmi RandomX yang
  dibangun dengan salt yang sama.
- MOD-13: mode cepat (dataset sekitar 2 GB) dan multi-utas.
- `pow_limit`, target awal, dan genesis testnet.

## Batas yang ditulis jujur
- Varian hanya penghalang kecil: CPU dan server sewaan tetap bisa berpindah (Konsep P13).
- `randomx-rs` tidak memeriksa VM yang gagal dibuat (misalnya JIT ditolak sistem): node berhenti, bukan menerima hash salah.
- Build butuh compiler C++; versinya dikunci di TSK-F10.
