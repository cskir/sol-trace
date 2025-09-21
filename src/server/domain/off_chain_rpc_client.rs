use async_trait::async_trait;

use crate::server::domain::Token;

#[async_trait]
pub trait OffChainRpcClient: Send + Sync {
    async fn get_tokens(
        &self,
        tokens: Vec<String>,
    ) -> Result<Vec<Token>, Box<dyn std::error::Error + Send + Sync>>;
}
