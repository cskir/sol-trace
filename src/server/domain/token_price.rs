use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct TokenPrice {
    #[serde(rename = "usdPrice")]
    pub usd_price: f64,
    #[serde(rename = "blockId")]
    pub block_id: u64,
    pub decimals: u8,
    #[serde(rename = "priceChange24h")]
    pub price_change_24h: Option<f64>,
}
