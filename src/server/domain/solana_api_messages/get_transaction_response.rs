use serde::Deserialize;

use crate::server::domain::ErrorResponse;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetTransactionResponse {
    Transaction(TransactionResponse),
    Error(ErrorResponse),
}

#[derive(Debug, Deserialize)]
pub struct TransactionResponse {
    pub result: Option<TransactionResult>,
    pub id: u64,
}

#[derive(Debug, Deserialize)]
pub struct TransactionResult {
    pub block_time: u64,
    pub slot: u64,
    pub version: String,
    pub transaction: EncodedTransaction,
    pub meta: Option<TransactionMeta>,
}

#[derive(Debug, Deserialize)]
pub struct EncodedTransaction {
    pub signatures: Vec<String>,
    pub message: TransactionMessage,
}

#[derive(Debug, Deserialize)]
pub struct TransactionMessage {
    #[serde(rename = "accountKeys")]
    pub account_keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionMeta {
    pub err: Option<serde_json::Value>,
    pub fee: u64,

    #[serde(rename = "preBalances")]
    pub pre_balances: Vec<u64>,

    #[serde(rename = "postBalances")]
    pub post_balances: Vec<u64>,

    #[serde(rename = "preTokenBalances")]
    pub pre_token_balances: Vec<TokenBalance>,

    #[serde(rename = "postTokenBalances")]
    pub post_token_balances: Vec<TokenBalance>,
}

#[derive(Debug, Deserialize)]
pub struct TokenBalance {
    //pub account_index: u64,
    pub mint: String,
    pub owner: Option<String>,
    #[serde(rename = "uiTokenAmount")]
    pub ui_token_amount: UiTokenAmount,
}

#[derive(Debug, Deserialize)]
pub struct UiTokenAmount {
    #[serde(rename = "uiAmount")]
    // pub ui_amount: Option<f64>, //deprecatod
    // #[serde(rename = "uiAmountString")]
    // pub ui_amount_string: String,
    pub decimals: u8,
    pub amount: String,
}
