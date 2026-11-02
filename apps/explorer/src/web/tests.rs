use std::path::PathBuf;

use super::{CONTENT_SECURITY_POLICY, content_type, serve};

/// Folder web sementara khusus satu test, dihapus saat selesai.
struct Site(PathBuf);

impl Site {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("homeostasis-q85-web-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("assets").join("sub")).unwrap();
        let files: [(&str, &str); 6] = [
            ("index.html", "<!doctype html><title>HSIS</title>"),
            ("robots.txt", "User-agent: *"),
            (".env", "rahasia"),
            ("assets/index-a1b2c3.js", "console.log(1)"),
            ("assets/index-a1b2c3.css", "body{}"),
            ("assets/run.sh", "echo"),
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
    assert_eq!(index.body, b"<!doctype html><title>HSIS</title>");
    assert_eq!(serve(&site.0, "/index.html"), index);
    let script = serve(&site.0, "/assets/index-a1b2c3.js");
    assert_eq!(script.status, 200);
    assert_eq!(script.content_type, "text/javascript; charset=utf-8");
    assert_eq!(script.cache_control, "public, max-age=31536000, immutable");
    assert_eq!(script.body, b"console.log(1)");
    let style = serve(&site.0, "/assets/index-a1b2c3.css");
    assert_eq!(style.content_type, "text/css; charset=utf-8");
    assert_eq!(style.cache_control, "public, max-age=31536000, immutable");
    let robots = serve(&site.0, "/robots.txt");
    assert_eq!(robots.content_type, "text/plain; charset=utf-8");
    assert_eq!(robots.cache_control, "no-cache");
    assert_eq!(content_type("a.svg"), Some("image/svg+xml"));
    assert_eq!(content_type("a.png"), Some("image/png"));
    assert_eq!(content_type("a.ico"), Some("image/x-icon"));
    assert_eq!(content_type("a.woff2"), Some("font/woff2"));
    assert!(CONTENT_SECURITY_POLICY.starts_with("default-src 'none'; script-src 'self';"));
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
        "/assets/../index.html",
        "/assets/./index-a1b2c3.js",
        "/assets/",
        "/assets",
        "/assets/sub",
        "/assets/run.sh",
        "/assets/index%2da1b2c3.js",
        "/assets\\index-a1b2c3.js",
        "/missing.js",
        "/a/b/c/d/e.js",
        "/INDEX.HTML",
    ];
    for path in paths {
        let response = serve(&site.0, path);
        assert_eq!(response.status, 404, "{path}");
        assert_eq!(response.content_type, "text/plain; charset=utf-8", "{path}");
        assert_eq!(response.body, b"not found\n", "{path}");
    }
    assert_eq!(content_type("noext"), None);
    assert_eq!(content_type("a.HTML"), None);
    assert_eq!(content_type("a.map"), None);
}
