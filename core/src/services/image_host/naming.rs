//! Object-key generator for hosts where the client chooses the key
//! (S3, R2, Aliyun OSS, Qiniu).
//!
//! Format: `{yyyy}/{mm}/{dd}/{8-char-blake3}-{sanitized-name}.{ext}`
//!
//! Sanitization: ASCII alphanumerics, `-`, `_`, `.` pass through unchanged;
//! spaces become `_`; everything else (including CJK) is dropped. If the
//! sanitized stem is empty (e.g. all-CJK filename), it falls back to
//! `image`. Total key length is capped at `MAX_KEY_LEN` to stay under
//! typical provider key limits.

use chrono::{DateTime, Datelike, Utc};

const MAX_KEY_LEN: usize = 200;

/// Generate an object key for the given filename + content + clock.
pub fn generate_key(filename: &str, bytes: &[u8], now: DateTime<Utc>) -> String {
    let (stem, ext) = split_ext(filename);
    let sanitized = sanitize_name(stem);
    let hash = blake3::hash(bytes).to_hex();
    let short_hash: String = hash.as_str().chars().take(8).collect();

    let prefix = format!(
        "{:04}/{:02}/{:02}/{}-",
        now.year(),
        now.month(),
        now.day(),
        short_hash,
    );
    let suffix = if ext.is_empty() {
        String::new()
    } else {
        format!(".{}", ext.to_ascii_lowercase())
    };

    let stem_budget = MAX_KEY_LEN.saturating_sub(prefix.len() + suffix.len());
    let trimmed_stem: String = sanitized.chars().take(stem_budget).collect();

    format!("{}{}{}", prefix, trimmed_stem, suffix)
}

fn split_ext(filename: &str) -> (&str, &str) {
    match filename.rfind('.') {
        Some(idx) if idx > 0 && idx < filename.len() - 1 => {
            (&filename[..idx], &filename[idx + 1..])
        }
        _ => (filename, ""),
    }
}

fn sanitize_name(stem: &str) -> String {
    let mut out = String::with_capacity(stem.len());
    for c in stem.chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
            out.push(c);
        } else if c == ' ' {
            out.push('_');
        } else if c.is_ascii() {
            // other ASCII punctuation/control — fold to underscore
            out.push('_');
        }
        // non-ASCII (CJK etc.) is dropped silently
    }
    if out.is_empty() {
        out.push_str("image");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn fixed() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 6, 12, 0, 0).unwrap()
    }

    #[test]
    fn key_has_date_prefix_hash_and_sanitized_name() {
        let key = generate_key("Hello World.PNG", b"some-bytes", fixed());
        assert!(key.starts_with("2026/05/06/"), "got: {}", key);
        assert!(key.ends_with("-Hello_World.png"), "got: {}", key);
        // 8 hex chars between prefix and dash
        let after = &key["2026/05/06/".len()..];
        let hash_part = &after[..8];
        assert!(
            hash_part.chars().all(|c| c.is_ascii_hexdigit()),
            "hash part not hex: {}",
            hash_part
        );
    }

    #[test]
    fn cjk_only_stem_falls_back_to_image() {
        let key = generate_key("照片.jpg", b"x", fixed());
        assert!(key.ends_with("-image.jpg"), "got: {}", key);
    }

    #[test]
    fn extension_is_lowercased() {
        let key = generate_key("photo.JPG", b"x", fixed());
        assert!(key.ends_with(".jpg"), "got: {}", key);
    }

    #[test]
    fn no_extension_means_no_dot() {
        let key = generate_key("README", b"x", fixed());
        // ends with "-README", no trailing dot
        assert!(key.ends_with("-README"), "got: {}", key);
        assert!(!key.ends_with("."), "got: {}", key);
    }

    #[test]
    fn long_name_capped_under_200_chars() {
        let huge = "a".repeat(500);
        let key = generate_key(&format!("{}.png", huge), b"x", fixed());
        assert!(key.len() <= 200, "key length {}: {}", key.len(), key);
        assert!(key.ends_with(".png"), "should preserve extension: {}", key);
    }

    #[test]
    fn deterministic_hash_for_same_bytes() {
        let a = generate_key("x.png", b"identical", fixed());
        let b = generate_key("x.png", b"identical", fixed());
        assert_eq!(a, b);
    }

    #[test]
    fn different_bytes_yield_different_hash() {
        let a = generate_key("x.png", b"one", fixed());
        let b = generate_key("x.png", b"two", fixed());
        assert_ne!(a, b);
    }

    #[test]
    fn punctuation_folded_to_underscore() {
        let key = generate_key("a@b#c.png", b"x", fixed());
        assert!(key.ends_with("-a_b_c.png"), "got: {}", key);
    }

    #[test]
    fn double_extension_keeps_last_only() {
        let key = generate_key("archive.tar.gz", b"x", fixed());
        assert!(key.ends_with(".gz"), "got: {}", key);
        assert!(key.contains("-archive.tar"), "got: {}", key);
    }
}
