
use clap::Parser;
use chain_ping::ping_endpoint;


/// This struct defines our CLI arguments using clap's derive feature
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Ethereum RPC endpoint URL (this comment becomes the help text)
    endpoint: String,
}


#[tokio::main]
async fn main() {
    
    let cli = Cli::parse();
    
    println!("Pinging: {}", cli.endpoint);

     match ping_endpoint(&cli.endpoint).await {
        Ok(latency) => println!("Success! Latency: {} ms", latency),
        Err(e) => println!("Error: {}", e),
    }
}