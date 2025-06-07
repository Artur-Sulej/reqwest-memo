use reqwest_middleware::ClientBuilder;
use memo::VcrCacheMiddleware;

#[tokio::main]
async fn main() {
    let vcr = VcrCacheMiddleware::new("cache_dir");
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(vcr)
        .build();

    let resp = client.get("https://httpbin.org/get").send().await.unwrap();
    println!("Status: {}", resp.status());
    println!("Body: {}", resp.text().await.unwrap());
}
