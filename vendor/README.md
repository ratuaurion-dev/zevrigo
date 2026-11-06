# vendor: salinan kode pihak ketiga (DEC-01, DEC-10 bagian 10)

Folder ini berisi salinan kode yang dibangun bersama proyek, sesuai Konsep Bagian 5.10 dan 5.11 (L5): kode RandomX disalin
ke repositori dengan versi dikunci. Folder ini bukan anggota workspace (`exclude` di `Cargo.toml` akar) dan tidak diperiksa
`make check-forbidden`; kode proyek tetap tanpa `unsafe` dan floating point.

## randomx-rs 1.6.0

| Hal | Isi |
|---|---|
| Sumber | crates.io `randomx-rs` 1.6.0 (The Tari Project, BSD-3-Clause), file `.crate` sha256 `34837f0cf043a56a66e7a07f835969714a27fc9b561d4743620c54a165a7fc8f`, commit `6733ed4d2156254122c7d893a46e92f67b0e60b5`. Di dalamnya `RandomX/`: salinan kode C++ resmi tevador/RandomX (BSD-3-Clause) |
| Dipakai lewat | `[patch.crates-io] randomx-rs = { path = "vendor/randomx-rs" }`, hanya MOD-03 |
| Tidak disalin | `Cargo.lock` milik paket (dependensi dikunci `Cargo.lock` akar) |
| Perubahan 1 (konsensus) | `RandomX/src/configuration.h`: `RANDOMX_ARGON_SALT` `"RandomX\x03"` → `"RandomZVRG\x01"` (11 byte), satu-satunya perubahan algoritma (Konsep Bagian 5.10). Parameter RandomX lain identik dengan resmi |
| Perubahan 2 (build saja) | `Cargo.toml`: bagian `[profile.release] lto = false` dihapus. Cargo mengabaikan profil paket yang bukan akar dan menulis peringatan setiap kali dijalankan |

Setiap perubahan lain di folder ini wajib lewat keputusan (DEC) dan skrip gerbang yang membandingkan isinya dengan sumber
asli.
