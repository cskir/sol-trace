use tonic::Status;

#[derive(Debug, PartialEq)]
pub enum InputValidationError {
    InvalidWalletAddress,
    MissingTokens,
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
