//! Amazon S3 / Cloudflare R2 / MinIO uploader (SigV4 PUT, path-style).
//!
//! Wire: PUT `<endpoint or https://s3.<region>.amazonaws.com>/<bucket>/<key>`
//! with body = raw image bytes. Headers:
//! - `Host`
//! - `x-amz-content-sha256` = hex(sha256(body))
//! - `x-amz-date` = `YYYYMMDDTHHMMSSZ`
//! - `Content-Type` (signed)
//! - `Content-Length` (auto-set by reqwest, NOT signed in canonical request
//!   here for simplicity — AWS allows omitting it from `SignedHeaders`)
//! - `Authorization` = SigV4 string
//!
//! Path-style is used unconditionally so the same code works for AWS,
//! Cloudflare R2, and MinIO without DNS setup. The final public URL falls
//! back to `custom_domain/<key>` when set.
//!
//! Reference: https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-header-based-auth.html

use crate::models::image_host::ProviderConfig;
use crate::services::image_host::types::{HostError, UploadInput, UploadResult};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

const SERVICE: &str = "s3";

pub async fn upload(
    config: &ProviderConfig,
    input: UploadInput,
) -> Result<UploadResult, HostError> {
    let (ak, sk, bucket, region, endpoint, path_prefix, custom_domain) = match config {
        ProviderConfig::S3 {
            access_key_id,
            secret_access_key,
            bucket,
            region,
            endpoint,
            path_prefix,
            custom_domain,
        } => (
            access_key_id,
            secret_access_key,
            bucket,
            region,
            endpoint.clone(),
            path_prefix.clone(),
            custom_domain.clone(),
        ),
        _ => return Err(HostError::BadConfig("not an S3 config".into())),
    };
    if ak.is_empty() || sk.is_empty() || bucket.is_empty() || region.is_empty() {
        return Err(HostError::BadConfig(
            "S3 config missing required field".into(),
        ));
    }

    let endpoint = endpoint
        .unwrap_or_else(|| format!("https://s3.{region}.amazonaws.com"))
        .trim_end_matches('/')
        .to_string();

    let final_key = match path_prefix.as_deref() {
        Some(p) if !p.is_empty() => {
            format!("{}/{}", p.trim_matches('/'), input.key)
        }
        _ => input.key.clone(),
    };

    let put_url = format!("{endpoint}/{bucket}/{final_key}");
    upload_to_url(
        ak,
        sk,
        bucket,
        region,
        &put_url,
        &final_key,
        &input,
        custom_domain.as_deref(),
        chrono::Utc::now(),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn upload_to_url(
    ak: &str,
    sk: &str,
    bucket: &str,
    region: &str,
    put_url: &str,
    final_key: &str,
    input: &UploadInput,
    custom_domain: Option<&str>,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<UploadResult, HostError> {
    let parsed =
        url::Url::parse(put_url).map_err(|e| HostError::BadConfig(format!("bad endpoint: {e}")))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| HostError::BadConfig("endpoint has no host".into()))?
        .to_string();
    let host_with_port = match parsed.port() {
        Some(p) => format!("{host}:{p}"),
        None => host.clone(),
    };

    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();
    let payload_hash = hex_sha256(&input.bytes);

    // Canonical request
    // path: from URL path verbatim, but each segment URL-encoded per AWS rules.
    let canonical_uri = canonicalize_uri_path(parsed.path());
    let canonical_query = String::new(); // PUT object has no query string here
    let canonical_headers = format!(
        "content-type:{}\nhost:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
        input.mime, host_with_port, payload_hash, amz_date,
    );
    let signed_headers = "content-type;host;x-amz-content-sha256;x-amz-date";
    let canonical_request = format!(
        "PUT\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}",
    );

    let credential_scope = format!("{date_stamp}/{region}/{SERVICE}/aws4_request");
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
        hex_sha256(canonical_request.as_bytes())
    );
    let signing_key = derive_signing_key(sk, &date_stamp, region, SERVICE)?;
    let signature = hex_hmac_sha256(&signing_key, string_to_sign.as_bytes())?;

    let auth = format!(
        "AWS4-HMAC-SHA256 Credential={ak}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}",
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| HostError::Network(e.to_string()))?;

    let resp = client
        .put(put_url)
        .header("Host", host_with_port)
        .header("Content-Type", &input.mime)
        .header("x-amz-content-sha256", payload_hash)
        .header("x-amz-date", amz_date)
        .header("Authorization", auth)
        .body(input.bytes.clone())
        .send()
        .await
        .map_err(|e| HostError::Network(e.to_string()))?;

    let status = resp.status();
    if status.is_success() {
        let url = match custom_domain {
            Some(d) => format!("{}/{}", d.trim_end_matches('/'), final_key),
            None => format!(
                "{}/{}/{}",
                parsed.origin().ascii_serialization(),
                bucket,
                final_key
            ),
        };
        Ok(UploadResult {
            url,
            remote_key: Some(final_key.to_string()),
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

fn hex_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_encode(&hasher.finalize())
}

fn hex_hmac_sha256(key: &[u8], data: &[u8]) -> Result<String, HostError> {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|e| HostError::BadConfig(format!("hmac init: {e}")))?;
    mac.update(data);
    Ok(hex_encode(&mac.finalize().into_bytes()))
}

fn hmac_sha256_raw(key: &[u8], data: &[u8]) -> Result<Vec<u8>, HostError> {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|e| HostError::BadConfig(format!("hmac init: {e}")))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn derive_signing_key(
    sk: &str,
    date_stamp: &str,
    region: &str,
    service: &str,
) -> Result<Vec<u8>, HostError> {
    let k_date = hmac_sha256_raw(format!("AWS4{sk}").as_bytes(), date_stamp.as_bytes())?;
    let k_region = hmac_sha256_raw(&k_date, region.as_bytes())?;
    let k_service = hmac_sha256_raw(&k_region, service.as_bytes())?;
    hmac_sha256_raw(&k_service, b"aws4_request")
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// AWS-style URI path encoding: leave RFC 3986 unreserved chars and `/`,
/// percent-encode everything else. (Object keys may contain `/` segments
/// — they are preserved.)
fn canonicalize_uri_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for c in path.chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' || c == '/' {
            out.push(c);
        } else {
            // ASCII non-special → simple percent-encode of UTF-8 bytes
            let mut buf = [0u8; 4];
            for b in c.encode_utf8(&mut buf).bytes() {
                out.push_str(&format!("%{b:02X}"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use wiremock::matchers::{header_exists, method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn input(key: &str) -> UploadInput {
        UploadInput {
            bytes: vec![1, 2, 3],
            filename: "x.png".into(),
            mime: "image/png".into(),
            key: key.into(),
        }
    }

    /// AWS published test vector: derive_signing_key for
    /// secret="wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
    /// date="20150830", region="us-east-1", service="iam"
    /// expected hex = c4afb1cc5771d871763a393e44b703571b55cc28424d1a5e86da6ed3c154a4b9
    /// (see https://docs.aws.amazon.com/general/latest/gr/signature-v4-examples.html)
    #[test]
    fn derive_signing_key_matches_aws_example() {
        let key = derive_signing_key(
            "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
            "20150830",
            "us-east-1",
            "iam",
        )
        .unwrap();
        assert_eq!(
            hex_encode(&key),
            "c4afb1cc5771d871763a393e44b703571b55cc28424d1a5e86da6ed3c154a4b9"
        );
    }

    #[test]
    fn hex_sha256_empty_string_known() {
        // sha256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(
            hex_sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn canonicalize_uri_keeps_slashes() {
        assert_eq!(
            canonicalize_uri_path("/bucket/2026/05/06/x.png"),
            "/bucket/2026/05/06/x.png"
        );
    }

    #[test]
    fn canonicalize_uri_encodes_spaces_and_punct() {
        assert_eq!(canonicalize_uri_path("/a b/c@d"), "/a%20b/c%40d");
    }

    #[tokio::test]
    async fn upload_to_url_success_uses_path_style_url() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path_regex(r"^/mybucket/.*"))
            .and(header_exists("Authorization"))
            .and(header_exists("x-amz-date"))
            .and(header_exists("x-amz-content-sha256"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        let put_url = format!("{}/mybucket/2026/05/06/x.png", server.uri());
        let result = upload_to_url(
            "AKIAEXAMPLE",
            "secret",
            "mybucket",
            "us-east-1",
            &put_url,
            "2026/05/06/x.png",
            &input("2026/05/06/x.png"),
            None,
            chrono::Utc.with_ymd_and_hms(2026, 5, 6, 12, 0, 0).unwrap(),
        )
        .await
        .unwrap();
        assert!(
            result.url.ends_with("/mybucket/2026/05/06/x.png"),
            "got {}",
            result.url
        );
    }

    #[tokio::test]
    async fn upload_to_url_uses_custom_domain_when_set() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        let put_url = format!("{}/mybucket/x.png", server.uri());
        let result = upload_to_url(
            "AK",
            "SK",
            "mybucket",
            "us-east-1",
            &put_url,
            "x.png",
            &input("x.png"),
            Some("https://images.example.com"),
            chrono::Utc.with_ymd_and_hms(2026, 5, 6, 12, 0, 0).unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(result.url, "https://images.example.com/x.png");
    }

    #[tokio::test]
    async fn upload_to_url_403_maps_to_auth_error() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .respond_with(ResponseTemplate::new(403).set_body_string("SignatureDoesNotMatch"))
            .mount(&server)
            .await;
        let put_url = format!("{}/mybucket/x.png", server.uri());
        let err = upload_to_url(
            "AK",
            "SK",
            "mybucket",
            "us-east-1",
            &put_url,
            "x.png",
            &input("x.png"),
            None,
            chrono::Utc.with_ymd_and_hms(2026, 5, 6, 12, 0, 0).unwrap(),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, HostError::Auth(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn empty_field_is_bad_config() {
        let cfg = ProviderConfig::S3 {
            access_key_id: "".into(),
            secret_access_key: "SK".into(),
            bucket: "b".into(),
            region: "us-east-1".into(),
            endpoint: None,
            path_prefix: None,
            custom_domain: None,
        };
        let err = upload(&cfg, input("x.png")).await.unwrap_err();
        assert!(matches!(err, HostError::BadConfig(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn r2_endpoint_override_works() {
        // Just verify the URL builder handles an explicit endpoint. We
        // don't actually hit Cloudflare; we hit a wiremock standing in.
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        let cfg = ProviderConfig::S3 {
            access_key_id: "AK".into(),
            secret_access_key: "SK".into(),
            bucket: "blog".into(),
            region: "auto".into(),
            endpoint: Some(server.uri()),
            path_prefix: None,
            custom_domain: Some("https://images.example.com".into()),
        };
        let result = upload(&cfg, input("foo.png")).await.unwrap();
        assert_eq!(result.url, "https://images.example.com/foo.png");
    }

    #[tokio::test]
    async fn path_prefix_is_prepended() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path_regex(r"^/blog/uploads/.*"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        let cfg = ProviderConfig::S3 {
            access_key_id: "AK".into(),
            secret_access_key: "SK".into(),
            bucket: "blog".into(),
            region: "us-east-1".into(),
            endpoint: Some(server.uri()),
            path_prefix: Some("uploads".into()),
            custom_domain: None,
        };
        let result = upload(&cfg, input("foo.png")).await.unwrap();
        assert!(
            result.url.contains("/blog/uploads/foo.png"),
            "got {}",
            result.url
        );
        assert_eq!(result.remote_key.as_deref(), Some("uploads/foo.png"));
    }
}
