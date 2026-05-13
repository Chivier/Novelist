//! Settings model for image-hosting providers.
//!
//! `ImageHostSettings` lives in `GlobalSettings.image_hosts`. Per-project
//! overrides are limited to `ProjectConfig.active_image_host_id` —
//! credentials are global only so they never accidentally land in a
//! per-project file the user might commit to git.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Type, PartialEq)]
pub struct ImageHostSettings {
    #[serde(default)]
    pub hosts: Vec<HostConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_host_id: Option<String>,
    #[serde(default)]
    pub auto_on_paste: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub struct HostConfig {
    /// Stable UUID. Survives renaming the user-facing label.
    pub id: String,
    /// User-facing label, e.g. "Personal R2".
    pub name: String,
    /// Provider-specific config; the discriminant lives at the
    /// `provider` field (flattened into the parent on the wire).
    #[serde(flatten)]
    pub config: ProviderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum ProviderConfig {
    Qiniu {
        access_key: String,
        secret_key: String,
        bucket: String,
        /// CDN domain that fronts the bucket, e.g. "https://cdn.example.com".
        domain: String,
    },
    AliyunOss {
        access_key_id: String,
        access_key_secret: String,
        bucket: String,
        /// Region endpoint, e.g. "oss-cn-hangzhou.aliyuncs.com".
        endpoint: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        custom_domain: Option<String>,
    },
    S3 {
        access_key_id: String,
        secret_access_key: String,
        bucket: String,
        region: String,
        /// Set for non-AWS S3-compatible endpoints (R2, MinIO).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        /// Optional key prefix prepended before the generated object key.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path_prefix: Option<String>,
        /// Custom CDN domain. Final URL = `<custom_domain>/<key>` when set.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        custom_domain: Option<String>,
    },
    Imgur {
        client_id: String,
    },
    Smms {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        api_token: Option<String>,
    },
    Custom {
        post_url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bearer: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_config_serializes_with_flattened_provider_tag() {
        let host = HostConfig {
            id: "h1".into(),
            name: "Personal Qiniu".into(),
            config: ProviderConfig::Qiniu {
                access_key: "ak".into(),
                secret_key: "sk".into(),
                bucket: "novelist".into(),
                domain: "https://cdn.example.com".into(),
            },
        };
        let json = serde_json::to_string(&host).unwrap();
        // Discriminant flattened into parent
        assert!(json.contains("\"provider\":\"qiniu\""));
        assert!(json.contains("\"id\":\"h1\""));
        assert!(json.contains("\"access_key\":\"ak\""));
    }

    #[test]
    fn host_config_roundtrips_through_json() {
        let original = HostConfig {
            id: "h2".into(),
            name: "R2".into(),
            config: ProviderConfig::S3 {
                access_key_id: "AKIA".into(),
                secret_access_key: "secret".into(),
                bucket: "blog".into(),
                region: "auto".into(),
                endpoint: Some("https://acct.r2.cloudflarestorage.com".into()),
                path_prefix: None,
                custom_domain: Some("https://images.example.com".into()),
            },
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: HostConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn image_host_settings_default_is_empty_no_active() {
        let s = ImageHostSettings::default();
        assert!(s.hosts.is_empty());
        assert!(s.active_host_id.is_none());
        assert!(!s.auto_on_paste);
    }

    #[test]
    fn omits_active_host_id_when_none() {
        let s = ImageHostSettings::default();
        let json = serde_json::to_string(&s).unwrap();
        assert!(!json.contains("active_host_id"));
    }

    #[test]
    fn parses_minimal_smms_with_no_token() {
        let host: HostConfig =
            serde_json::from_str(r#"{"id":"x","name":"sm.ms","provider":"smms"}"#).unwrap();
        assert_eq!(host.id, "x");
        match host.config {
            ProviderConfig::Smms { api_token } => assert!(api_token.is_none()),
            _ => panic!("expected Smms"),
        }
    }
}
