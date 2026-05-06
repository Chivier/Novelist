//! Minimal custom HTTP uploader.
//!
//! Wire: multipart POST to `post_url` with field `file=<bytes>`. Optional
//! `Authorization: Bearer <bearer>` header. Response must be JSON; the URL
//! is read from the field at `url` first, falling back to `data.url`.
//!
//! Intentionally minimal — no header/body templating, no JSONPath, no
//! signing. For S3-compatible buckets, S3-specific signing, or any
//! HMAC-based scheme, use one of the dedicated providers instead.

use crate::models::image_host::ProviderConfig;
use crate::services::image_host::types::{HostError, UploadInput, UploadResult};

pub async fn upload(config: &ProviderConfig, input: UploadInput) -> Result<UploadResult, HostError> {
    let (post_url, bearer) = match config {
        ProviderConfig::Custom { post_url, bearer } => (post_url.clone(), bearer.clone()),
        _ => return Err(HostError::BadConfig("not a Custom config".into())),
    };
    if post_url.is_empty() {
        return Err(HostError::BadConfig("custom post_url is empty".into()));
    }

    let part = reqwest::multipart::Part::bytes(input.bytes)
        .file_name(input.filename.clone())
        .mime_str(&input.mime)
        .map_err(|e| HostError::BadConfig(format!("bad mime: {e}")))?;
    let form = reqwest::multipart::Form::new().part("file", part);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| HostError::Network(e.to_string()))?;

    let mut req = client.post(&post_url).multipart(form);
    if let Some(b) = bearer.as_deref() {
        if !b.is_empty() {
            req = req.header("Authorization", format!("Bearer {b}"));
        }
    }
    let resp = req
        .send()
        .await
        .map_err(|e| HostError::Network(e.to_string()))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(match status.as_u16() {
            401 | 403 => HostError::Auth(resp.text().await.unwrap_or_default()),
            429 => HostError::QuotaExceeded(resp.text().await.unwrap_or_default()),
            s => HostError::HostError {
                status: s,
                message: resp.text().await.unwrap_or_default(),
            },
        });
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| HostError::UnexpectedResponse(format!("json: {e}")))?;
    let url = body
        .get("url")
        .and_then(|v| v.as_str())
        .or_else(|| body.pointer("/data/url").and_then(|v| v.as_str()))
        .ok_or_else(|| HostError::UnexpectedResponse("no `url` or `data.url` in response".into()))?;
    Ok(UploadResult {
        url: url.to_string(),
        remote_key: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn cfg(post_url: &str, bearer: Option<&str>) -> ProviderConfig {
        ProviderConfig::Custom {
            post_url: post_url.into(),
            bearer: bearer.map(|s| s.to_string()),
        }
    }

    fn input() -> UploadInput {
        UploadInput {
            bytes: vec![1, 2, 3],
            filename: "x.png".into(),
            mime: "image/png".into(),
            key: "ignored".into(),
        }
    }

    #[tokio::test]
    async fn upload_uses_url_field_at_top_level() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/upload"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "url": "https://cdn.example.com/x.png"
                })),
            )
            .mount(&server)
            .await;
        let url = format!("{}/upload", server.uri());
        let result = upload(&cfg(&url, None), input()).await.unwrap();
        assert_eq!(result.url, "https://cdn.example.com/x.png");
    }

    #[tokio::test]
    async fn upload_falls_back_to_data_url() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": { "url": "https://cdn.example.com/y.png" }
                })),
            )
            .mount(&server)
            .await;
        let url = format!("{}/upload", server.uri());
        let result = upload(&cfg(&url, None), input()).await.unwrap();
        assert_eq!(result.url, "https://cdn.example.com/y.png");
    }

    #[tokio::test]
    async fn bearer_header_added_when_set() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header("Authorization", "Bearer mytoken"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"url":"https://x"})),
            )
            .mount(&server)
            .await;
        let url = format!("{}/upload", server.uri());
        let result = upload(&cfg(&url, Some("mytoken")), input()).await.unwrap();
        assert_eq!(result.url, "https://x");
    }

    #[tokio::test]
    async fn missing_url_field_is_unexpected_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"hash":"only"})),
            )
            .mount(&server)
            .await;
        let url = format!("{}/upload", server.uri());
        let err = upload(&cfg(&url, None), input()).await.unwrap_err();
        assert!(matches!(err, HostError::UnexpectedResponse(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn empty_post_url_is_bad_config() {
        let err = upload(&cfg("", None), input()).await.unwrap_err();
        assert!(matches!(err, HostError::BadConfig(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn five_xx_maps_to_host_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(503).set_body_string("down"))
            .mount(&server)
            .await;
        let url = format!("{}/upload", server.uri());
        let err = upload(&cfg(&url, None), input()).await.unwrap_err();
        assert!(matches!(err, HostError::HostError { status: 503, .. }), "got {err:?}");
    }
}
