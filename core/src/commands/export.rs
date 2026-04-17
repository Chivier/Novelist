use crate::error::AppError;
use crate::services::pandoc;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct PandocStatus {
    pub available: bool,
    pub version: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub async fn check_pandoc() -> Result<PandocStatus, AppError> {
    let version = pandoc::detect_pandoc().await;
    Ok(PandocStatus {
        available: version.is_some(),
        version,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn export_project(
    input_files: Vec<String>,
    output_path: String,
    format: String,
    extra_args: Vec<String>,
) -> Result<String, AppError> {
    let allowed_formats = ["html", "pdf", "docx", "epub"];
    if !allowed_formats.contains(&format.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "Unsupported export format: {}. Allowed: {:?}",
            format, allowed_formats
        )));
    }

    for arg in &extra_args {
        let lower = arg.to_lowercase();
        if lower.starts_with("--output") || lower.starts_with("--extract-media") {
            return Err(AppError::InvalidInput(format!(
                "Forbidden argument in extra_args: {}",
                arg
            )));
        }
    }

    let temp_dir = std::env::temp_dir();
    let temp_input = temp_dir.join("novelist-export-input.md");

    let mut combined = String::new();
    for path in &input_files {
        let content = tokio::fs::read_to_string(path).await?;
        combined.push_str(&content);
        combined.push_str("\n\n");
    }
    tokio::fs::write(&temp_input, &combined).await?;

    // Run pandoc
    let result = pandoc::run_pandoc(
        &temp_input,
        std::path::Path::new(&output_path),
        &format,
        &extra_args,
    )
    .await;

    // Cleanup temp file
    let _ = tokio::fs::remove_file(&temp_input).await;

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pandoc_status_serialize() {
        let status = PandocStatus {
            available: true,
            version: Some("pandoc 3.1".to_string()),
        };
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["available"], true);
        assert_eq!(json["version"], "pandoc 3.1");
    }

    #[test]
    fn test_pandoc_status_unavailable() {
        let status = PandocStatus {
            available: false,
            version: None,
        };
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["available"], false);
        assert!(json["version"].is_null());
    }

    #[tokio::test]
    async fn test_export_rejects_unsupported_format() {
        let result = export_project(
            vec![],
            "/tmp/out.xyz".to_string(),
            "xyz".to_string(),
            vec![],
        )
        .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unsupported export format"));
    }

    #[tokio::test]
    async fn test_export_rejects_forbidden_args() {
        let result = export_project(
            vec![],
            "/tmp/out.html".to_string(),
            "html".to_string(),
            vec!["--output=/tmp/evil".to_string()],
        )
        .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Forbidden argument"));
    }

    #[tokio::test]
    async fn test_export_rejects_extract_media() {
        let result = export_project(
            vec![],
            "/tmp/out.html".to_string(),
            "html".to_string(),
            vec!["--extract-media=/tmp".to_string()],
        )
        .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Forbidden argument"));
    }

    #[tokio::test]
    async fn test_export_allows_valid_formats() {
        for fmt in &["html", "pdf", "docx", "epub"] {
            // These will fail because input_files is empty or pandoc isn't available,
            // but they should NOT fail on format validation
            let result = export_project(
                vec!["/nonexistent/file.md".to_string()],
                "/tmp/out".to_string(),
                fmt.to_string(),
                vec![],
            )
            .await;
            if let Err(e) = &result {
                let msg = e.to_string();
                assert!(
                    !msg.contains("Unsupported export format"),
                    "format '{}' was incorrectly rejected",
                    fmt
                );
            }
        }
    }
}
