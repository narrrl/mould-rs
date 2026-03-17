use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;

/// Configuration for the application's appearance.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct ThemeConfig {
    /// If true, skip rendering the background block to let the terminal's transparency show.
    pub transparent: bool,
    /// Default background.
    pub bg_normal: String,
    /// Background for selected items and standard UI elements.
    pub bg_highlight: String,
    /// Active element background (e.g., insert mode).
    pub bg_active: String,
    /// Active element background (e.g., search mode).
    pub bg_search: String,
    /// Standard text.
    pub fg_normal: String,
    /// Secondary/inactive text.
    pub fg_dimmed: String,
    /// Text on selected items.
    pub fg_highlight: String,
    /// Red/Alert color for missing items.
    pub fg_warning: String,
    /// Accent color for modified items.
    pub fg_modified: String,
    /// High-contrast accent for titles and active UI elements.
    pub fg_accent: String,
    /// Borders.
    pub border_normal: String,
    /// Active borders (e.g., input mode).
    pub border_active: String,
    /// Color for tree indentation depth 1.
    pub tree_depth_1: String,
    /// Color for tree indentation depth 2.
    pub tree_depth_2: String,
    /// Color for tree indentation depth 3.
    pub tree_depth_3: String,
    /// Color for tree indentation depth 4.
    pub tree_depth_4: String,
}

impl ThemeConfig {
    /// Internal helper to parse a hex color string ("#RRGGBB") into a TUI Color.
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

    pub fn bg_normal(&self) -> Color { Self::parse_hex(&self.bg_normal) }
    pub fn bg_highlight(&self) -> Color { Self::parse_hex(&self.bg_highlight) }
    pub fn bg_active(&self) -> Color { Self::parse_hex(&self.bg_active) }
    pub fn bg_search(&self) -> Color { Self::parse_hex(&self.bg_search) }
    pub fn fg_normal(&self) -> Color { Self::parse_hex(&self.fg_normal) }
    pub fn fg_dimmed(&self) -> Color { Self::parse_hex(&self.fg_dimmed) }
    pub fn fg_highlight(&self) -> Color { Self::parse_hex(&self.fg_highlight) }
    pub fn fg_warning(&self) -> Color { Self::parse_hex(&self.fg_warning) }
    pub fn fg_modified(&self) -> Color { Self::parse_hex(&self.fg_modified) }
    pub fn fg_accent(&self) -> Color { Self::parse_hex(&self.fg_accent) }
    pub fn border_normal(&self) -> Color { Self::parse_hex(&self.border_normal) }
    pub fn border_active(&self) -> Color { Self::parse_hex(&self.border_active) }
    pub fn tree_depth_1(&self) -> Color { Self::parse_hex(&self.tree_depth_1) }
    pub fn tree_depth_2(&self) -> Color { Self::parse_hex(&self.tree_depth_2) }
    pub fn tree_depth_3(&self) -> Color { Self::parse_hex(&self.tree_depth_3) }
    pub fn tree_depth_4(&self) -> Color { Self::parse_hex(&self.tree_depth_4) }
}

impl Default for ThemeConfig {
    /// Default theme: Semantic Catppuccin Mocha.
    fn default() -> Self {
        Self {
            transparent: false,
            bg_normal: "#1e1e2e".to_string(), // base
            bg_highlight: "#89b4fa".to_string(), // blue
            bg_active: "#a6e3a1".to_string(), // green
            bg_search: "#cba6f7".to_string(), // mauve
            fg_normal: "#cdd6f4".to_string(), // text
            fg_dimmed: "#6c7086".to_string(), // overlay0
            fg_highlight: "#1e1e2e".to_string(), // base (dark for contrast against highlights)
            fg_warning: "#f38ba8".to_string(), // red
            fg_modified: "#fab387".to_string(), // peach
            fg_accent: "#b4befe".to_string(), // lavender
            border_normal: "#45475a".to_string(), // surface1
            border_active: "#a6e3a1".to_string(), // green
            tree_depth_1: "#b4befe".to_string(), // lavender
            tree_depth_2: "#cba6f7".to_string(), // mauve
            tree_depth_3: "#89b4fa".to_string(), // blue
            tree_depth_4: "#fab387".to_string(), // peach
        }
    }
}

/// Custom keybindings for navigation and application control.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct KeybindsConfig {
    pub down: String,
    pub up: String,
    pub edit: String,
    pub edit_append: String,
    pub edit_substitute: String,
    pub save: String,
    pub quit: String,
    pub normal_mode: String,
    pub search: String,
    pub next_match: String,
    pub previous_match: String,
    pub jump_top: String,
    pub jump_bottom: String,
    pub append_item: String,
    pub prepend_item: String,
    pub delete_item: String,
}

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self {
            down: "j".to_string(),
            up: "k".to_string(),
            edit: "i".to_string(),
            edit_append: "A".to_string(),
            edit_substitute: "S".to_string(),
            save: ":w".to_string(),
            quit: ":q".to_string(),
            normal_mode: "Esc".to_string(),
            search: "/".to_string(),
            next_match: "n".to_string(),
            previous_match: "N".to_string(),
            jump_top: "gg".to_string(),
            jump_bottom: "G".to_string(),
            append_item: "o".to_string(),
            prepend_item: "O".to_string(),
            delete_item: "dd".to_string(),
        }
    }
}

/// Root configuration structure for mould.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub keybinds: KeybindsConfig,
}

/// Loads the configuration from the user's home config directory (~/.config/mould/config.toml).
/// Falls back to default settings if no configuration is found.
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
