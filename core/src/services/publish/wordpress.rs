//! WordPress (self-hosted, REST API) publish adapter.
//!
//! Wire (HTTP Basic with Application Password):
//! - GET  /wp-json/wp/v2/users/me           — connectivity check
//! - GET  /wp-json/wp/v2/tags?search=<name> — resolve tag id by name
//! - POST /wp-json/wp/v2/tags { name }      — create tag if missing
//! - POST /wp-json/wp/v2/media              — image upload (raw body
//!   with Content-Disposition + Content-Type), returns id + source_url
//! - POST /wp-json/wp/v2/posts              — create post with HTML
//!   body, status, title, slug, excerpt, tags (id array),
//!   featured_media (attachment id)
//!
//! Reference: https://developer.wordpress.org/rest-api/reference/posts/
//! Application Passwords:
//! https://make.wordpress.org/core/2020/11/05/application-passwords-integration-guide/

use crate::models::publish::PlatformConfig;
use crate::services::publish::types::{PublishError, PublishInput, PublishResult};
use base64::Engine;

/// Build the `Authorization: Basic ...` header value.
/// Spaces in the application password are stripped first
/// (per WP integration guide; the password as displayed contains
/// spaces for readability).
pub fn basic_auth_header(username: &str, app_password: &str) -> String {
    let pw = app_password.replace(' ', "");
    let pair = format!("{username}:{pw}");
    let encoded = base64::engine::general_purpose::STANDARD.encode(pair.as_bytes());
    format!("Basic {encoded}")
}

/// Read-only credentials check: `GET /wp-json/wp/v2/users/me`. Returns
/// the user's display name on success.
pub async fn verify(
    site_url: &str,
    username: &str,
    app_password: &str,
) -> Result<String, PublishError> {
    if site_url.is_empty() || username.is_empty() || app_password.is_empty() {
        return Err(PublishError::BadConfig(
            "wordpress config missing site_url / username / app_password".into(),
        ));
    }
    let auth = basic_auth_header(username, app_password);
    let endpoint = format!("{}/wp-json/wp/v2/users/me", site_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| PublishError::Network(e.to_string()))?;
    let resp = client
        .get(&endpoint)
        .header("Authorization", &auth)
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
        .unwrap_or("WordPress user");
    Ok(format!("Connected as {name}"))
}

