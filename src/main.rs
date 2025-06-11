use memo::middleware::MemoClientBuilder;

#[tokio::main]
async fn main() {
    let client = MemoClientBuilder::new().build();

    let resp = client.get("https://httpbin.org/get").send().await.unwrap();
    println!("Status: {}", resp.status());
    println!("Body: {}", resp.text().await.unwrap());
}
