# Spesifikasi Wallet Zevrigo Coin (ZVRG) v0.1: frasa BIP39, turunan BIP32, alamat, pemilihan koin, pemindaian saldo

> Diperbarui: 2 November 2026 · Pemilik: pencipta · Ditulis: Quick · Task: TSK-S03 · Status: 🟢 v0.1
> Lokasi di repo: `docs/reference/spesifikasi-wallet-v0.1.md`

| Status | Ringkasan |
|---|---|
| 🟢 Dikunci | Frasa BIP39 standar (DEC-22), turunan BIP32 dengan jalur `m/44'/1'/0'/k/i` untuk regtest dan testnet, kunci x-only, kunci akun publik untuk watch-only, alamat bech32m (DEC-11), pemilihan koin, pemindaian saldo, file wallet CLI regtest dan testnet |
| ⏳ Ditunda | Nomor SLIP-44 mainnet (`TODO(param)`), passphrase non-ASCII, keychain sistem operasi (wallet desktop FTR-07), filter blok kompak untuk wallet ringan, output jenis 1 di wallet |
| ➡️ Langkah berikutnya | TSK-M15 (frasa, turunan, alamat di MOD-09), TSK-M16 (penyusunan dan tanda tangan), TSK-M19 (CLI, DEC-21), lalu TSK-F03 |

Wallet **bukan konsensus**. Chain hanya menyimpan `kind` dan `payload` output; semua isi dokumen ini bisa diubah lewat versi
baru tanpa memecah chain.

---

## 1. Frasa pemulihan (BIP39, DEC-22)

| Hal | Nilai |
|---|---|
| Daftar kata | Daftar kata Inggris resmi BIP39, 2048 kata, urut, huruf kecil ASCII. File `english.txt` (13_116 byte, sha256 `2f5eed53a4727b4bf8880d8f3f199efc90e58503646d9ff8eff3a2ed3b24dbda`) disimpan di MOD-09 |
| Wallet baru | 24 kata dari 256 bit entropi. Entropi diambil pemanggil dari sumber acak sistem operasi (CLI v0.1: `/dev/urandom`); MOD-09 hanya menerima byte entropi, sehingga tetap deterministik dan bisa diuji |
| Pemulihan | 12, 15, 18, 21, atau 24 kata. Checksum BIP39 wajib cocok. Masukan dinormalkan: spasi di awal dan akhir dibuang, spasi ganda menjadi satu, huruf besar menjadi kecil |
| Passphrase | Opsional. v0.1 hanya menerima ASCII cetak (`0x20`–`0x7e`): untuk ASCII, normalisasi NFKD BIP39 tidak mengubah apa pun, jadi tetap kompatibel tanpa pustaka Unicode |
| Seed | `PBKDF2-HMAC-SHA512(password = frasa, salt = "mnemonic" ‖ passphrase, 2_048 iterasi, 64 byte)` |

Frasa dan passphrase yang sama di wallet BIP39 lain menghasilkan seed yang sama.

---

## 2. Turunan kunci (BIP32)

| Hal | Nilai |
|---|---|
| Kunci induk | BIP32: `HMAC-SHA512(kunci = "Bitcoin seed", seed)`. Konstanta BIP32 dipertahankan agar vektor resmi BIP32 bisa dipakai |
| Jalur | `m / 44' / coin' / account' / branch / index` |
| `coin` | `1` untuk regtest dan testnet (konvensi SLIP-44 "testnet semua koin"). Mainnet: `TODO(param)` sampai nomor SLIP-44 ZVRG didaftarkan |
| `account` | `0` di v0.1 |
| `branch` | `0` penerimaan, `1` kembalian |
| `index` | `0` sampai `2^31 − 1`, non-hardened |
| Kunci output jenis 0 | x-only = 32 byte koordinat x dari kunci publik BIP32 (byte awalan `02`/`03` dibuang) |
| Tanda tangan | Kunci rahasia BIP32 dipakai langsung. BIP340 menegasikan kunci secara internal jika koordinat y ganjil (sudah dilakukan k256 di MOD-02) |
| Kunci tidak sah | Jika turunan menghasilkan kunci tidak sah (peluang sekitar 2^-127), indeks itu dilewati, sesuai BIP32 |

