
use clap::Parser;
use chain_ping::{ping_endpoint, PingStatus};
use futures::future::join_all;

/// This struct defines our CLI arguments using clap's derive feature
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Ethereum RPC endpoint URLs (one or more)
    endpoints: Vec<String>,
}


#[tokio::main]
async fn main() {
    
    let cli = Cli::parse();
    
    if cli.endpoints.is_empty() {
        eprintln!("Error: At least one endpoint is required");
        return;
    }
    
    println!("Pinging {} endpoints concurrently...", cli.endpoints.len());

    let ping_futures: Vec<_> = cli.endpoints
        .iter()
        .map(|endpoint| ping_endpoint(endpoint))
        .collect();

    let results = join_all(ping_futures).await;
    
    for result in results  {
        match result.status {
            PingStatus::Success => {
                println!("SUCCESS");
                println!("  Endpoint: {}", result.endpoint);
                println!("  Latency: {} ms", result.latency_ms.unwrap());
                if let Some(block) = result.block_number {
                    println!("  Block: {}", block);
                }
            }
            PingStatus::JsonRpcError => {
                println!("JSON-RPC ERROR");
                println!("  Endpoint: {}", result.endpoint);
                if let Some(latency) = result.latency_ms {
                    println!("  Latency: {} ms", latency);
                }
                if let Some(error) = result.error_message {
                    println!("  Error: {}", error);
                }
            }
            PingStatus::HttpError => {
                println!("HTTP ERROR");
                println!("  Endpoint: {}", result.endpoint);
                if let Some(latency) = result.latency_ms {
                    println!("  Latency: {} ms", latency);
                }
                if let Some(error) = result.error_message {
                    println!("  Error: {}", error);
                }
            }
        PingStatus::Timeout => {
            println!("TIMEOUT");
            println!("  Endpoint: {}", result.endpoint);
            if let Some(error) = result.error_message {
                println!("  Error: {}", error);
                }
            }
        }    
        println!();
    }

}