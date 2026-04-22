use crate::services::sync::{self, SyncConfig, SyncStatus};
use crate::services::webdav;
use crate::AppError;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
pub struct SyncConfigMasked {
    pub enabled: bool,
    pub webdav_url: String,
    pub username: String,
    pub password: String,
    pub interval_minutes: u32,
}

impl From<SyncConfig> for SyncConfigMasked {
    fn from(c: SyncConfig) -> Self {
        Self {
            enabled: c.enabled,
            webdav_url: c.webdav_url,
            username: c.username,
            password: "****".to_string(),
            interval_minutes: c.interval_minutes,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_sync_config(project_dir: String) -> Result<SyncConfigMasked, AppError> {
    let config = sync::read_sync_config(&project_dir)?;
    Ok(config.into())
}

#[tauri::command]
#[specta::specta]
pub async fn save_sync_config(project_dir: String, mut config: SyncConfig) -> Result<(), AppError> {
    // If the frontend sends the masked password, preserve the existing one
    if config.password == "****" {
        if let Ok(existing) = sync::read_sync_config(&project_dir) {
            config.password = existing.password;
        }
    }
    sync::save_sync_config_to_disk(&project_dir, &config)
}

#[tauri::command]
#[specta::specta]
pub async fn sync_now(project_dir: String) -> Result<SyncStatus, AppError> {
    sync::perform_sync(&project_dir).await
}

#[tauri::command]
#[specta::specta]
pub async fn test_sync_connection(
    webdav_url: String,
    username: String,
    password: String,
) -> Result<bool, AppError> {
    let client = reqwest::Client::new();
    let auth = webdav::WebDavAuth { username, password };
    webdav::test_connection(&client, &webdav_url, &auth).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_config_masked_hides_password() {
        let config = SyncConfig {
            enabled: true,
            webdav_url: "https://dav.example.com".to_string(),
            username: "user".to_string(),
            password: "secret_password_123".to_string(),
            interval_minutes: 15,
        };
        let masked: SyncConfigMasked = config.into();
        assert_eq!(masked.password, "****");
        assert_eq!(masked.username, "user");
        assert_eq!(masked.webdav_url, "https://dav.example.com");
        assert!(masked.enabled);
        assert_eq!(masked.interval_minutes, 15);
    }

    #[test]
    fn test_sync_config_masked_serialize() {
        let masked = SyncConfigMasked {
            enabled: false,
            webdav_url: "https://example.com".to_string(),
            username: "test".to_string(),
            password: "****".to_string(),
            interval_minutes: 30,
        };
        let json = serde_json::to_value(&masked).unwrap();
        assert_eq!(json["enabled"], false);
        assert_eq!(json["password"], "****");
        assert_eq!(json["interval_minutes"], 30);
    }
}
