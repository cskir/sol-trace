use std::error::Error;

use spl_associated_token_account::get_associated_token_address;
use spl_token::solana_program::pubkey::Pubkey;
use tonic::Status;

use crate::proto::InitRequest;

pub fn validate_input(init_request: &InitRequest) -> Result<(), Status> {
    validate_address(&init_request.wallet)
        .map_err(|_| Status::invalid_argument("Invalid wallet address"))?;

    if init_request.tokens.len() == 0 {
        return Err(Status::invalid_argument("Missing tokens"))?;
    }

    let mut invalid_tokens: Vec<String> = vec![];

    for token_mint in &init_request.tokens {
        match validate_address(token_mint) {
            Ok(_) => {}
            Err(_) => invalid_tokens.push(token_mint.clone()),
        }
    }

    if invalid_tokens.len() > 0 {
        return Err(Status::invalid_argument(format!(
            "Invalid token{} {}",
            if invalid_tokens.len() > 1 { "s:" } else { ":" },
            invalid_tokens.join(",")
        )));
    }

    Ok(())
}

fn validate_address(address: &str) -> Result<(), Box<dyn Error>> {
    let _pubkey: Pubkey = address.parse()?;
    Ok(())
}

pub fn get_token_account(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address(&wallet, &mint)
}

pub fn gen_token_account(wallet: &String, mint: &String) -> String {
    // unwrap is safe at this point
    let wallet: Pubkey = wallet.parse().unwrap();
    let mint: Pubkey = mint.parse().unwrap();
    get_associated_token_address(&wallet, &mint).to_string()
}

#[cfg(test)]
mod tests {

    use crate::server::utils::constants::test::solana_data::{
        INVALID_TOKEN1, INVALID_WALLET, TOKEN1, WALLET,
    };

    use super::*;

    #[test]
    fn valid_init() {
        let init_request = InitRequest {
            wallet: WALLET.to_owned(),
            tokens: vec![TOKEN1.to_owned()],
        };

        let result = validate_input(&init_request);
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_wallet() {
        let init_request = InitRequest {
            wallet: INVALID_WALLET.to_owned(),
            tokens: vec![],
        };

        let result = validate_input(&init_request);
        assert!(result.is_err());

        assert_eq!("Invalid wallet address", result.unwrap_err().message())
    }

    #[test]
    fn missing_token() {
        let init_request = InitRequest {
            wallet: WALLET.to_owned(),
            tokens: vec![],
        };

        let result = validate_input(&init_request);
        assert!(result.is_err());

        assert_eq!("Missing tokens", result.unwrap_err().message())
    }

    #[test]
    fn invalid_token() {
        let init_request = InitRequest {
            wallet: WALLET.to_owned(),
            tokens: vec![INVALID_TOKEN1.to_string()],
        };

        let result = validate_input(&init_request);
        assert!(result.is_err());

        assert_eq!(
            format!("Invalid token: {}", INVALID_TOKEN1),
            result.unwrap_err().message()
        )
    }
}
