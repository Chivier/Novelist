use crate::error::AppError;
use crate::models::template::{TemplateInfo, TemplateMeta};
use crate::services::template_scaffold::{self, ScaffoldLocale};
use std::path::{Path, PathBuf};

/// Returns the base directory for user templates: ~/.novelist/templates/
fn templates_dir() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Custom("Cannot determine home directory".into()))?;
    Ok(home.join(".novelist").join("templates"))
}

/// Built-in templates that are always available (generated in-memory, not on disk).
///
/// The `name` / `description` fields are English-only fallbacks — the
/// frontend renders these templates via i18n keys (`template.<id>.name`
/// etc.) and only falls back to these strings if a key is missing.
fn builtin_templates() -> Vec<TemplateInfo> {
    vec![
        TemplateInfo {
            id: "blank".into(),
            name: "Blank".into(),
            description: "Empty project with default settings".into(),
            category: "general".into(),
            builtin: true,
        },
        TemplateInfo {
            id: "novel".into(),
            name: "Novel".into(),
            description: "Novel project with chapter structure".into(),
            category: "fiction".into(),
            builtin: true,
        },
        TemplateInfo {
            id: "long-novel".into(),
            name: "Long Novel".into(),
            description: "Multi-volume novel with character profiles, world-building notes, and chapter folders".into(),
            category: "fiction".into(),
            builtin: true,
        },
        TemplateInfo {
            id: "short-story".into(),
            name: "Short Story".into(),
            description: "Short story with a single manuscript file and planning notes".into(),
            category: "fiction".into(),
            builtin: true,
        },
        TemplateInfo {
            id: "screenplay".into(),
            name: "Screenplay".into(),
            description: "Three-act screenplay with character list and scene structure".into(),
            category: "fiction".into(),
            builtin: true,
        },
        TemplateInfo {
            id: "blog".into(),
            name: "Blog".into(),
            description: "Blog with posts and drafts folders".into(),
            category: "non-fiction".into(),
            builtin: true,
        },
        TemplateInfo {
            id: "journal".into(),
            name: "Journal".into(),
            description: "Daily journal with date-based files".into(),
            category: "personal".into(),
            builtin: true,
        },
    ]
}

#[tauri::command]
#[specta::specta]
pub async fn list_templates() -> Result<Vec<TemplateInfo>, AppError> {
    let mut templates = builtin_templates();

    let dir = templates_dir()?;
    if dir.exists() {
        let mut entries = tokio::fs::read_dir(&dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let meta_path = path.join("template.toml");
            if meta_path.exists() {
                let content = tokio::fs::read_to_string(&meta_path).await?;
                if let Ok(meta) = toml::from_str::<TemplateMeta>(&content) {
                    templates.push(TemplateInfo {
                        id: meta.id,
                        name: meta.name,
                        description: meta.description,
                        category: meta.category,
                        builtin: false,
                    });
                }
            }
        }
    }

    Ok(templates)
}

/// Scaffolds a built-in template into the given directory.
fn scaffold_builtin(
    template_id: &str,
    project_name: &str,
    dest: &Path,
    locale: ScaffoldLocale,
) -> Result<String, AppError> {
    let novelist_dir = dest.join(".novelist");
    std::fs::create_dir_all(&novelist_dir)?;

    for f in template_scaffold::files_for(template_id, locale, project_name) {
        let full = dest.join(&f.name);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full, &f.content)?;
    }

    let config = build_project_toml(
        project_name,
        template_scaffold::project_type(template_id),
        &template_scaffold::outline_order(template_id, locale),
        template_scaffold::daily_goal(template_id),
    );
    std::fs::write(novelist_dir.join("project.toml"), config)?;

    Ok(dest.to_string_lossy().to_string())
}

fn build_project_toml(
    project_name: &str,
    project_type: &str,
    outline_order: &[String],
    daily_goal: u32,
) -> String {
    let outline_block = if outline_order.is_empty() {
        String::new()
    } else {
        let items: Vec<String> = outline_order.iter().map(|s| format!("\"{}\"", s)).collect();
        format!("\n[outline]\norder = [{}]\n", items.join(", "))
    };

    format!(
        r#"[project]
name = "{name}"
type = "{ptype}"
version = "0.1.0"
{outline}
[writing]
daily_goal = {goal}
auto_save_minutes = 5
"#,
        name = project_name,
        ptype = project_type,
        outline = outline_block,
        goal = daily_goal,
    )
}

