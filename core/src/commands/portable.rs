//! IPC commands for portable mode introspection.

use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Serialize, Type)]
pub struct PortableModeInfo {
    pub enabled: bool,
    pub data_root: String,
}

#[tauri::command]
#[specta::specta]
pub async fn is_portable_mode() -> PortableModeInfo {
    let cfg = crate::services::portable::config();
    PortableModeInfo {
        enabled: cfg.enabled,
        data_root: cfg.data_root.to_string_lossy().to_string(),
    }
}
