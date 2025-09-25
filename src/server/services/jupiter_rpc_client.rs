use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::Client;

use crate::server::domain::{OffChainRpcClient, TokenInfo, TokenPrice};

pub struct JupiterRpcClient {
    token_api_url: String,
    price_api_url: String,
    client: Client,
}

impl JupiterRpcClient {
    pub fn build(client: Client) -> Self {
        let base_url = "https://lite-api.jup.ag";
        Self {
            token_api_url: format! {"{}/tokens/v2/search", base_url},
            price_api_url: format! {"{}/price/v3", base_url},
            client,
        }
    }
}

#[async_trait]
impl OffChainRpcClient for JupiterRpcClient {
    //TODO: improve to handle the api token's limit (100) for one rq with using chunks
    async fn get_tokens(
        &self,
        tokens: Vec<String>,
    ) -> Result<Vec<TokenInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let tokens = self
            .client
            .get(self.token_api_url.as_str())
            .query(&[("query", tokens.join(","))])
            .send()
            .await?
            .json::<Vec<TokenInfo>>()
            .await?;
        Ok(tokens)
    }

    async fn get_prices(
        &self,
        tokens: Vec<String>,
    ) -> Result<HashMap<String, TokenPrice>, Box<dyn std::error::Error + Send + Sync>> {
        let prices = self
            .client
            .get(self.price_api_url.as_str())
            .query(&[("ids", tokens.join(","))])
            .send()
            .await?
            .json::<HashMap<String, TokenPrice>>()
            .await?;
        Ok(prices)
    }
}
