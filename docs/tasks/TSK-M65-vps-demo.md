# TSK-M65 · Jaringan demo regtest di VPS dan komputer pencipta · 🟡 · Prioritas: Tinggi
- **Jenis:** task repo (Docker) · **Modul:** — (`compose.publik.yaml`, dokumen) · **Untuk:** FTR-11 (DEC-34, DEC-38)
- **Bergantung pada:** TSK-M47, TSK-M53, TSK-M54, TSK-M64 · **Ditulis:** Quick, 17 November 2026
- **Keputusan pencipta (17 November 2026):** VPS untuk seed, pengamat, dan explorer (beserta yang berbagi namespace seed) dan
  faucet; node penambang dan treasury tetap di PC; masih regtest

## Isi
| Bagian | Isi |
|---|---|
| Persiapan VPS (Q145, tanpa repo) | Kunci SSH ed25519 di WSL, user `zvrg`, login hanya kunci (`0-zevrigo.conf`, karena `00-vpsmurah.conf` penyedia dibaca lebih dulu), swap 2 GiB, Docker resmi, `ufw` hanya 22, pembaruan otomatis, NTP |
| Compose (Q146) | `profiles` `vps`/`pc` di setiap layanan `compose.publik.yaml`; contoh perintah di kepala berkas, README, Spesifikasi Docker Bagian 7.1 |
| Pemindahan (Q146) | Image `docker save`/`load` (ID sama), layanan `vps` di PC dihentikan, 4 volume tar → VPS, `create` + pulihkan + `up`, PeerId seed sama, node PC tersambung, blok baru sampai di explorer publik, container `vps` di PC dihapus (volume disimpan) |

## Definition of done
- [x] Q145 VPS siap (17 November 2026)
- [ ] Q146 commit profile compose dan dokumen
- [ ] Q146 pemindahan: semua hostname publik dari VPS, PeerId seed tetap, blok dari PC sampai di explorer
