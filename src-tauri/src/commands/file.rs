use crate::error::AppError;
use crate::models::file_state::FileEntry;
use std::path::Path;

#[tauri::command]
#[specta::specta]
pub async fn read_file(path: String) -> Result<String, AppError> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(AppError::FileNotFound(path));
    }
    let content = tokio::fs::read_to_string(&path).await?;
    Ok(content)
}

#[tauri::command]
#[specta::specta]
pub async fn write_file(path: String, content: String) -> Result<(), AppError> {
    tracing::info!(
        "[write_file] path={}, content_bytes={}, content_lines={}",
        path, content.len(), content.lines().count()
    );
    let temp_path = format!("{}.novelist-tmp", path);
    tokio::fs::write(&temp_path, &content).await?;
    tokio::fs::rename(&temp_path, &path).await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn list_directory(path: String) -> Result<Vec<FileEntry>, AppError> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(AppError::FileNotFound(path));
    }
    if !p.is_dir() {
        return Err(AppError::NotADirectory(path));
    }

    let mut entries = Vec::new();
    let mut read_dir = tokio::fs::read_dir(&path).await?;

    while let Some(entry) = read_dir.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/dirs, except .novelist
        if name.starts_with('.') && name != ".novelist" {
            continue;
        }

        let metadata = entry.metadata().await?;
        entries.push(FileEntry {
            name,
            path: entry.path().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
        });
    }

    // Sort: directories first, then alphabetical by name
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

