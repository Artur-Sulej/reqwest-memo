# reqwest-memo

[![Crates.io](https://img.shields.io/crates/v/reqwest-memo)](https://crates.io/crates/reqwest-memo)
[![Documentation](https://docs.rs/reqwest-memo/badge.svg)](https://docs.rs/reqwest-memo)

A Rust library that adds transparent request/response memoization to `reqwest`. It remembers exact request/response pairs and returns stored responses for identical HTTP requests, making it perfect for development, testing, and reducing API costs.

## Features

- **Request/Response Memoization**: Remembers exact request/response pairs for identical requests
- **Disk-based Storage**: Memoized responses are cached in files on disk for persistence between runs
- **Cache Key Generation**: Each request is uniquely identified by a hash of HTTP method, URL, and body.
- **Simple Integration**: Drop-in replacement for `reqwest` with minimal code changes
- **Configurable**: Customize storage directory location

## Why Use This?

- **Development Efficiency**: Speed up your development workflow by avoiding repeated API calls
- **Cost Savings**: Reduce token usage with paid APIs (e.g., OpenAI, Anthropic) during development
- **Reliable Testing**: Get consistent results for testing by using memoized responses
- **Audit Trail**: Maintain a complete history of API interactions for debugging and analysis
- **Offline Development**: Continue working without an internet connection using previously memoized responses
- **Performance**: Get instant responses for repeated requests without network latency

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
reqwest-memo = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

## Usage

### Basic Usage

```rust
use reqwest_memo::ClientBuilder;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with default settings (memoization directory: "memo_dir")
    let client = ClientBuilder::new().build();
    
    // First request - will be fetched from the network
    let response = client
        .post("https://httpbin.org/post")
        .json(&json!({ "query": "example" }))
        .send()
        .await?;
    
    println!("First request status: {}", response.status());
    
    // Identical second request - will be served from memoization
    let memoized_response = client
        .post("https://httpbin.org/post")
        .json(&json!({ "query": "example" }))
        .send()
        .await?;
    
    println!("Memoized response status: {}", memoized_response.status());
    
    // Different request (different body) - will go to the network
    let new_response = client
        .post("https://httpbin.org/post")
        .json(&json!({ "query": "different" }))
        .send()
        .await?;
    
    println!("New request status: {}", new_response.status());
    
    Ok(())
}
```

### Custom Memoization Directory

By default, memoized responses are stored in a directory called `memo_dir`. You can customize this location:

```rust
use reqwest_memo::ClientBuilder;

let client = ClientBuilder::new()
    .cache_dir("my_custom_cache_dir")  // Custom directory for memoized responses
    .build();

// Or use a platform-appropriate cache directory
let client = ClientBuilder::new()
    .cache_dir(dirs::cache_dir().unwrap().join("my_app/cache"))
    .build();
```

## How It Works

1. **Request Hashing**: When a request is made, the middleware generates a unique SHA-256 hash based on:
   - HTTP method (GET, POST, etc.)
   - Full URL including query parameters
   - Complete request body (for POST/PUT/PATCH requests)
   - Request headers (if you want to include them in the future)

2. **Memoization Lookup**: The middleware checks if a memoized response exists for this exact hash

3. **Response Handling**:
   - If found: Returns the memoized response immediately (no network call)
   - If not found: Forwards the request to the server, then stores the response for future use

4. **Storage**: Memoized responses are stored as JSON files in the specified directory, with filenames matching the request hash. Each file contains both the original request and response data, making it easy to inspect or debug.

### Security Note

Currently, this lib stores complete request and response data on disk, including any auth tokens or API keys. Please keep this in mind when handling sensitive data. Future versions may address this issue.

## License

This project is licensed under the MIT License – see the [LICENSE](LICENSE.md) file for details.
