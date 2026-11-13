Wallet desktop Zevrigo {{VERSI}} untuk Windows (x64), dibangun GitHub Actions dari tag `{{TAG}}` (commit `{{COMMIT}}`)
dengan Cargo.lock dan package-lock.json tetap.

**Jaringan demo regtest untuk belajar.** Koin demo tidak bernilai, tidak bisa dijual, dan bisa hilang saat jaringan
di-reset. Ini bukan penawaran investasi.

| Berkas | SHA-256 |
|---|---|
| `{{BERKAS}}` | `{{SHA256}}` |

Periksa unduhan di PowerShell: `Get-FileHash .\{{BERKAS}} -Algorithm SHA256`. Hasilnya harus sama dengan baris di atas dan
dengan `SHA256SUMS.txt`.

- **Belum ditandatangani.** Windows SmartScreen menampilkan peringatan ("Windows protected your PC"); pilih "More info"
  lalu "Run anyway" hanya jika SHA-256 cocok. Lihat [Code signing policy](https://github.com/ratuaurion-dev/zevrigo/blob/main/docs/code-signing-policy.md).
- **Privasi:** untuk regtest, wallet memakai gerbang demo publik `https://zevrigo-rpc.ratuaurion.store`, yang bisa melihat
  alamat yang dicek wallet. Frasa, password, dan kunci rahasia tidak pernah dikirim. Ganti dengan node sendiri di
  Pengaturan. Teks lengkap tampil saat instalasi.
- Butuh Microsoft Edge WebView2 (sudah ada di Windows 10 dan 11 yang diperbarui).
