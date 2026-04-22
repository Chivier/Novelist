//! Locale-keyed scaffolding data for built-in project templates.
//!
//! Each built-in template (`novel`, `long-novel`, `short-story`,
//! `screenplay`, `blog`, `journal`, `blank`) can be scaffolded in either
//! `En` or `ZhCN`. The frontend passes the active UI locale down through
//! `create_project_from_template`, and the scaffolding files + seed
//! content switch accordingly. Unknown locales fall back to `En`.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScaffoldLocale {
    En,
    ZhCN,
}

impl ScaffoldLocale {
    /// Map a BCP-47-ish language tag to one of the supported scaffold
    /// locales. Any tag starting with `zh` (case-insensitive) maps to
    /// `ZhCN` so that future `zh-TW` / `zh-HK` reuse the Chinese
    /// scaffolding without a code change; everything else falls back to
    /// English.
    pub fn from_tag(tag: &str) -> Self {
        if tag.to_ascii_lowercase().starts_with("zh") {
            ScaffoldLocale::ZhCN
        } else {
            ScaffoldLocale::En
        }
    }
}

#[derive(Debug)]
pub struct ScaffoldFile {
    pub name: String,
    pub content: String,
}

/// Project-type string written into `.novelist/project.toml` `[project].type`.
pub fn project_type(id: &str) -> &'static str {
    match id {
        "blog" => "blog",
        "journal" => "journal",
        _ => "novel",
    }
}

/// Suggested daily word goal seeded into `.novelist/project.toml`.
pub fn daily_goal(id: &str) -> u32 {
    match id {
        "short-story" => 1000,
        "screenplay" => 1500,
        _ => 2000,
    }
}

/// Files to write when creating a project from the given built-in template.
pub fn files_for(id: &str, locale: ScaffoldLocale, project_name: &str) -> Vec<ScaffoldFile> {
    match id {
        "novel" => novel_files(locale, project_name),
        "long-novel" => long_novel_files(locale),
        "short-story" => short_story_files(locale, project_name),
        "screenplay" => screenplay_files(locale),
        "blog" => blog_files(locale),
        "journal" => journal_files(),
        // "blank" or unknown: .novelist/project.toml is written by the
        // caller; no additional files.
        _ => Vec::new(),
    }
}

/// `[outline].order` entries. Empty → caller should omit `[outline]`.
pub fn outline_order(id: &str, locale: ScaffoldLocale) -> Vec<String> {
    match id {
        "novel" => match locale {
            ScaffoldLocale::En => vec![
                "Chapter 1.md".into(),
                "Chapter 2.md".into(),
                "Chapter 3.md".into(),
            ],
            ScaffoldLocale::ZhCN => vec![
                "第一章.md".into(),
                "第二章.md".into(),
                "第三章.md".into(),
            ],
        },
        "long-novel" => match locale {
            ScaffoldLocale::En => vec![
                "Outline.md".into(),
                "Characters.md".into(),
                "Worldbuilding.md".into(),
            ],
            ScaffoldLocale::ZhCN => vec![
                "大纲.md".into(),
                "人物设定.md".into(),
                "世界观.md".into(),
            ],
        },
        "short-story" => match locale {
            ScaffoldLocale::En => vec!["Manuscript.md".into(), "Notes.md".into()],
            ScaffoldLocale::ZhCN => vec!["正文.md".into(), "创作笔记.md".into()],
        },
        "screenplay" => match locale {
            ScaffoldLocale::En => vec![
                "Cast.md".into(),
                "Act 1.md".into(),
                "Act 2.md".into(),
                "Act 3.md".into(),
            ],
            ScaffoldLocale::ZhCN => vec![
                "人物表.md".into(),
                "第一幕.md".into(),
                "第二幕.md".into(),
                "第三幕.md".into(),
            ],
        },
        _ => Vec::new(),
    }
}

// --- Per-template builders --------------------------------------------------

fn novel_files(locale: ScaffoldLocale, project_name: &str) -> Vec<ScaffoldFile> {
    match locale {
        ScaffoldLocale::En => vec![
            file("Chapter 1.md", format!("# {}\n\n", project_name)),
            file("Chapter 2.md", "# Chapter 2\n\n".into()),
            file("Chapter 3.md", "# Chapter 3\n\n".into()),
        ],
        ScaffoldLocale::ZhCN => vec![
            file("第一章.md", format!("# {}\n\n", project_name)),
            file("第二章.md", "# 第二章\n\n".into()),
            file("第三章.md", "# 第三章\n\n".into()),
        ],
    }
}

