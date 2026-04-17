use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct TemplateMeta {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_category")]
    pub category: String,
}

fn default_category() -> String {
    "custom".to_string()
}

/// Info returned to the frontend for listing templates.
#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    /// Whether this is a built-in template (cannot be deleted).
    pub builtin: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_meta_deserialize() {
        let toml_str = r#"
id = "novel-blank"
name = "Blank Novel"
description = "A blank novel project"
category = "novel"
"#;
        let meta: TemplateMeta = toml::from_str(toml_str).unwrap();
        assert_eq!(meta.id, "novel-blank");
        assert_eq!(meta.name, "Blank Novel");
        assert_eq!(meta.category, "novel");
    }

    #[test]
    fn test_template_meta_defaults() {
        let toml_str = r#"
id = "test"
name = "Test"
"#;
        let meta: TemplateMeta = toml::from_str(toml_str).unwrap();
        assert_eq!(meta.description, "");
        assert_eq!(meta.category, "custom");
    }
}
