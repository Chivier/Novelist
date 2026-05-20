use crate::error::AppError;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct SnapshotMeta {
    pub id: String,
    pub name: String,
    pub timestamp: u64,
    pub file_count: usize,
    pub total_bytes: u64,
}

/// Returns `<novelist_home>/snapshots/{blake3_hash_of_project_dir}/`,
/// where `novelist_home` is `~/.novelist/` in standard mode or
/// `<exe_dir>/data/` in portable mode.
pub fn snapshots_dir(project_dir: &str) -> PathBuf {
    let hash = blake3::hash(project_dir.as_bytes()).to_hex();
    data_root().join("snapshots").join(hash.to_string())
}

fn data_root() -> PathBuf {
    // `NOVELIST_SNAPSHOTS_DATA_DIR` is a test seam (used by unit tests that need
    // per-test isolation and can't rely on `portable::init()` having run).
    // Production code goes through `portable::novelist_home`.
    #[cfg(test)]
    {
        if let Ok(p) = std::env::var("NOVELIST_SNAPSHOTS_DATA_DIR") {
            if !p.is_empty() {
                return PathBuf::from(p);
            }
        }
    }
    crate::services::portable::novelist_home().to_path_buf()
}

fn validate_snapshot_id(id: &str) -> Result<(), AppError> {
    if !id.starts_with("snap-") || !id["snap-".len()..].chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::InvalidInput(format!(
            "Invalid snapshot ID: {}",
            id
        )));
    }
    Ok(())
}

/// Allowed extensions for snapshot
fn is_snapshot_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("md" | "markdown" | "txt" | "json" | "jsonl" | "csv")
    )
}

/// Create a named snapshot of all .md/.markdown/.txt files in the project.
pub async fn create_snapshot(project_dir: &str, name: &str) -> Result<SnapshotMeta, AppError> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::Custom(e.to_string()))?
        .as_secs();

    let snap_id = format!("snap-{}", timestamp);
    let snap_dir = snapshots_dir(project_dir).join(&snap_id);
    let files_dir = snap_dir.join("files");
    tokio::fs::create_dir_all(&files_dir).await?;

    let mut file_count: usize = 0;
    let mut total_bytes: u64 = 0;

    for entry in WalkDir::new(project_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if !path.is_file() || !is_snapshot_file(path) {
            continue;
        }

        let relative = path
            .strip_prefix(project_dir)
            .map_err(|e| AppError::Custom(e.to_string()))?;

        // Skip hidden directories (like .novelist, .git, etc.)
        if relative
            .components()
            .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
        {
            continue;
        }
        let dest = files_dir.join(relative);

        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = tokio::fs::read(path).await?;
        total_bytes += content.len() as u64;
        tokio::fs::write(&dest, &content).await?;
        file_count += 1;
    }

    let meta = SnapshotMeta {
        id: snap_id,
        name: name.to_string(),
        timestamp,
        file_count,
        total_bytes,
    };

    let meta_path = snap_dir.join("metadata.json");
    let meta_json = serde_json::to_string_pretty(&meta)?;
    tokio::fs::write(&meta_path, meta_json).await?;

    Ok(meta)
}

/// List all snapshots for a project, sorted newest first.
pub async fn list_snapshots(project_dir: &str) -> Result<Vec<SnapshotMeta>, AppError> {
    let base = snapshots_dir(project_dir);
    if !base.exists() {
        return Ok(vec![]);
    }

    let mut snapshots = Vec::new();
    let mut entries = tokio::fs::read_dir(&base).await?;

    while let Some(entry) = entries.next_entry().await? {
        let meta_path = entry.path().join("metadata.json");
        if meta_path.exists() {
            let content = tokio::fs::read_to_string(&meta_path).await?;
            if let Ok(meta) = serde_json::from_str::<SnapshotMeta>(&content) {
                snapshots.push(meta);
            }
        }
    }

    // Sort newest first
    snapshots.sort_by_key(|s| std::cmp::Reverse(s.timestamp));
    Ok(snapshots)
}

/// Restore a snapshot by overwriting project files with snapshot files.
pub async fn restore_snapshot(project_dir: &str, snapshot_id: &str) -> Result<(), AppError> {
    validate_snapshot_id(snapshot_id)?;
    let snap_dir = snapshots_dir(project_dir).join(snapshot_id);
    let files_dir = snap_dir.join("files");

    if !files_dir.exists() {
        return Err(AppError::Custom(format!(
            "Snapshot not found: {}",
            snapshot_id
        )));
    }

    let project_path = Path::new(project_dir);

    for entry in WalkDir::new(&files_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let relative = path
            .strip_prefix(&files_dir)
            .map_err(|e| AppError::Custom(e.to_string()))?;
        let dest = project_path.join(relative);

        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = tokio::fs::read(path).await?;
        tokio::fs::write(&dest, &content).await?;
    }

    Ok(())
}

