use crate::server::domain::{OffChainRpcClient, TokenStore, WebSocketClient};
use crate::server::states::ClientState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type TokenStoreType = Arc<RwLock<dyn TokenStore + Send + Sync>>;
pub type OffChainRpcClientType = Arc<dyn OffChainRpcClient + Send + Sync>;

#[derive(Clone)]
pub struct AppState {
    pub token_store: TokenStoreType,
    pub off_chain_rpc_client: OffChainRpcClientType,
    pub ws_client_factory: Arc<dyn Fn() -> Box<dyn WebSocketClient + Send + Sync> + Send + Sync>,
    pub clients: Arc<RwLock<HashMap<Uuid, ClientState>>>,
}

impl AppState {
    pub fn new(
        token_store: TokenStoreType,
        off_chain_rpc_client: OffChainRpcClientType,
        ws_client_factory: Arc<dyn Fn() -> Box<dyn WebSocketClient + Send + Sync> + Send + Sync>,
    ) -> Self {
        Self {
            token_store,
            off_chain_rpc_client,
            clients: Arc::new(RwLock::new(HashMap::new())),
            ws_client_factory,
        }
    }
}