#[tauri::command]
#[specta::specta]
pub async fn create_project_from_template(
    template_id: String,
    project_name: String,
    parent_dir: String,
    locale: String,
) -> Result<String, AppError> {
    let dest = Path::new(&parent_dir).join(&project_name);
    if dest.exists() {
        return Err(AppError::InvalidInput(format!(
            "Directory already exists: {}",
            dest.display()
        )));
    }
    std::fs::create_dir_all(&dest)?;

    let scaffold_locale = ScaffoldLocale::from_tag(&locale);

    // Check if it's a built-in template
    let builtins: Vec<&str> = vec![
        "blank",
        "novel",
        "long-novel",
        "short-story",
        "screenplay",
        "blog",
        "journal",
    ];
    if builtins.contains(&template_id.as_str()) {
        return scaffold_builtin(&template_id, &project_name, &dest, scaffold_locale);
    }

    // User template: copy .novelist/ directory from template
    let tpl_dir = templates_dir()?.join(&template_id);
    let novelist_src = tpl_dir.join(".novelist");
    if !novelist_src.exists() {
        return Err(AppError::FileNotFound(format!(
            "Template .novelist dir not found: {}",
            novelist_src.display()
        )));
    }

    // Copy the .novelist directory
    let novelist_dest = dest.join(".novelist");
    copy_dir_recursive(&novelist_src, &novelist_dest)?;

    // Override project name in project.toml
    let config_path = novelist_dest.join("project.toml");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        if let Ok(mut config) = toml::from_str::<crate::models::project::ProjectConfig>(&content) {
            config.project.name = project_name.clone();
            let new_content = toml::to_string(&config)?;
            std::fs::write(&config_path, new_content)?;
        }
    }

    // Copy any non-.novelist files from template (sample chapters, etc.)
    copy_template_files(&tpl_dir, &dest)?;

    Ok(dest.to_string_lossy().to_string())
}

/// Recursively copy a directory.
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}