pub async fn upload_image(
    site_url: &str,
    auth: &str,
    bytes: Vec<u8>,
    filename: String,
    mime: String,
) -> Result<(String, u64), PublishError> {
    let endpoint = format!("{}/wp-json/wp/v2/media", site_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| PublishError::Network(e.to_string()))?;
    let resp = client
        .post(&endpoint)
        .header("Authorization", auth)
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

pub async fn resolve_tag_ids(
    site_url: &str,
    auth: &str,
    tag_names: &[String],
) -> Result<Vec<u64>, PublishError> {
    let mut out = Vec::with_capacity(tag_names.len());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| PublishError::Network(e.to_string()))?;
    let base = site_url.trim_end_matches('/');
    for name in tag_names {
        let resp = client
            .get(format!("{base}/wp-json/wp/v2/tags"))
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
        // Search might return multiple — find one whose `name` matches case-insensitively
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
        // Create the tag.
        let create = client
            .post(format!("{base}/wp-json/wp/v2/tags"))
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

pub async fn publish(
    config: &PlatformConfig,
    input: &PublishInput,
) -> Result<PublishResult, PublishError> {
    let (site_url, username, app_password) = match config {
        PlatformConfig::WordPressSelfHosted {
            site_url,
            username,
            app_password,
        } => (site_url, username, app_password),
        _ => return Err(PublishError::BadConfig("not a WordPress config".into())),
    };
    if site_url.is_empty() || username.is_empty() || app_password.is_empty() {
        return Err(PublishError::BadConfig(
            "wordpress config missing site_url / username / app_password".into(),
        ));
    }
    let auth = basic_auth_header(username, app_password);

    let tag_ids = if input.tags.is_empty() {
        Vec::new()
    } else {
        resolve_tag_ids(site_url, &auth, &input.tags).await?
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

    let endpoint = format!("{}/wp-json/wp/v2/posts", site_url.trim_end_matches('/'));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::publish::types::BodyFormat;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn cfg(site_url: &str) -> PlatformConfig {
        PlatformConfig::WordPressSelfHosted {
            site_url: site_url.into(),
            username: "alice".into(),
            app_password: "abcd EFGH 1234 ijkl MNOP 6789".into(),
        }
    }

    fn input() -> PublishInput {
        PublishInput {
            title: "Hello".into(),
            body: "<p>world</p>".into(),
            body_format: BodyFormat::Html,
            tags: vec!["rust".into()],
            slug: Some("hello".into()),
            excerpt: Some("brief".into()),
            status: "draft".into(),
            feature_image_url: None,
            featured_media_id: Some(7),
            publication_id: None,
        }
    }

    #[test]
    fn basic_auth_strips_spaces_from_password() {
        let h = basic_auth_header("alice", "abcd EFGH 1234");
        // base64("alice:abcdEFGH1234") = YWxpY2U6YWJjZEVGR0gxMjM0
        assert_eq!(h, "Basic YWxpY2U6YWJjZEVGR0gxMjM0");
    }

    #[tokio::test]
    async fn upload_image_returns_id_and_url() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/wp-json/wp/v2/media"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": 42,
                "source_url": "https://example.com/wp-content/uploads/x.png"
            })))
            .mount(&server)
            .await;
        let auth = basic_auth_header("alice", "p");
        let (url, id) = upload_image(
            &server.uri(),
            &auth,
            vec![1, 2, 3],
            "x.png".into(),
            "image/png".into(),
        )
        .await
        .unwrap();
        assert_eq!(id, 42);
        assert_eq!(url, "https://example.com/wp-content/uploads/x.png");
    }

    #[tokio::test]
    async fn resolve_tag_ids_finds_existing_or_creates() {
        let server = MockServer::start().await;
        // existing tag
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/tags"))
            .and(query_param("search", "rust"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"id": 10, "name": "rust"}
            ])))
            .mount(&server)
            .await;
        // missing tag → POST to create
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/tags"))
            .and(query_param("search", "tauri"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/wp-json/wp/v2/tags"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"id": 20, "name": "tauri"})),
            )
            .mount(&server)
            .await;
        let auth = basic_auth_header("alice", "p");
        let ids = resolve_tag_ids(&server.uri(), &auth, &["rust".into(), "tauri".into()])
            .await
            .unwrap();
        assert_eq!(ids, vec![10, 20]);
    }

    #[tokio::test]
    async fn publish_creates_post_with_resolved_tags() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"id": 10, "name": "rust"}
            ])))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/wp-json/wp/v2/posts"))
            .and(header(
                "Authorization",
                "Basic YWxpY2U6YWJjZEVGR0gxMjM0aWprbE1OT1A2Nzg5",
            ))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": 99,
                "link": "https://example.com/?p=99",
                "slug": "hello"
            })))
            .mount(&server)
            .await;
        let result = publish(&cfg(&server.uri()), &input()).await.unwrap();
        assert_eq!(result.remote_id, "99");
        assert_eq!(result.url, "https://example.com/?p=99");
    }

    #[tokio::test]
    async fn publish_401_is_auth_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(401).set_body_string("nope"))
            .mount(&server)
            .await;
        let err = publish(&cfg(&server.uri()), &input()).await.unwrap_err();
        assert!(matches!(err, PublishError::Auth(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn empty_field_is_bad_config() {
        let cfg = PlatformConfig::WordPressSelfHosted {
            site_url: "".into(),
            username: "a".into(),
            app_password: "p".into(),
        };
        let err = publish(&cfg, &input()).await.unwrap_err();
        assert!(matches!(err, PublishError::BadConfig(_)), "got {err:?}");
    }
}
