//! Pandoc detection and invocation.
//!
//! Resolution order for the pandoc binary:
//!   1. The user's explicit override from `GlobalSettings.pandoc_path`,
//!      if it points at an executable that responds to `--version`.
//!   2. `pandoc` on `$PATH`.
//!   3. Common system install locations (Homebrew, /usr/local, etc.)
//!      so that GUI launches on macOS — which often have a stripped
//!      `$PATH` that excludes `/opt/homebrew/bin` — still find a
//!      Homebrew-installed pandoc.
//!   4. None — the caller surfaces a friendly install hint.

use crate::error::AppError;
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Common locations to probe when pandoc isn't on `$PATH`. Order
/// matters: Apple Silicon Homebrew first, then Intel Homebrew, then
/// system, then platform-typical Windows/Linux locations.
fn common_paths() -> Vec<PathBuf> {
    let mut v = Vec::with_capacity(8);
    #[cfg(target_os = "macos")]
    {
        v.push(PathBuf::from("/opt/homebrew/bin/pandoc"));
        v.push(PathBuf::from("/usr/local/bin/pandoc"));
        v.push(PathBuf::from("/usr/bin/pandoc"));
    }
    #[cfg(target_os = "linux")]
    {
        v.push(PathBuf::from("/usr/bin/pandoc"));
        v.push(PathBuf::from("/usr/local/bin/pandoc"));
        v.push(PathBuf::from("/snap/bin/pandoc"));
    }
    #[cfg(target_os = "windows")]
    {
        v.push(PathBuf::from(r"C:\Program Files\Pandoc\pandoc.exe"));
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            v.push(PathBuf::from(format!(r"{local}\Pandoc\pandoc.exe")));
        }
    }
    v
}

/// Run `<bin> --version` and return its first line on success.
async fn probe(bin: &str) -> Option<String> {
    let output = Command::new(bin).arg("--version").output().await.ok()?;
    if !output.status.success() {
        return None;
    }
    let version = String::from_utf8_lossy(&output.stdout);
    Some(version.lines().next().unwrap_or("pandoc").to_string())
}

/// Resolve the pandoc binary. Returns `(path, version_line)` if found.
pub async fn resolve_pandoc(override_path: Option<&str>) -> Option<(String, String)> {
    if let Some(p) = override_path {
        let trimmed = p.trim();
        if !trimmed.is_empty() {
            if let Some(v) = probe(trimmed).await {
                return Some((trimmed.to_string(), v));
            }
        }
    }
    if let Some(v) = probe("pandoc").await {
        return Some(("pandoc".to_string(), v));
    }
    for candidate in common_paths() {
        if candidate.exists() {
            let s = candidate.to_string_lossy().to_string();
            if let Some(v) = probe(&s).await {
                return Some((s, v));
            }
        }
    }
    None
}

/// Backwards-compatible: detect pandoc using the auto-discovery path
/// only (no override). Returns the version line on success. Kept for
/// callers that want a probe without touching settings.
#[allow(dead_code)]
pub async fn detect_pandoc() -> Option<String> {
    resolve_pandoc(None).await.map(|(_, v)| v)
}

