# Code signing policy (kebijakan penandatanganan kode)

> Diperbarui: 13 November 2026 · Pemilik: pencipta · Task: TSK-M61 · Keputusan: DEC-07

## Status
Installer wallet desktop Windows **belum ditandatangani**. Permohonan tanda tangan gratis untuk proyek open source ke
[SignPath Foundation](https://signpath.org) sedang disiapkan (DEC-07). Sampai disetujui, Windows SmartScreen menampilkan
peringatan saat installer dibuka; periksa SHA-256 sebelum menjalankannya (bagian "Memeriksa unduhan").

Setelah permohonan disetujui, halaman ini dan halaman rilis memuat kalimat: Free code signing provided by
[SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

## Yang ditandatangani
- Hanya installer wallet desktop Windows (`zevrigo-wallet-<versi>-x64-setup.exe`) dan program di dalamnya.
- Dibangun GitHub Actions dari tag `wallet-v<versi>` di repo ini (`.github/workflows/rilis-wallet.yml`), dari kode sumber
  repo ini dengan `Cargo.lock` dan `package-lock.json` tetap. Tidak ada biner proyek lain yang ditandatangani dengan
  sertifikat proyek ini.
- Setiap rilis disetujui manual: workflow hanya membuat rilis draf; penyetuju memeriksa isi dan SHA-256, lalu
  menerbitkannya.

## Peran
| Peran | Anggota |
|---|---|
| Penulis (committers) | [ratuaurion-dev](https://github.com/ratuaurion-dev) |
| Peninjau (reviewers) | [ratuaurion-dev](https://github.com/ratuaurion-dev) |
| Penyetuju rilis (approvers) | [ratuaurion-dev](https://github.com/ratuaurion-dev) |

Perubahan dari orang di luar tim (pull request) ditinjau anggota tim sebelum digabung. Anggota tim wajib memakai
autentikasi dua langkah di GitHub dan SignPath.

## Kebijakan privasi
Teks lengkap ditampilkan saat instalasi: [apps/wallet/src-tauri/PRIVASI-DAN-LISENSI.txt](../apps/wallet/src-tauri/PRIVASI-DAN-LISENSI.txt).
Ringkas:
- Tanpa analitik, telemetri, atau laporan galat.
- Wallet hanya menghubungi alamat node di Pengaturan. Untuk regtest, bawaannya gerbang demo publik
  `https://zevrigo-rpc.ratuaurion.store`, yang bisa melihat alamat yang dicek wallet dan alamat IP lewat Cloudflare; gerbang
  tidak menyimpan alamat IP dan tidak mencatat isi permintaan. Untuk testnet, bawaannya node di komputer sendiri.
- Frasa pemulihan, password, dan kunci rahasia tidak pernah dikirim.
- Dimatikan dengan mengisi alamat node sendiri di Pengaturan.

## Memeriksa unduhan
Setiap rilis memuat `SHA256SUMS.txt`. Di PowerShell:

```powershell
Get-FileHash .\zevrigo-wallet-<versi>-x64-setup.exe -Algorithm SHA256
```

Hasilnya harus sama dengan isi `SHA256SUMS.txt` dan tabel di catatan rilis.
