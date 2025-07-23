//! Transparent HTTP request/response recording and replay for `reqwest`.
//!
//! This crate provides a middleware layer for `reqwest` that automatically records
//! each HTTP request and its response to separate files on disk. When an identical request
//! is made again, the stored response is returned instantly, skipping the network.
//!
//! # Example
//! ```rust
//! use reqwest_replay::ClientBuilder;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = ClientBuilder::new().cache_dir("docs_cache").build();
//!
//!     // First request - will be fetched from the network
//!     let response = client
//!         .post("https://httpbin.org/post")
//!         .json(&json!({ "query": "example" }))
//!         .send()
//!         .await
//!         .unwrap();
//!
//!     println!("First request status: {}", response.status());
//!
//!     // Identical second request - will be served from cache file
//!     let cached_response = client
//!         .post("https://httpbin.org/post")
//!         .json(&json!({ "query": "example" }))
//!         .send()
//!         .await
//!         .unwrap();
//!
//!     println!("Cached response status: {}", cached_response.status());
//! }
//! ```
//!

pub mod middleware;
pub use middleware::ClientBuilder;