fn long_novel_files(locale: ScaffoldLocale) -> Vec<ScaffoldFile> {
    match locale {
        ScaffoldLocale::En => vec![
            file(
                "Outline.md",
                "# Outline\n\n## Synopsis\n\n\n\n## Themes\n\n\n\n## Plotlines\n\n### Main Plot\n\n\n\n### Subplot\n\n\n"
                    .into(),
            ),
            file(
                "Characters.md",
                "# Characters\n\n## Protagonist\n\n**Name**:\n\n**Age**:\n\n**Personality**:\n\n**Background**:\n\n---\n\n## Supporting Cast\n\n\n\n---\n\n## Antagonist\n\n\n"
                    .into(),
            ),
            file(
                "Worldbuilding.md",
                "# Worldbuilding\n\n## Era\n\n\n\n## Geography\n\n\n\n## Society\n\n\n\n## Key Concepts\n\n\n"
                    .into(),
            ),
            file("Volume 1/Chapter 1.md", "# Chapter 1\n\n".into()),
            file("Volume 1/Chapter 2.md", "# Chapter 2\n\n".into()),
            file("Volume 1/Chapter 3.md", "# Chapter 3\n\n".into()),
            file("Volume 2/Chapter 4.md", "# Chapter 4\n\n".into()),
            file("Volume 2/Chapter 5.md", "# Chapter 5\n\n".into()),
        ],
        ScaffoldLocale::ZhCN => vec![
            file(
                "大纲.md",
                "# 大纲\n\n## 故事梗概\n\n\n\n## 主题\n\n\n\n## 故事线\n\n### 主线\n\n\n\n### 副线\n\n\n"
                    .into(),
            ),
            file(
                "人物设定.md",
                "# 人物设定\n\n## 主角\n\n**姓名**：\n\n**年龄**：\n\n**性格**：\n\n**背景**：\n\n---\n\n## 配角\n\n\n\n---\n\n## 反派\n\n\n"
                    .into(),
            ),
            file(
                "世界观.md",
                "# 世界观\n\n## 时代背景\n\n\n\n## 地理环境\n\n\n\n## 社会结构\n\n\n\n## 重要设定\n\n\n"
                    .into(),
            ),
            file("第一卷/第一章.md", "# 第一章\n\n".into()),
            file("第一卷/第二章.md", "# 第二章\n\n".into()),
            file("第一卷/第三章.md", "# 第三章\n\n".into()),
            file("第二卷/第四章.md", "# 第四章\n\n".into()),
            file("第二卷/第五章.md", "# 第五章\n\n".into()),
        ],
    }
}

fn short_story_files(locale: ScaffoldLocale, project_name: &str) -> Vec<ScaffoldFile> {
    match locale {
        ScaffoldLocale::En => vec![
            file("Manuscript.md", format!("# {}\n\n", project_name)),
            file(
                "Notes.md",
                "# Notes\n\n## Inspiration\n\n\n\n## Core Conflict\n\n\n\n## Character Sketches\n\n\n\n## Ending Ideas\n\n\n"
                    .into(),
            ),
        ],
        ScaffoldLocale::ZhCN => vec![
            file("正文.md", format!("# {}\n\n", project_name)),
            file(
                "创作笔记.md",
                "# 创作笔记\n\n## 灵感来源\n\n\n\n## 核心冲突\n\n\n\n## 人物速写\n\n\n\n## 结局构想\n\n\n"
                    .into(),
            ),
        ],
    }
}

fn screenplay_files(locale: ScaffoldLocale) -> Vec<ScaffoldFile> {
    match locale {
        ScaffoldLocale::En => vec![
            file(
                "Cast.md",
                "# Cast\n\n## Main Characters\n\n**Name**:\n**Role**:\n**Bio**:\n\n---\n\n## Supporting Characters\n\n\n"
                    .into(),
            ),
            file(
                "Act 1.md",
                "# Act 1\n\n## Scene 1\n\n**Location**:\n**Time**:\n\n---\n\n".into(),
            ),
            file(
                "Act 2.md",
                "# Act 2\n\n## Scene 1\n\n**Location**:\n**Time**:\n\n---\n\n".into(),
            ),
            file(
                "Act 3.md",
                "# Act 3\n\n## Scene 1\n\n**Location**:\n**Time**:\n\n---\n\n".into(),
            ),
        ],
        ScaffoldLocale::ZhCN => vec![
            file(
                "人物表.md",
                "# 人物表\n\n## 主要人物\n\n**角色名**：\n**身份**：\n**简介**：\n\n---\n\n## 次要人物\n\n\n"
                    .into(),
            ),
            file(
                "第一幕.md",
                "# 第一幕\n\n## 场景一\n\n**场景**：\n**时间**：\n\n---\n\n".into(),
            ),
            file(
                "第二幕.md",
                "# 第二幕\n\n## 场景一\n\n**场景**：\n**时间**：\n\n---\n\n".into(),
            ),
            file(
                "第三幕.md",
                "# 第三幕\n\n## 场景一\n\n**场景**：\n**时间**：\n\n---\n\n".into(),
            ),
        ],
    }
}

