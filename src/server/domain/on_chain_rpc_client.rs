use async_trait::async_trait;

use crate::server::domain::{BalanceResponse, TokenAccountBalanceResponse, TransactionResponse};

#[async_trait]
pub trait OnChainRpcClient: Send + Sync {
    async fn get_transaction(
        &self,
        signature: String,
    ) -> Result<TransactionResponse, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_token_account_balance(
        &self,
        pub_key: String,
    ) -> Result<TokenAccountBalanceResponse, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_balance(
        &self,
        pub_key: String,
    ) -> Result<BalanceResponse, Box<dyn std::error::Error + Send + Sync>>;
}
