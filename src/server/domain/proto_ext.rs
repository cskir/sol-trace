use crate::{
    proto::{Trade, Transfer},
    server::utils::{fmt_token, fmt_usd},
};

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

impl Transfer {
    fn to_string(&self) -> String {
        let mut token_info = String::new();
        if let Some(name) = &self.name {
            token_info.push_str(&name);
        }
        if let Some(symbol) = &self.symbol {
            if token_info.len() > 0 {
                token_info.push_str(" ");
            }
            token_info.push_str(&format!("[{symbol}]"));
        }
        if token_info.len() == 0 {
            token_info.push_str(&self.mint[0..4]);
            token_info.push_str("...");
            token_info.push_str(&self.mint[&self.mint.len() - 4..]);
        }

        let (price_str, value_str) = if let Some(price) = self.usd_price {
            (fmt_usd(price), fmt_usd(price * self.amount))
        } else {
            ("N/A".to_string(), "N/A".to_string())
        };

        format!(
            "  {} Amount: {} Current Value: {} (Price: {})",
            token_info,
            fmt_token(self.amount),
            value_str,
            price_str,
        )
    }
}

impl Trade {
    pub fn to_string(&self) -> String {
        let mut res = String::new();

        if self.from.len() == 1 && self.to.len() == 1 {
            res.push_str("Swap\t");
        } else if self.from.len() > 1 && self.to.len() == 1 {
            res.push_str("Multi Sell\t");
        } else if self.from.len() == 1 && self.to.len() > 1 {
            res.push_str("Multi Buy\t");
        } else {
            res.push_str("Multi Swap\t");
        }

        res.push_str("\nFrom:\t");

        for item in self.from.iter() {
            res.push_str(item.to_string().as_str());
        }

        res.push_str("\nTo:\t");

        for item in self.to.iter() {
            res.push_str(item.to_string().as_str());
        }

        res
    }
}