#[tauri::command]
#[specta::specta]
pub async fn create_file(parent_dir: String, filename: String) -> Result<String, AppError> {
    let file_path = Path::new(&parent_dir).join(&filename);
    if file_path.exists() {
        return Err(AppError::Custom(format!(
            "File already exists: {}",
            file_path.display()
        )));
    }
    tokio::fs::write(&file_path, "").await?;
    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn create_directory(parent_dir: String, name: String) -> Result<String, AppError> {
    let dir_path = Path::new(&parent_dir).join(&name);
    if dir_path.exists() {
        return Err(AppError::Custom(format!(
            "Directory already exists: {}",
            dir_path.display()
        )));
    }
    tokio::fs::create_dir(&dir_path).await?;
    Ok(dir_path.to_string_lossy().to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn rename_item(old_path: String, new_name: String) -> Result<String, AppError> {
    let old = Path::new(&old_path);
    if !old.exists() {
        return Err(AppError::FileNotFound(old_path));
    }
    let new_path = old
        .parent()
        .ok_or_else(|| AppError::Custom("Cannot determine parent directory".to_string()))?
        .join(&new_name);
    if new_path.exists() {
        return Err(AppError::Custom(format!(
            "Already exists: {}",
            new_path.display()
        )));
    }
    tokio::fs::rename(&old, &new_path).await?;
    Ok(new_path.to_string_lossy().to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_item(path: String) -> Result<(), AppError> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(AppError::FileNotFound(path));
    }
    if p.is_dir() {
        tokio::fs::remove_dir_all(&path).await?;
    } else {
        tokio::fs::remove_file(&path).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_read_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.md");
        fs::write(&file_path, "# Hello\n\nWorld").unwrap();
        let content = read_file(file_path.to_string_lossy().to_string())
            .await
            .unwrap();
        assert_eq!(content, "# Hello\n\nWorld");
    }

    #[tokio::test]
    async fn test_read_file_not_found() {
        let result = read_file("/nonexistent/path.md".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_write_file_atomic() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("output.md");
        write_file(
            file_path.to_string_lossy().to_string(),
            "# New Content".to_string(),
        )
        .await
        .unwrap();
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "# New Content");
        let temp_path = format!("{}.novelist-tmp", file_path.to_string_lossy());
        assert!(!std::path::Path::new(&temp_path).exists());
    }

    #[tokio::test]
    async fn test_list_directory() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("b.md"), "").unwrap();
        fs::write(dir.path().join("a.md"), "").unwrap();
        fs::create_dir(dir.path().join("chapters")).unwrap();
        fs::write(dir.path().join(".hidden"), "").unwrap();
        let entries = list_directory(dir.path().to_string_lossy().to_string())
            .await
            .unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].name, "chapters");
        assert!(entries[0].is_dir);
        assert_eq!(entries[1].name, "a.md");
        assert_eq!(entries[2].name, "b.md");
    }

    #[tokio::test]
    async fn test_list_directory_shows_novelist_dir() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join(".novelist")).unwrap();
        fs::write(dir.path().join(".other_hidden"), "").unwrap();
        let entries = list_directory(dir.path().to_string_lossy().to_string())
            .await
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, ".novelist");
    }

    #[tokio::test]
    async fn test_list_directory_not_found() {
        let result = list_directory("/nonexistent/dir".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_directory_not_a_dir() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("file.txt");
        fs::write(&file_path, "").unwrap();
        let result = list_directory(file_path.to_string_lossy().to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_file() {
        let dir = TempDir::new().unwrap();
        let result = create_file(
            dir.path().to_string_lossy().to_string(),
            "new.md".to_string(),
        )
        .await
        .unwrap();
        assert!(result.ends_with("new.md"));
        assert!(Path::new(&result).exists());
        assert_eq!(fs::read_to_string(&result).unwrap(), "");
    }

    #[tokio::test]
    async fn test_create_file_already_exists() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("exists.md"), "content").unwrap();
        let result = create_file(
            dir.path().to_string_lossy().to_string(),
            "exists.md".to_string(),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_directory() {
        let dir = TempDir::new().unwrap();
        let result = create_directory(
            dir.path().to_string_lossy().to_string(),
            "chapters".to_string(),
        )
        .await
        .unwrap();
        assert!(Path::new(&result).is_dir());
    }

    #[tokio::test]
    async fn test_create_directory_already_exists() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("chapters")).unwrap();
        let result = create_directory(
            dir.path().to_string_lossy().to_string(),
            "chapters".to_string(),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rename_item() {
        let dir = TempDir::new().unwrap();
        let old_path = dir.path().join("old.md");
        fs::write(&old_path, "content").unwrap();
        let new_path = rename_item(
            old_path.to_string_lossy().to_string(),
            "new.md".to_string(),
        )
        .await
        .unwrap();
        assert!(!old_path.exists());
        assert!(Path::new(&new_path).exists());
        assert_eq!(fs::read_to_string(&new_path).unwrap(), "content");
    }

    #[tokio::test]
    async fn test_rename_item_not_found() {
        let result = rename_item("/nonexistent.md".to_string(), "new.md".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rename_item_target_exists() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.md"), "").unwrap();
        fs::write(dir.path().join("b.md"), "").unwrap();
        let result = rename_item(
            dir.path().join("a.md").to_string_lossy().to_string(),
            "b.md".to_string(),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_file() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("delete_me.md");
        fs::write(&file, "content").unwrap();
        delete_item(file.to_string_lossy().to_string())
            .await
            .unwrap();
        assert!(!file.exists());
    }

    #[tokio::test]
    async fn test_delete_directory() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("file.md"), "").unwrap();
        delete_item(sub.to_string_lossy().to_string())
            .await
            .unwrap();
        assert!(!sub.exists());
    }

    #[tokio::test]
    async fn test_delete_item_not_found() {
        let result = delete_item("/nonexistent.md".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_write_file_creates_new() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("new.md");
        write_file(
            file_path.to_string_lossy().to_string(),
            "# Title\n\nBody".to_string(),
        )
        .await
        .unwrap();
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "# Title\n\nBody");
    }

    #[tokio::test]
    async fn test_write_file_overwrites() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("existing.md");
        fs::write(&file_path, "old content").unwrap();
        write_file(
            file_path.to_string_lossy().to_string(),
            "new content".to_string(),
        )
        .await
        .unwrap();
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "new content");
    }

    #[tokio::test]
    async fn test_read_file_utf8_cjk() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("cjk.md");
        let content = "# 第一章\n\n落霞与孤鹜齐飞，秋水共长天一色。";
        fs::write(&file_path, content).unwrap();
        let result = read_file(file_path.to_string_lossy().to_string())
            .await
            .unwrap();
        assert_eq!(result, content);
    }
}

#[cfg(test)]
mod large_file_tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_read_large_file_150k_lines() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("large.md");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            for i in 1..=150000 {
                writeln!(f, "Line {} of 150000", i).unwrap();
            }
        }
        let file_size = std::fs::metadata(&path).unwrap().len();
        println!("File size: {} bytes ({:.1} MB)", file_size, file_size as f64 / 1e6);

        let content = read_file(path.to_string_lossy().to_string()).await.unwrap();
        let line_count = content.lines().count();
        println!("Read {} lines, {} bytes", line_count, content.len());
        assert_eq!(line_count, 150000, "Line count mismatch: expected 150000, got {}", line_count);

        let last_line = content.lines().last().unwrap();
        assert_eq!(last_line, "Line 150000 of 150000", "Last line wrong: {}", last_line);
        println!("Last line: {}", last_line);
        println!("✓ readFile returns all 150000 lines");
    }
}
