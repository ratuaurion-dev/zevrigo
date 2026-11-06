# TSK-M16 · Pemilihan koin, biaya, kembalian, dan tanda tangan transaksi kirim di MOD-09 · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-09 (crates/wallet-core) · **Untuk:** FTR-03
- **Bergantung pada:** TSK-M03, TSK-M15, TSK-M24 · **Memblokir:** TSK-M19, TSK-F03, TSK-M17
- **Estimasi:** S · **Ditulis:** Quick (kode dan test), 1 November 2026

## Tujuan
Spesifikasi Wallet v0.1 Bagian 5: memilih UTXO jenis 0 milik wallet, menghitung biaya dan kembalian, lalu
menandatangani setiap input dengan BIP340 atas `sighash_v1` (DEC-13). Hasilnya transaksi yang lolos T1 sampai T6
selama koin yang diberikan memang ada di set UTXO. Pemanggil (CLI, TSK-M19) mengambil koin lewat `scan_utxos`,
menurunkan payload kembalian, dan memberikan `aux_rand` dari sumber acak sistem operasi.

## API (`send`, re-export dari lib.rs)
- `Coin::new(outpoint, amount, payload, spendable_at, branch, index)` dan getter-nya.
- `SendRequest { recipient_kind, recipient_payload, amount, fee_rate (quanta/byte), change_payload, tip_height }`.
- `plan(&[Coin], &SendRequest) -> Result<Plan, SendError>`: tanpa kunci rahasia.
- `Plan::{coins, outputs, fee, change, size}`.
- `sign_plan(&Plan, &ExtendedPrivKey akun, &network_id, &aux_rand) -> Result<Transaction, SendError>`.
- `build_transaction(coins, request, akun, network_id, aux_rand) -> Result<(Plan, Transaction), SendError>`.
- `SendError::{ZeroAmount, DuplicateCoin, InsufficientFunds { available, needed }, TooLarge { size },
  AmountOverflow, KeyMismatch, Derivation, Structure, Signing, SizeMismatch}`. `SINGLE_KEY_WITNESS_SIZE = 65`.

## Aturan (keputusan teknis Quick, dicatat juga di Spesifikasi Wallet Bagian 5)
1. Koin dengan `spendable_at ≤ tip_height + 1` diurutkan jumlah menurun lalu outpoint naik, diambil berurutan sampai
   `Σ input ≥ jumlah + tarif × ukuran tanpa kembalian`. Outpoint ganda ditolak.
2. Ukuran dihitung pasti: `3 + varint(k) + Σ input + output penerima + 65k`, ditambah `33 + varint(kembalian)` bila
   ada kembalian. Setelah menandatangani, ukuran encoding dicocokkan dengan rencana.
3. Sisa nol: tanpa kembalian. Sisa cukup untuk biaya output kembalian ditambah minimal 1 quanta: kembalian dibuat.
   Sisa lebih kecil dari itu: ikut menjadi biaya (tanpa output kembalian).
4. Panjang varint kembalian dicari dari 1 sampai 19 byte; dipakai panjang terkecil `L` yang memberi kembalian dengan
   varint ≤ `L`. Di batas varint (misalnya kembalian 128 butuh 2 byte, tetapi dengan biaya 2 byte tersisa 127),
   biaya menjadi `tarif × (ukuran + 1)`; selain itu biaya tepat `tarif × ukuran`.
5. Transaksi lebih dari 100_000 byte ditolak (`TooLarge`), sama dengan batas mempool.
6. Kunci input = `akun/branch/index` koin itu; x-only-nya wajib sama dengan payload koin (`KeyMismatch`).
   `aux_rand` yang sama dipakai untuk semua input; aman karena pesan tiap input berbeda (`input_index` ikut
   ditandatangani). Setiap tanda tangan diverifikasi ulang sebelum dikembalikan.
- Tanpa dependensi baru; Cargo.lock tidak berubah.

## Vektor
- T1 (implementasi referensi Python: BIP340 yang lolos vektor resmi, BLAKE3 `alat/blake3_ref.py`, kunci
  `alat/wallet_ref.py`): wallet W1, koin 5 ZVRG di 0/0, 3 ZVRG di 0/1, dan 7 ZVRG di 1/0 yang belum matang;
  kirim 6 ZVRG ke kunci contoh BIP340, tarif 1, kembalian ke 1/1. Hasil: 2 input, biaya 284, kembalian
  `2 ZVRG − 284`, 284 byte, txid `2dd7f50382bab1376d77991113bfde898a48d9f1594670dd88f74e61e2ea4d86`, encoding
  lengkap byte demi byte.
- E1 sampai E6: kasus batas biaya satu input (pas, sisa 30/33/34 ke biaya, kembalian 1 quanta, batas varint 127).
- Simulasi Python 20_000 kasus acak: Σ input = Σ output + biaya, biaya ≥ tarif × ukuran, prefiks terpendek.

## Test (wallet-core 26 → 35, workspace 280 → 289)
`crates/wallet-core/src/send/tests.rs`: `signed_vector_t1_two_inputs_with_change`,
`signatures_verify_and_commit_to_network_id`, `plan_orders_by_amount_then_outpoint`,
`plan_respects_spendable_at_boundary`, `plan_fee_and_change_edge_cases`,
`plan_rejects_zero_duplicate_and_insufficient`, `plan_rejects_transaction_larger_than_block`,
`sign_rejects_key_mismatch`, `prop_plan_balances_and_pays_at_least_rate`.
