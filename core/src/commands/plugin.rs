use crate::error::AppError;
use crate::models::plugin::{PluginInfo, PluginManifest, RegisteredCommandInfo};
use crate::services::plugin_host::sandbox::PluginHostState;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{Manager, State};

/// Known built-in plugin IDs that ship with Novelist.
const BUILTIN_PLUGIN_IDS: &[&str] = &["canvas", "mindmap"];

fn novelist_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".novelist")
}

fn plugins_dir() -> PathBuf {
    novelist_dir().join("plugins")
}

fn plugin_settings_path() -> PathBuf {
    novelist_dir().join("plugin-settings.json")
}

// --- Plugin Settings Persistence ---

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct PluginSettings {
    /// Map of plugin_id -> enabled (true/false).
    enabled: HashMap<String, bool>,
}

async fn read_plugin_settings() -> PluginSettings {
    let path = plugin_settings_path();
    if !path.exists() {
        return PluginSettings::default();
    }
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => PluginSettings::default(),
    }
}

async fn write_plugin_settings(settings: &PluginSettings) -> Result<(), AppError> {
    let dir = novelist_dir();
    if !dir.exists() {
        tokio::fs::create_dir_all(&dir).await?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| AppError::Custom(format!("Failed to serialize plugin settings: {e}")))?;
    tokio::fs::write(plugin_settings_path(), json).await?;
    Ok(())
}

// --- Bundled Plugin Installation ---

/// Install bundled plugins to ~/.novelist/plugins/ if they don't already exist.
/// Bundled plugin assets are embedded via include_dir at compile time.
async fn ensure_bundled_plugins(app_handle: &tauri::AppHandle) -> Result<(), AppError> {
    let target = plugins_dir();
    if !target.exists() {
        tokio::fs::create_dir_all(&target).await?;
    }

    // Bundled plugins are shipped in the Tauri resource directory under "bundled-plugins/"
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| AppError::Custom(format!("Failed to get resource dir: {e}")))?;
    let bundled_dir = resource_dir.join("bundled-plugins");

    if !bundled_dir.exists() {
        return Ok(());
    }

    for plugin_id in BUILTIN_PLUGIN_IDS {
        let src = bundled_dir.join(plugin_id);
        let dest = target.join(plugin_id);
        if !src.exists() {
            continue;
        }
        // Only install if the destination doesn't exist yet
        if dest.exists() {
            // Check if bundled version is newer
            let src_manifest = src.join("manifest.toml");
            let dest_manifest = dest.join("manifest.toml");
            if src_manifest.exists() && dest_manifest.exists() {
                if let (Ok(src_content), Ok(dest_content)) = (
                    tokio::fs::read_to_string(&src_manifest).await,
                    tokio::fs::read_to_string(&dest_manifest).await,
                ) {
                    if let (Ok(src_m), Ok(dest_m)) = (
                        toml::from_str::<PluginManifest>(&src_content),
                        toml::from_str::<PluginManifest>(&dest_content),
                    ) {
                        if src_m.plugin.version == dest_m.plugin.version {
                            continue;
                        }
                        // Bundled version differs, update it
                    }
                }
            } else {
                continue;
            }
        }
        copy_dir_recursive(&src, &dest).await?;
    }

    Ok(())
}

async fn copy_dir_recursive(src: &PathBuf, dest: &PathBuf) -> Result<(), AppError> {
    if !dest.exists() {
        tokio::fs::create_dir_all(dest).await?;
    }
    let mut entries = tokio::fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dest.join(&file_name);

        // Skip node_modules, dist, .DS_Store
        let name = file_name.to_string_lossy();
        if name == "node_modules" || name == ".DS_Store" || name == "target" {
            continue;
        }

        if src_path.is_dir() {
            Box::pin(copy_dir_recursive(&src_path, &dest_path)).await?;
        } else {
            tokio::fs::copy(&src_path, &dest_path).await?;
        }
    }
    Ok(())
}

/// Scan ~/.novelist/plugins/ and return info for each plugin found.
#[tauri::command]
#[specta::specta]
pub async fn list_plugins(
    state: State<'_, PluginHostState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<PluginInfo>, AppError> {
    // Ensure bundled plugins are installed
    ensure_bundled_plugins(&app_handle).await.ok();

    let dir = plugins_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let settings = read_plugin_settings().await;
    let loaded = state.list_loaded_plugins();
    let loaded_ids: std::collections::HashSet<String> =
        loaded.iter().map(|p| p.id.clone()).collect();

    let mut plugins = Vec::new();

    let mut entries = tokio::fs::read_dir(&dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let manifest_path = path.join("manifest.toml");
        if !manifest_path.exists() {
            continue;
        }
        let content = match tokio::fs::read_to_string(&manifest_path).await {
            Ok(c) => c,
            Err(_) => continue,
        };
        let manifest: PluginManifest = match toml::from_str(&content) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let plugin_id = &manifest.plugin.id;
        let is_builtin = BUILTIN_PLUGIN_IDS.contains(&plugin_id.as_str());
        let is_active = loaded_ids.contains(plugin_id);
        // Default: builtin plugins are disabled, user plugins are disabled
        let is_enabled = settings.enabled.get(plugin_id).copied().unwrap_or(false);

        plugins.push(PluginInfo {
            id: manifest.plugin.id,
            name: manifest.plugin.name,
            version: manifest.plugin.version,
            permissions: manifest.plugin.permissions,
            active: is_active,
            ui: manifest.ui,
            description: manifest.plugin.description,
            author: manifest.plugin.author,
            icon: manifest.plugin.icon,
            builtin: is_builtin,
            enabled: is_enabled,
        });
    }

    // Sort: builtin first, then by name
    plugins.sort_by(|a, b| b.builtin.cmp(&a.builtin).then(a.name.cmp(&b.name)));

    Ok(plugins)
}

