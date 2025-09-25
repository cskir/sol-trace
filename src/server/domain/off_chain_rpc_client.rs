use std::collections::HashMap;

use async_trait::async_trait;

use crate::server::domain::{TokenInfo, TokenPrice};

#[async_trait]
pub trait OffChainRpcClient: Send + Sync {
    async fn get_tokens(
        &self,
        tokens: Vec<String>,
    ) -> Result<Vec<TokenInfo>, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_prices(
        &self,
        tokens: Vec<String>,
    ) -> Result<HashMap<String, TokenPrice>, Box<dyn std::error::Error + Send + Sync>>;
}
