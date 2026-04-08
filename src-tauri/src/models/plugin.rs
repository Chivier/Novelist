use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct PluginManifest {
    pub plugin: PluginMeta,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct PluginMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub permissions: Vec<String>,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct RegisteredCommandInfo {
    pub plugin_id: String,
    pub command_id: String,
    pub label: String,
}
