//! Penyajian antarmuka web explorer (Spesifikasi Explorer Web v0.2 Bagian 7): file statis hasil `next build` (static
//! export dengan `trailingSlash`) di folder `--web-dir`, sebagai fungsi murni di atas sistem file.
//!
//! Jalur halaman: `/`, jalur berakhiran `/`, atau jalur yang segmen terakhirnya tanpa titik berarti `index.html` di
//! folder itu (`/blocks/` dan `/blocks` sama-sama `blocks/index.html`). Jalur lain adalah file dan harus berakhiran
//! salah satu ekstensi yang dikenal. Paling banyak 8 segmen; setiap segmen hanya huruf, angka, `.`, `_`, `-`, tidak
//! kosong, dan tidak diawali titik. Halaman yang tidak ada dijawab `404` dengan `404.html` hasil build (jika ada); file
//! yang tidak ada dan jalur yang tidak sah dijawab `404` teks.
//!
//! Next.js menyisipkan skrip inline (`self.__next_f`, data halaman) di setiap halaman HTML. Kebijakan keamanan konten
//! halaman itu menambahkan hash SHA-256 setiap skrip inline-nya ke `script-src` ([`page_policy`]), sehingga hanya
//! skrip hasil build yang berjalan; skrip lain tetap hanya dari origin yang sama.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// File halaman di setiap folder (halaman utama: `index.html` di akar folder web).
pub const INDEX_FILE: &str = "index.html";

/// Halaman `404` hasil build, dipakai untuk jalur halaman yang tidak ada.
pub const NOT_FOUND_FILE: &str = "404.html";

/// Kebijakan keamanan konten dasar untuk semua jawaban HTTP explorer: skrip, koneksi, gambar, dan font hanya dari
/// origin yang sama, tanpa frame. Gaya boleh inline (atribut `style` dari React dan radix-ui). Untuk halaman HTML,
/// `script-src` ditambah hash skrip inline halaman itu ([`page_policy`]).
/// Ditulis dengan `concat!` agar setiap baris punya tanda kutipnya sendiri: pemindai larangan
/// (`scripts/check-forbidden.sh`) membaca per baris, dan `'unsafe-inline'` di baris lanjutan string akan terbaca
/// sebagai kata kunci `unsafe`.
pub const CONTENT_SECURITY_POLICY: &str = concat!(
    "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; ",
    "img-src 'self' data:; font-src 'self'; connect-src 'self'; base-uri 'none'; ",
    "form-action 'none'; frame-ancestors 'none'",
);

/// Bagian `script-src` dasar di [`CONTENT_SECURITY_POLICY`].
const SCRIPT_SELF: &str = "script-src 'self'";

/// Kedalaman jalur file paling banyak (jumlah segmen).
const MAX_SEGMENTS: usize = 8;

/// Jenis isi halaman HTML.
const TEXT_HTML: &str = "text/html; charset=utf-8";

/// Jenis isi untuk teks biasa (juga data RSC `.txt` Next.js).
const TEXT_PLAIN: &str = "text/plain; charset=utf-8";

/// Folder file build Next.js yang namanya berisi hash isi.
const STATIC_DIR: &str = "_next/static";

/// Cache untuk file di `_next/static/`: tidak pernah berubah isinya.
const CACHE_IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// Cache untuk halaman dan file lain: selalu diperiksa ulang ke server.
const CACHE_REVALIDATE: &str = "no-cache";

/// Cache untuk galat.
const CACHE_NONE: &str = "no-store";

/// Awal dan akhir elemen skrip HTML.
const SCRIPT_OPEN: &[u8] = b"<script";
const SCRIPT_CLOSE: &[u8] = b"</script>";

/// Jawaban file web: kode status, jenis isi, aturan cache, kebijakan keamanan konten, dan isi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebResponse {
    /// Kode status HTTP.
    pub status: u16,
    /// Nilai kepala `Content-Type`.
    pub content_type: &'static str,
    /// Nilai kepala `Cache-Control`.
    pub cache_control: &'static str,
    /// Nilai kepala `Content-Security-Policy`: [`page_policy`] untuk halaman HTML, selain itu
    /// [`CONTENT_SECURITY_POLICY`].
    pub content_security_policy: String,
    /// Isi jawaban.
    pub body: Vec<u8>,
}

impl WebResponse {
    /// `404 not found` (teks biasa).
    pub(crate) fn not_found() -> Self {
        Self::text(404, b"not found\n")
    }

    /// `500 internal error` (teks biasa).
    pub(crate) fn internal() -> Self {
        Self::text(500, b"internal error\n")
    }

    fn text(status: u16, body: &[u8]) -> Self {
        Self {
            status,
            content_type: TEXT_PLAIN,
            cache_control: CACHE_NONE,
            content_security_policy: CONTENT_SECURITY_POLICY.to_owned(),
            body: body.to_vec(),
        }
    }

    /// Halaman HTML dengan kebijakan keamanan konten halaman itu.
    fn page(status: u16, body: Vec<u8>) -> Self {
        Self {
            status,
            content_type: TEXT_HTML,
            cache_control: if status == 200 {
                CACHE_REVALIDATE
            } else {
                CACHE_NONE
            },
            content_security_policy: page_policy(&body),
            body,
        }
    }
}

