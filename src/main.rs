use reqwest_memo::ClientBuilder;

#[tokio::main]
async fn main() {
    let client = ClientBuilder::new().build();

    let resp = client.get("https://httpbin.org/get").send().await.unwrap();
    println!("Status: {}", resp.status());
    println!("Body: {}", resp.text().await.unwrap());
}
