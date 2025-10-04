use crate::client::{Config, fmt_token, fmt_usd};
use crate::proto::{Holding, InitRequest, Trade, Transfer};
use std::fmt;

impl InitRequest {
    pub fn build(config: Config) -> Result<Self, String> {
        if config.wallet.is_empty() {
            return Err("Wallet address is required in config".to_string());
        }

        if config.token_mints.is_empty() {
            return Err("Error: No token mints specified in config".to_string());
        }

        Ok(Self {
            wallet: config.wallet,
            tokens: config.token_mints,
        })
    }
}

impl fmt::Display for Holding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let price_str = if let Some(price) = self.usd_price {
            fmt_usd(price)
        } else {
            "N/A".to_string()
        };
        let value_str = if let Some(value) = self.usd_value {
            fmt_usd(value)
        } else {
            "N/A".to_string()
        };
        write!(
            f,
            "  {} ({}) - Balance: {}, Price: {}, Value: {}",
            self.name, self.symbol, self.balance, price_str, value_str
        )
    }
}

impl Transfer {
    fn to_short_string(&self) -> String {
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

        let value_str = if let Some(price) = self.usd_price {
            fmt_usd(price * self.amount)
        } else {
            "N/A".to_string()
        };

        format!(
            "  {} Amount: {} Current Value: {}",
            token_info,
            fmt_token(self.amount),
            value_str,
        )
    }
}

impl Trade {
    pub fn to_string_lines(&self) -> Vec<String> {
        let mut res = vec![];

        if self.from.len() == 1 && self.to.len() == 1 {
            res.push("Swap".to_string());
        } else if self.from.len() > 1 && self.to.len() == 1 {
            res.push("Multi Sell".to_string());
        } else if self.from.len() == 1 && self.to.len() > 1 {
            res.push("Multi Buy".to_string());
        } else {
            res.push("Multi Swap".to_string());
        }

        res.push("From:".to_string());

        for item in self.from.iter() {
            res.push(item.to_short_string());
        }

        res.push("To:".to_string());

        for item in self.to.iter() {
            res.push(item.to_short_string());
        }

        res
    }
}
