use crate::server::{
    domain::TokenStoreError,
    states::app_state::{OffChainRpcClientType, TokenStoreType},
};

#[tracing::instrument(name = "Store tokens", skip_all)]
pub async fn store_tokens(
    token_mints: &Vec<String>,
    off_chain_rpc_client: OffChainRpcClientType,
    token_store: TokenStoreType,
) -> Result<(), TokenStoreError> {
    let mut token_store = token_store.write().await;

    let mut tokens_to_query: Vec<String> = vec![];

    for token_mint in token_mints {
        if !token_store.has_token(token_mint).await {
            tokens_to_query.push(token_mint.clone());
        }
    }

    if tokens_to_query.len() > 0 {
        match off_chain_rpc_client.get_tokens(tokens_to_query).await {
            Err(e) => {
                let error = TokenStoreError::TokenIsNotAvailable(e.to_string());
                tracing::error!("Failed to fetch tokens: {}", error);
                return Err(error);
            }
            Ok(tokens) => {
                for token in tokens.into_iter() {
                    if token_store.add_token(token.clone()).await.is_err() {
                        tracing::error!("Failed to store token {}", token.id);
                        return Err(TokenStoreError::UnexpectedError);
                    }
                }
            }
        }
    }

    Ok(())
}
