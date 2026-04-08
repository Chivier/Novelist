use crate::error::AppError;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    pub last_opened: String,
}

fn recent_projects_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".novelist")
        .join("recent-projects.json")
}

#[tauri::command]
#[specta::specta]
pub async fn get_recent_projects() -> Result<Vec<RecentProject>, AppError> {
    let path = recent_projects_path();
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = tokio::fs::read_to_string(&path).await?;
    let projects: Vec<RecentProject> = serde_json::from_str(&content).unwrap_or_default();
    Ok(projects)
}

#[tauri::command]
#[specta::specta]
pub async fn add_recent_project(path: String, name: String) -> Result<(), AppError> {
    let file_path = recent_projects_path();

    // Ensure ~/.novelist/ exists
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Read existing
    let mut projects: Vec<RecentProject> = if file_path.exists() {
        let content = tokio::fs::read_to_string(&file_path).await?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        vec![]
    };

    // Remove if already exists (will re-add at top)
    projects.retain(|p| p.path != path);

    // Add at front
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    projects.insert(
        0,
        RecentProject {
            path,
            name,
            last_opened: timestamp,
        },
    );

    // Keep max 20
    projects.truncate(20);

    // Write
    let json = serde_json::to_string_pretty(&projects)?;
    tokio::fs::write(&file_path, json).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_project_serialize() {
        let p = RecentProject {
            path: "/home/user/novel".to_string(),
            name: "My Novel".to_string(),
            last_opened: "1700000000".to_string(),
        };
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("My Novel"));

        let parsed: RecentProject = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "My Novel");
        assert_eq!(parsed.path, "/home/user/novel");
    }

    #[test]
    fn test_recent_projects_list_serialize() {
        let projects = vec![
            RecentProject {
                path: "/a".to_string(),
                name: "A".to_string(),
                last_opened: "100".to_string(),
            },
            RecentProject {
                path: "/b".to_string(),
                name: "B".to_string(),
                last_opened: "200".to_string(),
            },
        ];
        let json = serde_json::to_string_pretty(&projects).unwrap();
        let parsed: Vec<RecentProject> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].name, "A");
        assert_eq!(parsed[1].name, "B");
    }

    #[test]
    fn test_dedup_and_truncate_logic() {
        let mut projects: Vec<RecentProject> = (0..25)
            .map(|i| RecentProject {
                path: format!("/project/{}", i),
                name: format!("P{}", i),
                last_opened: i.to_string(),
            })
            .collect();

        // Simulate adding existing project — should move to front
        let existing_path = "/project/10".to_string();
        projects.retain(|p| p.path != existing_path);
        projects.insert(
            0,
            RecentProject {
                path: existing_path.clone(),
                name: "P10-updated".to_string(),
                last_opened: "999".to_string(),
            },
        );
        projects.truncate(20);

        assert_eq!(projects.len(), 20);
        assert_eq!(projects[0].path, "/project/10");
        assert_eq!(projects[0].name, "P10-updated");
        // Original P10 should be gone (deduped)
        assert_eq!(
            projects.iter().filter(|p| p.path == "/project/10").count(),
            1
        );
    }
}
