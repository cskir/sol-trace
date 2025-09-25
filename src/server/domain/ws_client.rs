use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::mpsc;
use tonic::Status;

use crate::{
    proto::SubscribeResponse,
    server::{
        domain::SubscriptionInput,
        states::app_state::{OffChainRpcClientType, OnChainRpcClientType},
    },
};

pub type WSCResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[async_trait]
pub trait WebSocketClient {
    async fn logs_subscribe(
        &mut self,
        subscription_input: Arc<SubscriptionInput>,
        off_chain_rpc_client: OffChainRpcClientType,
        on_chain_rpc_client: OnChainRpcClientType,
        tx: mpsc::Sender<Result<SubscribeResponse, Status>>,
    ) -> WSCResult<u64>;

    async fn logs_unsubscribe(&mut self, sub_id: u64) -> WSCResult<()>;
}
