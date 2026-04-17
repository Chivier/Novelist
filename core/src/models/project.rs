use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct ProjectConfig {
    pub project: ProjectMeta,
    #[serde(default)]
    pub outline: OutlineConfig,
    #[serde(default)]
    pub writing: WritingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct ProjectMeta {
    pub name: String,
    #[serde(rename = "type", default = "default_project_type")]
    pub project_type: String,
    #[serde(default = "default_version")]
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, Type)]
pub struct OutlineConfig {
    #[serde(default)]
    pub order: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct WritingConfig {
    #[serde(default = "default_daily_goal")]
    pub daily_goal: u32,
    #[serde(default = "default_auto_save_minutes")]
    pub auto_save_minutes: u32,
}

impl Default for WritingConfig {
    fn default() -> Self {
        Self {
            daily_goal: default_daily_goal(),
            auto_save_minutes: default_auto_save_minutes(),
        }
    }
}

fn default_project_type() -> String {
    "novel".to_string()
}
fn default_version() -> String {
    "0.1.0".to_string()
}
fn default_daily_goal() -> u32 {
    2000
}
fn default_auto_save_minutes() -> u32 {
    5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_config_deserialize() {
        let toml_str = r#"
[project]
name = "My Novel"
type = "novel"
version = "1.0.0"

[outline]
order = ["chapter1.md", "chapter2.md"]

[writing]
daily_goal = 3000
auto_save_minutes = 10
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.name, "My Novel");
        assert_eq!(config.project.project_type, "novel");
        assert_eq!(config.project.version, "1.0.0");
        assert_eq!(config.outline.order, vec!["chapter1.md", "chapter2.md"]);
        assert_eq!(config.writing.daily_goal, 3000);
        assert_eq!(config.writing.auto_save_minutes, 10);
    }

    #[test]
    fn test_minimal_config_defaults() {
        let toml_str = r#"
[project]
name = "Minimal"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.name, "Minimal");
        assert_eq!(config.project.project_type, "novel");
        assert_eq!(config.project.version, "0.1.0");
        assert!(config.outline.order.is_empty());
        assert_eq!(config.writing.daily_goal, 2000);
        assert_eq!(config.writing.auto_save_minutes, 5);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let config = ProjectConfig {
            project: ProjectMeta {
                name: "Test".to_string(),
                project_type: "novel".to_string(),
                version: "0.1.0".to_string(),
            },
            outline: OutlineConfig {
                order: vec!["a.md".to_string()],
            },
            writing: WritingConfig {
                daily_goal: 1500,
                auto_save_minutes: 3,
            },
        };

        let serialized = toml::to_string(&config).unwrap();
        let deserialized: ProjectConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.project.name, "Test");
        assert_eq!(deserialized.writing.daily_goal, 1500);
        assert_eq!(deserialized.outline.order, vec!["a.md"]);
    }

    #[test]
    fn test_writing_config_default() {
        let wc = WritingConfig::default();
        assert_eq!(wc.daily_goal, 2000);
        assert_eq!(wc.auto_save_minutes, 5);
    }

    #[test]
    fn test_partial_writing_config() {
        let toml_str = r#"
[project]
name = "Partial"

[writing]
daily_goal = 500
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.writing.daily_goal, 500);
        assert_eq!(config.writing.auto_save_minutes, 5); // default
    }

    #[test]
    fn test_custom_project_type() {
        let toml_str = r#"
[project]
name = "My Blog"
type = "blog"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.project_type, "blog");
    }
}
