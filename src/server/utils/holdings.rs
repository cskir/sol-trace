use std::{collections::HashMap, sync::Arc};

use crate::{
    proto::{Holding, HoldingsResponse},
    server::{
        states::app_state::{OffChainRpcClientType, OnChainRpcClientType, TokenStoreType},
        utils::constants::{SOL_DENOM, WSOL},
    },
};

pub async fn query_holdings(
    wallet: &String,
    token_account_map: Arc<HashMap<String, String>>,
    tokens_store: TokenStoreType,
    on_chain_rpc_client: OnChainRpcClientType,
    off_chain_rpc_client: OffChainRpcClientType,
) -> Result<HoldingsResponse, Box<dyn std::error::Error + Send + Sync>> {
    let mut holdings: Vec<Holding> = Vec::new();

    let mut token_balance_map: HashMap<String, f64> = HashMap::new();
    for (token_mint, token_account) in token_account_map.iter() {
        let token_balance = on_chain_rpc_client
            .get_token_account_balance(token_account.clone())
            .await?;

        if let Some(amount) = token_balance.result.as_ref().map(|res| res.value.to_f64()) {
            if amount > 0.0 {
                token_balance_map.insert(token_mint.clone(), amount);
            }
        }
    }

    let sol_balance = on_chain_rpc_client.get_balance(wallet.to_string()).await?;

    if let Some(amount) = sol_balance
        .result
        .as_ref()
        .map(|res| res.value as f64 / SOL_DENOM)
    {
        if amount > 0.0 {
            token_balance_map
                .entry(WSOL.to_string())
                .and_modify(|x| *x += amount)
                .or_insert(amount);
        }
    }

    let token_prices_map = off_chain_rpc_client
        .get_prices(token_balance_map.keys().cloned().collect())
        .await?;

    for token_address in token_balance_map.keys() {
        if let Some(token_info) = tokens_store
            .read()
            .await
            .get_token(token_address)
            .await
            .ok()
        {
            let balance = token_balance_map.get(token_address).cloned().unwrap_or(0.0);

            let (usd_price, usd_value) = match token_prices_map.get(token_address) {
                Some(token_price) => (
                    Some(token_price.usd_price),
                    Some(balance * token_price.usd_price),
                ),
                None => (None, None),
            };

            holdings.push(Holding {
                name: token_info.name.clone(),
                symbol: token_info.symbol.clone(),
                address: token_address.clone(),
                balance: balance.to_string(),
                usd_price,
                usd_value,
            });
        }
    }

    Ok(HoldingsResponse { holdings })
}
