//! Portable mode detection.
//!
//! Standard mode (no `portable.dat` next to exe): user data lives under
//! `~/.novelist/`. Portable mode (marker file present): user data lives under
//! `<exe_dir>/data/`, the updater plugin is skipped, and the UI shows a
//! banner so the user knows where their data is.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct PortableConfig {
    pub enabled: bool,
    pub data_root: PathBuf,
}

static CONFIG: OnceLock<PortableConfig> = OnceLock::new();

/// Initialize portable detection. Must be called once at startup before any
/// path-using code. Panics on portable mode if the data directory cannot be
/// created or is not writable — we never silently fall back to APPDATA.
pub fn init() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.canonicalize().ok())
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));
    let cfg = detect_with_exe_dir(&exe_dir);
    let _ = CONFIG.set(cfg);
}

/// Test seam: detect using a caller-provided exe directory.
pub fn detect_with_exe_dir(exe_dir: &Path) -> PortableConfig {
    let marker = exe_dir.join("portable.dat");
    if !marker.exists() {
        return PortableConfig {
            enabled: false,
            data_root: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("~"))
                .join(".novelist"),
        };
    }

    let data_root = exe_dir.join("data");
    std::fs::create_dir_all(&data_root).unwrap_or_else(|e| {
        panic!(
            "Portable mode: cannot create data directory at {}: {}. \
             Move Novelist out of Program Files or any read-only location.",
            data_root.display(),
            e
        )
    });

    let probe = data_root.join(".write-probe");
    std::fs::write(&probe, b"ok").unwrap_or_else(|e| {
        panic!(
            "Portable mode: data directory at {} is not writable: {}. \
             Move Novelist out of Program Files or any read-only location.",
            data_root.display(),
            e
        )
    });
    let _ = std::fs::remove_file(&probe);

    PortableConfig {
        enabled: true,
        data_root,
    }
}

pub fn config() -> &'static PortableConfig {
    CONFIG
        .get()
        .expect("portable::init() must be called before config()")
}

pub fn novelist_home() -> PathBuf {
    config().data_root.clone()
}

pub fn is_portable() -> bool {
    config().enabled
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn detect_no_marker_uses_home_dir() {
        let tmp = TempDir::new().unwrap();
        let cfg = detect_with_exe_dir(tmp.path());
        assert!(!cfg.enabled);
        let expected = dirs::home_dir().unwrap().join(".novelist");
        assert_eq!(cfg.data_root, expected);
    }

    #[test]
    fn detect_with_marker_writable_creates_data_dir() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("portable.dat"), b"").unwrap();
        let cfg = detect_with_exe_dir(tmp.path());
        assert!(cfg.enabled);
        assert_eq!(cfg.data_root, tmp.path().join("data"));
        assert!(cfg.data_root.is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn detect_with_marker_readonly_panics() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("portable.dat"), b"").unwrap();
        // Make the temp dir read-only so create_dir_all OR write fails.
        let mut perms = std::fs::metadata(tmp.path()).unwrap().permissions();
        perms.set_mode(0o555);
        std::fs::set_permissions(tmp.path(), perms.clone()).unwrap();

        let result = std::panic::catch_unwind(|| detect_with_exe_dir(tmp.path()));

        // Restore so TempDir can clean up.
        let mut restore = perms.clone();
        restore.set_mode(0o755);
        let _ = std::fs::set_permissions(tmp.path(), restore);

        assert!(result.is_err(), "expected panic on read-only directory");
    }
}
