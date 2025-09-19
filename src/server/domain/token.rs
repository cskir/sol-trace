use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Token {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub decimals: u8,
}
