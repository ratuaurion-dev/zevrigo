# Vektor Uji Konsensus (`tests/vectors/`)

Folder ini disiapkan untuk menyimpan **vektor uji konsensus berbentuk data murni** (independen terhadap bahasa pemrograman) yang dapat dipakai untuk memverifikasi kesesuaian hingga tingkat quanta/byte antar implementasi maupun lintas versi.

## Konvensi Penamaan Berkas

Gunakan format `<topik>_v<nomor_versi>.json` (huruf kecil dengan garis bawah):

- `supply_v1.json` — jadwal penerbitan koin (`base_reward`, `block_subsidy`, `mined_total_after`)
- `encoding_v1.json` — serialisasi kanonis transaksi dan blok
- `asert_v1.json` — perhitungan penyesuaian kesulitan ASERT dan aturan cap waktu
- `schnorr_musig2_v1.json` — vektor tanda tangan Schnorr BIP340 dan agregasi MuSig2

Semua angka jumlah quanta pada berkas JSON wajib disimpan sebagai **string bilangan bulat desimal** (bukan angka JSON yang rawan diparse sebagai float 64-bit).

## Status (R0)

Belum ada berkas vektor uji pada tahap R0; vektor akan ditambahkan seiring pengerjaan task konsensus di tahap berikutnya.
