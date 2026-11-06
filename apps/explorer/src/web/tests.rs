use std::path::PathBuf;

use super::{CONTENT_SECURITY_POLICY, base64, content_type, page_policy, serve};

/// Isi `index.html` uji: satu skrip dari file dan dua skrip inline gaya Next.js.
const INDEX: &str = r#"<!DOCTYPE html><html><head><script src="/_next/static/chunks/a1b2.js" async=""></script></head><body><script>(self.__next_f=self.__next_f||[]).push([0])</script></body></html>"#;

/// Hash skrip inline di [`INDEX`] (dihitung terpisah dengan Python hashlib).
const INDEX_HASH: &str = "'sha256-OBTN3RiyCV4Bq7dFqZ5a2pAXjnCcCYeTJMO2I/LYKeo='";

/// Folder web sementara khusus satu test (bentuk hasil `next build` static export), dihapus saat selesai.
struct Site(PathBuf);

impl Site {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("zevrigo-q90-web-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        for folder in [
            "_next/static/chunks",
            "_next/static/css",
            "blocks",
            "assets/sub",
        ] {
            std::fs::create_dir_all(path.join(folder)).unwrap();
        }
        let files: [(&str, &str); 10] = [
            ("index.html", INDEX),
            (
                "404.html",
                "<!DOCTYPE html><title>404</title><script><b>404</b></script>",
            ),
            ("robots.txt", "User-agent: *"),
            ("__next._tree.txt", "0:{}"),
            (".env", "rahasia"),
            ("blocks/index.html", "<!DOCTYPE html><title>blok</title>"),
            ("blocks/__next.blocks.__PAGE__.txt", "1:{}"),
            ("_next/static/chunks/a1b2.js", "console.log(1)"),
            ("_next/static/css/a1b2.css", "body{}"),
            ("_next/static/chunks/run.sh", "echo"),
        ];
        for (name, body) in files {
            std::fs::write(path.join(name), body).unwrap();
        }
        Self(path)
    }
}

