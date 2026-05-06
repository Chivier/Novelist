//! sm.ms uploader (v2 API).
//!
//! Wire: multipart POST to `https://sm.ms/api/v2/upload` with field
//! `smfile=<bytes>`. Optional `Authorization: <token>` (no `Bearer` prefix
//! per sm.ms docs).
//!
//! Response shapes:
//! - Success: `{ "success": true, "data": { "url": "..." } }`
//! - App-level error (status still 200): `{ "success": false, "code": "...", "message": "..." }`
//!
//! 5 MB hard limit per sm.ms docs.
//!
//! Reference: https://doc.sm.ms/#api-Image-Upload

use crate::models::image_host::ProviderConfig;
use crate::services::image_host::types::{HostError, UploadInput, UploadResult};

const ENDPOINT: &str = "https://sm.ms/api/v2/upload";
const MAX_BYTES: u64 = 5 * 1024 * 1024;

pub async fn upload(config: &ProviderConfig, input: UploadInput) -> Result<UploadResult, HostError> {
    upload_with_endpoint(config, input, ENDPOINT.to_string()).await
}

pub async fn upload_with_endpoint(
    config: &ProviderConfig,
    input: UploadInput,
    endpoint: String,
) -> Result<UploadResult, HostError> {
    let token = match config {
        ProviderConfig::Smms { api_token } => api_token.clone(),
        _ => return Err(HostError::BadConfig("not an sm.ms config".into())),
    };

    let actual = input.bytes.len() as u64;
    if actual > MAX_BYTES {
        return Err(HostError::FileTooLarge {
            limit: MAX_BYTES,
            actual,
        });
    }

    let part = reqwest::multipart::Part::bytes(input.bytes)
        .file_name(input.filename.clone())
        .mime_str(&input.mime)
        .map_err(|e| HostError::BadConfig(format!("bad mime: {e}")))?;
    let form = reqwest::multipart::Form::new().part("smfile", part);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| HostError::Network(e.to_string()))?;

    let mut req = client.post(&endpoint).multipart(form);
    if let Some(t) = token.as_deref() {
        if !t.is_empty() {
            req = req.header("Authorization", t);
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

    let success = body.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
    if !success {
        let code = body
            .get("code")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let message = body
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("(no message)");
        // sm.ms returns code "image_repeated" for already-uploaded files,
        // with the original URL in `images`. Treat that as success.
        if code == "image_repeated" {
            if let Some(url) = body.get("images").and_then(|v| v.as_str()) {
                return Ok(UploadResult {
                    url: url.to_string(),
                    remote_key: None,
                });
            }
        }
        return Err(HostError::HostError {
            status: 200,
            message: format!("{code}: {message}"),
        });
    }

    let url = body
        .pointer("/data/url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| HostError::UnexpectedResponse("no data.url".into()))?;
    Ok(UploadResult {
        url: url.to_string(),
        remote_key: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn cfg(token: Option<&str>) -> ProviderConfig {
        ProviderConfig::Smms {
            api_token: token.map(|s| s.to_string()),
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
    async fn upload_success_returns_url() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v2/upload"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": { "url": "https://s2.loli.net/abc.png" }
            })))
            .mount(&server)
            .await;
        let url = format!("{}/api/v2/upload", server.uri());
        let result = upload_with_endpoint(&cfg(Some("tok")), input(), url)
            .await
            .unwrap();
        assert_eq!(result.url, "https://s2.loli.net/abc.png");
    }

    #[tokio::test]
    async fn app_level_failure_maps_to_host_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": false,
                "code": "exception",
                "message": "no idea"
            })))
            .mount(&server)
            .await;
        let url = format!("{}/api/v2/upload", server.uri());
        let err = upload_with_endpoint(&cfg(None), input(), url).await.unwrap_err();
        match err {
            HostError::HostError { status, message } => {
                assert_eq!(status, 200);
                assert!(message.contains("exception"), "got: {message}");
            }
            other => panic!("expected HostError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn image_repeated_returns_existing_url() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": false,
                "code": "image_repeated",
                "message": "already uploaded",
                "images": "https://s2.loli.net/existing.png"
            })))
            .mount(&server)
            .await;
        let url = format!("{}/api/v2/upload", server.uri());
        let result = upload_with_endpoint(&cfg(None), input(), url)
            .await
            .unwrap();
        assert_eq!(result.url, "https://s2.loli.net/existing.png");
    }

    #[tokio::test]
    async fn rejects_files_over_5mb() {
        let big = UploadInput {
            bytes: vec![0u8; 5 * 1024 * 1024 + 1],
            filename: "x.png".into(),
            mime: "image/png".into(),
            key: "x".into(),
        };
        let err = upload(&cfg(None), big).await.unwrap_err();
        assert!(matches!(err, HostError::FileTooLarge { .. }), "got {err:?}");
    }

    #[tokio::test]
    async fn upload_429_maps_to_quota_exceeded() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(429).set_body_string("rate limited"))
            .mount(&server)
            .await;
        let url = format!("{}/api/v2/upload", server.uri());
        let err = upload_with_endpoint(&cfg(None), input(), url).await.unwrap_err();
        assert!(matches!(err, HostError::QuotaExceeded(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn no_token_works() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "success": true,
                    "data": { "url": "https://s2.loli.net/anon.png" }
                })),
            )
            .mount(&server)
            .await;
        let url = format!("{}/api/v2/upload", server.uri());
        let result = upload_with_endpoint(&cfg(None), input(), url)
            .await
            .unwrap();
        assert_eq!(result.url, "https://s2.loli.net/anon.png");
    }
}
