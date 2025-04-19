use anyhow::Result;
use std::fs;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub hostname: String,
    pub paste: PasteConfig,
    pub send: SendConfig,
}

impl Config {
    pub fn load_from_file() -> Result<Config> {
        let config_str = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}

#[derive(Deserialize)]
pub struct PasteConfig {
    pub allow_html_injection: bool,
    pub max_paste_size: Option<usize>,
}

#[derive(Deserialize)]
pub struct SendConfig {
    pub file_limit: Option<usize>,
    pub max_file_size: Option<usize>,
}
