//! WordPress.com publish adapter.
//!
//! Same wire shape as the self-hosted adapter except:
//! - Base URL: `https://public-api.wordpress.com/wp/v2/sites/<site_id_or_domain>`
//! - Auth: `Authorization: Bearer <access_token>` (OAuth2 personal
//!   access token from developer.wordpress.com)
//!
//! Adapter kept separate per design — different URL space + different
//! auth mode means a single combined file would be a forced
//! abstraction.

use crate::models::publish::PlatformConfig;
use crate::services::publish::types::{PublishError, PublishInput, PublishResult};

const DEFAULT_BASE: &str = "https://public-api.wordpress.com";

/// Read-only credentials check: `GET /wp/v2/sites/<id>/users/me`.
pub async fn verify(site: &str, token: &str) -> Result<String, PublishError> {
    if site.is_empty() || token.is_empty() {
        return Err(PublishError::BadConfig(
            "wordpress_com config missing site or access_token".into(),
        ));
    }
    let endpoint = format!("{DEFAULT_BASE}/wp/v2/sites/{site}/users/me");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| PublishError::Network(e.to_string()))?;
    let resp = client
        .get(&endpoint)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| PublishError::Network(e.to_string()))?;
    let resp = crate::services::publish::require_success(resp).await?;
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| PublishError::UnexpectedResponse(format!("json: {e}")))?;
    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("WordPress.com user");
    Ok(format!("Connected as {name}"))
}

pub async fn upload_image(
    site: &str,
    token: &str,
    bytes: Vec<u8>,
    filename: String,
    mime: String,
) -> Result<(String, u64), PublishError> {
    upload_image_with_base(site, token, bytes, filename, mime, DEFAULT_BASE).await
}

pub async fn upload_image_with_base(
    site: &str,
    token: &str,
    bytes: Vec<u8>,
    filename: String,
    mime: String,
    base: &str,
) -> Result<(String, u64), PublishError> {
    if site.is_empty() || token.is_empty() {
        return Err(PublishError::BadConfig("missing site or token".into()));
    }
    let endpoint = format!("{base}/wp/v2/sites/{site}/media");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| PublishError::Network(e.to_string()))?;
    let resp = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", &mime)
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{filename}\""),
        )
        .body(bytes)
        .send()
        .await
        .map_err(|e| PublishError::Network(e.to_string()))?;
    let resp = crate::services::publish::require_success(resp).await?;
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| PublishError::UnexpectedResponse(format!("json: {e}")))?;
    let id = body
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| PublishError::UnexpectedResponse("no id".into()))?;
    let url = body
        .get("source_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PublishError::UnexpectedResponse("no source_url".into()))?
        .to_string();
    Ok((url, id))
}

pub async fn publish(
    config: &PlatformConfig,
    input: &PublishInput,
) -> Result<PublishResult, PublishError> {
    publish_with_base(config, input, DEFAULT_BASE).await
}

pub async fn publish_with_base(
    config: &PlatformConfig,
    input: &PublishInput,
    base: &str,
) -> Result<PublishResult, PublishError> {
    let (site, token) = match config {
        PlatformConfig::WordPressCom {
            site_id_or_domain,
            access_token,
        } => (site_id_or_domain, access_token),
        _ => return Err(PublishError::BadConfig("not a WordPress.com config".into())),
    };
    if site.is_empty() || token.is_empty() {
        return Err(PublishError::BadConfig(
            "wordpress_com config missing site or access_token".into(),
        ));
    }
    let auth = format!("Bearer {token}");
    let tag_ids = if input.tags.is_empty() {
        Vec::new()
    } else {
        resolve_tag_ids_with_base(site, &auth, &input.tags, base).await?
    };

    let mut payload = serde_json::json!({
        "title": input.title,
        "content": input.body,
        "status": input.status,
        "tags": tag_ids,
    });
    if let Some(slug) = &input.slug {
        payload["slug"] = serde_json::Value::String(slug.clone());
    }
    if let Some(excerpt) = &input.excerpt {
        payload["excerpt"] = serde_json::Value::String(excerpt.clone());
    }
    if let Some(media_id) = input.featured_media_id {
        payload["featured_media"] = serde_json::Value::Number(media_id.into());
    }

    let endpoint = format!("{base}/wp/v2/sites/{site}/posts");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| PublishError::Network(e.to_string()))?;
    let resp = client
        .post(&endpoint)
        .header("Authorization", &auth)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| PublishError::Network(e.to_string()))?;
    let resp = crate::services::publish::require_success(resp).await?;
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| PublishError::UnexpectedResponse(format!("json: {e}")))?;
    let id = body
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| PublishError::UnexpectedResponse("no id".into()))?;
    let url = body
        .get("link")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PublishError::UnexpectedResponse("no link".into()))?
        .to_string();
    Ok(PublishResult {
        url,
        remote_id: id.to_string(),
    })
}

