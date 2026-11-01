# TSK-M15 · Frasa BIP39, turunan BIP32, kunci akun publik, dan alamat bech32m di MOD-09 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-09 (crates/wallet-core), ditambah satu modul kecil di MOD-02 (crates/crypto) · **Untuk:** FTR-03
- **Bergantung pada:** TSK-M04, TSK-S03, DEC-10 bagian 4, DEC-22 · **Memblokir:** TSK-M16, TSK-M19, TSK-M17
- **Estimasi:** M · **Ditulis:** Quick (kode dan test), 1 November 2026

## Tujuan
Bagian kunci dan alamat dari Spesifikasi Wallet v0.1 (Bagian 1 sampai 4): frasa pemulihan BIP39, seed, turunan
BIP32 dengan jalur `m/44'/coin'/0'/branch/index`, kunci akun publik untuk watch-only, dan alamat bech32m (DEC-11).
Penyusunan dan tanda tangan transaksi tetap di TSK-M16.

## API MOD-02 (`tweak`, re-export dari lib.rs)
Kunci publik dalam SEC1 terkompresi 33 byte; skalar 32 byte big-endian. Bukan konsensus.
- `public_key_compressed(&[u8; 32]) -> Result<[u8; 33], TweakError>`: `k·G`.
- `secret_tweak_add(&secret, &tweak) -> Result<[u8; 32], TweakError>`: `(k + t) mod n` (CKDpriv).
- `public_tweak_add(&point, &tweak) -> Result<[u8; 33], TweakError>`: `P + t·G` (CKDpub); tweak 0 sah.
- `is_valid_public_key(&[u8; 33]) -> bool`, `COMPRESSED_PUBLIC_KEY_LEN = 33`.
- `TweakError::{InvalidSecretKey, InvalidPublicKey, InvalidTweak, InvalidResult}`.

## API MOD-09
- `mnemonic`: `Mnemonic::{from_entropy, parse, phrase, entropy, word_count, to_seed}`, `Seed::as_bytes`,
  `MnemonicError::{InvalidEntropyLength, InvalidWordCount, UnknownWord, BadChecksum, InvalidPassphrase, Wordlist}`.
  Daftar kata `src/mnemonic/english.txt` lewat `include_str!` (sha256 `2f5eed53…24dbda`, sama dengan
  `ai_advisory_hub/alat/bip39_english.txt`). Entropi 16 sampai 32 byte; wallet baru 32 byte (`NEW_WALLET_ENTROPY_LEN`).
- `bip32`: `ExtendedPrivKey::{master, derive_child, derive_path, to_public, secret_bytes, chain_code}`,
  `ExtendedPubKey::{from_parts, derive_child, derive_path, public_key, chain_code, x_only, to_account_hex,
  from_account_hex}`, `coin_type`, `account_path`, `account_key`, `hardened`, konstanta `HARDENED`, `PURPOSE`,
  `TESTNET_COIN_TYPE`, `ACCOUNT`, `RECEIVE_BRANCH`, `CHANGE_BRANCH`, `ACCOUNT_KEY_HEX_LEN`.
  `Bip32Error::{InvalidSeedLength, InvalidMasterKey, InvalidChild, HardenedFromPublic, InvalidPublicKey,
  InvalidAccountKey, MainnetCoinTypeUnset}`.
- `address`: `encode_address(Network, OutputKind, &[u8; 32]) -> String`,
  `decode_address(Network, &str) -> Result<(OutputKind, [u8; 32]), AddressError>`,
  `AddressError::{MixedCase, MissingSeparator, WrongNetwork, InvalidLength, InvalidCharacter, InvalidChecksum,
  UnknownKind, InvalidPadding}`.

## Keputusan teknis (Quick, tanpa DEC)
- Penjumlahan skalar dan titik ada di MOD-02 (modul `tweak`, di atas k256 yang sudah ada), sesuai Spesifikasi
  Wallet Bagian 2, sehingga MOD-09 tidak memakai k256 langsung. Operator `+`/`*` k256 dipanggil sebagai fungsi trait
  (`Add::add`, `Mul::mul`) dan `Scalar::add`, agar lint `arithmetic_side_effects` tetap aktif.
- Kunci publik terkompresi hanya diterima dengan awalan `02`/`03`. `sec1` 0.7.3 (di bawah k256) juga menerima awalan
  `05` ("compact", 33 byte) sebagai titik sah; awalan itu ditolak lebih dulu di MOD-02 (ditemukan test
  `invalid_public_keys_are_rejected` di Q72b).