- `44'` hanya struktur jalur. ZVRG tidak punya skrip, jadi tidak ada arti P2PKH seperti di Bitcoin.
- **Watch-only (MOD-10):** kunci akun publik = kunci publik terkompresi 33 byte ‖ chain code 32 byte di `m/44'/coin'/0'`,
  ditulis hex huruf kecil 130 karakter. Dari kunci ini, `branch/index` diturunkan dengan CKDpub BIP32 tanpa kunci rahasia.
  Format xpub Base58 BIP32 tidak dipakai di v0.1.
- **Lokasi kode:** HMAC-SHA512 dan PBKDF2 di MOD-09 (DEC-10 bagian 4). Penjumlahan skalar dan titik secp256k1 di MOD-02
  (fungsi baru di atas k256 yang sudah ada), sehingga MOD-09 tidak memakai k256 langsung.

---

## 3. Alamat (DEC-11)

Sesuai Spesifikasi Format v0.1 Bagian 8 (sudah final): `bech32m(HRP, [kind] ‖ payload)`, HRP `rzvrg` (regtest), `tzvrg`
(testnet), `zvrg` (mainnet). Alamat penerimaan wallet v0.1 selalu jenis 0 (`…1q…`). Wallet menolak alamat ber-HRP jaringan
lain. Output jenis 1 (kondisi m-dari-n dengan kunci waktu) belum dibuat wallet v0.1.

---

## 4. Vektor uji

Dihitung Quick dengan implementasi referensi Python yang lebih dulu lolos vektor resmi BIP39 (Trezor), BIP32 vektor 1
(`m`, `m/0'`, `m/0'/1`, termasuk CKDpub), dan contoh alamat Spesifikasi Format Bagian 8.

| ID | Masukan | Hasil |
|---|---|---|
| W1 | entropi 32 byte nol | `abandon` ×23 lalu `art` |
| W2 | seed W1, tanpa passphrase | `408b285c123836004f4b8842c89324c1f01382450c0d439af345ba7fc49acf705489c6fc77dbd4e3dc1dd8cc6bc9f043db8ada1e243c4a0eafb290d399480840` |
| W3 | seed W1, passphrase `TREZOR` (vektor resmi) | `bda85446c68413707090a52022edd26a1c9462295029f2e60cd7c4f2bbd3097170af7a4d73245cafa9c3cca8d561a7c3de6f5d4a10be8ed2a5e608d68f92fcc8` |
| W4 | kunci akun publik `m/44'/1'/0'` dari W2 | `03b9385a6fbee0caa6e79407f83b0053e2d864b6c7a8f32ce61c3f495b5dd5d02712ed17c3e72223beb38ea98ec1317581f02458f02b0ecf4abe02c6490eca170a` |
| W5 | `m/44'/1'/0'/0/0` dari W2 | rahasia `11831b777e3fad99962ab773b23bad02812b85d8ac452e6c92434b4d7a79d9a3`, x-only `5a6f28253f35e2ff1ab0b086b8c87f731159661157e5be2ca98188369625dc32`, alamat `rzvrg1qtfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939mseq59qlxg` |
| W6 | `m/44'/1'/0'/0/1` dari W2 | rahasia `e5dde95cb4c071fe5cc1bb06b5d8183e2ae5bec60ae6fb45c53085aaee8aa7df`, x-only `8f33099c2cc3739e24c329d9e2d72ff7ce8bb4f4491b1763f69ca8693ea34806`, alamat `rzvrg1q3uesn8pvcdeeufxr98v794e07l8ghd85fyd3wclknj5xj04rfqrqyeht59` |
| W7 | `m/44'/1'/0'/1/0` dari W2 | rahasia `f6615fe1ba1d1c685ac5eb2697b6b377aafc30a03c4b5aeb14d0bec5a7b26869`, x-only `c71cdd713224479fff24b56db524c2385acf09725605e7440de0c34272e2abf5`, alamat `rzvrg1qcuwd6ufjy3relleyk4km2fxz8pdv7ztj2cz7w3qdurp5yuhz406swzcwe2` |
| W8 | 12 kata `legal winner thank year wave sausage worth useful legal winner thank yellow`, tanpa passphrase, `m/44'/1'/0'/0/0` | x-only `080faab833c4787baabac2eb47f64a2654f92a1b70ca8dc2f09679179ac362a1`, alamat `rzvrg1qpq864wpnc3u8h246ct450aj2ye20j2smwr9gmshsjeu30xkrv2ss80zac3` |
| W9 | W1 dengan passphrase `TREZOR`, `m/44'/1'/0'/0/0` | alamat `rzvrg1q2x064qcn4zk84y9zjtpjt8e8ksjjja7peenap4lcse2d0vmw2xyqm2rpq4` |
| W10 | x-only W5 di testnet | `tzvrg1qtfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939mseq4f4dkt` |

