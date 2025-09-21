use tonic::{Code, Status};

use crate::server::domain::Token;

#[async_trait::async_trait]
pub trait TokenStore {
    async fn add_token(&mut self, token: Token) -> Result<(), TokenStoreError>;
    async fn get_token(&self, id: &String) -> Result<Token, TokenStoreError>;
    async fn has_token(&self, id: &String) -> bool;
}

#[derive(Debug, PartialEq)]
pub enum TokenStoreError {
    TokenNotFound,
    TokenAlreadyExists,
    UnexpectedError,
}

impl From<TokenStoreError> for Status {
    fn from(err: TokenStoreError) -> Self {
        match err {
            TokenStoreError::TokenNotFound => Status::new(Code::Internal, "Token not found"),
            TokenStoreError::TokenAlreadyExists => {
                Status::new(Code::InvalidArgument, "Token already exists")
            }
            TokenStoreError::UnexpectedError => Status::new(Code::Unknown, "Unexpected error"),
        }
    }
}
