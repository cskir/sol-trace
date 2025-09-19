use async_trait::async_trait;

use crate::server::domain::Token;

#[async_trait]
pub trait RpcClient: Send + Sync {
    async fn get_tokens(
        &self,
        tokens: Vec<Token>,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>>;
}
