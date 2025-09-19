use std::error::Error;

use spl_associated_token_account::get_associated_token_address;
use spl_token::solana_program::pubkey::Pubkey;

pub fn validate_address(address: &str) -> Result<(), Box<dyn Error>> {
    let _pubkey: Pubkey = address.parse()?;
    Ok(())
}

pub fn get_token_account(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address(&wallet, &mint)
}
