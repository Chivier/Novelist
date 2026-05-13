//! Shared types for publish adapters.

use serde::{Deserialize, Serialize};
use specta::Type;

/// Body format selector. Ghost / WordPress consume HTML; Medium
/// consumes Markdown natively.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BodyFormat {
    Html,
    Markdown,
}

/// Inputs handed to a platform adapter's `publish()` function. The
/// frontend builds this from the publish dialog plus pre-publish
/// image rewrite.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PublishInput {
    pub title: String,
    /// Already-rewritten body. For Ghost / WordPress this is HTML
    /// (from Pandoc); for Medium this is Markdown.
    pub body: String,
    pub body_format: BodyFormat,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    /// Platform-specific status string. See per-platform spec.
    pub status: String,
    /// Already-uploaded feature image URL on the platform.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_image_url: Option<String>,
    /// WordPress-specific: pre-uploaded media attachment id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub featured_media_id: Option<u64>,
    /// Medium-only: when set, post to a publication instead of the
    /// authenticated user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication_id: Option<String>,
}

/// Result returned to the frontend on success.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PublishResult {
    /// Canonical URL of the new post on the platform.
    pub url: String,
    /// Platform post id (string for portability across platforms).
    pub remote_id: String,
}

/// All errors a publish adapter can return. Mapped to
/// `AppError::Custom(...)` at the Tauri command boundary.
#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Authentication rejected: {0}")]
    Auth(String),
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
    #[error("Bad config: {0}")]
    BadConfig(String),
    #[error("Platform returned {status}: {message}")]
    Server { status: u16, message: String },
    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),
    #[error("Pandoc conversion failed: {0}")]
    PandocFailed(String),
    #[error("Image upload failed for {ref_path}: {cause}")]
    ImageUploadFailed { ref_path: String, cause: String },
}
