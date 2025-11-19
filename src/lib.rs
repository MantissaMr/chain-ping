
// --- IMPORTS ---

use serde::Serialize;
use std::time::{Duration, Instant};
use thiserror::Error;


// --- DATA STRUCTURES ---
#[derive(Debug, Serialize)]
/// The final, aggregated result of pinging an endpoint multiple times
pub struct PingResult {
    pub endpoint: String,
    pub avg_latency_ms: Option<u128>,
    pub min_latency_ms: Option<u128>,
    pub max_latency_ms: Option<u128>,
    pub block_number: Option<String>,
    pub ping_count: usize,
    pub success_count: usize,
    pub status: PingStatus,
    pub error_message: Option<String>,
}

/// A simple summary of the outcome
#[derive(Debug, Serialize, PartialEq, Copy, Clone)]
pub enum PingStatus {
    Success,
    PartialSuccess,
    Failure,
} 

#[derive(Debug, Error)]
pub enum PingError { // Custom error type for core logic 
    #[error("Request failed: {0}")] 
    RequestError(#[from] reqwest::Error),
    #[error("JSON-RPC error: {0}")]
    JsonRpcError(String),
}

type PingAttemptResult = Result<(Duration, String), PingError>;


// --- CORE LOGIC ---
/// Pings an endpoint ONCE and returns its latency and block number, or an error
async fn ping_once(client: &reqwest::Client, url: &str) -> PingAttemptResult {
    let request_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1,
    });

    let start = Instant::now();

    let response = client
        .post(url)
        .json(&request_payload)
        .send()
        .await
        .map_err(PingError::RequestError)?; // Convert reqwest error into our custom error

    let latency = start.elapsed();

    if !response.status().is_success() {
        return Err(PingError::RequestError(response.error_for_status().unwrap_err()));
    }

    let json_response: serde_json::Value = response.json().await.map_err(PingError::RequestError)?;
    
    if let Some(error) = json_response.get("error") {
        return Err(PingError::JsonRpcError(error.to_string()));
    }
    
    if let Some(result) = json_response.get("result") {
        // We have a success! Return the latency and the block number string.
        Ok((latency, result.to_string()))
    } else {
        Err(PingError::JsonRpcError("Missing 'result' field in response".to_string()))
    }
}

/// Pings an endpoint multiple times and aggregates the results
pub async fn ping_endpoint_multiple(url: &str, count: usize, timeout_secs: u64) -> PingResult {    
    let client =  match reqwest::Client::builder()
    .timeout(Duration::from_secs(timeout_secs))
    .build() {
        Ok(c) => c,
        Err(e) => {
            // If we can't even build the client, the entire process has failed.
            // We return a failure PingResult immediately.
            return PingResult {
                endpoint: url.to_string(),
                status: PingStatus::Failure,
                error_message: Some(format!("Failed to build HTTP client: {}", e)),
                // ... all other fields are None or 0 ...
                avg_latency_ms: None,
                min_latency_ms: None,
                max_latency_ms: None,
                block_number: None,
                ping_count: count,
                success_count: 0,
            };
        }
    }; 

    let mut latencies = Vec::new();
    let mut successes = 0;
    let mut last_block_number = None;
    let mut last_error_message = None;

    for _ in 0..count {
        match ping_once(&client, url).await {
            Ok((latency, block_number)) => {
                successes += 1;
                latencies.push(latency.as_millis());
                last_block_number = Some(block_number);
            }
            Err(e) => {
                if let PingError::RequestError(ref req_err) = e {
                    if req_err.is_timeout() {
                        last_error_message = Some("Request timed out".to_string());
                    } else if req_err.is_connect() {
                        last_error_message = Some("Connection failed".to_string());
                    } else if let Some(status) = req_err.status() {
                        last_error_message = Some(format!("HTTP Error: {}", status));
                    } else {
                        last_error_message = Some(e.to_string());
                    }
                } else {
                    last_error_message = Some(e.to_string());
                }
            }
        }
    }

    let status = if successes == count {
        PingStatus::Success
    } else if successes > 0 {
        PingStatus::PartialSuccess
    } else {
        PingStatus::Failure
    };
    
    let (avg, min, max) = calculate_stats(&latencies);

    PingResult {
        endpoint: url.to_string(),        
        avg_latency_ms: avg,
        min_latency_ms: min,
        max_latency_ms: max,
        block_number: last_block_number,
        ping_count: count,
        success_count: successes,
        status,
        error_message: last_error_message,
    }
}

fn calculate_stats(latencies: &[u128]) -> (Option<u128>, Option<u128>, Option<u128>) {
    if latencies.is_empty() {
        return (None, None, None);
    }
    let sum: u128 = latencies.iter().sum();
    let avg = sum / latencies.len() as u128;
    let min = latencies.iter().min().copied();
    let max = latencies.iter().max().copied();
    (Some(avg), min, max)
}

// --- TESTS ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_stats() {
        let data = vec![100, 200, 300];
        let (avg, min, max) = calculate_stats(&data);
        assert_eq!(avg, Some(200));
        assert_eq!(min, Some(100));
        assert_eq!(max, Some(300));
    }

    #[test]
    fn test_calculate_stats_empty() {
        let data = vec![];
        let (avg, min, max) = calculate_stats(&data);
        assert_eq!(avg, None);
        assert_eq!(min, None);
        assert_eq!(max, None);
    }
}