//! Penyajian antarmuka web explorer (Spesifikasi Explorer Web v0.1 Bagian 7): file statis hasil build
//! `apps/explorer/web` dari folder `--web-dir`, sebagai fungsi murni di atas sistem file.
//!
//! Hanya jalur yang aman yang dibaca: `/` berarti `index.html`; jalur lain paling banyak 4 segmen, setiap segmen hanya
//! huruf, angka, `.`, `_`, `-`, tidak diawali titik, dan berakhiran salah satu ekstensi yang dikenal. Selain itu `404`,
//! tanpa fallback ke `index.html` (antarmuka web memakai jalur hash `#/…`, sehingga server cukup menyajikan file yang
//! ada).

use std::path::{Path, PathBuf};

/// File halaman utama di folder web.
pub const INDEX_FILE: &str = "index.html";

/// Kebijakan keamanan konten untuk semua jawaban HTTP explorer: skrip, gaya, dan koneksi hanya dari origin yang sama,
/// tanpa sumber luar, tanpa frame.
pub const CONTENT_SECURITY_POLICY: &str = "default-src 'none'; script-src 'self'; style-src 'self'; \
     img-src 'self' data:; font-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; \
     frame-ancestors 'none'";

/// Kedalaman jalur file paling banyak (jumlah segmen).
const MAX_SEGMENTS: usize = 4;

/// Jenis isi untuk teks biasa.
const TEXT_PLAIN: &str = "text/plain; charset=utf-8";

/// Cache untuk file di `assets/` (nama file berisi hash isi dari build Vite).
const CACHE_IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// Cache untuk `index.html` dan file lain: selalu diperiksa ulang ke server.
const CACHE_REVALIDATE: &str = "no-cache";

/// Jawaban file web: kode status, jenis isi, aturan cache, dan isi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebResponse {
    /// Kode status HTTP.
    pub status: u16,
    /// Nilai kepala `Content-Type`.
    pub content_type: &'static str,
    /// Nilai kepala `Cache-Control`.
    pub cache_control: &'static str,
    /// Isi jawaban.
    pub body: Vec<u8>,
}

impl WebResponse {
    /// `404 not found` (teks biasa).
    pub(crate) fn not_found() -> Self {
        Self {
            status: 404,
            content_type: TEXT_PLAIN,
            cache_control: "no-store",
            body: b"not found\n".to_vec(),
        }
    }

    /// `500 internal error` (teks biasa).
    pub(crate) fn internal() -> Self {
        Self {
            status: 500,
            content_type: TEXT_PLAIN,
            cache_control: "no-store",
            body: b"internal error\n".to_vec(),
        }
    }
}

/// Jenis isi menurut ekstensi nama file, `None` untuk ekstensi yang tidak disajikan.
#[must_use]
pub fn content_type(name: &str) -> Option<&'static str> {
    let (_, extension) = name.rsplit_once('.')?;
    match extension {
        "html" => Some("text/html; charset=utf-8"),
        "js" => Some("text/javascript; charset=utf-8"),
        "css" => Some("text/css; charset=utf-8"),
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

/// Jalur relatif file di folder web untuk jalur permintaan `path`, `None` jika jalur itu tidak boleh disajikan.
fn file_path(path: &str) -> Option<PathBuf> {
    let rest = path.strip_prefix('/')?;
    if rest.is_empty() {
        return Some(PathBuf::from(INDEX_FILE));
    }
    let segments: Vec<&str> = rest.split('/').collect();
    if segments.len() > MAX_SEGMENTS || !segments.iter().all(|segment| valid_segment(segment)) {
        return None;
    }
    content_type(segments.last()?).map(|_| segments.iter().copied().collect())
}

/// Menyajikan file untuk jalur permintaan `path` dari folder web `dir`.
#[must_use]
pub fn serve(dir: &Path, path: &str) -> WebResponse {
    let Some(relative) = file_path(path) else {
        return WebResponse::not_found();
    };
    let Some(content_type) = relative
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(content_type)
    else {
        return WebResponse::not_found();
    };
    match std::fs::read(dir.join(&relative)) {
        Ok(body) => WebResponse {
            status: 200,
            content_type,
            cache_control: if relative.starts_with("assets") {
                CACHE_IMMUTABLE
            } else {
                CACHE_REVALIDATE
            },
            body,
        },
        Err(_) => WebResponse::not_found(),
    }
}

#[cfg(test)]
mod tests;
