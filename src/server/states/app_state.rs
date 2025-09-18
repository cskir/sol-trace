use crate::server::domain::WebSocketClient;
use crate::server::states::ClientState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub ws_client_factory: Arc<dyn Fn() -> Box<dyn WebSocketClient + Send + Sync> + Send + Sync>,
    pub clients: Arc<RwLock<HashMap<Uuid, ClientState>>>,
}

impl AppState {
    pub fn new(
        ws_client_factory: Arc<dyn Fn() -> Box<dyn WebSocketClient + Send + Sync> + Send + Sync>,
    ) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            ws_client_factory,
        }
    }
}
