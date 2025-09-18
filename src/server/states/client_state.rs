use crate::{
    proto::InitRequest,
    server::{
        domain::{SubscriptionInput, WebSocketClient},
        states::SubscriptionState,
    },
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type WebSocketClientType = Arc<RwLock<Box<dyn WebSocketClient + Send + Sync>>>;

#[derive(Clone)]
pub struct ClientState {
    pub ws_client: WebSocketClientType,
    pub subscription_input: Arc<SubscriptionInput>,
    pub logs_subscription: Option<SubscriptionState>,
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
            logs_subscription: None,
        }
    }
}
