# TSK-M61 · Wallet desktop versi demo dan workflow rilis · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-15 (`apps/wallet`), repo (CI, dokumen) · **Untuk:** FTR-11 tahap 1 (DEC-34, DEC-07)
- **Bergantung pada:** TSK-M60 · **Memblokir:** TSK-M59 bagian 1, permohonan SignPath Foundation
- **Estimasi:** M · **Ditulis:** Quick, 13 November 2026

## Tujuan
Orang awam dengan laptop Windows bisa memasang wallet dan langsung tersambung ke jaringan demo, tanpa menjalankan node.
Installer dibangun dari repo publik secara terverifikasi, dengan kebijakan privasi, sebagai syarat permohonan SignPath.

## Isi
| Bagian | Isi |
|---|---|
| Rust (`commands`) | `DEMO_RPC_URL` = `https://zevrigo-rpc.ratuaurion.store`; `wallet_default_rpc_url`: regtest gerbang demo, testnet/mainnet alamat bawaan CLI (node lokal); `wallet_info.defaultRpcUrl` memakainya. Test baru `regtest_defaults_to_demo_gateway_and_testnet_to_local_node` |
| Antarmuka | Jaringan bawaan `regtest` (pengaturan lama tetap); pita "REGTEST · jaringan demo …" di bawah bilah status; catatan privasi gerbang di kartu Node Pengaturan; pesan offline khusus regtest; teks kaki 0.2. Teks dua bahasa |
| Installer | `tauri.conf.json`: versi 0.2.0, `bundle.licenseFile` `PRIVASI-DAN-LISENSI.txt` (halaman lisensi NSIS, tauri-bundler 2.10.2), `homepage`, `copyright` |
| Privasi | `apps/wallet/src-tauri/PRIVASI-DAN-LISENSI.txt` (Indonesia, Inggris): data yang dikirim ke node, gerbang demo, Cloudflare, cara mematikan, lisensi |
| Workflow | `.github/workflows/rilis-wallet.yml`: tag `wallet-v*` atau manual, `windows-latest`; versi tag wajib sama dengan `tauri.conf.json`; toolchain dari `rust-toolchain.toml`; `npm ci`, `npm run check`, `tauri build --locked`; nama tetap `zevrigo-wallet-<versi>-x64-setup.exe` + `SHA256SUMS.txt`; artefak 30 hari; rilis **draf** prerelease dengan catatan `apps/wallet/rilis-catatan.md` (hanya untuk tag) |
| Dokumen | `docs/code-signing-policy.md` (status, yang ditandatangani, peran, privasi, memeriksa unduhan); README akar (wallet demo, Code signing policy); README wallet; MOD-15; DEC-07; docs/tasks |

## Gerbang (Q129)
`make check-all` paling sedikit 452 test; `make wallet-check`; workflow dibaca sebagai YAML dengan pemicu tag `wallet-v*`,
`windows-latest`, `--locked`, rilis `--draft`; build Windows dari salinan working tree di `C:\zevrigo-build` menghasilkan
`Zevrigo Wallet_0.2.0_x64-setup.exe` dengan halaman lisensi (`license_file` di folder kerja NSIS); `zevrigo-cli.exe` hasil
build Windows menjawab `info` lewat `https://zevrigo-rpc.ratuaurion.store` (TLS di Windows); commit tanpa push. Uji
pencipta sesudahnya: pasang installer, halaman privasi tampil, wallet regtest tersambung ke gerbang.

## Batas
- Workflow GitHub Actions baru teruji saat pencipta mem-push tag pertama (`wallet-v0.2.0`); build Windows lokal Q129 memakai
  perintah yang sama.
- Belum ditandatangani: SmartScreen tetap memperingatkan sampai SignPath disetujui dan reputasi terbentuk.
- Action GitHub dirujuk dengan tag versi (`@v4`), seperti `ci.yml`; penguncian per commit dikerjakan bersama integrasi
  SignPath.
- Wallet belum mengenal output jenis 1 (koin terkunci): tahap 2 (TSK-M50).

## Definition of done
- [x] Gerbang Q129 lolos
- [x] MOD-15, README, README wallet, DEC-07, Code signing policy, docs/tasks
