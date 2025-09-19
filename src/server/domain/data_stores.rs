use crate::server::domain::Token;

#[async_trait::async_trait]
pub trait TokenStore {
    async fn add_token(&mut self, token: Token) -> Result<(), TokenStoreError>;
    async fn get_token(&self, id: &String) -> Result<Token, TokenStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum TokenStoreError {
    TokenNotFound,
    TokenAlreadyExists,
    UnexpectedError,
}
