use async_trait::async_trait;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
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
struct CachedResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
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
            if let Ok(cached) = serde_json::from_slice::<CachedResponse>(&bytes) {
                println!("Cache HIT for {}", req.url());

                let mut response_builder = http::Response::builder().status(cached.status);
                for (k, v) in cached.headers {
                    response_builder = response_builder.header(&k, &v);
                }
                let http_response = response_builder.body(cached.body).unwrap();
                return Ok(Response::from(http_response));
            }
        }

        println!("Cache MISS for {}", req.url());

        // Make the actual request
        let response = next.run(req, extensions).await?;
        let status = response.status().as_u16();
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response.bytes().await?.to_vec();

        // Save to cache
        let cached = CachedResponse {
            status,
            headers: headers.clone(),
            body: body.clone(),
        };
        fs::create_dir_all(&self.cache_dir).await.ok();
        fs::write(&cache_path, serde_json::to_vec(&cached).unwrap())
            .await
            .ok();

        // Rebuild response
        let mut response_builder = http::Response::builder().status(status);
        for (k, v) in headers {
            response_builder = response_builder.header(&k, &v);
        }
        let http_response = response_builder.body(body).unwrap();
        Ok(Response::from(http_response))
    }
}