Selain W1–W10, test MOD-09 wajib memakai vektor resmi BIP39 (Trezor, passphrase `TREZOR`) dan BIP32 vektor 1 dan 2.
Kunci rahasia di W5–W7 milik frasa contoh yang publik; jangan dipakai untuk koin bernilai.

---

## 5. Pemilihan koin dan biaya

| Hal | Aturan |
|---|---|
| UTXO yang boleh dipakai | Hasil `scan_utxos` dengan `spendable_at ≤ tinggi ujung + 1` |
| Urutan pilihan | Jumlah menurun, lalu `(txid, index)` naik. Diambil berurutan sampai Σ input ≥ jumlah kirim + biaya. Deterministik |
| Biaya | `tarif × ukuran` encoding kanonis lengkap transaksi bertanda tangan. Tarif bawaan CLI 1 quanta/byte, bisa diubah. Tidak ada biaya minimum di konsensus maupun mempool v0.1 |
| Ukuran | Bisa dihitung pasti sebelum menandatangani, karena tanda tangan BIP340 selalu 64 byte (witness jenis 0 selalu 65 byte) |
| Kembalian | Ke alamat `branch 1` indeks berikutnya yang belum pernah dipakai. Jika kembalian 0, tanpa output kembalian. Kembalian 1 quanta tetap sah (tanpa batas dust) |
| Urutan output | Penerima, lalu kembalian |
| Sisa kecil | Jika sisa setelah biaya tanpa kembalian tidak cukup untuk biaya output kembalian ditambah 1 quanta, sisa itu ikut menjadi biaya (tanpa output kembalian) |
| Batas varint | Panjang varint kembalian `L` dicari dari 1 byte ke atas; dipakai `L` terkecil yang memberi kembalian dengan varint ≤ `L`. Di batas varint biaya bisa `tarif × (ukuran + 1)` agar kembalian tetap unik (TSK-M16) |
| Ukuran maksimum | Transaksi lebih dari 100_000 byte ditolak wallet (sama dengan batas mempool) |

---

## 6. Pemindaian saldo

- Wallet menurunkan alamat `branch 0` dan `branch 1` sampai 20 alamat berturut-turut tanpa UTXO di atas indeks terpakai
  tertinggi (gap limit 20), lalu memanggil `scan_utxos` per 64 payload.
- Keterbatasan v0.1: `scan_utxos` hanya melihat UTXO yang masih ada, jadi riwayat alamat yang sudah kosong tidak terlihat.
  Karena itu indeks terpakai disimpan di file wallet agar alamat tidak dipakai ulang.
- **Transaksi tertunda (TSK-M36):** sesudah `scan_utxos`, wallet memanggil `scan_mempool` untuk semua payload kedua
  cabang sampai batas pemindaian (indeks terpakai + 20), per 64 payload. UTXO wallet yang sedang dibelanjakan di mempool
  dihitung `keluar tertunda` (bukan `bisa dipakai`) dan tidak dipilih saat mengirim; output jenis 0 ke alamat wallet
  (kiriman masuk dan kembalian) dihitung `masuk tertunda`. `saldo setelah tertunda masuk blok` = saldo − keluar tertunda
  + masuk tertunda. Output tertunda belum bisa dibelanjakan (mempool v0.1 tanpa rantai transaksi tertunda).
- Filter blok kompak (gaya BIP158) untuk wallet ringan ditunda ke TSK-F07/F08.

---

## 7. File wallet CLI (v0.1, regtest dan testnet)

