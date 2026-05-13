//! Qiniu Kodo uploader.
//!
//! Wire (form-upload v2):
//! 1. Build upload policy JSON: `{ "scope": "<bucket>:<key>", "deadline": <unix-ts + 3600> }`
//! 2. base64-urlsafe (no padding) encode the policy → `encoded_policy`.
//! 3. HMAC-SHA1(secret_key, encoded_policy) → base64-urlsafe (no padding) → `encoded_sign`.
//! 4. `upload_token = "<access_key>:<encoded_sign>:<encoded_policy>"`.
//! 5. multipart POST to `https://upload.qiniup.com` with text fields
//!    `key`, `token` and the file part `file`.
//! 6. Response JSON contains `key` and `hash`. Final URL = `<domain>/<key>`.
//!
//! Reference: https://developer.qiniu.com/kodo/1312/upload

use crate::models::image_host::ProviderConfig;
use crate::services::image_host::types::{HostError, UploadInput, UploadResult};
use base64::Engine;
use hmac::{Hmac, Mac};
use sha1::Sha1;

const DEFAULT_ENDPOINT: &str = "https://upload.qiniup.com";

pub async fn upload(
    config: &ProviderConfig,
    input: UploadInput,
) -> Result<UploadResult, HostError> {
    upload_with_endpoint(config, input, DEFAULT_ENDPOINT.to_string()).await
}

pub async fn upload_with_endpoint(
    config: &ProviderConfig,
    input: UploadInput,
    endpoint: String,
) -> Result<UploadResult, HostError> {
    let (ak, sk, bucket, domain) = match config {
        ProviderConfig::Qiniu {
            access_key,
            secret_key,
            bucket,
            domain,
        } => (access_key, secret_key, bucket, domain),
        _ => return Err(HostError::BadConfig("not a Qiniu config".into())),
    };
    if ak.is_empty() || sk.is_empty() || bucket.is_empty() || domain.is_empty() {
        return Err(HostError::BadConfig(
            "Qiniu config has empty access_key / secret_key / bucket / domain".into(),
        ));
    }

    let token = build_upload_token(ak, sk, bucket, &input.key)?;

    let part = reqwest::multipart::Part::bytes(input.bytes)
        .file_name(input.filename.clone())
        .mime_str(&input.mime)
        .map_err(|e| HostError::BadConfig(format!("bad mime: {e}")))?;
    let form = reqwest::multipart::Form::new()
        .text("key", input.key.clone())
        .text("token", token)
        .part("file", part);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| HostError::Network(e.to_string()))?;

    let resp = client
        .post(&endpoint)
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
        let key = body
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HostError::UnexpectedResponse("no `key` in response".into()))?;
        let url = format!("{}/{}", domain.trim_end_matches('/'), key);
        Ok(UploadResult {
            url,
            remote_key: Some(key.to_string()),
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

fn build_upload_token(ak: &str, sk: &str, bucket: &str, key: &str) -> Result<String, HostError> {
    let scope = format!("{bucket}:{key}");
    let deadline = (chrono::Utc::now().timestamp() as u64) + 3600;
    let policy = serde_json::json!({ "scope": scope, "deadline": deadline });
    let policy_str =
        serde_json::to_string(&policy).map_err(|e| HostError::BadConfig(format!("policy: {e}")))?;
    let encoded_policy =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(policy_str.as_bytes());

    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(sk.as_bytes())
        .map_err(|e| HostError::BadConfig(format!("hmac init: {e}")))?;
    mac.update(encoded_policy.as_bytes());
    let sign = mac.finalize().into_bytes();
    let encoded_sign = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sign);

    Ok(format!("{ak}:{encoded_sign}:{encoded_policy}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn cfg(domain: &str) -> ProviderConfig {
        ProviderConfig::Qiniu {
            access_key: "ak123".into(),
            secret_key: "sk456".into(),
            bucket: "novelist".into(),
            domain: domain.into(),
        }
    }

    fn input(key: &str) -> UploadInput {
        UploadInput {
            bytes: vec![1, 2, 3],
            filename: "x.png".into(),
            mime: "image/png".into(),
            key: key.into(),
        }
    }

    #[tokio::test]
    async fn token_has_three_colon_separated_parts() {
        let token = build_upload_token("ak", "sk", "bucket", "k").unwrap();
        let parts: Vec<&str> = token.split(':').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "ak");
        // sign and policy are non-empty
        assert!(!parts[1].is_empty());
        assert!(!parts[2].is_empty());
    }

    #[tokio::test]
    async fn upload_success_returns_cdn_url() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "key": "2026/05/06/abc-x.png",
                "hash": "FooBar"
            })))
            .mount(&server)
            .await;
        let result = upload_with_endpoint(
            &cfg("https://cdn.example.com"),
            input("2026/05/06/abc-x.png"),
            server.uri(),
        )
        .await
        .unwrap();
        assert_eq!(result.url, "https://cdn.example.com/2026/05/06/abc-x.png");
        assert_eq!(result.remote_key.as_deref(), Some("2026/05/06/abc-x.png"));
    }

    #[tokio::test]
    async fn trailing_slash_in_domain_is_stripped() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"key":"x.png"})),
            )
            .mount(&server)
            .await;
        let result = upload_with_endpoint(
            &cfg("https://cdn.example.com/"),
            input("x.png"),
            server.uri(),
        )
        .await
        .unwrap();
        assert_eq!(result.url, "https://cdn.example.com/x.png");
    }

    #[tokio::test]
    async fn upload_401_maps_to_auth_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401).set_body_string("bad token"))
            .mount(&server)
            .await;
        let err = upload_with_endpoint(&cfg("https://cdn"), input("x"), server.uri())
            .await
            .unwrap_err();
        assert!(matches!(err, HostError::Auth(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn empty_field_is_bad_config() {
        let cfg = ProviderConfig::Qiniu {
            access_key: "".into(),
            secret_key: "sk".into(),
            bucket: "b".into(),
            domain: "d".into(),
        };
        let err = upload(&cfg, input("x")).await.unwrap_err();
        assert!(matches!(err, HostError::BadConfig(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn malformed_response_is_unexpected() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"hash":"only"})),
            )
            .mount(&server)
            .await;
        let err = upload_with_endpoint(&cfg("https://cdn"), input("x"), server.uri())
            .await
            .unwrap_err();
        assert!(
            matches!(err, HostError::UnexpectedResponse(_)),
            "got {err:?}"
        );
    }
}
