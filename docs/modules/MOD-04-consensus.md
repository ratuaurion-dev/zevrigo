# MOD-04: Consensus (`zevrigo-consensus`)

- **ID:** `MOD-04`
- **Path:** `crates/consensus`
- **Paket:** `zevrigo-consensus`
- **Status:** 🟢 selesai untuk regtest (`TSK-M07`, `TSK-M08`, `TSK-M09`); V3 RandomX testnet/mainnet (`TSK-M06` bagian B, DEC-01)
- **Bergantung pada:** `MOD-01`, `MOD-02`, `MOD-03`
- **Dipakai oleh:** `MOD-07`, `MOD-11`, `MOD-17`

## Tujuan

Menjadi satu-satunya tempat penulisan seluruh aturan konsensus protokol Zevrigo Coin.

## Tanggung Jawab

- Rumus jadwal pasokan koin (`base_reward`, *slow start*, pasokan ekor, alokasi genesis).
- Penyesuaian kesulitan ASERT per blok dan validasi cap waktu blok.
- Validasi transaksi (transfer, multi-tanda tangan, kunci waktu, kematangan coinbase), validasi blok, dan perhitungan kerja kumulatif rantai. Aturan lengkap: [spesifikasi-validasi-v0.1.md](../reference/spesifikasi-validasi-v0.1.md).
- PoW per jaringan (V3): `check_header_pow` dan `check_block_pow` dengan kunci RandomX (`randomx_key_height`) dan penghitung `RandomxHasher`; `randomx_hasher` memulai layanan RandomX MOD-03. `check_header_context_free` dan `check_block_context_free` memeriksa tanpa keduanya (regtest).
- Parameter dan genesis testnet (TSK-M06 C2): `ChainParams::testnet`, `ChainParams::for_network`, `testnet_genesis`, `testnet_network_id`, `known_genesis` (genesis dari `network_id`); `ParamsError::Unavailable` untuk mainnet.

## Bukan Tanggung Jawab

- Penyimpanan persisten blok dan basis data disk (`MOD-05`).
- Komunikasi jaringan P2P (`MOD-06`) maupun antarmuka JSON-RPC (`MOD-08`).

## Task terkait
- [TSK-M07](../tasks/TSK-M07-consensus-pasokan.md) — 🟢 selesai (rumus pasokan dan vektor uji)
- [TSK-M08](../tasks/TSK-M08-consensus-target-asert.md) — 🟢 selesai (target compact, ASERT, chainwork, cap waktu, parameter jaringan, genesis regtest)
- [TSK-M09](../tasks/TSK-M09-consensus-validasi.md) — 🟢 selesai (validasi transaksi dan blok, V1–V10)
- [TSK-M28](../tasks/TSK-M28-p2p-daemon.md) — 🟢 `check_header_context_free` (V2–V3 untuk header tanpa badan, sinkronisasi header P2P)
- [TSK-M06](../tasks/TSK-M06-pow-randomx.md) — 🟡 bagian B: V3 RandomX dengan kunci (`check_header_pow`, `check_block_pow`, `RandomxHasher`)