- Nomor SLIP-44 mainnet: `MAINNET_COIN_TYPE: Option<u32> = None` bertanda `TODO(param)`; `coin_type(Mainnet)`
  mengembalikan `MainnetCoinTypeUnset`, sehingga wallet mainnet tidak mungkin dibuat diam-diam dengan nomor karangan.
- Checksum frasa diperiksa dengan menyusun ulang frasa dari entropi lalu membandingkan teksnya.
- Normalisasi frasa memakai `split_whitespace` (tab dan baris baru juga dianggap spasi) dan huruf kecil ASCII.
- Turunan yang menghasilkan kunci tidak sah mengembalikan `InvalidChild { index }`; pemanggil (TSK-M16/M19) melewati
  indeks itu, sesuai BIP32. Tidak ada vektor alami untuk kasus ini (peluang sekitar 2^-127).
- `Debug` untuk `Mnemonic`, `Seed`, dan `ExtendedPrivKey` tidak mencetak isi rahasia. Penghapusan memori rahasia
  (zeroize) belum dilakukan karena butuh dependensi langsung baru; dicatat bersama enkripsi file wallet (sebelum
  TSK-F06).
- Dependensi MOD-09: `sha2`, `hmac`, `pbkdf2` (DEC-10 bagian 4), `thiserror`; dev: `proptest`. Cargo.lock +2 paket
  (`hmac` 0.12.1, `pbkdf2` 0.12.2); `digest` 0.10.7 kini juga memakai `subtle` (fitur `mac`).

## Vektor
- BIP39 resmi Trezor (24 vektor bahasa Inggris, passphrase `TREZOR`): entropi, frasa, seed, dan kunci induk BIP32
  (kunci rahasia dan chain code dari xprv resmi).
- BIP32 resmi vektor 1, 2, dan 3 (semua jalur): chain code, kunci rahasia, kunci publik; CKDpub untuk setiap langkah
  non-hardened.
- W1 sampai W10 Spesifikasi Wallet v0.1 Bagian 4, termasuk W7 lewat watch-only dari kunci akun publik W4.
- Alamat contoh Spesifikasi Format Bagian 8 (empat alamat), vektor bech32m BIP350 `a1lqfn3a` dan `abcdef1l7aum…`.
- Titik `k·G` untuk k = 1, 2, 3, n − 1 dan kunci vektor BIP340 #1, dihitung dengan `alat/wallet_ref.py`.

## Test (crypto 30 → 38, wallet-core 1 → 26, workspace 247 → 280)
- `crates/crypto/src/tweak/tests.rs`: `public_key_of_known_scalars`, `public_key_rejects_invalid_secret`,
  `x_coordinate_matches_bip340_public_key`, `secret_tweak_add_vectors`, `public_tweak_add_vectors`,
  `invalid_public_keys_are_rejected`, `prop_public_tweak_matches_secret_tweak`. `src/tests.rs`:
  `tweak_reexport_is_accessible`.
- `crates/wallet-core/src/mnemonic/tests.rs`: `wordlist_is_official_bip39_english`,
  `trezor_vectors_phrase_seed_and_master_key`, `spec_vectors_w1_to_w3`, `parse_normalizes_spaces_and_case`,
  `parse_rejects_bad_count_unknown_word_and_checksum`, `from_entropy_rejects_bad_length`,
  `seed_rejects_non_printable_passphrase`, `debug_never_prints_secrets`, `prop_entropy_round_trip`.
- `crates/wallet-core/src/bip32/tests.rs`: `bip32_official_vector_1`, `bip32_official_vector_2`,
  `bip32_official_vector_3_keeps_leading_zeros`, `spec_vectors_w4_to_w7_private_and_watch_only`,
  `account_path_and_coin_type_per_network`, `hardened_child_from_public_key_is_rejected`,
  `master_rejects_bad_seed_length`, `account_hex_rejects_malformed_input`, `debug_never_prints_private_key`.
- `crates/wallet-core/src/address/tests.rs`: `format_spec_examples_round_trip`, `bip350_checksum_constant_vectors`,
  `decode_accepts_uppercase_and_rejects_mixed_case`, `decode_rejects_wrong_network_separator_length_and_characters`,
  `decode_rejects_bad_checksum_kind_and_padding`, `prop_round_trip_all_networks_and_kinds`.
- `crates/wallet-core/src/tests.rs`: `module_id_matches`, `spec_wallet_vectors_w5_to_w10_end_to_end`.