/// Delete a snapshot directory.
pub async fn delete_snapshot(project_dir: &str, snapshot_id: &str) -> Result<(), AppError> {
    validate_snapshot_id(snapshot_id)?;
    let snap_dir = snapshots_dir(project_dir).join(snapshot_id);
    if snap_dir.exists() {
        tokio::fs::remove_dir_all(&snap_dir).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Tests that touch snapshots_dir mutate `NOVELIST_SNAPSHOTS_DATA_DIR`
    // (process-global env). Serialize so they don't race.
    static DATA_DIR_MUTEX: Mutex<()> = Mutex::new(());

    fn set_data_dir(p: &std::path::Path) -> Option<std::ffi::OsString> {
        let old = std::env::var_os("NOVELIST_SNAPSHOTS_DATA_DIR");
        unsafe {
            std::env::set_var("NOVELIST_SNAPSHOTS_DATA_DIR", p);
        }
        old
    }

    fn restore_data_dir(old: Option<std::ffi::OsString>) {
        if let Some(v) = old {
            unsafe {
                std::env::set_var("NOVELIST_SNAPSHOTS_DATA_DIR", v);
            }
        } else {
            unsafe {
                std::env::remove_var("NOVELIST_SNAPSHOTS_DATA_DIR");
            }
        }
    }

    fn lock_data_dir() -> std::sync::MutexGuard<'static, ()> {
        DATA_DIR_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[tokio::test]
    async fn test_snapshots_dir_is_deterministic() {
        let _guard = lock_data_dir();
        let tmp = TempDir::new().unwrap();
        let old = set_data_dir(tmp.path());
        let d1 = snapshots_dir("/home/user/novel");
        let d2 = snapshots_dir("/home/user/novel");
        assert_eq!(d1, d2);
        restore_data_dir(old);
    }

    #[tokio::test]
    async fn test_create_and_list_snapshot() {
        let _guard = lock_data_dir();
        let data_tmp = TempDir::new().unwrap();
        let old = set_data_dir(data_tmp.path());

        let dir = TempDir::new().unwrap();
        let project = dir.path().to_string_lossy().to_string();

        // Create some files
        std::fs::write(dir.path().join("chapter1.md"), "# Chapter 1").unwrap();
        std::fs::write(dir.path().join("notes.txt"), "Some notes").unwrap();
        std::fs::write(dir.path().join("image.png"), [0u8; 100]).unwrap();

        let meta = create_snapshot(&project, "First draft").await.unwrap();
        assert_eq!(meta.name, "First draft");
        assert_eq!(meta.file_count, 2);
        assert!(meta.id.starts_with("snap-"));

        let list = list_snapshots(&project).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, meta.id);

        restore_data_dir(old);
    }

    #[tokio::test]
    async fn test_restore_snapshot() {
        let _guard = lock_data_dir();
        let data_tmp = TempDir::new().unwrap();
        let old = set_data_dir(data_tmp.path());

        let dir = TempDir::new().unwrap();
        let project = dir.path().to_string_lossy().to_string();

        std::fs::write(dir.path().join("chapter1.md"), "Original").unwrap();
        let meta = create_snapshot(&project, "Before edit").await.unwrap();

        // Modify the file
        std::fs::write(dir.path().join("chapter1.md"), "Modified").unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.path().join("chapter1.md")).unwrap(),
            "Modified"
        );

        // Restore
        restore_snapshot(&project, &meta.id).await.unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.path().join("chapter1.md")).unwrap(),
            "Original"
        );

        restore_data_dir(old);
    }

    #[tokio::test]
    async fn test_delete_snapshot() {
        let _guard = lock_data_dir();
        let data_tmp = TempDir::new().unwrap();
        let old = set_data_dir(data_tmp.path());

        let dir = TempDir::new().unwrap();
        let project = dir.path().to_string_lossy().to_string();

        std::fs::write(dir.path().join("test.md"), "content").unwrap();
        let meta = create_snapshot(&project, "temp").await.unwrap();

        let list = list_snapshots(&project).await.unwrap();
        assert_eq!(list.len(), 1);

        delete_snapshot(&project, &meta.id).await.unwrap();

        let list = list_snapshots(&project).await.unwrap();
        assert_eq!(list.len(), 0);

        restore_data_dir(old);
    }

    #[tokio::test]
    async fn test_list_empty() {
        let _guard = lock_data_dir();
        let data_tmp = TempDir::new().unwrap();
        let old = set_data_dir(data_tmp.path());

        let dir = TempDir::new().unwrap();
        let project = dir.path().to_string_lossy().to_string();
        let list = list_snapshots(&project).await.unwrap();
        assert!(list.is_empty());

        restore_data_dir(old);
    }
}
