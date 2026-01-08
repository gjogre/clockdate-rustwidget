use ratatui::style::Color;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub colors: ColorConfig,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub fonts: FontConfig,
}

#[derive(Deserialize)]
pub struct ColorConfig {
    pub time: String,
    pub date: String,
}

#[derive(Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_margin_top")]
    pub margin_top: i32,
    #[serde(default = "default_margin_right")]
    pub margin_right: i32,
    #[serde(default = "default_width")]
    pub width: i32,
    #[serde(default = "default_height")]
    pub height: i32,
    #[serde(default = "default_monitor")]
    pub monitor: String,
    #[serde(default = "default_date_offset")]
    pub date_offset: i32,
}

#[derive(Deserialize)]
pub struct FontConfig {
    #[serde(default = "default_time_size")]
    pub time_size: i32,
    #[serde(default = "default_date_size")]
    pub date_size: i32,
}

fn default_margin_top() -> i32 { 10 }
fn default_margin_right() -> i32 { 10 }
fn default_width() -> i32 { 400 }
fn default_height() -> i32 { 180 }
fn default_time_size() -> i32 { 12 }
fn default_date_size() -> i32 { 10 }
fn default_monitor() -> String { "DP-1".to_string() }
fn default_date_offset() -> i32 { -65 }

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            margin_top: default_margin_top(),
            margin_right: default_margin_right(),
            width: default_width(),
            height: default_height(),
            monitor: default_monitor(),
            date_offset: default_date_offset(),
        }
    }
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            time_size: default_time_size(),
            date_size: default_date_size(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Try multiple locations for config file
        let config_paths = vec![
            std::env::var("HOME").ok().map(|h| format!("{}/.config/clockdate/config.toml", h)),
            Some(std::env::current_dir()?.join("config.toml").to_string_lossy().to_string()),
        ];

        for path in config_paths.into_iter().flatten() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str(&content) {
                    return Ok(config);
                }
            }
        }

        Err("Config file not found".into())
    }

    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_else(|_| Config {
            colors: ColorConfig {
                time: "Blue".to_string(),
                date: "DarkGray".to_string(),
            },
            window: WindowConfig::default(),
            fonts: FontConfig::default(),
        })
    }
}

pub fn parse_color(color_str: &str) -> Color {
    // Check if it's a hex color (e.g., "#A020F0" for purple)
    if color_str.starts_with('#') && color_str.len() == 7 {
        if let Ok(r) = u8::from_str_radix(&color_str[1..3], 16) {
            if let Ok(g) = u8::from_str_radix(&color_str[3..5], 16) {
                if let Ok(b) = u8::from_str_radix(&color_str[5..7], 16) {
                    return Color::Rgb(r, g, b);
                }
            }
        }
    }

    match color_str {
        "Black" => Color::Black,
        "Red" => Color::Red,
        "Green" => Color::Green,
        "Yellow" => Color::Yellow,
        "Blue" => Color::Blue,
        "Magenta" => Color::Magenta,
        "Purple" => Color::Rgb(160, 32, 240), // Purple color
        "Cyan" => Color::Cyan,
        "Gray" => Color::Gray,
        "DarkGray" => Color::DarkGray,
        "LightRed" => Color::LightRed,
        "LightGreen" => Color::LightGreen,
        "LightYellow" => Color::LightYellow,
        "LightBlue" => Color::LightBlue,
        "LightMagenta" => Color::LightMagenta,
        "LightCyan" => Color::LightCyan,
        "White" => Color::White,
        _ => Color::Blue, // Default fallback
    }
}
