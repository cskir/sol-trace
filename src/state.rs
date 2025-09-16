use crate::{proto::InitRequest, server::domain::WebSocketClient};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type WebSocketClientType = Arc<RwLock<Box<dyn WebSocketClient + Send + Sync>>>;

#[derive(Clone)]
pub struct ClientState {
    pub ws_client: WebSocketClientType,
    pub subscription_input: Arc<SubscriptionInput>,
    pub subscription: Option<SubscriptionState>,
}

impl ClientState {
    pub fn build(
        request: InitRequest,
        factory: Arc<dyn Fn() -> Box<dyn WebSocketClient + Send + Sync> + Send + Sync>,
    ) -> Self {
        let ws_client = factory();
        Self {
            ws_client: Arc::new(RwLock::new(ws_client)),
            subscription_input: Arc::new(SubscriptionInput::new(request)),
            subscription: None,
        }
    }
}

#[derive(Clone)]
pub struct SubscriptionState {
    pub sub_id: u64,
}

#[derive(Clone)]
pub struct SubscriptionInput {
    pub wallet: String,
    pub tokens: Vec<String>,
}

impl SubscriptionInput {
    pub fn new(init_request: InitRequest) -> Self {
        Self {
            wallet: init_request.wallet,
            tokens: init_request.tokens,
        }
    }
}

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
