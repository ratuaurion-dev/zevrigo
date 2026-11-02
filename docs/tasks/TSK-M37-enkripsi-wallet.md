# TSK-M37 · Enkripsi file wallet dengan password · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-09 (wallet-core), MOD-12 (cli) · **Untuk:** persiapan testnet (FTR-06), FTR-07
- **Bergantung pada:** TSK-M19, DEC-27, DEC-10 bagian 8 · **Memblokir:** TSK-F06
- **Estimasi:** M · **Ditulis:** Quick (kode dan test), 2 November 2026

## Tujuan
File wallet CLI menyimpan frasa tanpa enkripsi. Sekarang frasa dikunci password (Spesifikasi Wallet v0.1 Bagian 7),
tanpa mempersulit perintah yang tidak butuh kunci rahasia.

## Isi
| Modul | Isi |
|---|---|
| MOD-09 `vault` (baru) | `KdfParams { memory_kib, iterations, parallelism }` (`DEFAULT` 64 MiB/3/1, batas 1 GiB/64/16), `Sealed { kdf, salt, nonce, ciphertext }`, `seal`, `unseal` (hasil `Zeroizing`), `VaultError`; Argon2id 0x13 dengan memori kerja yang dihapus sesudah dipakai, XChaCha20-Poly1305 dengan AAD parameter dan salt |
| MOD-09 | `Drop` yang menghapus `Mnemonic` (entropi dan frasa), `Seed`, dan `ExtendedPrivKey` dari memori |
| MOD-12 `wallet_file` | Versi 2: `account`, `encrypted` (`EncryptedPhrase`), versi 1 tetap terbaca; `new_encrypted`, `is_encrypted`, `account`, `sealed`, `WalletFileError::Encrypted` |
| MOD-12 `commands` | `Protection`, `wallet_create`/`wallet_restore` (passphrase + protection), `wallet_encrypt`, `show_phrase`, `check_new_password` (≥ 8 karakter, sama), `PasswordPrompt`, `terminal_password` (`HOMEOSTASIS_WALLET_PASSWORD` atau `rpassword`), `send` + password dan pemeriksaan passphrase; `run(config, random, password)` |
| MOD-12 `args` | Perintah `wallet-encrypt`, `wallet-show-phrase`; teks bantuan |

## Keputusan teknis (Quick, tanpa DEC)
- **Seri digest 0.10** (`argon2` 0.5.3, `chacha20poly1305` 0.10.1), bukan versi terbaru yang tertulis di usul DEC-27:
  Cargo.lock hanya bertambah `argon2`, `base64ct`, `rpassword`, `rtoolbox`.
- **`hash_password_into_with_memory`** dengan memori milik pemanggil, tanpa fitur `alloc`/`password-hash` argon2
  (fitur itu membawa `password-hash` dan `rand_core`).
- **Passphrase versi 2** dipakai saat wallet dibuat dan saat `send`; `address` dan `balance` memakai kunci akun publik
  di file, sehingga tidak butuh password maupun passphrase.
- **Argon2 dan BLAKE2 `opt-level = 3` di profil dev**, agar uji biner dengan parameter penuh cepat.
- Vektor: RFC 9106 (Argon2id), draft XChaCha A.3.1, serta V1/V2 modul ini dari implementasi rujukan Python yang lolos
  vektor resmi Argon2d/i/id dan XChaCha.

## Test (workspace 369 → 378)
- MOD-09: `argon2id_matches_rfc9106`, `xchacha20poly1305_matches_draft_vector`, `seal_matches_reference_vectors`,
  `unseal_round_trips_and_rejects_wrong_password_or_tampering`, `kdf_params_are_bounded`.
- MOD-12: `encrypted_wallet_rules`, `encrypted_json_format_is_exact_and_round_trips`,
  `from_json_rejects_bad_encrypted_content`, `encrypted_wallet_with_password_from_environment` (biner, parameter
  penuh); test lama memakai wallet versi 2.
- MOD-17: `two_cli_wallets_send_and_receive_over_rpc` memakai wallet terkunci password.