/// Enable or disable a plugin (persists to settings file).
/// When enabling, also loads the plugin. When disabling, unloads it.
#[tauri::command]
#[specta::specta]
pub async fn set_plugin_enabled(
    plugin_id: String,
    enabled: bool,
    state: State<'_, PluginHostState>,
) -> Result<(), AppError> {
    // Persist the setting
    let mut settings = read_plugin_settings().await;
    settings.enabled.insert(plugin_id.clone(), enabled);
    write_plugin_settings(&settings).await?;

    if enabled {
        // Load the plugin
        let plugin_dir = plugins_dir().join(&plugin_id);
        let manifest_path = plugin_dir.join("manifest.toml");
        let index_path = plugin_dir.join("index.js");

        if manifest_path.exists() && index_path.exists() {
            let manifest_content = tokio::fs::read_to_string(&manifest_path).await?;
            let manifest: PluginManifest = toml::from_str(&manifest_content)?;
            let source = tokio::fs::read_to_string(&index_path).await?;
            state
                .load_plugin(manifest, &source)
                .map_err(AppError::Custom)?;
        }
    } else {
        // Unload the plugin
        state.unload_plugin(&plugin_id).ok();
    }

    Ok(())
}

/// Load and activate a plugin by its ID.
#[tauri::command]
#[specta::specta]
pub async fn load_plugin(
    plugin_id: String,
    state: State<'_, PluginHostState>,
) -> Result<(), AppError> {
    let plugin_dir = plugins_dir().join(&plugin_id);
    let manifest_path = plugin_dir.join("manifest.toml");
    let index_path = plugin_dir.join("index.js");

    if !manifest_path.exists() {
        return Err(AppError::FileNotFound(format!(
            "Plugin manifest not found: {}",
            manifest_path.display()
        )));
    }
    if !index_path.exists() {
        return Err(AppError::FileNotFound(format!(
            "Plugin entry point not found: {}",
            index_path.display()
        )));
    }

    let manifest_content = tokio::fs::read_to_string(&manifest_path).await?;
    let manifest: PluginManifest = toml::from_str(&manifest_content)?;

    let source = tokio::fs::read_to_string(&index_path).await?;

    state
        .load_plugin(manifest, &source)
        .map_err(AppError::Custom)?;

    Ok(())
}

/// Unload (deactivate) a plugin.
#[tauri::command]
#[specta::specta]
pub async fn unload_plugin(
    plugin_id: String,
    state: State<'_, PluginHostState>,
) -> Result<(), AppError> {
    state.unload_plugin(&plugin_id).map_err(AppError::Custom)?;
    Ok(())
}

/// Get all commands registered by active plugins.
#[tauri::command]
#[specta::specta]
pub async fn get_plugin_commands(
    state: State<'_, PluginHostState>,
) -> Result<Vec<RegisteredCommandInfo>, AppError> {
    Ok(state.get_registered_commands())
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct PluginReplacementResult {
    pub from: usize,
    pub to: usize,
    pub text: String,
}

/// Execute a registered plugin command. Returns any text replacements the plugin wants to make.
#[tauri::command]
#[specta::specta]
pub async fn invoke_plugin_command(
    plugin_id: String,
    command_id: String,
    state: State<'_, PluginHostState>,
) -> Result<Vec<PluginReplacementResult>, AppError> {
    let replacements = state
        .invoke_command(&plugin_id, &command_id)
        .map_err(AppError::Custom)?;

    Ok(replacements
        .into_iter()
        .map(|r| PluginReplacementResult {
            from: r.from,
            to: r.to,
            text: r.text,
        })
        .collect())
}

/// Update the document state that plugins can read.
#[tauri::command]
#[specta::specta]
pub async fn set_plugin_document_state(
    content: String,
    selection_from: u32,
    selection_to: u32,
    word_count: u32,
    state: State<'_, PluginHostState>,
) -> Result<(), AppError> {
    state.set_document_state(
        content,
        selection_from as usize,
        selection_to as usize,
        word_count as usize,
    );
    Ok(())
}
