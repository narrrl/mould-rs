use serde::{Deserialize, Serialize};
use std::fs;
use ratatui::style::Color;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ThemeConfig {
    #[serde(default)]
    pub transparent: bool,
    pub crust: String,
    pub surface0: String,
    pub surface1: String,
    pub text: String,
    pub blue: String,
    pub green: String,
    pub lavender: String,
    pub mauve: String,
    pub peach: String,
}

impl ThemeConfig {
    fn parse_hex(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            Color::Rgb(r, g, b)
        } else {
            Color::Reset
        }
    }

    pub fn crust(&self) -> Color { Self::parse_hex(&self.crust) }
    pub fn surface0(&self) -> Color { Self::parse_hex(&self.surface0) }
    pub fn surface1(&self) -> Color { Self::parse_hex(&self.surface1) }
    pub fn text(&self) -> Color { Self::parse_hex(&self.text) }
    pub fn blue(&self) -> Color { Self::parse_hex(&self.blue) }
    pub fn green(&self) -> Color { Self::parse_hex(&self.green) }
    pub fn lavender(&self) -> Color { Self::parse_hex(&self.lavender) }
    pub fn mauve(&self) -> Color { Self::parse_hex(&self.mauve) }
    pub fn peach(&self) -> Color { Self::parse_hex(&self.peach) }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            transparent: false,
            crust: "#11111b".to_string(),
            surface0: "#313244".to_string(),
            surface1: "#45475a".to_string(),
            text: "#cdd6f4".to_string(),
            blue: "#89b4fa".to_string(),
            green: "#a6e3a1".to_string(),
            lavender: "#b4befe".to_string(),
            mauve: "#cba6f7".to_string(),
            peach: "#fab387".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KeybindsConfig {
    pub down: String,
    pub up: String,
    pub edit: String,
    pub save: String,
    pub quit: String,
    pub normal_mode: String,
}

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self {
            down: "j".to_string(),
            up: "k".to_string(),
            edit: "i".to_string(),
            save: ":w".to_string(),
            quit: ":q".to_string(),
            normal_mode: "Esc".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub keybinds: KeybindsConfig,
}

pub fn load_config() -> Config {
    if let Some(mut config_dir) = dirs::config_dir() {
        config_dir.push("mould");
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