/// Jenis isi menurut ekstensi nama file, `None` untuk ekstensi yang tidak disajikan.
#[must_use]
pub fn content_type(name: &str) -> Option<&'static str> {
    let (_, extension) = name.rsplit_once('.')?;
    match extension {
        "html" => Some(TEXT_HTML),
        "js" => Some("text/javascript; charset=utf-8"),
        "css" => Some("text/css; charset=utf-8"),
        "json" => Some("application/json"),
        "svg" => Some("image/svg+xml"),
        "png" => Some("image/png"),
        "ico" => Some("image/x-icon"),
        "woff2" => Some("font/woff2"),
        "txt" => Some(TEXT_PLAIN),
        _ => None,
    }
}

fn valid_segment(segment: &str) -> bool {
    !segment.is_empty()
        && !segment.starts_with('.')
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

/// Isi folder web yang diminta.
enum Target {
    /// Halaman: `index.html` di folder yang diminta.
    Page(PathBuf),
    /// File dengan ekstensi yang dikenal dan jenis isinya.
    File(PathBuf, &'static str),
}

/// Jalur relatif di folder web untuk jalur permintaan `path`, `None` jika jalur itu tidak boleh disajikan.
fn target(path: &str) -> Option<Target> {
    let rest = path.strip_prefix('/')?;
    let folder = rest.strip_suffix('/');
    let trimmed = folder.unwrap_or(rest);
    let segments: Vec<&str> = if trimmed.is_empty() {
        Vec::new()
    } else {
        trimmed.split('/').collect()
    };
    if segments.len() > MAX_SEGMENTS || !segments.iter().all(|segment| valid_segment(segment)) {
        return None;
    }
    let mut relative: PathBuf = segments.iter().collect();
    match segments.last() {
        Some(last) if folder.is_none() && last.contains('.') => {
            content_type(last).map(|content_type| Target::File(relative, content_type))
        }
        _ => {
            relative.push(INDEX_FILE);
            Some(Target::Page(relative))
        }
    }
}

/// Menyajikan file untuk jalur permintaan `path` dari folder web `dir`.
#[must_use]
pub fn serve(dir: &Path, path: &str) -> WebResponse {
    match target(path) {
        None => WebResponse::not_found(),
        Some(Target::Page(relative)) => match std::fs::read(dir.join(relative)) {
            Ok(body) => WebResponse::page(200, body),
            Err(_) => match std::fs::read(dir.join(NOT_FOUND_FILE)) {
                Ok(body) => WebResponse::page(404, body),
                Err(_) => WebResponse::not_found(),
            },
        },
        Some(Target::File(relative, content_type)) => match std::fs::read(dir.join(&relative)) {
            Ok(body) if content_type == TEXT_HTML => WebResponse::page(200, body),
            Ok(body) => WebResponse {
                status: 200,
                content_type,
                cache_control: if relative.starts_with(STATIC_DIR) {
                    CACHE_IMMUTABLE
                } else {
                    CACHE_REVALIDATE
                },
                content_security_policy: CONTENT_SECURITY_POLICY.to_owned(),
                body,
            },
            Err(_) => WebResponse::not_found(),
        },
    }
}

/// Kebijakan keamanan konten untuk halaman HTML `html`: [`CONTENT_SECURITY_POLICY`] dengan hash SHA-256 (base64)
/// setiap skrip inline di halaman itu ditambahkan ke `script-src`, sesuai urutan dan tanpa duplikat. Skrip inline
/// adalah elemen `<script>` tanpa atribut `src`; isinya di-hash persis seperti tertulis.
#[must_use]
pub fn page_policy(html: &[u8]) -> String {
    let mut sources: Vec<String> = Vec::new();
    for script in inline_scripts(html) {
        let digest = Sha256::digest(script);
        let source = format!("'sha256-{}'", base64(&digest));
        if !sources.contains(&source) {
            sources.push(source);
        }
    }
    if sources.is_empty() {
        return CONTENT_SECURITY_POLICY.to_owned();
    }
    let scripts = format!("{SCRIPT_SELF} {}", sources.join(" "));
    CONTENT_SECURITY_POLICY.replacen(SCRIPT_SELF, &scripts, 1)
}

/// Posisi pertama `needle` (tidak kosong) di `haystack`.
fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Isi setiap elemen `<script>` tanpa atribut `src` di `html`, sesuai urutan.
fn inline_scripts(html: &[u8]) -> Vec<&[u8]> {
    let mut scripts = Vec::new();
    let mut rest = html;
    while let Some(start) = find(rest, SCRIPT_OPEN) {
        let after = &rest[start + SCRIPT_OPEN.len()..];
        let is_script = after
            .first()
            .is_some_and(|byte| *byte == b'>' || byte.is_ascii_whitespace());
        if !is_script {
            rest = after;
            continue;
        }
        let Some(tag_end) = find(after, b">") else {
            break;
        };
        let body = &after[tag_end + 1..];
        let Some(end) = find(body, SCRIPT_CLOSE) else {
            break;
        };
        if find(&after[..tag_end], b"src=").is_none() {
            scripts.push(&body[..end]);
        }
        rest = &body[end + SCRIPT_CLOSE.len()..];
    }
    scripts
}

/// Base64 standar dengan padding (RFC 4648 Bagian 4).
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let byte = |index: usize| usize::from(chunk.get(index).copied().unwrap_or(0));
        let group = (byte(0) << 16) | (byte(1) << 8) | byte(2);
        for position in 0..4 {
            if position <= chunk.len() {
                out.push(char::from(ALPHABET[(group >> (18 - 6 * position)) & 63]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
