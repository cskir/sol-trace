use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

pub const WSOL: &str = "So11111111111111111111111111111111111111112";
pub const SOL_DENOM: f64 = 1_000_000_000.0; // lamports in 1 SOL

lazy_static! {
    pub static ref SOLANA_WS_URL: String = set_solana_ws_url();
    pub static ref SOLANA_RPC_URL: String = set_solana_rpc_url();
}

fn set_solana_ws_url() -> String {
    dotenv().ok(); // Load environment variables
    let ws_url = std_env::var(env::SOLANA_WS_URL_ENV_VAR).expect("SOLANA_WS_URL must be set.");
    if ws_url.is_empty() {
        panic!("SOLANA_WS_URL must not be empty.");
    }
    ws_url
}

fn set_solana_rpc_url() -> String {
    dotenv().ok(); // Load environment variables
    let rpc_url = std_env::var(env::SOLANA_RPC_URL_ENV_VAR).expect("SOLANA_RPC_URL must be set.");
    if rpc_url.is_empty() {
        panic!("SOLANA_RPC_URL must not be empty.");
    }
    rpc_url
}

pub mod env {
    pub const SOLANA_WS_URL_ENV_VAR: &str = "SOLANA_WS_URL";
    pub const SOLANA_RPC_URL_ENV_VAR: &str = "SOLANA_RPC_URL";
}

pub mod test {
    pub mod solana_data {
        pub const WALLET: &str = "9AhKqLR67hwapvG8SA2JFXaCshXc9nALJjpKaHZrsbkw";
        pub const TOKEN1: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";

        pub const INVALID_WALLET: &str = "9AhKqLR67hwapvG8SA2JFXaCshXc9nALJjpKaHZrsbk_";
        pub const INVALID_TOKEN1: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB26_";
    }
}
