use async_trait::async_trait;

use crate::server::domain::TransactionResponse;

#[async_trait]
pub trait OnChainRpcClient: Send + Sync {
    async fn get_transaction(
        &self,
        signature: String,
    ) -> Result<TransactionResponse, Box<dyn std::error::Error + Send + Sync>>;
}
