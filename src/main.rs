
// use assert_cmd::output;
use clap::Parser;
use chain_ping::{ping_endpoint_multiple, PingStatus};
use futures::future::join_all;
use comfy_table::Table;
use comfy_table::presets::UTF8_FULL;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;

/// This struct defines our CLI arguments using clap's derive feature
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Ethereum RPC endpoint URLs (one or more)
    endpoints: Vec<String>,

    /// Number of pints to perform per endpoint (default: 1)
    # [arg(short, long, default_value = "1")]
    pings: usize,

    /// Output format: table or json (default: table)
    #[arg(short, long, default_value = "table")]
    format: String,
}


#[tokio::main]
async fn main() {
    
    let cli = Cli::parse();
    
    if cli.endpoints.is_empty() {
        eprintln!("Error: At least one endpoint is required");
        return;
    }
    
    println!("Pinging {} endpoints {} times each...", cli.endpoints.len(), cli.pings);

    let ping_futures: Vec<_> = cli.endpoints
        .iter()
        .map(|endpoint| ping_endpoint_multiple(endpoint, cli.pings))
        .collect();

    let results = join_all(ping_futures).await;

    match cli.format.as_str() {
    "json" => output_json(&results),
    "table" => output_table(&results),
    _ => eprintln!("Error: Unknown format '{}'. Use 'table' or 'json'.", cli.format),
    }
}

fn output_table(results: &[chain_ping::PingResult]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Endpoint", "Status", "Avg (ms)", "Min/Max (ms)", "Success", "Block"]);
    for result in results {
        let endpoint = &result.endpoint;
        
        // Convert PingStatus enum to human-readable string
        let status = match result.status {
            PingStatus::Success => "SUCCESS",
            PingStatus::PartialSuccess => "PARTIAL",
            PingStatus::HttpError => "HTTP_ERROR",
            PingStatus::JsonRpcError => "JSON_RPC_ERROR",
            PingStatus::Timeout => "TIMEOUT",
        };
        let latency = if let Some(avg) = result.latency_ms {
            format!("{}", avg) 
        } else {
            "-".to_string()
        };
        
        // Calculate success count from success rate
        let success_rate = format!("{}/{}", (result.success_rate * result.ping_count as f32) as usize, result.ping_count);
        
        // Handle optional block number
        let block = result.block_number.as_deref().unwrap_or("-");

        // Add a row to the table
        table.add_row(vec![
            endpoint,
            status,
            &latency,
            &format!("{}/{}", result.min_latency_ms.unwrap_or(0), result.max_latency_ms.unwrap_or(0)),
            &success_rate,
            block,
        ]);
    }

    println!("{table}");
}

fn output_json(results: &[chain_ping::PingResult]) {
    if let Ok(json_string) = serde_json::to_string_pretty(results) {
        println!("{}", json_string);
    } else {
        eprintln!("Error: Failed to serialize results to JSON");
    }
}