//! Ghost Admin API publish adapter.
//!
//! Wire (Admin API key + per-request HS256 JWT, 5-min TTL):
//! 1. Split api_key on `:` into `id` and `secret_hex`.
//! 2. hex-decode `secret_hex` → bytes (this is the HMAC key — a common
//!    mistake is to sign with the ASCII hex string directly).
//! 3. JWT header `{"alg":"HS256","kid":"<id>","typ":"JWT"}`.
//! 4. JWT payload `{"iat":<now>,"exp":<now+300>,"aud":"/admin/"}`.
//! 5. token = base64url(header) + "." + base64url(payload) + "." +
//!    base64url(HMAC-SHA256(key=secret_bytes, data=header.payload)).
//! 6. Per-request headers: `Authorization: Ghost <token>`,
//!    `Accept-Version: v5.0`, `Content-Type: application/json`.
//!
//! Endpoints:
//! - POST /ghost/api/admin/posts/?source=html — create. Body:
//!   `{posts:[{title, html, tags:[{name}], status, slug,
//!   custom_excerpt, feature_image}]}`. The `?source=html` query
//!   param tells Ghost to convert the HTML body to its internal
//!   Lexical format server-side.
//! - POST /ghost/api/admin/images/upload/ — multipart, field `file`,
//!   optional `purpose=image`. Returns `images[0].url`.
//! - GET /ghost/api/admin/site/ — connectivity smoke check.
//!
//! Reference: https://docs.ghost.org/admin-api/

use crate::models::publish::PlatformConfig;
use crate::services::publish::types::{PublishError, PublishInput, PublishResult};
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;

const ADMIN_PATH_PREFIX: &str = "/ghost/api/admin";
const ACCEPT_VERSION: &str = "v5.0";
const TOKEN_TTL_SECONDS: i64 = 5 * 60;

/// Read-only credentials check: `GET /ghost/api/admin/site/` with a
/// fresh JWT. Returns the site title on success — surfaces nicely in
/// the Test button's status pane.
pub async fn verify(admin_url: &str, api_key: &str) -> Result<String, PublishError> {
    if admin_url.is_empty() || api_key.is_empty() {
        return Err(PublishError::BadConfig(
            "ghost config missing admin_url or api_key".into(),
        ));
    }
    let token = make_jwt(api_key)?;
    let endpoint = format!(
        "{}{}/site/",
        admin_url.trim_end_matches('/'),
        ADMIN_PATH_PREFIX
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| PublishError::Network(e.to_string()))?;
    let resp = client
        .get(&endpoint)
        .header("Authorization", format!("Ghost {token}"))
        .header("Accept-Version", ACCEPT_VERSION)
        .send()
        .await
        .map_err(|e| PublishError::Network(e.to_string()))?;
    let resp = crate::services::publish::require_success(resp).await?;
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| PublishError::UnexpectedResponse(format!("json: {e}")))?;
    let title = body
        .pointer("/site/title")
        .and_then(|v| v.as_str())
        .unwrap_or("Ghost site");
    Ok(format!("Connected to {title}"))
}

/// Split `id:secret_hex`, hex-decode the secret, build a fresh JWT.
pub fn make_jwt(api_key: &str) -> Result<String, PublishError> {
    make_jwt_with_clock(api_key, chrono::Utc::now().timestamp())
}

pub fn make_jwt_with_clock(api_key: &str, now_secs: i64) -> Result<String, PublishError> {
    let trimmed = api_key.trim();
    let (id, secret_hex) = trimmed.split_once(':').ok_or_else(|| {
        PublishError::BadConfig(
            "Ghost api_key must be 'id:secret' (Admin API Key from Ghost Admin → \
             Integrations). The Content API Key is a single hex string with no \
             colon and cannot publish — use the Admin API Key instead."
                .into(),
        )
    })?;
    if id.is_empty() || secret_hex.is_empty() {
        return Err(PublishError::BadConfig(
            "Ghost api_key has empty id or secret half".into(),
        ));
    }
    let key_bytes = hex_decode(secret_hex).map_err(|e| {
        PublishError::BadConfig(format!(
            "Ghost api_key secret half must be hex characters (got: {e}). \
             Make sure you copied the full Admin API Key from Ghost Admin → \
             Integrations → your integration."
        ))
    })?;

    let header = serde_json::json!({"alg":"HS256","kid":id,"typ":"JWT"});
    let header_b64 = b64url(&serde_json::to_vec(&header).unwrap());

    let payload = serde_json::json!({
        "iat": now_secs,
        "exp": now_secs + TOKEN_TTL_SECONDS,
        "aud": "/admin/",
    });
    let payload_b64 = b64url(&serde_json::to_vec(&payload).unwrap());

    let signing_input = format!("{header_b64}.{payload_b64}");
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(&key_bytes)
        .map_err(|e| PublishError::BadConfig(format!("hmac init: {e}")))?;
    mac.update(signing_input.as_bytes());
    let sig = mac.finalize().into_bytes();
    let sig_b64 = b64url(&sig);
    Ok(format!("{signing_input}.{sig_b64}"))
}

fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err(format!("odd length: {}", s.len()));
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    for chunk in bytes.chunks(2) {
        let hi = hex_nibble(chunk[0])?;
        let lo = hex_nibble(chunk[1])?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

fn hex_nibble(b: u8) -> Result<u8, String> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(format!("not hex: {b:#x}")),
    }
}

pub async fn upload_image(
    admin_url: &str,
    api_key: &str,
    bytes: Vec<u8>,
    filename: String,
    mime: String,
) -> Result<String, PublishError> {
    let token = make_jwt(api_key)?;
    let endpoint = format!(
        "{}{}/images/upload/",
        admin_url.trim_end_matches('/'),
        ADMIN_PATH_PREFIX
    );
    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name(filename)
        .mime_str(&mime)
        .map_err(|e| PublishError::BadConfig(format!("bad mime: {e}")))?;
    let form = reqwest::multipart::Form::new()
        .text("purpose", "image")
        .part("file", part);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| PublishError::Network(e.to_string()))?;
    let resp = client
        .post(&endpoint)
        .header("Authorization", format!("Ghost {token}"))
        .header("Accept-Version", ACCEPT_VERSION)
        .multipart(form)
        .send()
        .await
        .map_err(|e| PublishError::Network(e.to_string()))?;
    let resp = crate::services::publish::require_success(resp).await?;
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| PublishError::UnexpectedResponse(format!("json: {e}")))?;
    body.pointer("/images/0/url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| PublishError::UnexpectedResponse("no images[0].url".into()))
}

