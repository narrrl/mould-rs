use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ThemeConfig {
    pub name: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "catppuccin_mocha".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub theme: ThemeConfig,
}

pub fn load_config() -> Config {
    if let Some(mut config_dir) = dirs::config_dir() {
        config_dir.push("cenv-rs");
        config_dir.push("config.toml");

        if config_dir.exists() {
            if let Ok(content) = fs::read_to_string(config_dir) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
    }
    Config::default()
}
