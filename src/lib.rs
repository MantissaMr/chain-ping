use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<String>,
    pub id: u32,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<String>,
    pub error: Option<JsonRpcError>,
    pub id: u32,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum ChainPingError {
    #[error("HTTP request failed: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("JSON-RPC error: {message} (code: {code})")]
    JsonRpcError { code: i32, message: String },
}

pub type Result<T> = std::result::Result<T, ChainPingError>;

pub async fn ping_endpoint(url: &str) -> Result<u128> {
    let client = reqwest::Client::new();
    let request_payload = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "eth_blockNumber".to_string(),
        params: Vec::new(),
        id: 1,
    };

    let start_time = std::time::Instant::now();

    let response = client
        .post(url)
        .json(&request_payload)
        .send()
        .await?;

    let json_response: JsonRpcResponse = response.json().await?;

    // Check if the JSON-RPC response contains an error
    if let Some(error) = json_response.error {
        return Err(ChainPingError::JsonRpcError {
            code: error.code,
            message: error.message,
        });
    }

    let latency = start_time.elapsed().as_millis();
    Ok(latency)
}