pub async fn publish(
    config: &PlatformConfig,
    input: &PublishInput,
) -> Result<PublishResult, PublishError> {
    let (admin_url, api_key) = match config {
        PlatformConfig::Ghost { admin_url, api_key } => (admin_url, api_key),
        _ => return Err(PublishError::BadConfig("not a Ghost config".into())),
    };
    if admin_url.is_empty() || api_key.is_empty() {
        return Err(PublishError::BadConfig(
            "ghost config missing admin_url or api_key".into(),
        ));
    }
    let token = make_jwt(api_key)?;

    let tags: Vec<serde_json::Value> = input
        .tags
        .iter()
        .map(|t| serde_json::json!({"name": t}))
        .collect();

    let mut post = serde_json::json!({
        "title": input.title,
        "html": input.body,
        "tags": tags,
        "status": input.status,
    });
    if let Some(slug) = &input.slug {
        post["slug"] = serde_json::Value::String(slug.clone());
    }
    if let Some(excerpt) = &input.excerpt {
        post["custom_excerpt"] = serde_json::Value::String(excerpt.clone());
    }
    if let Some(image) = &input.feature_image_url {
        post["feature_image"] = serde_json::Value::String(image.clone());
    }
    let payload = serde_json::json!({"posts": [post]});

    let endpoint = format!(
        "{}{}/posts/?source=html",
        admin_url.trim_end_matches('/'),
        ADMIN_PATH_PREFIX
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| PublishError::Network(e.to_string()))?;
    let resp = client
        .post(&endpoint)
        .header("Authorization", format!("Ghost {token}"))
        .header("Accept-Version", ACCEPT_VERSION)
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
        .pointer("/posts/0/id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PublishError::UnexpectedResponse("no posts[0].id".into()))?
        .to_string();
    let url = body
        .pointer("/posts/0/url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PublishError::UnexpectedResponse("no posts[0].url".into()))?
        .to_string();
    Ok(PublishResult { url, remote_id: id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::publish::types::BodyFormat;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn cfg(admin_url: &str) -> PlatformConfig {
        PlatformConfig::Ghost {
            admin_url: admin_url.into(),
            // 32-byte secret as 64 hex chars
            api_key: "abc:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd".into(),
        }
    }

    fn input() -> PublishInput {
        PublishInput {
            title: "Hello".into(),
            body: "<p>body</p>".into(),
            body_format: BodyFormat::Html,
            tags: vec!["rust".into()],
            slug: Some("hello".into()),
            excerpt: Some("brief".into()),
            status: "draft".into(),
            feature_image_url: None,
            featured_media_id: None,
            publication_id: None,
        }
    }

    #[test]
    fn make_jwt_has_three_dot_segments_with_correct_kid() {
        let token = make_jwt_with_clock(
            "abc:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd",
            1_700_000_000,
        )
        .unwrap();
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
        // Decode header and verify kid
        let header_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[0])
            .unwrap();
        let header: serde_json::Value = serde_json::from_slice(&header_bytes).unwrap();
        assert_eq!(header["kid"], "abc");
        assert_eq!(header["alg"], "HS256");
        assert_eq!(header["typ"], "JWT");
        // Decode payload and verify exp = iat + 300
        let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).unwrap();
        assert_eq!(payload["iat"], 1_700_000_000);
        assert_eq!(payload["exp"], 1_700_000_000 + 300);
        assert_eq!(payload["aud"], "/admin/");
    }

    #[test]
    fn make_jwt_signature_is_deterministic_for_fixed_inputs() {
        let a = make_jwt_with_clock(
            "abc:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd",
            1_700_000_000,
        )
        .unwrap();
        let b = make_jwt_with_clock(
            "abc:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd",
            1_700_000_000,
        )
        .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn malformed_api_key_is_bad_config() {
        let err = make_jwt("no-colon-here").unwrap_err();
        assert!(matches!(err, PublishError::BadConfig(_)), "got {err:?}");
        // The error should mention Admin vs Content API key distinction so
        // a user pasting the wrong key type gets actionable guidance.
        if let PublishError::BadConfig(msg) = err {
            assert!(
                msg.contains("Admin API Key") && msg.contains("Content API Key"),
                "error should distinguish Admin vs Content API Key, got: {msg}"
            );
        }
        let err = make_jwt("only-id:").unwrap_err();
        assert!(matches!(err, PublishError::BadConfig(_)), "got {err:?}");
    }

    #[test]
    fn whitespace_in_api_key_is_trimmed() {
        // Users sometimes paste with surrounding whitespace.
        let token = make_jwt_with_clock(
            "  abc:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd  ",
            1_700_000_000,
        )
        .unwrap();
        assert_eq!(token.split('.').count(), 3);
    }

    #[tokio::test]
    async fn upload_image_returns_url() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/ghost/api/admin/images/upload/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "images": [{"url": "https://blog.example.com/content/images/x.png", "ref": null}]
            })))
            .mount(&server)
            .await;
        let url = upload_image(
            &server.uri(),
            "abc:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd",
            vec![1, 2, 3],
            "x.png".into(),
            "image/png".into(),
        )
        .await
        .unwrap();
        assert_eq!(url, "https://blog.example.com/content/images/x.png");
    }

    #[tokio::test]
    async fn publish_creates_post_via_source_html() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/ghost/api/admin/posts/"))
            .and(query_param("source", "html"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "posts": [{
                    "id": "p123",
                    "url": "https://blog.example.com/hello/",
                    "title": "Hello"
                }]
            })))
            .mount(&server)
            .await;
        let result = publish(&cfg(&server.uri()), &input()).await.unwrap();
        assert_eq!(result.remote_id, "p123");
        assert_eq!(result.url, "https://blog.example.com/hello/");
    }

    #[tokio::test]
    async fn publish_401_is_auth_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401).set_body_string("invalid token"))
            .mount(&server)
            .await;
        let err = publish(&cfg(&server.uri()), &input()).await.unwrap_err();
        assert!(matches!(err, PublishError::Auth(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn publish_5xx_is_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(503).set_body_string("down"))
            .mount(&server)
            .await;
        let err = publish(&cfg(&server.uri()), &input()).await.unwrap_err();
        assert!(
            matches!(err, PublishError::Server { status: 503, .. }),
            "got {err:?}"
        );
    }

    #[tokio::test]
    async fn empty_field_is_bad_config() {
        let cfg = PlatformConfig::Ghost {
            admin_url: "".into(),
            api_key: "id:secret".into(),
        };
        let err = publish(&cfg, &input()).await.unwrap_err();
        assert!(matches!(err, PublishError::BadConfig(_)), "got {err:?}");
    }
}
