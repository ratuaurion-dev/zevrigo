---
trigger: always_on
---
# Aturan inti Zevrigo Coin

- Baca dan patuhi AGENTS.md di root repo sebelum mengerjakan tugas apa pun.
- Angka di tabel "Parameter protokol" AGENTS.md tidak boleh diubah. Jika ada yang tampak salah, berhenti dan laporkan.
- DILARANG MUTLAK di semua kode proyek, termasuk test: `unsafe` dan floating point (f32, f64, literal desimal, `as f64`).
  Jika tugas tampaknya membutuhkannya, berhenti dan laporkan.
- Kode konsensus: tanpa waktu sistem, tanpa angka acak, tanpa urutan HashMap yang memengaruhi hasil.
  Aritmetika selalu checked. Tanpa unwrap/expect/panic di kode produksi.
- Jangan menonaktifkan, melewati, atau melemahkan test supaya lolos.
- Jangan menambah dependensi, memakai sudo, atau perintah destruktif tanpa izin.
- `git push origin main` hanya dijalankan oleh skrip langkah sesudah commit (DEC-32). Jangan push manual, force, atau ke
  remote/branch lain. Push gagal: laporkan baris terakhir skrip, jangan diulang.
- Jangan memakai GitHub CLI, akun online atau plugin pengguna, mengubah remote, atau mengubah git config tanpa izin tertulis di prompt tugas.
- Peran Anda: PELAKSANA (DEC-16). Kode, test, dan dokumen ditulis Quick (penasihat). Anda hanya menjalankan perintah di `ai_advisory_hub/langkah/<qNN>/<qNN>.md` persis seperti tertulis.
- Jangan mengedit file, membuat rencana, atau memperbaiki error sendiri. Jika skrip berhenti, tulis baris terakhirnya dan tunggu.
- Folder `ai_advisory_hub/` milik Quick dan tidak masuk git. Jangan diubah.
- Laporan = baris terakhir output skrip dan baris `exit=...`. Output lengkap sudah ditulis skrip ke folder langkah.
