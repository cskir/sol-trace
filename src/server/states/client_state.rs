use crate::{
    proto::InitRequest,
    server::{
        domain::{SubscriptionInput, WebSocketClient},
        states::SubscriptionState,
        utils::gen_token_account,
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::RwLock;

pub type WebSocketClientType = Arc<RwLock<Box<dyn WebSocketClient + Send + Sync>>>;

#[derive(Clone)]
pub struct ClientState {
    pub ws_client: WebSocketClientType,
    pub subscription_input: Arc<SubscriptionInput>,
    pub token_account_map: Arc<HashMap<String, String>>,
    pub logs_subscription: Option<SubscriptionState>,
}

impl ClientState {
    pub fn build(
        request: InitRequest,
        factory: Arc<dyn Fn() -> Box<dyn WebSocketClient + Send + Sync> + Send + Sync>,
    ) -> Self {
        let ws_client = factory();

        let tokens: HashSet<String> = request.tokens.into_iter().collect();

        // !! mutabale only here
        let mut token_account_map = HashMap::new();

        for token_mint in &tokens {
            token_account_map.insert(
                token_mint.clone(),
                gen_token_account(&request.wallet, token_mint),
            );
        }

        Self {
            ws_client: Arc::new(RwLock::new(ws_client)),
            subscription_input: Arc::new(SubscriptionInput::new(request.wallet, tokens)),
            token_account_map: Arc::new(token_account_map),
            logs_subscription: None,
        }
    }
}
