//! Effective settings commands.
//!
//! Global defaults live in `~/.novelist/settings.json`; per-project overrides
//! live in `<project>/.novelist/project.toml`'s `[view] / [new_file] /
//! [plugins]` sections. `get_effective_settings` merges them (project wins);
//! `write_project_settings` / `write_global_settings` each patch one file.
//!
//! Section granularity: frontend passes `Option<Section>` — `Some(..)` means
//! "replace this section", `None` means "leave it alone". To delete an
//! override, pass `Some(Section::default())` (all fields None / empty map).

use crate::error::AppError;
use crate::models::project::ProjectConfig;
use crate::models::settings::{
    resolve, EffectiveSettings, GlobalSettings, NewFileConfig, PluginsConfig, ViewConfig,
};
use std::path::{Path, PathBuf};

fn global_settings_path() -> PathBuf {
    // `NOVELIST_DATA_DIR` is a test seam (used by unit tests that need
    // per-test isolation and can't rely on `portable::init()` having run).
    // Production code goes through `portable::novelist_home`.
    #[cfg(test)]
    {
        if let Ok(p) = std::env::var("NOVELIST_DATA_DIR") {
            if !p.is_empty() {
                return PathBuf::from(p).join("settings.json");
            }
        }
    }
    crate::services::portable::novelist_home().join("settings.json")
}

pub(crate) async fn read_global_settings() -> GlobalSettings {
    let path = global_settings_path();
    if !path.exists() {
        return GlobalSettings::default();
    }
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => GlobalSettings::default(),
    }
}

pub(crate) async fn write_global_settings_to_disk(
    settings: &GlobalSettings,
) -> Result<(), AppError> {
    let path = global_settings_path();
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| AppError::Custom(format!("Failed to serialize global settings: {e}")))?;
    let temp = format!("{}.novelist-tmp", path.display());
    tokio::fs::write(&temp, json).await?;
    tokio::fs::rename(&temp, &path).await?;
    Ok(())
}

async fn read_project_config_if_any(dir_path: &str) -> Option<ProjectConfig> {
    let cfg_path = Path::new(dir_path).join(".novelist").join("project.toml");
    if !cfg_path.exists() {
        return None;
    }
    let content = tokio::fs::read_to_string(&cfg_path).await.ok()?;
    toml::from_str(&content).ok()
}

async fn write_project_config(dir_path: &str, config: &ProjectConfig) -> Result<(), AppError> {
    let novelist_dir = Path::new(dir_path).join(".novelist");
    if !novelist_dir.exists() {
        tokio::fs::create_dir_all(&novelist_dir).await?;
    }
    let cfg_path = novelist_dir.join("project.toml");
    let serialized = toml::to_string(config)
        .map_err(|e| AppError::Custom(format!("Failed to serialize project config: {e}")))?;
    let temp = format!("{}.novelist-tmp", cfg_path.display());
    tokio::fs::write(&temp, serialized).await?;
    tokio::fs::rename(&temp, &cfg_path).await?;
    Ok(())
}

/// Return raw global defaults from `~/.novelist/settings.json` without
/// merging in any project overlay. Used by the frontend to compute plugin
/// delta overrides (only entries differing from global are persisted).
#[tauri::command]
#[specta::specta]
pub async fn get_global_settings() -> Result<GlobalSettings, AppError> {
    Ok(read_global_settings().await)
}

/// Return effective settings, merging global defaults with an optional
/// project overlay. `dir_path = None` returns global-only (scratch mode).
#[tauri::command]
#[specta::specta]
pub async fn get_effective_settings(
    dir_path: Option<String>,
) -> Result<EffectiveSettings, AppError> {
    let global = read_global_settings().await;
    let (view, new_file, plugins) = match dir_path {
        Some(d) => {
            let project = read_project_config_if_any(&d).await;
            match project {
                Some(p) => (Some(p.view), Some(p.new_file), Some(p.plugins)),
                None => (None, None, None),
            }
        }
        None => (None, None, None),
    };
    Ok(resolve(
        &global,
        view.as_ref(),
        new_file.as_ref(),
        plugins.as_ref(),
    ))
}

/// Patch the global `~/.novelist/settings.json`. Only the `Some(..)` sections
/// are replaced; unspecified sections keep their current value on disk.
#[tauri::command]
#[specta::specta]
pub async fn write_global_settings(
    view: Option<ViewConfig>,
    new_file: Option<NewFileConfig>,
    plugins: Option<PluginsConfig>,
) -> Result<(), AppError> {
    let mut current = read_global_settings().await;
    if let Some(v) = view {
        current.view = v;
    }
    if let Some(n) = new_file {
        current.new_file = n;
    }
    if let Some(p) = plugins {
        current.plugins = p;
    }
    write_global_settings_to_disk(&current).await
}

