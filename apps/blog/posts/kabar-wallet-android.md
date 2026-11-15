---
judul: Kabar wallet Android, APK uji pertama lolos uji
ringkasan: Wallet Zevrigo kini bisa dibangun untuk Android dengan inti yang sama dengan versi desktop. APK uji pertama lolos uji di HP pengembang; APK rilis bertanda tangan sedang disiapkan.
tanggal: 2026-11-15
kategori: Pengumuman
penulis: Badak Besi
---
Wallet desktop Zevrigo dibuat dengan Tauri 2, dan inti wallet-nya ditulis dalam Rust. Pada 14 November 2026, inti yang sama berhasil dibangun untuk Android. Hasilnya adalah APK uji pertama untuk HP arm64.

## Yang sama dengan versi desktop

- Membuat wallet baru dengan 24 kata pemulihan, atau memulihkan wallet dari frasa.
- Menerima, mengirim, dan melihat riwayat transaksi.
- Frasa pemulihan dikunci password di perangkat, dengan Argon2id dan XChaCha20-Poly1305.
- Langsung memakai jaringan demo regtest lewat gerbang `zevrigo-rpc.ratuaurion.store`.

## Yang khusus Android

- Wallet disimpan di folder data milik aplikasi itu sendiri. Folder ini ikut terhapus kalau aplikasi dihapus, jadi 24 kata tetap satu-satunya cadangan.
- Sertifikat HTTPS gerbang RPC diperiksa dengan daftar sertifikat akar sistem Android, sama seperti aplikasi lain di HP.
- Di layar HP, menu pindah ke bawah dengan lima tombol besar.

## Status

APK uji ini dibangun dalam mode debug. Pada 15 November 2026 APK itu lolos uji pertama di HP pengembang: membuat wallet, tersambung ke gerbang RPC lewat HTTPS, menerima koin dari faucet, dan mengirim transaksi.

APK uji ini **tidak dibagikan**. Yang akan dibagikan adalah APK rilis yang ditandatangani kunci rilis. APK itu akan muncul di [halaman Unduh](/unduh/) beserta sidik SHA-256-nya. Jangan memasang APK Zevrigo dari sumber lain.

Yang akan dikerjakan sebelum APK dibagikan:

1. Kunci tanda tangan rilis, beserta sidik SHA-256 yang diumumkan bersama berkasnya.
2. Tombol faucet langsung di aplikasi.
3. Pendaftaran Investor Demo dari aplikasi, dengan pesan bertanda tangan.
4. Tampilan koin terkunci untuk vesting di jaringan demo.

Sambil menunggu, wallet desktop sudah bisa dicoba lewat [panduan wallet dan faucet](/artikel/panduan-wallet-dan-faucet/).
