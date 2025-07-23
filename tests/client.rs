use reqwest_replay::ClientBuilder;
use serde_json::json;

#[tokio::test]
async fn cached_response() {
    let client = ClientBuilder::new().build();
    let response = client
        .post("http://localhost:3000")
        .json(&json!({ "query": "crab" }))
        .send()
        .await
        .unwrap();
    let resp_status = response.status();
    let resp_text = response.text().await.unwrap();

    assert_eq!(resp_status, 200);
    assert!(resp_text.contains("Clumsy crab counts clouds."));
}

#[tokio::test]
async fn cached_response_custom_dir() {
    let client = ClientBuilder::new().cache_dir("cache_v2").build();
    let response = client.get("http://localhost:3000").send().await.unwrap();
    let resp_status = response.status();
    let resp_text = response.text().await.unwrap();

    assert_eq!(resp_status, 200);
    assert!(resp_text.contains("Crab wears jellyfish hat."));
}

#[tokio::test]
async fn http_response() {
    let _ = std::fs::remove_file(
        "cache/9d734edfacdb4968bf6f1c90ee7adf5a274c63b5edc30605ea9933cca6d7792e.json",
    );

    let client = ClientBuilder::new().build();
    let response = client
        .get("https://httpbin.org/get?other")
        .send()
        .await
        .unwrap();
    let resp_text = response.text().await.unwrap();

    assert!(!resp_text.contains("Clumsy crab counts clouds."));
}
