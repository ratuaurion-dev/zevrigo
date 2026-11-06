# TSK-M21 · ID transaksi dan blok di MOD-02 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-02 (crates/crypto) · **Untuk:** FTR-01 sampai FTR-05
- **Bergantung pada:** TSK-M02, TSK-M03 · **Memblokir:** TSK-M09, TSK-M10, TSK-M12, TSK-M16
- **Estimasi:** S

## Tujuan
Menjadi satu-satunya tempat di proyek yang menghitung identitas data chain: txid, wtxid, block_id,
hash kondisi (payload output jenis 1), dan Merkle root sebuah blok. Semua dihitung dari encoding kanonis
MOD-01 (TSK-M03) dengan fungsi hash berpemisah domain dari TSK-M02. Tidak ada encoding baru dan tidak ada
aturan validasi di task ini.

## Konteks
- Sumber kebenaran: `docs/reference/spesifikasi-format-v0.1.md`
  Bagian 3 (domain hash), 4.1 (kondisi), 5.3 (txid dan wtxid), 6.1 (block_id), 6.3 (Merkle), dan 7 (ID jaringan).
  Jika kartu ini dan spesifikasi berbeda, BERHENTI dan laporkan.
- MOD-02 sudah bergantung pada MOD-01. Pakai `Transaction`, `Header`, `Block`, `Condition`, dan trait `Encode` dari
  zevrigo-primitives, serta `hash`, `Domain`, dan `merkle_root` dari modul TSK-M02.
- Lint konsensus aktif di crate ini. Jangan dinonaktifkan.

## Ruang lingkup
- Boleh diubah: crates/crypto/**; status TSK-M21 di docs/tasks/README.md dan docs/modules/MOD-02-crypto.md.
- Tanpa dependensi baru. Cargo.toml (root dan crate) serta Cargo.lock tidak berubah.
- Jangan diubah: modul `hash`, `merkle`, `schnorr` (kecuali menambah re-export di lib.rs), crate lain,
  termasuk crates/primitives. Jika butuh perubahan di MOD-01, BERHENTI dan tanya.
- Di luar lingkup: pemeriksaan bahwa merkle_root di header cocok dengan isi blok (TSK-M09), sighash (DEC-13),
  PoW (TSK-M05), format alamat bech32m.

## Persyaratan (modul `ids`)
1. `pub fn txid(tx: &Transaction) -> Hash32` = `hash(Domain::Txid, encoding badan saja)`.
   Encoding badan saja adalah fungsi dari TSK-M03 (bahan txid, contoh V1b).
2. `pub fn wtxid(tx: &Transaction) -> Hash32` = `hash(Domain::Wtxid, encoding lengkap)`.
   Untuk coinbase, encoding lengkap sama dengan encoding badan (tanpa witness).
3. `pub fn block_id(header: &Header) -> Hash32` = `hash(Domain::BlockId, 96 byte header)`.
4. `pub fn condition_hash(condition: &Condition) -> Hash32` = `hash(Domain::Condition, encoding kondisi)`.
   Inilah nilai `payload` output jenis 1.
5. `pub fn block_merkle_root(block: &Block) -> Option<Hash32>` = `merkle_root` atas daftar wtxid semua transaksi
   di blok, sesuai urutannya. `None` hanya jika blok tidak punya transaksi (tidak mungkin lewat API MOD-01,
   tetapi tetap ditangani tanpa unwrap).
6. `pub fn network_id(genesis_header: &Header) -> Hash32` = `block_id(genesis_header)`. Dokumentasikan bahwa
   fungsi ini hanya bermakna untuk header genesis.
7. Re-export semua fungsi di atas dari lib.rs.
8. Implementasi memakai `DomainHasher` atau `hash` dari TSK-M02, tidak memanggil crate blake3 langsung.

## Vektor uji (wajib dari alat independen, bukan dari kode kita)
Bahan byte: vektor V1, V1b, V2, V3, V4, V5, dan V8 dari kartu TSK-M03 (hex-nya sudah ada di test crate primitives).
Hitung nilai harapan di shell dengan b3sum dan xxd, lalu tulis sebagai konstanta di test, dengan perintah
pembuatnya sebagai komentar di atas setiap konstanta. Pola perintah:
`printf '%s' '<hex>' | xxd -r -p | b3sum --derive-key "<konteks>" --no-names`

| ID | Fungsi | Bahan byte | Konteks |
|---|---|---|---|
| H1 | txid(V1) | V1b (121 byte) | `zevrigo 2026-10-28 txid` |
| H2 | wtxid(V1) | V1 (186 byte) | `zevrigo 2026-10-28 wtxid` |
| H3 | txid(V2) | V2 (53 byte) | `zevrigo 2026-10-28 txid` |
| H4 | wtxid(V2) | V2 (53 byte) | `zevrigo 2026-10-28 wtxid` |
| H5 | txid(V3) dan wtxid(V3) | V3 (89 byte) | keduanya |
| H6 | block_id(V4) | V4 (96 byte) | `zevrigo 2026-10-28 block id` |
| H7 | condition_hash(V5) | V5 (107 byte) | `zevrigo 2026-10-28 condition` |
| H8 | block_merkle_root(V8) | daun = H4 lalu H2 (urutan transaksi V8: coinbase V2, lalu V1). Hitung leaf dan node di shell seperti TSK-M02 | `merkle leaf` dan `merkle node` |
| H9 | block_id(header V8) | sama dengan H6 (header V8 adalah V4) | — |

## Test lain
- txid(V2) ≠ wtxid(V2) walaupun bahan bytenya sama (domain berbeda).
- Mengganti byte tanda tangan di witness V1 mengubah wtxid tetapi TIDAK mengubah txid.
- Mengganti satu byte di badan V1 mengubah txid dan wtxid.
- Merkle root blok berubah jika urutan transaksi ditukar (lewat merkle_root langsung atas daftar wtxid).
- Proptest: untuk transaksi acak yang valid (pakai generator dari test MOD-01 jika bisa diakses; jika tidak,
  buat generator sederhana sendiri di test crate ini), txid tidak bergantung pada witness.

## Kriteria penerimaan
- [ ] H1–H9 cocok dengan nilai dari b3sum
- [ ] Semua test lain lolos
- [ ] Tanpa dependensi baru; `#![forbid(unsafe_code)]` dan lint konsensus tetap aktif; tanpa unwrap/expect/panic di kode non-test
- [ ] Unit test di `src/ids/tests.rs`, terpisah dari kode
- [ ] `make check-all` lolos
- [ ] Status TSK-M21 di docs/tasks/README.md menjadi 🟢, dan kartu MOD-02 mencatat TSK-M21 selesai

## Verifikasi
```bash
make check-all
git diff --stat HEAD~1
```

## Laporan akhir
Format AGENTS.md. Sertakan daftar API publik dan output semua perintah b3sum yang dipakai (H1–H9).
