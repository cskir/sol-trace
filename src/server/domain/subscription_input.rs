use crate::proto::InitRequest;

#[derive(Clone)]
pub struct SubscriptionInput {
    pub wallet: String,
    pub tokens: Vec<String>,
}

impl SubscriptionInput {
    pub fn new(init_request: InitRequest) -> Self {
        Self {
            wallet: init_request.wallet,
            tokens: init_request.tokens,
        }
    }
}
