//! Markdown → HTML helper for publish adapters that need HTML
//! (Ghost, WordPress self-hosted, WordPress.com).
//!
//! Uses the system Pandoc binary (we do NOT bundle Pandoc — bundle
//! size matters). The binary is resolved by
//! `services::pandoc::resolve_with_settings`, which honors the user's
//! `pandoc_path` override from settings, then probes `$PATH`, then
//! common install locations.
//!
//! On a system without Pandoc the error message points the user to
//! the install page so they can either run `brew install pandoc` /
//! download an installer, or set the binary path in Settings.

use crate::services::pandoc;
use crate::services::publish::types::PublishError;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const INSTALL_HINT: &str =
    "Pandoc not found on PATH. Install from https://pandoc.org/installing.html (e.g. `brew install pandoc` on macOS) or set the binary path in Settings → Editor → Pandoc.";

/// Convert Markdown to HTML via the resolved Pandoc binary.
pub async fn markdown_to_html(md: &str) -> Result<String, PublishError> {
    let (binary, _version) = pandoc::resolve_with_settings()
        .await
        .ok_or_else(|| PublishError::PandocFailed(INSTALL_HINT.to_string()))?;
    markdown_to_html_with_binary(md, &binary).await
}

pub async fn markdown_to_html_with_binary(md: &str, binary: &str) -> Result<String, PublishError> {
    let mut child = Command::new(binary)
        .args(["-f", "markdown", "-t", "html"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            // ENOENT (os error 2) means the binary path didn't exist.
            // Surface the install hint instead of the raw OS message.
            if e.kind() == std::io::ErrorKind::NotFound {
                PublishError::PandocFailed(INSTALL_HINT.to_string())
            } else {
                PublishError::PandocFailed(format!("spawn pandoc: {e}"))
            }
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(md.as_bytes())
            .await
            .map_err(|e| PublishError::PandocFailed(format!("write stdin: {e}")))?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| PublishError::PandocFailed(format!("wait pandoc: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(PublishError::PandocFailed(stderr));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| PublishError::PandocFailed(format!("non-utf8 output: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pandoc may not be present on every dev box; guard each test on
    /// `which pandoc` so CI without the binary skips cleanly.
    fn pandoc_available() -> bool {
        std::process::Command::new("pandoc")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[tokio::test]
    async fn paragraph_converts_to_p_tag() {
        if !pandoc_available() {
            eprintln!("skipping: pandoc not installed");
            return;
        }
        let html = markdown_to_html_with_binary("hello world", "pandoc")
            .await
            .unwrap();
        assert!(html.contains("<p>hello world</p>"), "got: {html}");
    }

    #[tokio::test]
    async fn heading_converts() {
        if !pandoc_available() {
            return;
        }
        let html = markdown_to_html_with_binary("# Title\n\nbody", "pandoc")
            .await
            .unwrap();
        assert!(
            html.contains("<h1") && html.contains("Title"),
            "got: {html}"
        );
    }

    #[tokio::test]
    async fn cjk_paragraph_is_preserved() {
        if !pandoc_available() {
            return;
        }
        let html = markdown_to_html_with_binary("中文段落，混合 English。", "pandoc")
            .await
            .unwrap();
        assert!(html.contains("中文段落"), "got: {html}");
    }

    #[tokio::test]
    async fn missing_binary_surfaces_install_hint() {
        let err = markdown_to_html_with_binary("x", "pandoc-nonexistent-binary-xxxxx")
            .await
            .unwrap_err();
        let PublishError::PandocFailed(msg) = err else {
            panic!("expected PandocFailed");
        };
        assert!(
            msg.contains("Install") || msg.contains("https://pandoc.org"),
            "missing install hint: {msg}"
        );
    }
}
