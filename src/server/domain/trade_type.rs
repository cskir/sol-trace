use std::fmt;

use crate::server::domain::TokenTrade;

pub enum TradeType {
    SingleSwap {
        from: TokenTrade,
        to: TokenTrade,
    },
    MultiBuy {
        from: TokenTrade,
        to: Vec<TokenTrade>,
    },
    MultiSell {
        from: Vec<TokenTrade>,
        to: TokenTrade,
    },
    MultiSwap {
        from: Vec<TokenTrade>,
        to: Vec<TokenTrade>,
    },
}

impl fmt::Debug for TradeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradeType::SingleSwap { from, to } => write!(f, "Swap (from:{:?}, to:{:?})", from, to),
            TradeType::MultiBuy { from, to } => {
                write!(f, "Multi Buy (from:{:?}, to:{:?})", from, to)
            }
            TradeType::MultiSell { from, to } => {
                write!(f, "Multi Sell (from:{:?}, to:{:?})", from, to)
            }
            TradeType::MultiSwap { from, to } => {
                write!(f, "Multi Swap (from:{:?}, to:{:?})", from, to)
            }
        }
    }
}