| Hal | Nilai |
|---|---|
| Lokasi | `<datadir>/<jaringan>/wallet.json`: `regtest` (bawaan) atau `testnet` (`--testnet`, TSK-M06 bagian C3b). Kunci akun sama untuk kedua jaringan (`coin` 1, Bagian 3); yang berbeda folder, `network` di file, dan HRP alamat. File yang `network`-nya bukan jaringan CLI ditolak |
| Isi versi 2 (DEC-27, semua wallet baru) | `version` (2), `network` (`regtest` atau `testnet`), `account` (kunci akun publik `m/44'/1'/0'`, hex 130 karakter, Bagian 3), `encrypted` (`kdf` `argon2id`, `memory_kib`, `iterations`, `parallelism`, `salt` hex 16 byte, `cipher` `xchacha20poly1305`, `nonce` hex 24 byte, `ciphertext` hex = frasa ternormalkan terenkripsi diikuti tag 16 byte), `next_index` untuk `branch 0` dan `branch 1` |
| Isi versi 1 (lama) | `version` (1), `network`, `mnemonic` tanpa enkripsi, `next_index`. Tetap bisa dipakai; `wallet-encrypt` mengubahnya ke versi 2 dengan indeks yang sama |
| Enkripsi (DEC-27, TSK-M37) | Kunci 32 byte = Argon2id versi 0x13 (password UTF-8, salt; bawaan 64 MiB, 3 putaran, 1 jalur; file dengan memori di atas 1 GiB, putaran di atas 64, atau jalur di atas 16 ditolak). Frasa dienkripsi XChaCha20-Poly1305 dengan AAD = `zevrigo wallet vault v1` ‖ `memory_kib` ‖ `iterations` ‖ `parallelism` (masing-masing `u32` big-endian) ‖ salt. Salt dan nonce dari `/dev/urandom` setiap kali file dikunci |
| Password | Minimal 8 karakter, diketik dua kali saat membuat (`wallet-create`, `wallet-restore`, `wallet-encrypt`), dibaca tanpa tampil di layar. Hanya diminta saat kunci rahasia dibutuhkan: `send` dan `wallet-show-phrase`. `address` dan `balance` memakai `account` tanpa password. Untuk skrip regtest boleh lewat variabel lingkungan `ZEVRIGO_WALLET_PASSWORD` |
| Passphrase | Tidak disimpan. Versi 2: dipakai saat wallet dibuat (menentukan `account`) dan saat `send` (wajib menghasilkan `account` yang sama, jika tidak ditolak); `address` dan `balance` tidak memakainya. Versi 1: diminta lewat opsi setiap kali dibutuhkan |
| Izin file | `0600`, dibuat baru dan tidak menimpa file yang sudah ada |
| Memori | Frasa, seed, kunci rahasia, kunci turunan, memori kerja Argon2id, dan password dihapus dari memori sesudah dipakai (`zeroize`, upaya terbaik) |
| Belum | Keychain sistem operasi dan ganti password (menyusul bersama wallet desktop FTR-07) |

---

## 8. Lokasi kode dan task turunan

| Isi | Modul | Task |
|---|---|---|
| Penjumlahan skalar dan titik secp256k1 untuk turunan BIP32 | MOD-02 | TSK-M15 |
| Daftar kata, frasa, seed, turunan BIP32, kunci akun publik, alamat bech32m (encode dan decode) | MOD-09 | TSK-M15 |
| Pemilihan koin, penyusunan, dan tanda tangan transaksi (sighash v1, DEC-13) | MOD-09 | TSK-M16 |
| Perintah wallet CLI (buat, pulihkan, alamat, saldo, kirim) dan file wallet | MOD-12 (DEC-21) | TSK-M19 |
| Saldo dan pemilihan koin dengan transaksi tertunda (`scan_mempool`) | MOD-08, MOD-11, MOD-12 | TSK-M36 |
| Enkripsi frasa dengan password (modul `vault`), penghapusan rahasia dari memori, file wallet versi 2, `wallet-encrypt`, `wallet-show-phrase` | MOD-09, MOD-12 | TSK-M37 |
| Kirim dan terima ujung ke ujung | MOD-17 | TSK-F03 |

---

## 9. Keputusan

| ID | Keputusan | Status |
|---|---|---|
| DEC-10 bagian 4 | `sha2` 0.10.9, `hmac` 0.12.1, `pbkdf2` 0.12.2 khusus MOD-09 | 🟢 31 Oktober 2026 |
| DEC-21 | MOD-12 CLI bergantung pada MOD-09 wallet-core | 🟢 31 Oktober 2026 |
| DEC-22 | Frasa pemulihan BIP39 standar | 🟢 31 Oktober 2026 |
| DEC-27 | File wallet dikunci password: Argon2id + XChaCha20-Poly1305; password hanya untuk membuat, memulihkan, mengirim, dan menampilkan frasa | 🟢 2 November 2026 |
| DEC-10 bagian 8 | `argon2` 0.5.3, `chacha20poly1305` 0.10.1, `zeroize` 1.9.1 (MOD-09), `rpassword` 7.5.4 dan `zeroize` (MOD-12) | 🟢 2 November 2026 (konsekuensi DEC-27) |
