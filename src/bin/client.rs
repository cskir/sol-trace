use clap::Parser;
use sol_trace::client::{CliArgs, run_cli_client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CliArgs::parse();
    run_cli_client(cli).await
}
