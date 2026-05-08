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
