# MOD-04: Consensus (`homeostasis-consensus`)

- **ID:** `MOD-04`
- **Path:** `crates/consensus`
- **Paket:** `homeostasis-consensus`
- **Status:** 🟡 sedang berjalan (`TSK-M07` dan `TSK-M08` selesai; menunggu `TSK-M09`)
- **Bergantung pada:** `MOD-01`, `MOD-02`, `MOD-03`
- **Dipakai oleh:** `MOD-07`, `MOD-11`, `MOD-17`

## Tujuan

Menjadi satu-satunya tempat penulisan seluruh aturan konsensus protokol Homeostasis Coin.

## Tanggung Jawab

- Rumus jadwal pasokan koin (`base_reward`, *slow start*, pasokan ekor, alokasi genesis).
- Penyesuaian kesulitan ASERT per blok dan validasi cap waktu blok.
- Validasi transaksi (transfer, multi-tanda tangan, kunci waktu, kematangan coinbase), validasi blok, dan perhitungan kerja kumulatif rantai. Aturan lengkap: [spesifikasi-validasi-v0.1.md](../reference/spesifikasi-validasi-v0.1.md).

## Bukan Tanggung Jawab

- Penyimpanan persisten blok dan basis data disk (`MOD-05`).
- Komunikasi jaringan P2P (`MOD-06`) maupun antarmuka JSON-RPC (`MOD-08`).

## Task terkait
- [TSK-M07](../tasks/TSK-M07-consensus-pasokan.md) — 🟢 selesai (rumus pasokan dan vektor uji)
- [TSK-M08](../tasks/TSK-M08-consensus-target-asert.md) — 🟢 selesai (target compact, ASERT, chainwork, cap waktu, parameter jaringan, genesis regtest)
- [TSK-M09](../tasks/README.md) — 🟡 siap (validasi transaksi dan blok)
