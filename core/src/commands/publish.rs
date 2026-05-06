//! Tauri commands for publishing + per-platform media upload + the
//! Pandoc Markdown→HTML helper.
//!
//! Mirrors `commands/image_host.rs` — one command per platform, no
//! generic dispatcher. The frontend's `app/lib/services/publish.ts`
//! routes by `PlatformConfig.platform` discriminant.

use crate::error::AppError;
use crate::models::publish::{PlatformConfig, PublishSettings};
use crate::services::publish::types::{PublishError, PublishInput, PublishResult};
use crate::services::publish::{ghost, medium, pandoc_html, wordpress, wordpress_com};

impl From<PublishError> for AppError {
    fn from(e: PublishError) -> AppError {
        AppError::Custom(e.to_string())
    }
}

#[tauri::command]
#[specta::specta]
pub async fn publish_to_ghost(
    input: PublishInput,
    config: PlatformConfig,
) -> Result<PublishResult, AppError> {
    Ok(ghost::publish(&config, &input).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn publish_to_wordpress_self_hosted(
    input: PublishInput,
    config: PlatformConfig,
) -> Result<PublishResult, AppError> {
    Ok(wordpress::publish(&config, &input).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn publish_to_wordpress_com(
    input: PublishInput,
    config: PlatformConfig,
) -> Result<PublishResult, AppError> {
    Ok(wordpress_com::publish(&config, &input).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn publish_to_medium(
    input: PublishInput,
    config: PlatformConfig,
) -> Result<PublishResult, AppError> {
    Ok(medium::publish(&config, &input).await?)
}

/// Returns `(hosted_url, attachment_id_or_zero)`. Only WordPress
/// returns a non-zero attachment id (used for `featured_media`).
#[derive(serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PostImageUploadResult {
    pub url: String,
    pub attachment_id: u64,
}

#[tauri::command]
#[specta::specta]
pub async fn upload_post_image_ghost(
    bytes: Vec<u8>,
    filename: String,
    mime: String,
    config: PlatformConfig,
) -> Result<PostImageUploadResult, AppError> {
    let (admin_url, api_key) = match &config {
        PlatformConfig::Ghost {
            admin_url,
            api_key,
        } => (admin_url, api_key),
        _ => return Err(AppError::Custom("not a Ghost config".into())),
    };
    let url = ghost::upload_image(admin_url, api_key, bytes, filename, mime).await?;
    Ok(PostImageUploadResult {
        url,
        attachment_id: 0,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn upload_post_image_wordpress_self_hosted(
    bytes: Vec<u8>,
    filename: String,
    mime: String,
    config: PlatformConfig,
) -> Result<PostImageUploadResult, AppError> {
    let (site_url, username, app_password) = match &config {
        PlatformConfig::WordPressSelfHosted {
            site_url,
            username,
            app_password,
        } => (site_url, username, app_password),
        _ => return Err(AppError::Custom("not a WordPress config".into())),
    };
    let auth = wordpress::basic_auth_header(username, app_password);
    let (url, id) = wordpress::upload_image(site_url, &auth, bytes, filename, mime).await?;
    Ok(PostImageUploadResult {
        url,
        attachment_id: id,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn upload_post_image_wordpress_com(
    bytes: Vec<u8>,
    filename: String,
    mime: String,
    config: PlatformConfig,
) -> Result<PostImageUploadResult, AppError> {
    let (site, token) = match &config {
        PlatformConfig::WordPressCom {
            site_id_or_domain,
            access_token,
        } => (site_id_or_domain, access_token),
        _ => return Err(AppError::Custom("not a WordPress.com config".into())),
    };
    let (url, id) = wordpress_com::upload_image(site, token, bytes, filename, mime).await?;
    Ok(PostImageUploadResult {
        url,
        attachment_id: id,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn upload_post_image_medium(
    bytes: Vec<u8>,
    filename: String,
    mime: String,
    config: PlatformConfig,
) -> Result<PostImageUploadResult, AppError> {
    let token = match &config {
        PlatformConfig::Medium { token } => token,
        _ => return Err(AppError::Custom("not a Medium config".into())),
    };
    let url = medium::upload_image(token, bytes, filename, mime).await?;
    Ok(PostImageUploadResult {
        url,
        attachment_id: 0,
    })
}

/// Convert Markdown to HTML via the bundled / system Pandoc binary.
/// Used by the frontend orchestrator before submitting to Ghost / WP /
/// WP.com (Medium consumes Markdown directly).
#[tauri::command]
#[specta::specta]
pub async fn convert_markdown_to_html(markdown: String) -> Result<String, AppError> {
    Ok(pandoc_html::markdown_to_html(&markdown).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn get_publish_settings() -> Result<PublishSettings, AppError> {
    let g = crate::commands::settings::read_global_settings().await;
    Ok(g.publish)
}

#[tauri::command]
#[specta::specta]
pub async fn set_publish_settings(settings: PublishSettings) -> Result<(), AppError> {
    let mut g = crate::commands::settings::read_global_settings().await;
    g.publish = settings;
    crate::commands::settings::write_global_settings_to_disk(&g).await
}
