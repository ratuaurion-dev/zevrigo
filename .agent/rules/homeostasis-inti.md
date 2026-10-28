---
trigger: always_on
---
# Aturan inti Homeostasis Coin

- Baca dan patuhi AGENTS.md di root repo sebelum mengerjakan tugas apa pun.
- Angka di tabel "Parameter protokol" AGENTS.md tidak boleh diubah. Jika ada yang tampak salah, berhenti dan laporkan.
- DILARANG MUTLAK di semua kode proyek, termasuk test: `unsafe` dan floating point (f32, f64, literal desimal, `as f64`).
  Jika tugas tampaknya membutuhkannya, berhenti dan laporkan.
- Kode konsensus: tanpa waktu sistem, tanpa angka acak, tanpa urutan HashMap yang memengaruhi hasil.
  Aritmetika selalu checked. Tanpa unwrap/expect/panic di kode produksi.
- Jangan menonaktifkan, melewati, atau melemahkan test supaya lolos.
- Jangan menambah dependensi, memakai sudo, git push, atau perintah destruktif tanpa izin.
- Jangan memakai GitHub CLI, akun online atau plugin pengguna, mengubah remote, atau mengubah git config tanpa izin tertulis di prompt tugas.
- Kerjakan satu tugas saja per percakapan. Akhiri dengan laporan sesuai format di AGENTS.md.
