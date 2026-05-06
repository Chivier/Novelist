//! imgur uploader.
//!
//! Wire: multipart POST to https://api.imgur.com/3/image with field
//! `image=<bytes>` and header `Authorization: Client-ID <client_id>`.
//! Response: `{ "data": { "link": "..." }, "success": true }`.
//! Anonymous (client-credentials only); no per-user OAuth.
//!
//! 10 MB hard limit per imgur's docs — checked client-side before upload.

use crate::models::image_host::ProviderConfig;
use crate::services::image_host::types::{HostError, UploadInput, UploadResult};

const ENDPOINT: &str = "https://api.imgur.com/3/image";
const MAX_BYTES: u64 = 10 * 1024 * 1024;

pub async fn upload(
    config: &ProviderConfig,
    input: UploadInput,
) -> Result<UploadResult, HostError> {
    upload_with_endpoint(config, input, ENDPOINT.to_string()).await
}

pub async fn upload_with_endpoint(
    config: &ProviderConfig,
    input: UploadInput,
    endpoint: String,
) -> Result<UploadResult, HostError> {
    let client_id = match config {
        ProviderConfig::Imgur { client_id } => client_id,
        _ => return Err(HostError::BadConfig("not an imgur config".into())),
    };
    if client_id.is_empty() {
        return Err(HostError::BadConfig("imgur client_id is empty".into()));
    }

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
    let form = reqwest::multipart::Form::new().part("image", part);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| HostError::Network(e.to_string()))?;

    let resp = client
        .post(&endpoint)
        .header("Authorization", format!("Client-ID {client_id}"))
        .multipart(form)
        .send()
        .await
        .map_err(|e| HostError::Network(e.to_string()))?;

    let status = resp.status();
    if status.is_success() {
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| HostError::UnexpectedResponse(format!("json: {e}")))?;
        let link = body
            .pointer("/data/link")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HostError::UnexpectedResponse("no data.link".into()))?;
        Ok(UploadResult {
            url: link.to_string(),
            remote_key: None,
        })
    } else if matches!(status.as_u16(), 401 | 403) {
        Err(HostError::Auth(resp.text().await.unwrap_or_default()))
    } else if status.as_u16() == 429 {
        Err(HostError::QuotaExceeded(
            resp.text().await.unwrap_or_default(),
        ))
    } else {
        Err(HostError::Server {
            status: status.as_u16(),
            message: resp.text().await.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn cfg() -> ProviderConfig {
        ProviderConfig::Imgur {
            client_id: "xyz".into(),
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
    async fn upload_success_returns_link() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/3/image"))
            .and(header("Authorization", "Client-ID xyz"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "link": "https://i.imgur.com/abc.png" },
                "success": true
            })))
            .mount(&server)
            .await;
        let url = format!("{}/3/image", server.uri());
        let result = upload_with_endpoint(&cfg(), input(), url).await.unwrap();
        assert_eq!(result.url, "https://i.imgur.com/abc.png");
        assert!(result.remote_key.is_none());
    }

    #[tokio::test]
    async fn upload_401_maps_to_auth_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401).set_body_string("bad client_id"))
            .mount(&server)
            .await;
        let url = format!("{}/3/image", server.uri());
        let err = upload_with_endpoint(&cfg(), input(), url)
            .await
            .unwrap_err();
        assert!(matches!(err, HostError::Auth(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn upload_429_maps_to_quota_exceeded() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(429).set_body_string("too fast"))
            .mount(&server)
            .await;
        let url = format!("{}/3/image", server.uri());
        let err = upload_with_endpoint(&cfg(), input(), url)
            .await
            .unwrap_err();
        assert!(matches!(err, HostError::QuotaExceeded(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn rejects_files_over_10mb() {
        let cfg = ProviderConfig::Imgur {
            client_id: "xyz".into(),
        };
        let bytes = vec![0u8; 10 * 1024 * 1024 + 1];
        let big = UploadInput {
            bytes,
            filename: "x.png".into(),
            mime: "image/png".into(),
            key: "x".into(),
        };
        let err = upload(&cfg, big).await.unwrap_err();
        assert!(matches!(err, HostError::FileTooLarge { .. }), "got {err:?}");
    }

    #[tokio::test]
    async fn empty_client_id_is_bad_config() {
        let cfg = ProviderConfig::Imgur {
            client_id: "".into(),
        };
        let err = upload(&cfg, input()).await.unwrap_err();
        assert!(matches!(err, HostError::BadConfig(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn malformed_response_is_unexpected() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&server)
            .await;
        let url = format!("{}/3/image", server.uri());
        let err = upload_with_endpoint(&cfg(), input(), url)
            .await
            .unwrap_err();
        assert!(
            matches!(err, HostError::UnexpectedResponse(_)),
            "got {err:?}"
        );
    }

    #[tokio::test]
    async fn wrong_provider_config_is_bad_config() {
        let cfg = ProviderConfig::Smms { api_token: None };
        let err = upload(&cfg, input()).await.unwrap_err();
        assert!(matches!(err, HostError::BadConfig(_)), "got {err:?}");
    }
}
