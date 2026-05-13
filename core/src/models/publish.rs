//! Settings model for publish channels.
//!
//! `PublishSettings` lives in `GlobalSettings.publish`. No per-project
//! override (matches the v0.2.4 image-host convention — credentials
//! never leak into per-project files). No "active channel" pointer:
//! the user picks a channel per publish action.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Type, PartialEq)]
pub struct PublishSettings {
    #[serde(default)]
    pub channels: Vec<ChannelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub struct ChannelConfig {
    /// Stable UUID. Survives renaming the user-facing label.
    pub id: String,
    /// User-facing label, e.g. "Personal Ghost".
    pub name: String,
    /// Platform-specific config; the discriminant lives at the
    /// `platform` field (flattened into the parent on the wire).
    #[serde(flatten)]
    pub config: PlatformConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(tag = "platform")]
pub enum PlatformConfig {
    #[serde(rename = "ghost")]
    Ghost {
        /// Admin URL ending without a slash, e.g. "https://blog.example.com".
        admin_url: String,
        /// Admin API key in the canonical "<id>:<secret_hex>" form.
        api_key: String,
    },
    #[serde(rename = "wordpress_self_hosted")]
    WordPressSelfHosted {
        /// Site URL ending without a slash, e.g. "https://blog.example.com".
        site_url: String,
        username: String,
        /// Application Password, the 24-char string from WP Admin →
        /// Users → Application Passwords. Spaces optional.
        app_password: String,
    },
    #[serde(rename = "wordpress_com")]
    WordPressCom {
        /// Site id (numeric) or domain ("myblog.wordpress.com").
        site_id_or_domain: String,
        /// OAuth2 access token from developer.wordpress.com.
        access_token: String,
    },
    #[serde(rename = "medium")]
    Medium {
        /// Integration token from Settings → Security → Integration
        /// Tokens. The Medium UI for generating new tokens was removed
        /// in late 2024; legacy users only.
        token: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_config_serializes_with_flattened_platform_tag() {
        let ch = ChannelConfig {
            id: "h1".into(),
            name: "Personal Ghost".into(),
            config: PlatformConfig::Ghost {
                admin_url: "https://blog.example.com".into(),
                api_key: "abc:1234".into(),
            },
        };
        let json = serde_json::to_string(&ch).unwrap();
        assert!(json.contains("\"platform\":\"ghost\""));
        assert!(json.contains("\"admin_url\":\"https://blog.example.com\""));
    }

    #[test]
    fn channel_config_roundtrips_through_json() {
        let original = ChannelConfig {
            id: "wp1".into(),
            name: "Company WP".into(),
            config: PlatformConfig::WordPressSelfHosted {
                site_url: "https://blog.example.com".into(),
                username: "alice".into(),
                app_password: "abcd EFGH 1234 ijkl MNOP 6789".into(),
            },
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: ChannelConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn parses_minimal_medium_config() {
        let ch: ChannelConfig =
            serde_json::from_str(r#"{"id":"m","name":"Medium","platform":"medium","token":"t"}"#)
                .unwrap();
        assert_eq!(ch.id, "m");
        match ch.config {
            PlatformConfig::Medium { token } => assert_eq!(token, "t"),
            _ => panic!("expected Medium"),
        }
    }

    #[test]
    fn publish_settings_default_is_empty() {
        let s = PublishSettings::default();
        assert!(s.channels.is_empty());
    }

    #[test]
    fn parses_wordpress_com_variant() {
        let ch: ChannelConfig = serde_json::from_str(
            r#"{"id":"wp","name":"WP.com","platform":"wordpress_com","site_id_or_domain":"myblog.wordpress.com","access_token":"tok"}"#,
        )
        .unwrap();
        match ch.config {
            PlatformConfig::WordPressCom {
                site_id_or_domain,
                access_token,
            } => {
                assert_eq!(site_id_or_domain, "myblog.wordpress.com");
                assert_eq!(access_token, "tok");
            }
            _ => panic!("expected WordPressCom"),
        }
    }
}
