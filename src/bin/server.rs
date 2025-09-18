use std::sync::Arc;

use sol_trace::{
    server::states::AppState,
    server::{domain::WebSocketClient, run_server, services::SolanaWebSocketClient},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051";
    let ws_url = "wss://api.mainnet-beta.solana.com/".to_string();

    let ws_client_factory: Arc<dyn Fn() -> Box<dyn WebSocketClient + Send + Sync> + Send + Sync> =
        Arc::new(move || Box::new(SolanaWebSocketClient::new(ws_url.clone().as_str())));

    let state = AppState::new(ws_client_factory);

    run_server(addr, state).await
}
