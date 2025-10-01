use async_trait::async_trait;
use reqwest::Client;

use crate::server::domain::{GetTransactionResponse, OnChainRpcClient, TransactionResponse};

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
}
