use std::{collections::HashMap, sync::Arc};

use crate::{
    proto::{Trade, Transfer},
    server::{
        domain::{EncodedTransaction, SubscriptionInput, TransactionMeta},
        states::app_state::{OffChainRpcClientType, OnChainRpcClientType, TokenStoreType},
        utils::{
            constants::{SOL_DENOM, WSOL},
            store_tokens,
        },
    },
};

#[tracing::instrument(name = "Handle transaction", skip_all)]
pub async fn handle_transaction(
    signature: String,
    subscription_input: Arc<SubscriptionInput>,
    off_chain_rpc_client: OffChainRpcClientType,
    token_store: TokenStoreType,
    on_chain_rpc_client: OnChainRpcClientType,
) -> Result<Option<Trade>, Box<dyn std::error::Error + Send + Sync>> {
    let transaction = on_chain_rpc_client.get_transaction(signature).await?;

    if let Some(encoded_transaction) = transaction.result.as_ref().map(|res| &res.transaction) {
        if !wallet_is_the_fee_payer(&subscription_input.wallet, encoded_transaction) {
            return Ok(None);
        }
    } else {
        return Ok(None);
    }

    let transaction_meta = transaction
        .result
        .as_ref()
        .and_then(|res| res.meta.as_ref());

    if let Some(transaction_meta) = transaction_meta {
        let trades = build_trades(
            transaction_meta,
            &subscription_input,
            off_chain_rpc_client,
            token_store,
        )
        .await;
        return Ok(trades);
    }

    Ok(None)
}

fn wallet_is_the_fee_payer(wallet: &String, enc_transaction: &EncodedTransaction) -> bool {
    if let Some(fee_payer) = enc_transaction.message.account_keys.first() {
        wallet == fee_payer
    } else {
        false
    }
}

fn calc_sol_change(transaction_meta: &TransactionMeta) -> f64 {
    // it is safe because here pre_balances and post_balances have element at index 0
    let pre_balance = transaction_meta.pre_balances.get(0).cloned().unwrap_or(0);
    let post_balance = transaction_meta.post_balances.get(0).cloned().unwrap_or(0);
    (post_balance as f64 - pre_balance as f64) / SOL_DENOM
}

fn calc_fee(transaction_meta: &TransactionMeta) -> f64 {
    let fee = transaction_meta.fee;
    fee as f64 / SOL_DENOM
}

fn calc_token_changes_for_wallet(
    transaction_meta: &TransactionMeta,
    subscription_input: &SubscriptionInput,
) -> HashMap<String, f64> {
    let mut token_changes: HashMap<String, f64> = HashMap::new();

    let wallet_str = subscription_input.wallet.as_str();

    let sol_changes = calc_sol_change(transaction_meta) - calc_fee(transaction_meta);

    token_changes.insert(WSOL.to_string(), sol_changes);

    for token_balance in &transaction_meta.pre_token_balances {
        if token_balance.owner.as_deref() == Some(wallet_str) {
            token_changes.insert(
                token_balance.mint.clone(),
                -token_balance.ui_token_amount.to_f64(),
            );
        }
    }

    for token_balance in &transaction_meta.post_token_balances {
        if token_balance.owner.as_deref() == Some(wallet_str) {
            let entry = token_changes
                .entry(token_balance.mint.clone())
                .or_insert(0.0);
            *entry += token_balance.ui_token_amount.to_f64();
        }
    }

    token_changes.retain(|_, v| *v != 0.0);
    token_changes
}

#[tracing::instrument(name = "Build trades", skip_all)]
async fn build_trades(
    transaction_meta: &TransactionMeta,
    subscription_input: &SubscriptionInput,
    off_chain_rpc_client: OffChainRpcClientType,
    token_store: TokenStoreType,
) -> Option<Trade> {
    let mut sells: Vec<Transfer> = vec![];
    let mut buys: Vec<Transfer> = vec![];

    let token_changes = calc_token_changes_for_wallet(transaction_meta, subscription_input);
    if token_changes.is_empty() {
        return None;
    }

    let token_prices_map = off_chain_rpc_client
        .get_prices(token_changes.keys().cloned().collect())
        .await
        .ok();

    store_tokens(
        &token_changes.keys().cloned().collect(),
        off_chain_rpc_client,
        token_store.clone(),
    )
    .await
    .ok();

    for (mint, amount) in token_changes.into_iter() {
        let mut transfer = Transfer::new(mint.clone(), amount.abs());
        if let Some(token_prices_map) = token_prices_map.as_ref() {
            if let Some(token_price) = token_prices_map.get(&mint) {
                transfer.usd_price = Some(token_price.usd_price);
            }
            if let Ok(token_info) = token_store.clone().read().await.get_token(&mint).await {
                transfer.symbol = Some(token_info.symbol.clone());
                transfer.name = Some(token_info.name.clone());
            }
        }

        if amount < 0.0 {
            sells.push(transfer);
        } else if amount > 0.0 {
            buys.push(transfer);
        }
    }

    if sells.len() == 0 || buys.len() == 0 {
        return None;
    }

    Some(Trade {
        from: sells,
        to: buys,
    })
}
