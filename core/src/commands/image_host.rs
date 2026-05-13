//! Tauri commands for image-host uploads + settings.
//!
//! One command per provider — no generic dispatcher — because each
//! provider's wire format and config shape differ enough that a unified
//! trait would be a forced abstraction. The frontend's
//! `app/lib/services/image-host.ts` routes to the right command based on
//! the active host's `provider` discriminant.
//!
//! Settings live in `GlobalSettings.image_hosts` (always global —
//! credentials never leak into per-project files). Per-project overrides
//! are limited to `ProjectConfig.active_image_host_id` (the pointer only,
//! handled in `commands/settings.rs`).

use crate::error::AppError;
use crate::models::image_host::{ImageHostSettings, ProviderConfig};
use crate::services::image_host::naming;
use crate::services::image_host::types::{HostError, UploadInput, UploadResult};
use crate::services::image_host::{aliyun_oss, custom, imgur, qiniu, s3, smms};

impl From<HostError> for AppError {
    fn from(e: HostError) -> AppError {
        AppError::Custom(e.to_string())
    }
}

fn build_input(bytes: Vec<u8>, filename: String, mime: String) -> UploadInput {
    let key = naming::generate_key(&filename, &bytes, chrono::Utc::now());
    UploadInput {
        bytes,
        filename,
        mime,
        key,
    }
}

#[tauri::command]
#[specta::specta]
pub async fn upload_image_qiniu(
    bytes: Vec<u8>,
    filename: String,
    mime: String,
    config: ProviderConfig,
) -> Result<UploadResult, AppError> {
    Ok(qiniu::upload(&config, build_input(bytes, filename, mime)).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn upload_image_aliyun_oss(
    bytes: Vec<u8>,
    filename: String,
    mime: String,
    config: ProviderConfig,
) -> Result<UploadResult, AppError> {
    Ok(aliyun_oss::upload(&config, build_input(bytes, filename, mime)).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn upload_image_s3(
    bytes: Vec<u8>,
    filename: String,
    mime: String,
    config: ProviderConfig,
) -> Result<UploadResult, AppError> {
    Ok(s3::upload(&config, build_input(bytes, filename, mime)).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn upload_image_imgur(
    bytes: Vec<u8>,
    filename: String,
    mime: String,
    config: ProviderConfig,
) -> Result<UploadResult, AppError> {
    Ok(imgur::upload(&config, build_input(bytes, filename, mime)).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn upload_image_smms(
    bytes: Vec<u8>,
    filename: String,
    mime: String,
    config: ProviderConfig,
) -> Result<UploadResult, AppError> {
    Ok(smms::upload(&config, build_input(bytes, filename, mime)).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn upload_image_custom(
    bytes: Vec<u8>,
    filename: String,
    mime: String,
    config: ProviderConfig,
) -> Result<UploadResult, AppError> {
    Ok(custom::upload(&config, build_input(bytes, filename, mime)).await?)
}

/// Read the global `image_hosts` settings block.
#[tauri::command]
#[specta::specta]
pub async fn get_image_host_settings() -> Result<ImageHostSettings, AppError> {
    let g = crate::commands::settings::read_global_settings().await;
    Ok(g.image_hosts)
}

/// Replace the global `image_hosts` settings block atomically.
#[tauri::command]
#[specta::specta]
pub async fn set_image_host_settings(settings: ImageHostSettings) -> Result<(), AppError> {
    let mut g = crate::commands::settings::read_global_settings().await;
    g.image_hosts = settings;
    crate::commands::settings::write_global_settings_to_disk(&g).await
}

/// Read a file as raw bytes for upload. The frontend uses this to load
/// local image files referenced in Markdown into memory before handing
/// them to one of the `upload_image_*` commands.
#[tauri::command]
#[specta::specta]
pub async fn read_image_bytes(path: String) -> Result<Vec<u8>, AppError> {
    use tokio::fs;
    let p = std::path::PathBuf::from(&path);
    if !p.exists() {
        return Err(AppError::FileNotFound(path));
    }
    let bytes = fs::read(&p).await?;
    Ok(bytes)
}