/// Copy non-.novelist files from template root to destination.
fn copy_template_files(tpl_dir: &Path, dest: &Path) -> Result<(), AppError> {
    for entry in std::fs::read_dir(tpl_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        // Skip .novelist dir and template.toml meta
        if name_str == ".novelist" || name_str == "template.toml" {
            continue;
        }
        let src_path = entry.path();
        let dest_path = dest.join(&name);
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn save_project_as_template(
    project_dir: String,
    template_name: String,
) -> Result<TemplateInfo, AppError> {
    let src = Path::new(&project_dir);
    let novelist_src = src.join(".novelist");
    if !novelist_src.exists() {
        return Err(AppError::InvalidInput(
            "Not a Novelist project (no .novelist directory)".into(),
        ));
    }

    // Generate ID from name
    let id = template_name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>();
    let id = id.trim_matches('-').to_string();
    if id.is_empty() {
        return Err(AppError::InvalidInput("Template name is empty".into()));
    }

    let tpl_dir = templates_dir()?.join(&id);
    if tpl_dir.exists() {
        return Err(AppError::InvalidInput(format!(
            "Template '{}' already exists",
            id
        )));
    }

    std::fs::create_dir_all(&tpl_dir)?;

    // Copy .novelist directory
    copy_dir_recursive(&novelist_src, &tpl_dir.join(".novelist"))?;

    // Copy sample files (non-hidden, non-.novelist)
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') {
            continue;
        }
        let src_path = entry.path();
        let dest_path = tpl_dir.join(&name);
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }

    // Write template.toml
    let meta = TemplateMeta {
        id: id.clone(),
        name: template_name.clone(),
        description: String::new(),
        category: "custom".into(),
    };
    let toml_content = toml::to_string(&meta)?;
    std::fs::write(tpl_dir.join("template.toml"), toml_content)?;

    Ok(TemplateInfo {
        id,
        name: template_name,
        description: String::new(),
        category: "custom".into(),
        builtin: false,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn delete_template(template_id: String) -> Result<(), AppError> {
    let tpl_dir = templates_dir()?.join(&template_id);
    if !tpl_dir.exists() {
        return Err(AppError::FileNotFound(format!(
            "Template not found: {}",
            template_id
        )));
    }
    std::fs::remove_dir_all(&tpl_dir)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn import_template_zip(zip_path: String) -> Result<TemplateInfo, AppError> {
    let zip_file = Path::new(&zip_path);
    if !zip_file.exists() {
        return Err(AppError::FileNotFound(zip_path));
    }

    // Use the zip filename (without extension) as a temporary ID
    let stem = zip_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("imported");

    // Create a temp dir, extract, then move to templates
    let tmp_dir = std::env::temp_dir().join(format!("novelist-import-{}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir)?;

    // Extract zip using system unzip command
    let output = std::process::Command::new("unzip")
        .arg("-o")
        .arg(zip_file)
        .arg("-d")
        .arg(&tmp_dir)
        .output()?;

    if !output.status.success() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(AppError::Custom(format!(
            "Failed to extract zip: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // Find the actual content root (might be nested in a single directory)
    let content_root = find_content_root(&tmp_dir)?;

    // Check for template.toml or .novelist directory
    let meta_path = content_root.join("template.toml");
    let novelist_dir = content_root.join(".novelist");

    let (id, name, description, category) = if meta_path.exists() {
        let content = std::fs::read_to_string(&meta_path)?;
        let meta: TemplateMeta = toml::from_str(&content)?;
        (meta.id, meta.name, meta.description, meta.category)
    } else if novelist_dir.exists() {
        // No template.toml but has .novelist — use zip stem as name
        let id = stem
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();
        (id, stem.to_string(), String::new(), "custom".to_string())
    } else {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(AppError::InvalidInput(
            "Zip must contain a .novelist directory or template.toml".into(),
        ));
    };

    let tpl_dir = templates_dir()?.join(&id);
    if tpl_dir.exists() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(AppError::InvalidInput(format!(
            "Template '{}' already exists",
            id
        )));
    }

    // Move content to templates dir
    std::fs::create_dir_all(tpl_dir.parent().unwrap())?;
    copy_dir_recursive(&content_root, &tpl_dir)?;

    // Ensure template.toml exists
    if !tpl_dir.join("template.toml").exists() {
        let meta = TemplateMeta {
            id: id.clone(),
            name: name.clone(),
            description: description.clone(),
            category: category.clone(),
        };
        std::fs::write(tpl_dir.join("template.toml"), toml::to_string(&meta)?)?;
    }

    // Clean up temp dir
    let _ = std::fs::remove_dir_all(&tmp_dir);

    Ok(TemplateInfo {
        id,
        name,
        description,
        category,
        builtin: false,
    })
}

/// If a zip extracts to a single directory, use that as the root.
fn find_content_root(dir: &Path) -> Result<PathBuf, AppError> {
    let entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();

    // Skip __MACOSX directory
    let real_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.file_name().to_string_lossy() != "__MACOSX")
        .collect();

    if real_entries.len() == 1 && real_entries[0].path().is_dir() {
        Ok(real_entries[0].path())
    } else {
        Ok(dir.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_list_templates_includes_builtins() {
        let templates = list_templates().await.unwrap();
        assert!(templates.iter().any(|t| t.id == "blank"));
        assert!(templates.iter().any(|t| t.id == "novel"));
        assert!(templates.iter().any(|t| t.id == "long-novel"));
        assert!(templates.iter().any(|t| t.id == "short-story"));
        assert!(templates.iter().any(|t| t.id == "screenplay"));
        assert!(templates.iter().any(|t| t.id == "blog"));
        assert!(templates.iter().any(|t| t.id == "journal"));
    }

    #[tokio::test]
    async fn test_list_templates_fallback_strings_are_english() {
        // After the i18n refactor the Rust-side name/description are
        // rendered by the frontend only when an i18n key is missing; they
        // must all be English so a missing-key fallback never produces a
        // mixed EN/ZH UI.
        let templates = list_templates().await.unwrap();
        for t in templates.iter().filter(|t| t.builtin) {
            assert!(
                t.name.is_ascii(),
                "builtin template {} has non-ASCII name {:?}",
                t.id,
                t.name
            );
            assert!(
                t.description.is_ascii(),
                "builtin template {} has non-ASCII description",
                t.id
            );
        }
    }

    #[tokio::test]
    async fn test_create_blank_project() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        let result = create_project_from_template(
            "blank".into(),
            "TestProject".into(),
            parent,
            "en".into(),
        )
        .await
        .unwrap();
        let project_dir = Path::new(&result);
        assert!(project_dir.join(".novelist").join("project.toml").exists());
    }

    #[tokio::test]
    async fn test_create_novel_project_en() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        let result = create_project_from_template(
            "novel".into(),
            "MyNovel".into(),
            parent,
            "en".into(),
        )
        .await
        .unwrap();
        let project_dir = Path::new(&result);
        assert!(project_dir.join("Chapter 1.md").exists());
        assert!(project_dir.join("Chapter 2.md").exists());
        assert!(project_dir.join("Chapter 3.md").exists());
        assert!(!project_dir.join("第一章.md").exists());
        assert!(project_dir.join(".novelist").join("project.toml").exists());
    }

    #[tokio::test]
    async fn test_create_novel_project_zh() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        let result = create_project_from_template(
            "novel".into(),
            "中文小说".into(),
            parent,
            "zh-CN".into(),
        )
        .await
        .unwrap();
        let project_dir = Path::new(&result);
        assert!(project_dir.join("第一章.md").exists());
        assert!(project_dir.join("第二章.md").exists());
        assert!(project_dir.join("第三章.md").exists());
        assert!(!project_dir.join("Chapter 1.md").exists());
    }

    #[tokio::test]
    async fn test_create_long_novel_project_zh() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        let result = create_project_from_template(
            "long-novel".into(),
            "长篇".into(),
            parent,
            "zh-CN".into(),
        )
        .await
        .unwrap();
        let project_dir = Path::new(&result);
        assert!(project_dir.join("大纲.md").exists());
        assert!(project_dir.join("人物设定.md").exists());
        assert!(project_dir.join("世界观.md").exists());
        assert!(project_dir.join("第一卷").join("第一章.md").exists());
        assert!(project_dir.join("第二卷").join("第四章.md").exists());
    }

    #[tokio::test]
    async fn test_create_long_novel_project_en() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        let result = create_project_from_template(
            "long-novel".into(),
            "Epic".into(),
            parent,
            "en".into(),
        )
        .await
        .unwrap();
        let project_dir = Path::new(&result);
        assert!(project_dir.join("Outline.md").exists());
        assert!(project_dir.join("Characters.md").exists());
        assert!(project_dir.join("Worldbuilding.md").exists());
        assert!(project_dir.join("Volume 1").join("Chapter 1.md").exists());
        assert!(project_dir.join("Volume 1").join("Chapter 2.md").exists());
        assert!(project_dir.join("Volume 1").join("Chapter 3.md").exists());
        assert!(project_dir.join("Volume 2").join("Chapter 4.md").exists());
        assert!(project_dir.join("Volume 2").join("Chapter 5.md").exists());
        assert!(!project_dir.join("大纲.md").exists());
    }

    #[tokio::test]
    async fn test_create_short_story_project_zh() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        let result = create_project_from_template(
            "short-story".into(),
            "短篇".into(),
            parent,
            "zh-CN".into(),
        )
        .await
        .unwrap();
        let project_dir = Path::new(&result);
        assert!(project_dir.join("正文.md").exists());
        assert!(project_dir.join("创作笔记.md").exists());
    }

    #[tokio::test]
    async fn test_create_short_story_project_en() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        let result = create_project_from_template(
            "short-story".into(),
            "Shortie".into(),
            parent,
            "en".into(),
        )
        .await
        .unwrap();
        let project_dir = Path::new(&result);
        assert!(project_dir.join("Manuscript.md").exists());
        assert!(project_dir.join("Notes.md").exists());
        assert!(!project_dir.join("正文.md").exists());
    }

    #[tokio::test]
    async fn test_create_screenplay_project_zh() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        let result = create_project_from_template(
            "screenplay".into(),
            "剧本".into(),
            parent,
            "zh-CN".into(),
        )
        .await
        .unwrap();
        let project_dir = Path::new(&result);
        assert!(project_dir.join("人物表.md").exists());
        assert!(project_dir.join("第一幕.md").exists());
        assert!(project_dir.join("第二幕.md").exists());
        assert!(project_dir.join("第三幕.md").exists());
    }

    #[tokio::test]
    async fn test_create_screenplay_project_en() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        let result = create_project_from_template(
            "screenplay".into(),
            "Script".into(),
            parent,
            "en".into(),
        )
        .await
        .unwrap();
        let project_dir = Path::new(&result);
        assert!(project_dir.join("Cast.md").exists());
        assert!(project_dir.join("Act 1.md").exists());
        assert!(project_dir.join("Act 2.md").exists());
        assert!(project_dir.join("Act 3.md").exists());
        assert!(!project_dir.join("人物表.md").exists());
    }

    #[tokio::test]
    async fn test_create_blog_project_en() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        let result = create_project_from_template(
            "blog".into(),
            "MyBlog".into(),
            parent,
            "en".into(),
        )
        .await
        .unwrap();
        let project_dir = Path::new(&result);
        assert!(project_dir.join("posts").join("first-post.md").exists());
        assert!(project_dir.join("drafts").exists());
    }

    #[tokio::test]
    async fn test_create_blog_project_zh() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        let result = create_project_from_template(
            "blog".into(),
            "博客".into(),
            parent,
            "zh-CN".into(),
        )
        .await
        .unwrap();
        let project_dir = Path::new(&result);
        assert!(project_dir.join("posts").join("第一篇.md").exists());
        assert!(project_dir.join("drafts").exists());
    }

    #[tokio::test]
    async fn test_unknown_locale_falls_back_to_english() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        let result = create_project_from_template(
            "novel".into(),
            "FR".into(),
            parent,
            "fr".into(),
        )
        .await
        .unwrap();
        let project_dir = Path::new(&result);
        assert!(project_dir.join("Chapter 1.md").exists());
    }

    #[tokio::test]
    async fn test_outline_order_matches_locale_en() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        let result = create_project_from_template(
            "long-novel".into(),
            "Epic".into(),
            parent,
            "en".into(),
        )
        .await
        .unwrap();
        let config =
            std::fs::read_to_string(Path::new(&result).join(".novelist/project.toml")).unwrap();
        assert!(
            config.contains("\"Outline.md\""),
            "project.toml missing EN outline entry: {}",
            config
        );
    }

    #[tokio::test]
    async fn test_outline_order_matches_locale_zh() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        let result = create_project_from_template(
            "long-novel".into(),
            "长".into(),
            parent,
            "zh-CN".into(),
        )
        .await
        .unwrap();
        let config =
            std::fs::read_to_string(Path::new(&result).join(".novelist/project.toml")).unwrap();
        assert!(
            config.contains("\"大纲.md\""),
            "project.toml missing ZH outline entry: {}",
            config
        );
    }

    #[tokio::test]
    async fn test_create_duplicate_fails() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_string_lossy().to_string();
        create_project_from_template(
            "blank".into(),
            "Dup".into(),
            parent.clone(),
            "en".into(),
        )
        .await
        .unwrap();
        let result = create_project_from_template(
            "blank".into(),
            "Dup".into(),
            parent,
            "en".into(),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_save_and_delete_template() {
        let dir = TempDir::new().unwrap();
        let project = dir.path().join("myproject");
        std::fs::create_dir_all(project.join(".novelist")).unwrap();
        std::fs::write(
            project.join(".novelist").join("project.toml"),
            "[project]\nname = \"Test\"\n",
        )
        .unwrap();
        std::fs::write(project.join("chapter.md"), "# Chapter\n").unwrap();

        let info = save_project_as_template(
            project.to_string_lossy().to_string(),
            "My Test Template".into(),
        )
        .await
        .unwrap();
        assert_eq!(info.name, "My Test Template");
        assert!(!info.builtin);

        // Verify it appears in list
        let templates = list_templates().await.unwrap();
        assert!(templates.iter().any(|t| t.id == info.id));

        // Delete it
        delete_template(info.id.clone()).await.unwrap();
        let templates = list_templates().await.unwrap();
        assert!(!templates.iter().any(|t| t.id == info.id));
    }
}
