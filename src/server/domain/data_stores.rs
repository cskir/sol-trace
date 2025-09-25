use tonic::{Code, Status};

use crate::server::domain::TokenInfo;

#[async_trait::async_trait]
pub trait TokenStore {
    async fn add_token(&mut self, token: TokenInfo) -> Result<(), TokenStoreError>;
    async fn get_token(&self, id: &String) -> Result<TokenInfo, TokenStoreError>;
    async fn has_token(&self, id: &String) -> bool;
}

#[derive(Debug, PartialEq)]
pub enum TokenStoreError {
    TokenNotFound,
    TokenAlreadyExists,
    TokenIsNotAvailable(String),
    UnexpectedError,
}

impl From<TokenStoreError> for Status {
    fn from(err: TokenStoreError) -> Self {
        match err {
            TokenStoreError::TokenNotFound => Status::new(Code::Internal, "Token not found"),
            TokenStoreError::TokenAlreadyExists => {
                Status::new(Code::InvalidArgument, "Token already exists")
            }
            TokenStoreError::TokenIsNotAvailable(msg) => {
                Status::new(Code::Unavailable, format!("Token is not available {}", msg))
            }
            TokenStoreError::UnexpectedError => Status::new(Code::Unknown, "Unexpected error"),
        }
    }
}
