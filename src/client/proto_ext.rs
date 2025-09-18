use crate::client::Config;
use crate::proto::InitRequest;

impl InitRequest {
    pub fn build(config: Config) -> Result<Self, String> {
        if config.wallet.is_empty() {
            return Err("Wallet address is required in config".to_string());
        }

        if config.token_mints.is_empty() {
            return Err("Error: No token mints specified in config".to_string());
        }

        Ok(Self {
            wallet: config.wallet,
            tokens: config.token_mints,
        })
    }
}
