use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub wallet: String,
    pub token_mints: Vec<String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_data = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_data)?;
        Ok(config)
    }
}