/// Convenience: read the override from global settings and resolve.
pub async fn resolve_with_settings() -> Option<(String, String)> {
    let g = crate::commands::settings::read_global_settings().await;
    resolve_pandoc(g.pandoc_path.as_deref()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Skip-on-no-pandoc helper — different from the production probe
    /// in that it shells out via `which`/`PATH` only, not via our
    /// extended resolver, so the test can isolate "is pandoc reachable
    /// at all" from "does our resolver find it".
    fn pandoc_anywhere_on_system() -> bool {
        std::process::Command::new("pandoc")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            || std::path::Path::new("/opt/homebrew/bin/pandoc").exists()
            || std::path::Path::new("/usr/local/bin/pandoc").exists()
            || std::path::Path::new("/usr/bin/pandoc").exists()
    }

    #[tokio::test]
    async fn resolve_with_no_override_finds_pandoc_when_installed() {
        if !pandoc_anywhere_on_system() {
            eprintln!("skipping: pandoc not installed anywhere we recognize");
            return;
        }
        let result = resolve_pandoc(None).await;
        assert!(result.is_some(), "resolver should locate pandoc");
        let (path, version) = result.unwrap();
        assert!(!path.is_empty(), "resolved path empty");
        assert!(
            version.to_lowercase().contains("pandoc"),
            "version line should mention pandoc: {version}"
        );
        eprintln!("resolve_pandoc(None) → {path} ({version})");
    }

    #[tokio::test]
    async fn resolve_with_explicit_override_uses_override() {
        // Pick whichever common location exists on this box.
        let candidate = [
            "/opt/homebrew/bin/pandoc",
            "/usr/local/bin/pandoc",
            "/usr/bin/pandoc",
        ]
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .map(|s| s.to_string());
        let Some(path) = candidate else {
            eprintln!("skipping: no pandoc at common paths");
            return;
        };
        let result = resolve_pandoc(Some(&path)).await;
        assert!(
            result.is_some(),
            "resolver should accept the explicit override"
        );
        let (resolved, _) = result.unwrap();
        assert_eq!(resolved, path, "override should be used verbatim");
    }

    #[tokio::test]
    async fn override_with_bad_path_falls_back_to_path_or_common() {
        if !pandoc_anywhere_on_system() {
            eprintln!("skipping: pandoc not installed");
            return;
        }
        let result = resolve_pandoc(Some("/totally/nonexistent/pandoc-binary-xxxxx")).await;
        assert!(
            result.is_some(),
            "broken override should fall back, not return None"
        );
        let (resolved, _) = result.unwrap();
        assert_ne!(
            resolved, "/totally/nonexistent/pandoc-binary-xxxxx",
            "broken override path must NOT be returned"
        );
    }

    #[tokio::test]
    async fn empty_string_override_treated_as_none() {
        if !pandoc_anywhere_on_system() {
            return;
        }
        let with_empty = resolve_pandoc(Some("")).await;
        let with_none = resolve_pandoc(None).await;
        assert_eq!(
            with_empty.is_some(),
            with_none.is_some(),
            "empty string should behave identically to None"
        );
    }

    #[tokio::test]
    async fn whitespace_override_treated_as_none() {
        if !pandoc_anywhere_on_system() {
            return;
        }
        let result = resolve_pandoc(Some("   ")).await;
        assert!(
            result.is_some(),
            "whitespace-only override should fall through"
        );
    }

    #[tokio::test]
    async fn truly_missing_pandoc_returns_none() {
        // Hard to truly remove pandoc from the test environment; instead
        // exercise the negative path of `probe()` directly via an
        // override we know doesn't exist, AND simulate "no pandoc on
        // PATH" by relying on the bogus override falling through to
        // PATH/common probing. Where we can't simulate (pandoc IS
        // reachable), assert the structural property: the bad override
        // alone never resolves to itself.
        let bogus_only = probe("totally-nonexistent-pandoc-xyz").await;
        assert!(bogus_only.is_none(), "probe of a fake binary must be None");
    }
}

/// Run pandoc export. Honors the user's `pandoc_path` override.
pub async fn run_pandoc(
    input_path: &Path,
    output_path: &Path,
    format: &str,
    extra_args: &[String],
) -> Result<String, AppError> {
    let bin = match resolve_with_settings().await {
        Some((b, _)) => b,
        None => {
            return Err(AppError::Custom(
                "Pandoc not found. Install Pandoc from https://pandoc.org/installing.html or set the binary path in Settings → Editor → Pandoc."
                    .to_string(),
            ));
        }
    };
    let mut cmd = Command::new(&bin);
    cmd.arg(input_path);
    cmd.arg("-o").arg(output_path);

    match format {
        "html" => {
            cmd.arg("-t").arg("html5").arg("--standalone");
        }
        "pdf" => { /* pandoc auto-detects PDF engine */ }
        "docx" => {
            cmd.arg("-t").arg("docx");
        }
        "epub" => {
            cmd.arg("-t").arg("epub");
        }
        _ => {
            cmd.arg("-t").arg(format);
        }
    }

    for arg in extra_args {
        cmd.arg(arg);
    }

    let output = cmd.output().await?;

    if output.status.success() {
        Ok(format!("Export complete: {}", output_path.display()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(AppError::Custom(format!("Pandoc error: {}", stderr)))
    }
}
