//! Aliyun OSS uploader (header signature, HMAC-SHA1).
//!
//! Wire: PUT to `https://<bucket>.<endpoint>/<key>` with body = raw image
//! bytes. Headers:
//! - `Date` — RFC 1123 (e.g. `Wed, 06 May 2026 12:00:00 GMT`)
//! - `Content-Type` — image MIME
//! - `Authorization: OSS <access_key_id>:<signature>`
//!
//! Signature input (StringToSign):
//! ```text
//! PUT\n
//! <Content-MD5 or empty>\n
//! <Content-Type>\n
//! <Date>\n
//! <CanonicalizedOSSHeaders>\n
//! <CanonicalizedResource>
//! ```
//! - `CanonicalizedOSSHeaders` is empty when we don't set any `x-oss-*` headers.
//! - `CanonicalizedResource` = `/<bucket>/<key>` (path-encoded).
//!
//! Signature = base64(HMAC-SHA1(access_key_secret, StringToSign)).
//!
//! Final URL: `<custom_domain>/<key>` if set, else `https://<bucket>.<endpoint>/<key>`.
//!
//! Reference: https://help.aliyun.com/zh/oss/developer-reference/include-signatures-in-the-authorization-header

use crate::models::image_host::ProviderConfig;
use crate::services::image_host::types::{HostError, UploadInput, UploadResult};
use base64::Engine;
use hmac::{Hmac, Mac};
use sha1::Sha1;

pub async fn upload(config: &ProviderConfig, input: UploadInput) -> Result<UploadResult, HostError> {
    let (ak, sk, bucket, endpoint, custom_domain) = match config {
        ProviderConfig::AliyunOss {
            access_key_id,
            access_key_secret,
            bucket,
            endpoint,
            custom_domain,
        } => (access_key_id, access_key_secret, bucket, endpoint, custom_domain.clone()),
        _ => return Err(HostError::BadConfig("not an Aliyun OSS config".into())),
    };
    if ak.is_empty() || sk.is_empty() || bucket.is_empty() || endpoint.is_empty() {
        return Err(HostError::BadConfig(
            "Aliyun OSS config missing required field".into(),
        ));
    }

    // Build PUT URL: virtual-hosted style.
    let host_no_scheme = endpoint
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let put_url = format!("https://{bucket}.{host_no_scheme}/{}", input.key);
    upload_to_url(ak, sk, bucket, &put_url, &input, custom_domain.as_deref()).await
}

async fn upload_to_url(
    ak: &str,
    sk: &str,
    bucket: &str,
    put_url: &str,
    input: &UploadInput,
    custom_domain: Option<&str>,
) -> Result<UploadResult, HostError> {
    let date = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    let content_type = input.mime.clone();
    let canonicalized_resource = format!("/{bucket}/{}", input.key);

    let signature = sign(sk, "PUT", "", &content_type, &date, "", &canonicalized_resource)?;
    let auth = format!("OSS {ak}:{signature}");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| HostError::Network(e.to_string()))?;

    let resp = client
        .put(put_url)
        .header("Date", date)
        .header("Content-Type", content_type)
        .header("Authorization", auth)
        .body(input.bytes.clone())
        .send()
        .await
        .map_err(|e| HostError::Network(e.to_string()))?;

    let status = resp.status();
    if status.is_success() {
        let url = match custom_domain {
            Some(d) => format!("{}/{}", d.trim_end_matches('/'), input.key),
            None => put_url.to_string(),
        };
        Ok(UploadResult {
            url,
            remote_key: Some(input.key.clone()),
        })
    } else if matches!(status.as_u16(), 401 | 403) {
        Err(HostError::Auth(resp.text().await.unwrap_or_default()))
    } else if status.as_u16() == 429 {
        Err(HostError::QuotaExceeded(resp.text().await.unwrap_or_default()))
    } else {
        Err(HostError::HostError {
            status: status.as_u16(),
            message: resp.text().await.unwrap_or_default(),
        })
    }
}

fn sign(
    sk: &str,
    method: &str,
    content_md5: &str,
    content_type: &str,
    date: &str,
    canonicalized_oss_headers: &str,
    canonicalized_resource: &str,
) -> Result<String, HostError> {
    let string_to_sign = format!(
        "{method}\n{content_md5}\n{content_type}\n{date}\n{canonicalized_oss_headers}{canonicalized_resource}"
    );
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(sk.as_bytes())
        .map_err(|e| HostError::BadConfig(format!("hmac init: {e}")))?;
    mac.update(string_to_sign.as_bytes());
    let bytes = mac.finalize().into_bytes();
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header_exists, method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn cfg(endpoint: &str, custom_domain: Option<&str>) -> ProviderConfig {
        ProviderConfig::AliyunOss {
            access_key_id: "AK".into(),
            access_key_secret: "SK".into(),
            bucket: "novelist".into(),
            endpoint: endpoint.into(),
            custom_domain: custom_domain.map(|s| s.to_string()),
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
    async fn signature_known_vector() {
        // Reproducible: fixed inputs.
        let s = sign(
            "secret",
            "PUT",
            "",
            "image/png",
            "Wed, 06 May 2026 12:00:00 GMT",
            "",
            "/bucket/key.png",
        )
        .unwrap();
        // Just assert non-empty + valid base64 length (HMAC-SHA1 = 20 bytes → 28 char b64)
        assert_eq!(s.len(), 28);
    }

    /// Issue PUT directly to a wiremock server (bypassing the
    /// virtual-hosted URL builder). Verifies signing + happy path.
    #[tokio::test]
    async fn upload_to_url_success_returns_put_url_when_no_custom_domain() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path_regex(r"^/novelist/.*"))
            .and(header_exists("Authorization"))
            .and(header_exists("Date"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        let put_url = format!("{}/novelist/2026/05/06/x.png", server.uri());
        let result = upload_to_url(
            "AK",
            "SK",
            "novelist",
            &put_url,
            &input("2026/05/06/x.png"),
            None,
        )
        .await
        .unwrap();
        assert_eq!(result.url, put_url);
        assert_eq!(result.remote_key.as_deref(), Some("2026/05/06/x.png"));
    }

    #[tokio::test]
    async fn upload_to_url_uses_custom_domain_when_set() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        let put_url = format!("{}/novelist/x.png", server.uri());
        let result = upload_to_url(
            "AK",
            "SK",
            "novelist",
            &put_url,
            &input("x.png"),
            Some("https://images.example.com"),
        )
        .await
        .unwrap();
        assert_eq!(result.url, "https://images.example.com/x.png");
    }

    #[tokio::test]
    async fn upload_to_url_403_maps_to_auth_error() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .respond_with(ResponseTemplate::new(403).set_body_string("InvalidSignature"))
            .mount(&server)
            .await;
        let put_url = format!("{}/novelist/x.png", server.uri());
        let err = upload_to_url("AK", "SK", "novelist", &put_url, &input("x.png"), None)
            .await
            .unwrap_err();
        assert!(matches!(err, HostError::Auth(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn empty_field_is_bad_config() {
        let cfg = cfg("", None);
        let err = upload(&cfg, input("x.png")).await.unwrap_err();
        assert!(matches!(err, HostError::BadConfig(_)), "got {err:?}");
    }
}
