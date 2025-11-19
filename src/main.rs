
use clap::Parser;
use chain_ping::{ping_endpoint_multiple, PingStatus, PingResult};
use futures::future::join_all;
use comfy_table::{Table, presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS, Color, Cell};

/// A high-performance CLI tool for benchmarking Ethereum RPC endpoints.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Ethereum RPC endpoint URLs (one or more)
    endpoints: Vec<String>,

    /// Number of pints to perform per endpoint
    # [arg(short, long, default_value = "4")]
    pings: usize,

    /// Timeout limit for each individual request in seconds
    #[arg(short, long, default_value = "10")]
    timeout: u64,

    /// Output format: table or json 
    #[arg(short, long, default_value = "table")]
    format: String,
}


#[tokio::main]
async fn main() {
    
    let cli = Cli::parse();
    
    if cli.endpoints.is_empty() {
        eprintln!("Error: At least one endpoint URL is required");
        std::process::exit(1);
    }

    let endpoint_str = if cli.endpoints.len() == 1 { "endpoint" } else { "endpoints" };
    let ping_str = if cli.pings == 1 { "request" } else { "requests" };
    
    eprintln!("Pinging {} {} ({} {} each)...", cli.endpoints.len(), endpoint_str, cli.pings, ping_str);

    let ping_futures: Vec<_> = cli.endpoints
        .iter()
        .map(|endpoint| ping_endpoint_multiple(endpoint, cli.pings,cli.timeout))
        .collect();
    
    let mut results = join_all(ping_futures).await;

    // Sort results by average latency, fastest first. Failures go to the bottom.
    results.sort_by_key(|r| r.avg_latency_ms.unwrap_or(u128::MAX));

    match cli.format.as_str() {
    "json" => output_json(&results),
    "table" => output_table(&results),
    _ => eprintln!("Error: Unknown format '{}'. Use 'table' or 'json'.", cli.format),
    }
}

fn output_table(results: &[PingResult]) {
    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);

    let multiple_pings = results.get(0).map_or(false, |r| r.ping_count > 1);

    if multiple_pings {
        // Mode A: Multiple Pings. We show "Avg Latency", "Min", and "Max".
        table.set_header(vec!["Endpoint", "Status", "Avg Latency", "Min", "Max", "Success", "Block Number", "Last Error"]);
    } else {
        // Mode B: Single Ping. We show "Latency" and REMOVE "Min", "Max", and "Success" (Success count)
        table.set_header(vec!["Endpoint", "Status", "Latency", "Block Number", "Last Error"]);
    }

    for result in results {
        let latency_value = result.avg_latency_ms.map(|ms| format!("{}ms", ms)).unwrap_or_else(|| "-".to_string());       
        let success_count = format!("{}/{}", result.success_count, result.ping_count);
        let block = result.block_number.as_deref().unwrap_or("-");
        let error = result.error_message.as_deref().unwrap_or("-");        
        let endpoint_display = if result.endpoint.len() > 50 {
            format!("{}...", &result.endpoint[..47])
        } else {
            result.endpoint.clone()
        };

        // Color Logic: We create a Cell and apply the color directly to it.
        let status_cell = match result.status {
            PingStatus::Success => Cell::new("SUCCESS").fg(Color::Green),
            PingStatus::PartialSuccess => Cell::new("PARTIAL").fg(Color::Yellow),
            PingStatus::Failure => Cell::new("FAILURE").fg(Color::Red),
        };

        let error_display = if error.len() > 40 {
            format!("{}...", &error[..37])
        } else {
            error.to_string()
        };

        // Add rows based on whether we are in multiple pings mode or not
        if multiple_pings {
            // Multiple pings mode: Show Avg, Min, Max
            let min_latency = result.min_latency_ms.map(|ms| format!("{}ms", ms)).unwrap_or_else(|| "-".to_string());
            let max_latency = result.max_latency_ms.map(|ms| format!("{}ms", ms)).unwrap_or_else(|| "-".to_string());
            
            table.add_row(vec![
                Cell::new(&endpoint_display),
                status_cell,
                Cell::new(&latency_value),
                Cell::new(&min_latency),
                Cell::new(&max_latency),
                Cell::new(&success_count),
                Cell::new(block),
                Cell::new(&error_display),
            ]);
        } else {
            // Single ping mode: No Min, Max, and Success. And display Latency (instead if Avg Latency)
            table.add_row(vec![
                Cell::new(&endpoint_display),
                status_cell,
                Cell::new(&latency_value),
                Cell::new(block),
                Cell::new(&error_display),
            ]);
        }
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