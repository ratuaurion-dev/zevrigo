#!/usr/bin/env bash
# Pemindai larangan mutlak kode Rust Zevrigo Coin (ZVRG):
# - Tipe floating point kata utuh: f32, f64
# - Literal floating point (desimal seperti 1.0 / 1.5 atau eksponen seperti 2e3)
# - Kata kunci unsafe
#
# Catatan desain:
# - Komentar satu baris (//, ///, //!) dan isi string literal ("...") diabaikan
#   agar doc comment maupun pesan teks tetap boleh menyebut istilah tersebut.
# - Komentar blok /* ... */ sengaja diperlakukan konservatif sebagai kode biasa.

set -euo pipefail

scan_dirs=()
for d in crates apps tests; do
    if [[ -d "$d" ]]; then
        scan_dirs+=("$d")
    fi
done

if [[ "${#scan_dirs[@]}" -eq 0 ]]; then
    exit 0
fi

mapfile -t rs_files < <(find "${scan_dirs[@]}" -type f -name '*.rs' | LC_ALL=C sort)

if [[ "${#rs_files[@]}" -eq 0 ]]; then
    exit 0
fi

found=0

for f in "${rs_files[@]}"; do
    # Kosongkan isi string literal satu baris terlebih dahulu, lalu hapus komentar //
    cleaned="$(sed -E 's/"([^"\\]|\\.)*"/""/g; s|//.*||' "$f")"

    if matches="$(grep -En '(^|[^[:alnum:]_])(f32|f64)([^[:alnum:]_]|$)' <<< "$cleaned" || true)" && [[ -n "$matches" ]]; then
        while IFS= read -r line; do
            lineno="${line%%:*}"
            echo "${f}:${lineno}: ditemukan tipe floating point terlarang (f32/f64)"
        done <<< "$matches"
        found=1
    fi

    if matches="$(grep -En '(^|[^[:alnum:]_.])[0-9][0-9_]*\.([0-9]|[^.0-9[:alnum:]_]|$)|(^|[^[:alnum:]_])[0-9][0-9_]*(\.[0-9][0-9_]*)?[eE][+-]?[0-9_]+([^[:alnum:]_]|$)' <<< "$cleaned" || true)" && [[ -n "$matches" ]]; then
        while IFS= read -r line; do
            lineno="${line%%:*}"
            echo "${f}:${lineno}: ditemukan literal floating point terlarang"
        done <<< "$matches"
        found=1
    fi

    if matches="$(grep -En '(^|[^[:alnum:]_])unsafe([^[:alnum:]_]|$)' <<< "$cleaned" || true)" && [[ -n "$matches" ]]; then
        while IFS= read -r line; do
            lineno="${line%%:*}"
            echo "${f}:${lineno}: ditemukan kata kunci unsafe terlarang"
        done <<< "$matches"
        found=1
    fi
done

if [[ "$found" -ne 0 ]]; then
    exit 1
fi
