//! Project-scoped AI assets and sessions.
//!
//! These commands intentionally store JSON/Markdown blobs without knowing the
//! frontend schema. The AI panels evolve faster than the Rust core; Rust owns
//! path safety and atomic writes, while TypeScript owns session shape.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
pub enum AiSessionKind {
    Talk,
    Agent,
}

impl AiSessionKind {
    fn prefix(self) -> &'static str {
        match self {
            Self::Talk => "talk",
            Self::Agent => "agent",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AiSessionFile {
    pub id: String,
    pub kind: AiSessionKind,
    pub path: String,
    pub updated_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AiPromptAsset {
    pub id: String,
    pub kind: String,
    pub path: String,
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AiPromptAssets {
    pub commands: Vec<AiPromptAsset>,
    pub skills: Vec<AiPromptAsset>,
    pub memory: Option<AiPromptAsset>,
}

fn validate_project_dir(project_dir: &str) -> Result<PathBuf, AppError> {
    if project_dir.trim().is_empty() {
        return Err(AppError::InvalidInput("project_dir is required".into()));
    }
    Ok(PathBuf::from(project_dir))
}

fn validate_id(id: &str) -> Result<(), AppError> {
    if id.is_empty() || id.len() > 96 {
        return Err(AppError::InvalidInput(format!(
            "AI session id must be 1..=96 chars: {id}"
        )));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::InvalidInput(format!(
            "AI session id contains illegal characters: {id}"
        )));
    }
    Ok(())
}

fn ai_dir(project_dir: &str) -> Result<PathBuf, AppError> {
    Ok(validate_project_dir(project_dir)?
        .join(".novelist")
        .join("ai"))
}

fn sessions_dir(project_dir: &str) -> Result<PathBuf, AppError> {
    Ok(ai_dir(project_dir)?.join("sessions"))
}

fn session_path(project_dir: &str, kind: AiSessionKind, id: &str) -> Result<PathBuf, AppError> {
    validate_id(id)?;
    Ok(sessions_dir(project_dir)?.join(format!("{}-{id}.json", kind.prefix())))
}

fn atomic_write(path: &Path, content: &str) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!(
        "{}.novelist-tmp",
        path.extension().and_then(|s| s.to_str()).unwrap_or("tmp")
    ));
    fs::write(&tmp, content)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn list_ai_sessions(
    project_dir: String,
    kind: AiSessionKind,
) -> Result<Vec<AiSessionFile>, AppError> {
    let dir = sessions_dir(&project_dir)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let prefix = format!("{}-", kind.prefix());
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !name.starts_with(&prefix) || !name.ends_with(".json") {
            continue;
        }
        let id = name[prefix.len()..name.len() - ".json".len()].to_string();
        if validate_id(&id).is_err() {
            continue;
        }
        let updated_at = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64);
        out.push(AiSessionFile {
            id,
            kind,
            path: path.to_string_lossy().to_string(),
            updated_at,
        });
    }
    out.sort_by(|a, b| {
        b.updated_at
            .cmp(&a.updated_at)
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(out)
}

#[tauri::command]
#[specta::specta]
pub fn read_ai_session(
    project_dir: String,
    kind: AiSessionKind,
    id: String,
) -> Result<Option<String>, AppError> {
    let path = session_path(&project_dir, kind, &id)?;
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(fs::read_to_string(path)?))
}

#[tauri::command]
#[specta::specta]
pub fn write_ai_session(
    project_dir: String,
    kind: AiSessionKind,
    id: String,
    body_json: String,
) -> Result<(), AppError> {
    // Validate JSON early so corrupted session files do not get produced by
    // accidental callers.
    serde_json::from_str::<serde_json::Value>(&body_json)
        .map_err(|e| AppError::InvalidInput(format!("Invalid AI session JSON: {e}")))?;
    let path = session_path(&project_dir, kind, &id)?;
    atomic_write(&path, &body_json)
}

#[tauri::command]
#[specta::specta]
pub fn delete_ai_session(
    project_dir: String,
    kind: AiSessionKind,
    id: String,
) -> Result<(), AppError> {
    let path = session_path(&project_dir, kind, &id)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn read_asset_file(path: &Path, root: &Path, kind: &str) -> Option<AiPromptAsset> {
    if !path.is_file() || path.file_name()?.to_str()?.starts_with('.') {
        return None;
    }
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    if ext != "md" {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    let rel = path.strip_prefix(root).ok()?.to_string_lossy().to_string();
    let name = path.file_stem()?.to_string_lossy().to_string();
    Some(AiPromptAsset {
        id: rel.replace('\\', "/"),
        kind: kind.to_string(),
        path: path.to_string_lossy().to_string(),
        name,
        content,
    })
}

fn collect_markdown_files(dir: &Path, root: &Path, kind: &str) -> Vec<AiPromptAsset> {
    let mut out = Vec::new();
    if !dir.exists() {
        return out;
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
        let Ok(entries) = fs::read_dir(&cur) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            if name.to_string_lossy().starts_with('.') {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else if let Some(asset) = read_asset_file(&path, root, kind) {
                out.push(asset);
            }
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

#[tauri::command]
#[specta::specta]
pub fn list_ai_prompt_assets(project_dir: String) -> Result<AiPromptAssets, AppError> {
    let root = ai_dir(&project_dir)?;
    let commands = collect_markdown_files(&root.join("commands"), &root, "command");
    let skills = collect_markdown_files(&root.join("skills"), &root, "skill");
    let memory_path = root.join("memory.md");
    let memory = if memory_path.exists() {
        read_asset_file(&memory_path, &root, "memory")
    } else {
        None
    };
    Ok(AiPromptAssets {
        commands,
        skills,
        memory,
    })
}

#[tauri::command]
#[specta::specta]
pub fn write_ai_memory(project_dir: String, body: String) -> Result<(), AppError> {
    let path = ai_dir(&project_dir)?.join("memory.md");
    atomic_write(&path, &body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn rejects_path_traversal_ids() {
        let dir = tempdir().unwrap();
        let err = write_ai_session(
            dir.path().to_string_lossy().to_string(),
            AiSessionKind::Talk,
            "../bad".into(),
            "{}".into(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("illegal"));
    }

    #[test]
    fn write_read_list_delete_session_roundtrip() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_string_lossy().to_string();
        write_ai_session(
            root.clone(),
            AiSessionKind::Agent,
            "abc_123".into(),
            "{\"x\":1}".into(),
        )
        .unwrap();
        let list = list_ai_sessions(root.clone(), AiSessionKind::Agent).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "abc_123");
        let body = read_ai_session(root.clone(), AiSessionKind::Agent, "abc_123".into()).unwrap();
        assert_eq!(body.as_deref(), Some("{\"x\":1}"));
        delete_ai_session(root.clone(), AiSessionKind::Agent, "abc_123".into()).unwrap();
        let body = read_ai_session(root, AiSessionKind::Agent, "abc_123".into()).unwrap();
        assert!(body.is_none());
    }

    #[test]
    fn prompt_assets_skip_hidden_and_unsupported() {
        let dir = tempdir().unwrap();
        let root = dir.path().join(".novelist").join("ai");
        fs::create_dir_all(root.join("commands")).unwrap();
        fs::create_dir_all(root.join("skills").join("line-editor")).unwrap();
        fs::write(root.join("commands").join("rewrite.md"), "rewrite").unwrap();
        fs::write(root.join("commands").join(".hidden.md"), "hidden").unwrap();
        fs::write(root.join("commands").join("notes.txt"), "ignored").unwrap();
        fs::write(
            root.join("skills").join("line-editor").join("SKILL.md"),
            "skill",
        )
        .unwrap();
        fs::write(root.join("memory.md"), "memory").unwrap();

        let assets = list_ai_prompt_assets(dir.path().to_string_lossy().to_string()).unwrap();
        assert_eq!(assets.commands.len(), 1);
        assert_eq!(assets.commands[0].name, "rewrite");
        assert_eq!(assets.skills.len(), 1);
        assert_eq!(assets.memory.as_ref().unwrap().content, "memory");
    }

    #[test]
    fn write_memory_overwrites_memory_md() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_string_lossy().to_string();
        write_ai_memory(root.clone(), "first".into()).unwrap();
        write_ai_memory(root.clone(), "second".into()).unwrap();
        let assets = list_ai_prompt_assets(root).unwrap();
        assert_eq!(assets.memory.unwrap().content, "second");
    }
}
