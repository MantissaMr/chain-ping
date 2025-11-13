
use std::time::Instant;

#[tokio::main] 
async fn main() {
    println!("Chain Ping starting up...");

    // Hardcode the URL for now
    let url = "https://eth.llamarpc.com";

    println!("Pinging: {}", url);

    let start = Instant::now();

    let client = reqwest::Client::new();
    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });

    let response = client
        .post(url)
        .json(&request_body)
        .send()
        .await;

    let latency = start.elapsed().as_millis();

    // Check the result
    match response {
        Ok(resp) => {
            println!("Success! HTTP Status: {}", resp.status());
            // Try to get the text of the response
            match resp.text().await {
                Ok(text) => println!("Response Body: {}", text),
                Err(e) => println!("Failed to read response body: {}", e),
            }
            println!("Latency: {} ms", latency);
        }
        Err(e) => {
            println!("Request failed entirely: {}", e);
            println!("Latency (until failure): {} ms", latency);
        }
    }
}