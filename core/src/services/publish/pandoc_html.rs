//! Markdown → HTML helper for publish adapters that need HTML
//! (Ghost, WordPress self-hosted, WordPress.com).
//!
//! Spawns `pandoc -f markdown -t html`, writes Markdown to stdin,
//! reads HTML from stdout. Reuses pandoc location detection from
//! `services::pandoc`.

use crate::services::publish::types::PublishError;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Convert Markdown to HTML via the bundled / system Pandoc binary.
pub async fn markdown_to_html(md: &str) -> Result<String, PublishError> {
    markdown_to_html_with_binary(md, "pandoc").await
}

pub async fn markdown_to_html_with_binary(
    md: &str,
    binary: &str,
) -> Result<String, PublishError> {
    let mut child = Command::new(binary)
        .args(["-f", "markdown", "-t", "html"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| PublishError::PandocFailed(format!("spawn pandoc: {e}")))?;

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
        let html = markdown_to_html("hello world").await.unwrap();
        assert!(html.contains("<p>hello world</p>"), "got: {html}");
    }

    #[tokio::test]
    async fn heading_converts() {
        if !pandoc_available() {
            return;
        }
        let html = markdown_to_html("# Title\n\nbody").await.unwrap();
        assert!(html.contains("<h1") && html.contains("Title"), "got: {html}");
    }

    #[tokio::test]
    async fn cjk_paragraph_is_preserved() {
        if !pandoc_available() {
            return;
        }
        let html = markdown_to_html("中文段落，混合 English。").await.unwrap();
        assert!(html.contains("中文段落"), "got: {html}");
    }

    #[tokio::test]
    async fn missing_binary_is_pandoc_failed() {
        let err = markdown_to_html_with_binary("x", "pandoc-nonexistent-binary-xxxxx")
            .await
            .unwrap_err();
        assert!(matches!(err, PublishError::PandocFailed(_)), "got {err:?}");
    }
}