async fn resolve_tag_ids_with_base(
    site: &str,
    auth: &str,
    tag_names: &[String],
    base: &str,
) -> Result<Vec<u64>, PublishError> {
    let mut out = Vec::with_capacity(tag_names.len());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| PublishError::Network(e.to_string()))?;
    for name in tag_names {
        let resp = client
            .get(format!("{base}/wp/v2/sites/{site}/tags"))
            .query(&[("search", name)])
            .header("Authorization", auth)
            .send()
            .await
            .map_err(|e| PublishError::Network(e.to_string()))?;
        let resp = crate::services::publish::require_success(resp).await?;
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| PublishError::UnexpectedResponse(format!("json: {e}")))?;
        let matched_id = body.as_array().and_then(|arr| {
            arr.iter()
                .find(|t| {
                    t.get("name")
                        .and_then(|n| n.as_str())
                        .map(|n| n.eq_ignore_ascii_case(name))
                        .unwrap_or(false)
                })
                .and_then(|t| t.get("id").and_then(|v| v.as_u64()))
        });
        if let Some(id) = matched_id {
            out.push(id);
            continue;
        }
        let create = client
            .post(format!("{base}/wp/v2/sites/{site}/tags"))
            .header("Authorization", auth)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({"name": name}))
            .send()
            .await
            .map_err(|e| PublishError::Network(e.to_string()))?;
        let create = crate::services::publish::require_success(create).await?;
        let body: serde_json::Value = create
            .json()
            .await
            .map_err(|e| PublishError::UnexpectedResponse(format!("json: {e}")))?;
        let id = body
            .get("id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| PublishError::UnexpectedResponse("no id on tag create".into()))?;
        out.push(id);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::publish::types::BodyFormat;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn cfg(site: &str) -> PlatformConfig {
        PlatformConfig::WordPressCom {
            site_id_or_domain: site.into(),
            access_token: "tok".into(),
        }
    }

    fn input() -> PublishInput {
        PublishInput {
            title: "Hello".into(),
            body: "<p>world</p>".into(),
            body_format: BodyFormat::Html,
            tags: vec![],
            slug: None,
            excerpt: None,
            status: "draft".into(),
            feature_image_url: None,
            featured_media_id: None,
            publication_id: None,
        }
    }

    #[tokio::test]
    async fn publish_creates_post() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/wp/v2/sites/myblog.example.com/posts"))
            .and(header("Authorization", "Bearer tok"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": 7,
                "link": "https://myblog.example.com/?p=7"
            })))
            .mount(&server)
            .await;
        let result = publish_with_base(&cfg("myblog.example.com"), &input(), &server.uri())
            .await
            .unwrap();
        assert_eq!(result.remote_id, "7");
        assert!(result.url.contains("?p=7"), "got {}", result.url);
    }

    #[tokio::test]
    async fn upload_image_returns_id_and_url() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/wp/v2/sites/myblog/media"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": 11,
                "source_url": "https://wp.com/files/x.png"
            })))
            .mount(&server)
            .await;
        let (url, id) = upload_image_with_base(
            "myblog",
            "tok",
            vec![1, 2],
            "x.png".into(),
            "image/png".into(),
            &server.uri(),
        )
        .await
        .unwrap();
        assert_eq!(id, 11);
        assert_eq!(url, "https://wp.com/files/x.png");
    }

    #[tokio::test]
    async fn publish_401_is_auth_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401).set_body_string("invalid_token"))
            .mount(&server)
            .await;
        let err = publish_with_base(&cfg("myblog"), &input(), &server.uri())
            .await
            .unwrap_err();
        assert!(matches!(err, PublishError::Auth(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn empty_field_is_bad_config() {
        let cfg = PlatformConfig::WordPressCom {
            site_id_or_domain: "".into(),
            access_token: "t".into(),
        };
        let err = publish(&cfg, &input()).await.unwrap_err();
        assert!(matches!(err, PublishError::BadConfig(_)), "got {err:?}");
    }
}
