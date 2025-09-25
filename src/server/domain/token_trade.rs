#[derive(Debug)]
pub struct TokenTrade {
    pub mint: String,
    pub amount: f64,
    pub usd_price: Option<f64>,
}

impl TokenTrade {
    pub fn new(mint: String, amount: f64) -> Self {
        Self {
            mint,
            amount,
            usd_price: None,
        }
    }
}
