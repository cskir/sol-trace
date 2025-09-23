use std::collections::HashSet;

#[derive(Clone)]
pub struct SubscriptionInput {
    pub wallet: String,
    pub tokens: HashSet<String>,
}

impl SubscriptionInput {
    pub fn new(wallet: String, tokens: HashSet<String>) -> Self {
        Self { wallet, tokens }
    }
}
