use crate::proto::Transfer;

impl Transfer {
    pub fn new(mint: String, amount: f64) -> Self {
        Self {
            mint,
            symbol: None,
            name: None,
            amount,
            usd_price: None,
        }
    }
}
