use ratatui::style::Color;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub colors: ColorConfig,
}

#[derive(Deserialize)]
pub struct ColorConfig {
    pub time: String,
    pub date: String,
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
        })
    }
}

pub fn parse_color(color_str: &str) -> Color {
    match color_str {
        "Black" => Color::Black,
        "Red" => Color::Red,
        "Green" => Color::Green,
        "Yellow" => Color::Yellow,
        "Blue" => Color::Blue,
        "Magenta" => Color::Magenta,
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
