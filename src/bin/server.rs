use std::sync::Arc;

use sol_trace::server::{
    domain::WebSocketClient,
    run_server,
    services::{HashmapTokenStore, SolanaWebSocketClient},
    states::AppState,
};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051";

    let token_store = Arc::new(RwLock::new(HashmapTokenStore::default()));

    let ws_url = "wss://api.mainnet-beta.solana.com/".to_string();
    let ws_client_factory: Arc<dyn Fn() -> Box<dyn WebSocketClient + Send + Sync> + Send + Sync> =
        Arc::new(move || Box::new(SolanaWebSocketClient::new(ws_url.clone().as_str())));

    let state = AppState::new(token_store, ws_client_factory);

    run_server(addr, state).await
}
