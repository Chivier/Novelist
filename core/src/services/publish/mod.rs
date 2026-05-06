//! Publish-channel adapters. One module per platform; no shared trait.
//!
//! Each platform's public `publish(config, input)` async fn talks to
//! the platform's REST API directly using `reqwest`. They share
//! `types.rs` (request/response/error shapes) and `pandoc_html.rs`
//! (Markdown → HTML conversion).

pub mod types;

pub mod pandoc_html;

// Provider modules added one at a time as each is implemented.
// pub mod ghost;
// pub mod medium;
// pub mod wordpress;
// pub mod wordpress_com;
