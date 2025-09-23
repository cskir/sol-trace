use async_trait::async_trait;
use reqwest::Client;

use crate::server::domain::{OffChainRpcClient, Token};

pub struct JupiterRpcClient {
    token_api_url: String,
    client: Client,
}

impl JupiterRpcClient {
    pub fn build(client: Client) -> Self {
        let base_url = "https://lite-api.jup.ag";
        Self {
            token_api_url: format! {"{}/tokens/v2", base_url},
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
    ) -> Result<Vec<Token>, Box<dyn std::error::Error + Send + Sync>> {
        let tokens = self
            .client
            .get(self.token_api_url.as_str())
            .query(&[("query", tokens.join(","))])
            .send()
            .await?
            .json::<Vec<Token>>()
            .await?;
        Ok(tokens)
    }
}
