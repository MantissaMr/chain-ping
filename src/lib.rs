
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug)]
/// Complete snapshot of a single ping attempt, containing all relevant metrics and metadata
pub struct PingResult {
    pub endpoint: String,
    pub latency_ms: Option<u128>,
    pub status: PingStatus,
    pub block_number: Option<String>,
    pub error_message: Option<String>,

}

/// Whether the ping was successful or what type of failure occurred
#[derive(Debug)]
/// Categorizes the outcome of a ping attempt for quick status checking  
pub enum PingStatus {
    Success,
    HttpError,
    JsonRpcError,
    Timeout,
    // We can add more variants as needed
}    

#[derive(Debug, Serialize)]
/// Standard JSON-RPC 2.0 request structure for Ethereum API calls
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<String>,
    pub id: u32,
}

#[derive(Debug, Deserialize)]
/// Standard JSON-RPC 2.0 response structure, handling both success and error cases
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<String>,
    pub error: Option<JsonRpcError>,
    pub id: u32,
}

#[derive(Debug, Deserialize)]
/// Error payload within a JSON-RPC response according to JSON-RPC 2.0 spec
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Error)]
/// Represents all possible error conditions that can occur during RPC pinging
pub enum ChainPingError {
    #[error("HTTP request failed: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("JSON-RPC error: {message} (code: {code})")]
    JsonRpcError { code: i32, message: String },
    #[error("Request timeout")]
    Timeout,
    // We'll add more variants as we encounter new error cases
}

/// Convenience type alias for functions that can return our custom error type
pub type Result<T> = std::result::Result<T, ChainPingError>;

/// Pings a given Ethereum JSON-RPC endpoint and returns a structured result
pub async fn ping_endpoint(url: &str) -> PingResult {
    let endpoint = url.to_string();
    
    let client = reqwest::Client::new();
    let request_payload = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "eth_blockNumber".to_string(),
        params: Vec::new(),
        id: 1,
    };

    let start_time = std::time::Instant::now();

    // We'll handle the entire request in a match to capture different error types
    let result = client
        .post(url)
        .json(&request_payload)
        .send()
        .await;

    let latency = start_time.elapsed().as_millis();

    match result {
        Ok(response) => {
            // We got an HTTP response, now parse the JSON
            match response.json::<JsonRpcResponse>().await {
                Ok(json_response) => {
                    if let Some(error) = json_response.error {
                        // JSON-RPC level error
                        PingResult {
                            endpoint,
                            latency_ms: Some(latency),
                            status: PingStatus::JsonRpcError,
                            block_number: None,
                            error_message: Some(format!("JSON-RPC error: {} (code: {})", error.message, error.code)),
                        }
                    } else {
                        // Success case - we have a block number!
                        PingResult {
                            endpoint,
                            latency_ms: Some(latency),
                            status: PingStatus::Success,
                            block_number: json_response.result,
                            error_message: None,
                        }
                    }
                }
                Err(e) => {
                    // Failed to parse JSON
                    PingResult {
                        endpoint,
                        latency_ms: Some(latency),
                        status: PingStatus::HttpError,
                        block_number: None,
                        error_message: Some(format!("Failed to parse response: {}", e)),
                    }
                }
            }
        }
        Err(e) => {
            // HTTP request failed entirely
            PingResult {
                endpoint,
                latency_ms: None, // No latency measurement if request never completed
                status: PingStatus::HttpError,
                block_number: None,
                error_message: Some(format!("HTTP request failed: {}", e)),
            }
        }
    }
}