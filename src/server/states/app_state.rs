use crate::server::domain::{TokenStore, WebSocketClient};
use crate::server::states::ClientState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type TokenStoreType = Arc<RwLock<dyn TokenStore + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub token_store: TokenStoreType,
    pub ws_client_factory: Arc<dyn Fn() -> Box<dyn WebSocketClient + Send + Sync> + Send + Sync>,
    pub clients: Arc<RwLock<HashMap<Uuid, ClientState>>>,
}

impl AppState {
    pub fn new(
        token_store: TokenStoreType,
        ws_client_factory: Arc<dyn Fn() -> Box<dyn WebSocketClient + Send + Sync> + Send + Sync>,
    ) -> Self {
        Self {
            token_store,
            clients: Arc::new(RwLock::new(HashMap::new())),
            ws_client_factory,
        }
    }
}
