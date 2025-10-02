use async_trait::async_trait;
use reqwest::Client;

use crate::server::domain::{
    BalanceResponse, GetBalanceResponse, GetTokenAccountBalanceResponse, GetTransactionResponse,
    OnChainRpcClient, TokenAccountBalanceResponse, TransactionResponse,
};

pub struct SolanaRpcClient {
    solana_url: String,
    client: Client,
}

impl SolanaRpcClient {
    pub fn build(client: Client) -> Self {
        let base_url = "https://api.mainnet-beta.solana.com/";
        Self {
            solana_url: base_url.to_owned(),
            client,
        }
    }
}

#[async_trait]
impl OnChainRpcClient for SolanaRpcClient {
    #[tracing::instrument(name = "Get transaction", skip_all)]
    async fn get_transaction(
        &self,
        signature: String,
    ) -> Result<TransactionResponse, Box<dyn std::error::Error + Send + Sync>> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": [ signature,{
                "commitment": "confirmed", // 31 blocks finality -> confirmed ?
                "maxSupportedTransactionVersion": 0,
                "encoding": "json"}]
        });

        let response = self
            .client
            .post(&self.solana_url)
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            match response.json::<GetTransactionResponse>().await? {
                GetTransactionResponse::Transaction(resp) => Ok(resp),
                GetTransactionResponse::Error(resp) => {
                    Err(format!("Transaction not found. Error: {}", resp.error.message).into())
                }
            }
        } else {
            Err(format!("Request failed with status: {}", response.status()).into())
        }
    }

    #[tracing::instrument(name = "Get token balance", skip_all)]
    async fn get_token_account_balance(
        &self,
        pub_key: String,
    ) -> Result<TokenAccountBalanceResponse, Box<dyn std::error::Error + Send + Sync>> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTokenAccountBalance",
            "params": [ pub_key,
                {
                    "commitment": "finalized", // 31 blocks finality -> confirmed
                }
            ]
        });

        let response = self
            .client
            .post(&self.solana_url)
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            match response.json::<GetTokenAccountBalanceResponse>().await? {
                GetTokenAccountBalanceResponse::Balance(resp) => Ok(resp),
                GetTokenAccountBalanceResponse::Error(resp) => {
                    Err(format!("Balance not found. Error: {}", resp.error.message).into())
                }
            }
        } else {
            Err(format!("Request failed with status: {}", response.status()).into())
        }
    }

    #[tracing::instrument(name = "Get SOL balance", skip_all)]
    async fn get_balance(
        &self,
        pub_key: String,
    ) -> Result<BalanceResponse, Box<dyn std::error::Error + Send + Sync>> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [ pub_key,
                {
                    "commitment": "finalized", // 31 blocks finality -> confirmed
                }
            ]
        });

        let response = self
            .client
            .post(&self.solana_url)
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            match response.json::<GetBalanceResponse>().await? {
                GetBalanceResponse::Balance(resp) => Ok(resp),
                GetBalanceResponse::Error(resp) => {
                    Err(format!("Sol balance not found. Error: {}", resp.error.message).into())
                }
            }
        } else {
            Err(format!("Request failed with status: {}", response.status()).into())
        }
    }
}
