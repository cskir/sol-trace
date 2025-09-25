#[derive(Debug)]
pub struct TokenTrade {
    pub mint: String,
    pub symbol: Option<String>,
    pub amount: f64,
    pub usd_price: Option<f64>,
}

impl TokenTrade {
    pub fn new(mint: String, amount: f64) -> Self {
        Self {
            mint,
            symbol: None,
            amount,
            usd_price: None,
        }
    }
}