impl Drop for Site {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn serves_files_from_web_dir() {
    let site = Site::new("serve");
    let index = serve(&site.0, "/");
    assert_eq!(index.status, 200);
    assert_eq!(index.content_type, "text/html; charset=utf-8");
    assert_eq!(index.cache_control, "no-cache");
    assert_eq!(index.body, INDEX.as_bytes());
    assert_eq!(index.content_security_policy, page_policy(INDEX.as_bytes()));
    assert!(index.content_security_policy.contains(INDEX_HASH));
    assert_eq!(serve(&site.0, "/index.html"), index);
    let blocks = serve(&site.0, "/blocks/");
    assert_eq!(blocks.status, 200);
    assert_eq!(blocks.body, b"<!DOCTYPE html><title>blok</title>");
    assert_eq!(blocks.content_security_policy, CONTENT_SECURITY_POLICY);
    assert_eq!(serve(&site.0, "/blocks"), blocks);
    let script = serve(&site.0, "/_next/static/chunks/a1b2.js");
    assert_eq!(script.status, 200);
    assert_eq!(script.content_type, "text/javascript; charset=utf-8");
    assert_eq!(script.cache_control, "public, max-age=31536000, immutable");
    assert_eq!(script.content_security_policy, CONTENT_SECURITY_POLICY);
    assert_eq!(script.body, b"console.log(1)");
    let style = serve(&site.0, "/_next/static/css/a1b2.css");
    assert_eq!(style.content_type, "text/css; charset=utf-8");
    assert_eq!(style.cache_control, "public, max-age=31536000, immutable");
    for (path, body) in [
        ("/robots.txt", "User-agent: *"),
        ("/__next._tree.txt", "0:{}"),
        ("/blocks/__next.blocks.__PAGE__.txt", "1:{}"),
    ] {
        let text = serve(&site.0, path);
        assert_eq!(text.status, 200, "{path}");
        assert_eq!(text.content_type, "text/plain; charset=utf-8", "{path}");
        assert_eq!(text.cache_control, "no-cache", "{path}");
        assert_eq!(text.body, body.as_bytes(), "{path}");
    }
    assert_eq!(content_type("a.json"), Some("application/json"));
    assert_eq!(content_type("a.svg"), Some("image/svg+xml"));
    assert_eq!(content_type("a.png"), Some("image/png"));
    assert_eq!(content_type("a.ico"), Some("image/x-icon"));
    assert_eq!(content_type("a.woff2"), Some("font/woff2"));
    assert!(
        CONTENT_SECURITY_POLICY.starts_with("default-src 'none'; script-src 'self'; style-src")
    );
    assert!(CONTENT_SECURITY_POLICY.contains("style-src 'self' 'unsafe-inline';"));
    assert!(CONTENT_SECURITY_POLICY.contains("connect-src 'self'"));
    assert!(CONTENT_SECURITY_POLICY.ends_with("frame-ancestors 'none'"));
    assert!(!CONTENT_SECURITY_POLICY.contains("  "));
}

#[test]
fn rejects_unsafe_or_unknown_paths() {
    let site = Site::new("reject");
    let paths = [
        "",
        "index.html",
        "//index.html",
        "/.env",
        "/../index.html",
        "/blocks/../index.html",
        "/_next/./static/chunks/a1b2.js",
        "/blocks//",
        "/_next/static/chunks/run.sh",
        "/_next/static/chunks/a1b2%2ejs",
        "/_next\\static\\chunks\\a1b2.js",
        "/missing.js",
        "/a/b/c/d/e/f/g/h/i.js",
        "/INDEX.HTML",
    ];
    for path in paths {
        let response = serve(&site.0, path);
        assert_eq!(response.status, 404, "{path}");
        assert_eq!(response.content_type, "text/plain; charset=utf-8", "{path}");
        assert_eq!(response.cache_control, "no-store", "{path}");
        assert_eq!(
            response.content_security_policy, CONTENT_SECURITY_POLICY,
            "{path}"
        );
        assert_eq!(response.body, b"not found\n", "{path}");
    }
    assert_eq!(content_type("noext"), None);
    assert_eq!(content_type("a.HTML"), None);
    assert_eq!(content_type("a.map"), None);
}

#[test]
fn serves_not_found_page_for_missing_pages() {
    let site = Site::new("missing");
    let not_found = std::fs::read(site.0.join("404.html")).unwrap();
    for path in [
        "/tidak-ada/",
        "/tidak-ada",
        "/assets",
        "/assets/sub/",
        "/blocks/lama/",
    ] {
        let page = serve(&site.0, path);
        assert_eq!(page.status, 404, "{path}");
        assert_eq!(page.content_type, "text/html; charset=utf-8", "{path}");
        assert_eq!(page.cache_control, "no-store", "{path}");
        assert_eq!(page.body, not_found, "{path}");
        assert!(
            page.content_security_policy.contains(
                "script-src 'self' 'sha256-9O/IAZ2+9XJrxeTVaIW91OJ0VsvwvdiPBgGUQkox3Fo=';"
            ),
            "{path}"
        );
    }
    std::fs::remove_file(site.0.join("404.html")).unwrap();
    let plain = serve(&site.0, "/tidak-ada/");
    assert_eq!(plain.status, 404);
    assert_eq!(plain.content_type, "text/plain; charset=utf-8");
    assert_eq!(plain.body, b"not found\n");
}

#[test]
fn page_policy_hashes_inline_scripts() {
    for (input, output) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ] {
        assert_eq!(base64(input.as_bytes()), output, "{input}");
    }
    // Contoh di spesifikasi CSP Level 3: hash `alert('Hello, world.');`.
    let example = page_policy(b"<p>x</p><script>alert('Hello, world.');</script>");
    assert!(
        example
            .contains("script-src 'self' 'sha256-qznLcsROx4GACP2dm0UCKCzCG+HiZ1guq6ZZDob/Tng=';")
    );
    let html = concat!(
        r#"<script src="/_next/static/chunks/a1b2.js" async=""></script>"#,
        "<script>(self.__next_f=self.__next_f||[]).push([0])</script>",
        r#"<script>self.__next_f.push([1,"0:[\"$\",\"html\"]\n"])</script>"#,
        "<script>(self.__next_f=self.__next_f||[]).push([0])</script>",
        "<scripts>bukan skrip</scripts><noscript>x</noscript>",
        r#"<script type="application/json" id="data"></script>"#,
    );
    let expected = CONTENT_SECURITY_POLICY.replacen(
        "script-src 'self'",
        "script-src 'self' 'sha256-OBTN3RiyCV4Bq7dFqZ5a2pAXjnCcCYeTJMO2I/LYKeo=' \
         'sha256-b69jhbvTdHmt2iNapvsv5zhrrf5ZpreAa5n1KUo5H9Y=' \
         'sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU='",
        1,
    );
    assert_eq!(page_policy(html.as_bytes()), expected);
    assert_eq!(page_policy(b"<p>tanpa skrip</p>"), CONTENT_SECURITY_POLICY);
    assert_eq!(
        page_policy(b"<script>tanpa penutup"),
        CONTENT_SECURITY_POLICY
    );
}
