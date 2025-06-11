use async_trait::async_trait;
use reqwest::{Request, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware, Next, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use task_local_extensions::Extensions;
use tokio::fs;

pub struct VcrCacheMiddleware {
    cache_dir: PathBuf,
}

impl VcrCacheMiddleware {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
        }
    }

    fn cache_file_path(&self, req: &Request) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(req.method().as_str());
        hasher.update(req.url().as_str());
        if let Some(body) = req.body() {
            if let Some(bytes) = body.as_bytes() {
                hasher.update(bytes);
            }
        }
        let hash = hex::encode(hasher.finalize());
        self.cache_dir.join(format!("{}.json", hash))
    }
}

#[derive(Serialize, Deserialize)]
struct CachedRequest {
    method: String,
    url: String,
    body: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CachedResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

#[derive(Serialize, Deserialize)]
struct CachedEntry {
    request: CachedRequest,
    response: CachedResponse,
}

#[async_trait]
impl Middleware for VcrCacheMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let cache_path = self.cache_file_path(&req);

        // Try to load from cache
        if let Ok(bytes) = fs::read(&cache_path).await {
            if let Ok(entry) = serde_json::from_slice::<CachedEntry>(&bytes) {
                let cached = entry.response;
                println!("Cache HIT for {} (file: {})", req.url(), cache_path.display());

                let mut response_builder = http::Response::builder().status(cached.status);
                for (k, v) in cached.headers {
                    response_builder = response_builder.header(&k, &v);
                }
                let http_response = response_builder.body(cached.body.into_bytes()).unwrap();
                return Ok(Response::from(http_response));
            }
        }

        println!("Cache MISS for {}", req.url());

        // Prepare request info for caching
        let method = req.method().as_str().to_string();
        let url = req.url().to_string();
        let body = req
            .body()
            .and_then(|b| b.as_bytes().map(|b| String::from_utf8_lossy(b).to_string()));
        let cached_request = CachedRequest { method, url, body };

        // Make the actual request
        let response = next.run(req, extensions).await?;
        let status = response.status().as_u16();
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let bytes = response.bytes().await?;
        let body_str = String::from_utf8_lossy(&bytes).to_string();

        // Save to cache
        let cached_response = CachedResponse {
            status,
            headers: headers.clone(),
            body: body_str.clone(),
        };
        let entry = CachedEntry {
            request: cached_request,
            response: cached_response,
        };
        fs::create_dir_all(&self.cache_dir).await.ok();
        fs::write(&cache_path, serde_json::to_string_pretty(&entry).unwrap())
            .await
            .ok();

        // Rebuild response
        let mut response_builder = http::Response::builder().status(status);
        for (k, v) in headers {
            response_builder = response_builder.header(&k, &v);
        }
        let http_response = response_builder.body(bytes.to_vec()).unwrap();
        Ok(Response::from(http_response))
    }
}

// --- MemoClientBuilder for ergonomic client construction ---
pub struct MemoClientBuilder {
    cache_dir: String,
    // add more options here in the future
}

impl MemoClientBuilder {
    pub fn new() -> Self {
        Self {
            cache_dir: "cache_dir".to_string(),
        }
    }

    pub fn cache_dir(mut self, dir: impl Into<String>) -> Self {
        self.cache_dir = dir.into();
        self
    }

    pub fn build(self) -> ClientWithMiddleware {
        let vcr = VcrCacheMiddleware::new(self.cache_dir);
        ClientBuilder::new(reqwest::Client::new()).with(vcr).build()
    }
}