/// Patch `<dir>/.novelist/project.toml`. Only the provided sections are replaced.
/// Requires that a `project.toml` already exists (i.e. `dir` is a Novelist project).
#[tauri::command]
#[specta::specta]
pub async fn write_project_settings(
    dir_path: String,
    view: Option<ViewConfig>,
    new_file: Option<NewFileConfig>,
    plugins: Option<PluginsConfig>,
) -> Result<(), AppError> {
    let mut config = read_project_config_if_any(&dir_path)
        .await
        .ok_or_else(|| AppError::FileNotFound(format!("{}/.novelist/project.toml", dir_path)))?;
    if let Some(v) = view {
        config.view = v;
    }
    if let Some(n) = new_file {
        config.new_file = n;
    }
    if let Some(p) = plugins {
        config.plugins = p;
    }
    write_project_config(&dir_path, &config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Tests that touch the global settings path mutate `NOVELIST_DATA_DIR`
    // (process-global env). Serialize through this mutex so they don't race.
    static DATA_DIR_MUTEX: Mutex<()> = Mutex::new(());

    fn write_minimal_project(dir: &TempDir, overlay_toml: &str) {
        let novelist_dir = dir.path().join(".novelist");
        std::fs::create_dir(&novelist_dir).unwrap();
        let content = format!("[project]\nname = \"T\"\n\n{overlay_toml}");
        std::fs::write(novelist_dir.join("project.toml"), content).unwrap();
    }

    fn set_data_dir(p: &Path) -> Option<std::ffi::OsString> {
        let old = std::env::var_os("NOVELIST_DATA_DIR");
        std::env::set_var("NOVELIST_DATA_DIR", p);
        old
    }

    fn restore_data_dir(old: Option<std::ffi::OsString>) {
        if let Some(v) = old {
            std::env::set_var("NOVELIST_DATA_DIR", v);
        } else {
            std::env::remove_var("NOVELIST_DATA_DIR");
        }
    }

    #[tokio::test]
    async fn write_project_settings_patches_view_and_leaves_new_file_untouched() {
        let dir = TempDir::new().unwrap();
        write_minimal_project(
            &dir,
            r#"[new_file]
template = "Chapter {N}"
"#,
        );
        let dir_str = dir.path().to_string_lossy().to_string();

        write_project_settings(
            dir_str.clone(),
            Some(ViewConfig {
                sort_mode: Some("name-desc".into()),
                show_hidden_files: Some(true),
                wrap_file_names: None,
            }),
            None,
            None,
        )
        .await
        .unwrap();

        let config = read_project_config_if_any(&dir_str).await.unwrap();
        assert_eq!(config.view.sort_mode.as_deref(), Some("name-desc"));
        assert_eq!(config.view.show_hidden_files, Some(true));
        assert_eq!(
            config.new_file.template.as_deref(),
            Some("Chapter {N}"),
            "existing new_file template must be preserved"
        );
    }

    #[tokio::test]
    async fn get_effective_settings_merges_global_with_project_overlay() {
        let _guard = DATA_DIR_MUTEX.lock().unwrap();
        let global_tmp = TempDir::new().unwrap();
        let old = set_data_dir(global_tmp.path());

        let dir = TempDir::new().unwrap();
        write_minimal_project(
            &dir,
            r#"[view]
show_hidden_files = true
"#,
        );
        // Can't write ~/.novelist/settings.json in tests — exercise directly via
        // resolve() in the settings model (covered in models/settings.rs tests).
        // This test checks that project overlay flows through the command.
        let eff = get_effective_settings(Some(dir.path().to_string_lossy().into()))
            .await
            .unwrap();
        assert!(eff.view.show_hidden_files);
        assert!(eff.is_project_scoped);

        restore_data_dir(old);
    }

    #[tokio::test]
    async fn get_effective_settings_scratch_mode_is_not_project_scoped() {
        let _guard = DATA_DIR_MUTEX.lock().unwrap();
        let global_tmp = TempDir::new().unwrap();
        let old = set_data_dir(global_tmp.path());

        let eff = get_effective_settings(None).await.unwrap();
        assert!(!eff.is_project_scoped);

        restore_data_dir(old);
    }

    #[tokio::test]
    async fn write_project_settings_errors_without_project_toml() {
        let dir = TempDir::new().unwrap();
        let res = write_project_settings(
            dir.path().to_string_lossy().to_string(),
            Some(ViewConfig::default()),
            None,
            None,
        )
        .await;
        assert!(matches!(res, Err(AppError::FileNotFound(_))));
    }

    #[tokio::test]
    async fn write_project_settings_roundtrip_survives_existing_fields() {
        // Verifies the whole-config roundtrip: patching [view] must not clobber
        // [project] or [writing].
        let dir = TempDir::new().unwrap();
        let novelist_dir = dir.path().join(".novelist");
        std::fs::create_dir(&novelist_dir).unwrap();
        std::fs::write(
            novelist_dir.join("project.toml"),
            r#"[project]
name = "Preserve Me"
type = "novel"
version = "0.1.0"

[writing]
daily_goal = 1234
auto_save_minutes = 7
"#,
        )
        .unwrap();

        let dir_str = dir.path().to_string_lossy().to_string();
        write_project_settings(
            dir_str.clone(),
            Some(ViewConfig {
                sort_mode: Some("mtime-desc".into()),
                show_hidden_files: None,
                wrap_file_names: Some(true),
            }),
            None,
            None,
        )
        .await
        .unwrap();

        let config = read_project_config_if_any(&dir_str).await.unwrap();
        assert_eq!(config.project.name, "Preserve Me");
        assert_eq!(config.writing.daily_goal, 1234);
        assert_eq!(config.writing.auto_save_minutes, 7);
        assert_eq!(config.view.sort_mode.as_deref(), Some("mtime-desc"));
        assert_eq!(config.view.wrap_file_names, Some(true));
    }
}