fn blog_files(locale: ScaffoldLocale) -> Vec<ScaffoldFile> {
    // Directory names `posts/` and `drafts/` stay in English by design so
    // routing conventions (e.g. `/posts/<slug>`) stay intuitive.
    match locale {
        ScaffoldLocale::En => vec![
            file("posts/first-post.md", "# My First Post\n\n".into()),
            file("drafts/.gitkeep", String::new()),
        ],
        ScaffoldLocale::ZhCN => vec![
            file("posts/第一篇.md", "# 第一篇\n\n".into()),
            file("drafts/.gitkeep", String::new()),
        ],
    }
}

fn journal_files() -> Vec<ScaffoldFile> {
    let today = today_ymd();
    vec![file(
        &format!("{}.md", today),
        format!("# {}\n\n", today),
    )]
}

// --- helpers ---------------------------------------------------------------

fn file(name: &str, content: String) -> ScaffoldFile {
    ScaffoldFile {
        name: name.to_string(),
        content,
    }
}

/// Returns today's date as `YYYY-MM-DD` without pulling in chrono.
/// (Copy of the helper originally in `commands::template`; kept here so
/// scaffolding is self-contained.)
fn today_ymd() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = now / 86400;
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let month_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1u32;
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }
    let d = remaining + 1;
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_tag_maps_zh_prefix_to_zh_cn() {
        assert_eq!(ScaffoldLocale::from_tag("zh"), ScaffoldLocale::ZhCN);
        assert_eq!(ScaffoldLocale::from_tag("zh-CN"), ScaffoldLocale::ZhCN);
        assert_eq!(ScaffoldLocale::from_tag("zh-TW"), ScaffoldLocale::ZhCN);
        assert_eq!(ScaffoldLocale::from_tag("ZH-HK"), ScaffoldLocale::ZhCN);
    }

    #[test]
    fn from_tag_falls_back_to_en() {
        assert_eq!(ScaffoldLocale::from_tag("en"), ScaffoldLocale::En);
        assert_eq!(ScaffoldLocale::from_tag("en-US"), ScaffoldLocale::En);
        assert_eq!(ScaffoldLocale::from_tag("ja"), ScaffoldLocale::En);
        assert_eq!(ScaffoldLocale::from_tag(""), ScaffoldLocale::En);
    }

    #[test]
    fn novel_en_has_english_chapters() {
        let files = files_for("novel", ScaffoldLocale::En, "Hello");
        let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"Chapter 1.md"));
        assert!(names.contains(&"Chapter 2.md"));
        assert!(names.contains(&"Chapter 3.md"));
        assert!(files[0].content.starts_with("# Hello\n"));
    }

    #[test]
    fn novel_zh_has_chinese_chapters() {
        let files = files_for("novel", ScaffoldLocale::ZhCN, "你好");
        let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"第一章.md"));
        assert!(names.contains(&"第二章.md"));
        assert!(names.contains(&"第三章.md"));
        assert!(files[0].content.starts_with("# 你好\n"));
    }

    #[test]
    fn long_novel_en_has_planning_docs_and_volumes() {
        let files = files_for("long-novel", ScaffoldLocale::En, "Epic");
        let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"Outline.md"));
        assert!(names.contains(&"Characters.md"));
        assert!(names.contains(&"Worldbuilding.md"));
        assert!(names.contains(&"Volume 1/Chapter 1.md"));
        assert!(names.contains(&"Volume 2/Chapter 4.md"));
    }

    #[test]
    fn outline_order_matches_locale() {
        let en = outline_order("long-novel", ScaffoldLocale::En);
        assert_eq!(en[0], "Outline.md");
        let zh = outline_order("long-novel", ScaffoldLocale::ZhCN);
        assert_eq!(zh[0], "大纲.md");
    }

    #[test]
    fn blank_has_no_files() {
        assert!(files_for("blank", ScaffoldLocale::En, "X").is_empty());
        assert!(files_for("blank", ScaffoldLocale::ZhCN, "X").is_empty());
    }

    #[test]
    fn project_type_and_daily_goal() {
        assert_eq!(project_type("blog"), "blog");
        assert_eq!(project_type("journal"), "journal");
        assert_eq!(project_type("novel"), "novel");
        assert_eq!(project_type("long-novel"), "novel");

        assert_eq!(daily_goal("short-story"), 1000);
        assert_eq!(daily_goal("screenplay"), 1500);
        assert_eq!(daily_goal("novel"), 2000);
    }
}
