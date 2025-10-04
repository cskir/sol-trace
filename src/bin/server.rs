use std::sync::Arc;

use sol_trace::server::{
    domain::WebSocketClient,
    run_server,
    services::{HashmapTokenStore, JupiterRpcClient, SolanaRpcClient, SolanaWebSocketClient},
    states::AppState,
    utils::{constants::SOLANA_WS_URL, init_tracing},
};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing");

    let addr = "127.0.0.1:50051";

    let token_store = Arc::new(RwLock::new(HashmapTokenStore::default()));

    let client = reqwest::Client::new();
    let off_chain_rpc_client = Arc::new(JupiterRpcClient::build(client));

    let client2 = reqwest::Client::new();
    let on_chain_rpc_client = Arc::new(SolanaRpcClient::build(client2));

    //let ws_url = "wss://api.mainnet-beta.solana.com/".to_string();
    let ws_client_factory: Arc<dyn Fn() -> Box<dyn WebSocketClient + Send + Sync> + Send + Sync> =
        Arc::new(move || Box::new(SolanaWebSocketClient::new(&SOLANA_WS_URL)));

    let state = AppState::new(
        token_store,
        off_chain_rpc_client,
        on_chain_rpc_client,
        ws_client_factory,
    );

    run_server(addr, state).await
}
