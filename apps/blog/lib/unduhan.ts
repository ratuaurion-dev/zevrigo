/**
 * Daftar unduhan untuk halaman `/unduh/` (TSK-M64, keputusan pencipta 15 November 2026): halaman di blog, berkasnya di
 * GitHub Releases proyek. Blog tidak menyajikan berkas besar sendiri.
 *
 * `sha256` diisi sesudah rilis diterbitkan, dari `SHA256SUMS.txt` rilis itu (disalin pencipta), agar sidik berkas tampil di
 * dua tempat: halaman ini dan halaman rilis. Selama `null`, halaman meminta pengunjung membandingkan dengan `SHA256SUMS.txt`.
 */

export const RELEASES_URL = 'https://github.com/ratuaurion-dev/zevrigo/releases';

export type Unduhan = {
  id: 'windows' | 'android';
  platform: string;
  syarat: string;
  status: 'tersedia' | 'segera';
  versi: string;
  /** Hanya untuk status `tersedia`. */
  berkas?: {
    nama: string;
    url: string;
    halamanRilis: string;
    sha256Sums: string;
    sha256: string | null;
  };
  catatan: string[];
};

export const UNDUHAN: Unduhan[] = [
  {
    id: 'windows',
    platform: 'Windows',
    syarat: 'Windows 10 atau 11, 64-bit',
    status: 'tersedia',
    versi: '0.2.0',
    berkas: {
      nama: 'zevrigo-wallet-0.2.0-x64-setup.exe',
      url: `${RELEASES_URL}/download/wallet-v0.2.0/zevrigo-wallet-0.2.0-x64-setup.exe`,
      halamanRilis: `${RELEASES_URL}/tag/wallet-v0.2.0`,
      sha256Sums: `${RELEASES_URL}/download/wallet-v0.2.0/SHA256SUMS.txt`,
      // SHA256SUMS.txt rilis wallet-v0.2.0 (terbit 16 November 2026, build GitHub Actions dari 76a6124).
      sha256: 'bc22c6b96fbbcaeb73a19200191653e430dabe94237e32386df18ab1e4826d70',
    },
    catatan: [
      'Dibangun GitHub Actions dari kode sumber pada tag wallet-v0.2.0.',
      'Installer belum ditandatangani, jadi Windows SmartScreen menampilkan peringatan. Pilih Info selengkapnya lalu Tetap jalankan, hanya jika SHA-256 cocok.',
      'Langsung memakai jaringan demo regtest lewat gerbang zevrigo-rpc.ratuaurion.store.',
    ],
  },
  {
    id: 'android',
    platform: 'Android',
    syarat: 'Android 7.0 atau lebih baru, HP arm64',
    status: 'segera',
    versi: '0.2.0',
    catatan: [
      'APK uji sudah lolos uji di HP pengembang.',
      'Yang dibagikan nanti adalah APK rilis bertanda tangan kunci rilis, dengan sidik SHA-256 di halaman ini.',
      'Jangan memasang APK Zevrigo dari sumber lain.',
    ],
  },
];
