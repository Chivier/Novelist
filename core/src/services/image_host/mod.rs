//! Image-host upload services. One module per provider; no shared trait.
//!
//! Each provider's public `upload(config, input)` async fn talks directly
//! to the provider's REST API using `reqwest`. They share `types.rs` for
//! request/response/error shapes and `naming.rs` for object-key generation.

pub mod naming;
pub mod types;

// Provider modules are added one at a time as each is implemented.
pub mod aliyun_oss;
pub mod custom;
pub mod imgur;
pub mod qiniu;
pub mod s3;
pub mod smms;
