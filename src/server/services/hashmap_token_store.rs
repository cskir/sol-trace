use std::collections::HashMap;

use crate::server::domain::{TokenInfo, TokenStore, TokenStoreError};

#[derive(Default, Clone)]
pub struct HashmapTokenStore {
    tokens: HashMap<String, TokenInfo>,
}

#[async_trait::async_trait]
impl TokenStore for HashmapTokenStore {
    async fn add_token(&mut self, token: TokenInfo) -> Result<(), TokenStoreError> {
        if self.tokens.contains_key(&token.id) {
            return Err(TokenStoreError::TokenAlreadyExists);
        }
        self.tokens.insert(token.id.clone(), token);
        Ok(())
    }

    async fn get_token(&self, address: &String) -> Result<TokenInfo, TokenStoreError> {
        match self.tokens.get(address) {
            Some(token) => Ok(token.clone()),
            None => Err(TokenStoreError::TokenNotFound),
        }
    }

    async fn has_token(&self, address: &String) -> bool {
        self.tokens.contains_key(address)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_add_token() {
        let mut store = HashmapTokenStore::default();
        let token = TokenInfo {
            id: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_owned(),
            name: "Bonk".to_owned(),
            symbol: "Bonk".to_owned(),
            icon: Some(
                "https://arweave.net/hQiPZOsRZXGXBJd_82PhVdlM_hACsT_q6wqwf5cSY7I".to_owned(),
            ),
            decimals: 5,
        };
        let token_clone = token.clone();

        let result = store.add_token(token).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());
        assert_eq!(store.tokens.len(), 1);

        let result = store.add_token(token_clone).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TokenStoreError::TokenAlreadyExists);
    }

    #[tokio::test]
    async fn test_get_token() {
        let mut store = HashmapTokenStore::default();
        let token = TokenInfo {
            id: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_owned(),
            name: "Bonk".to_owned(),
            symbol: "Bonk".to_owned(),
            icon: None,
            decimals: 5,
        };
        let token_clone = token.clone();
        store.add_token(token).await.unwrap();

        let retr_token_ok = store.get_token(&token_clone.id).await;
        assert!(retr_token_ok.is_ok());
        assert_eq!(retr_token_ok.unwrap(), token_clone);

        let other_token_mint = "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm".to_owned();
        let retr_token_not_found = store.get_token(&other_token_mint).await;
        assert!(retr_token_not_found.is_err());
        assert_eq!(
            retr_token_not_found.unwrap_err(),
            TokenStoreError::TokenNotFound
        );
    }
    #[tokio::test]
    async fn test_has_token() {
        let mut store = HashmapTokenStore::default();
        let token_mint = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_owned();

        let token = TokenInfo {
            id: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_owned(),
            name: "Bonk".to_owned(),
            symbol: "Bonk".to_owned(),
            icon: None,
            decimals: 5,
        };

        assert!(!store.has_token(&token_mint).await);

        store.add_token(token).await.unwrap();

        assert!(store.has_token(&token_mint).await);
    }
}
