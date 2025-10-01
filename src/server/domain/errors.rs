use thiserror::Error;
use tonic::Status;

#[derive(Debug, PartialEq, Clone, Error)]
pub enum InputValidationError {
    #[error("Invalid wallet address")]
    InvalidWalletAddress,
    #[error("Missing tokens")]
    MissingTokens,
    #[error("{0}")]
    InvalidTokenAddress(String),
}

impl From<InputValidationError> for Status {
    fn from(err: InputValidationError) -> Self {
        match err {
            InputValidationError::InvalidWalletAddress => {
                Status::invalid_argument("Invalid wallet address")
            }
            InputValidationError::MissingTokens => Status::invalid_argument("Missing tokens"),
            InputValidationError::InvalidTokenAddress(msg) => Status::invalid_argument(msg),
        }
    }
}
