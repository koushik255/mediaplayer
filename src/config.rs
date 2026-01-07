use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default)]
    pub default_video_path: Option<String>,

    #[serde(default)]
    pub screenshot_folder: Option<String>,

    #[serde(default)]
    pub subtitle_offset: f64,

    #[serde(default)]
    pub subtitle_offset_vertical: f64,

    #[serde(default)]
    pub subtitle_offset_horizontal: f64,

    #[serde(default)]
    pub subtitle_timing_offset: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_video_path: None,
            screenshot_folder: None,
            subtitle_offset: 100.0,
            subtitle_offset_vertical: 100.0,
            subtitle_offset_horizontal: 0.0,
            subtitle_timing_offset: 0.0,
        }
    }
}

fn get_config_path() -> PathBuf {
    std::env::current_dir()
        .expect("Failed to get current directory")
        .join("config.toml")
}

pub fn load_config() -> Result<AppConfig, String> {
    let config_path = get_config_path();

    if !config_path.exists() {
        println!("Config file does not exist, using defaults");
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: AppConfig =
        toml::from_str(&content).map_err(|e| format!("Failed to parse config file: {}", e))?;

    println!("Loaded config from file: {:?}", config);
    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path();

    let content =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, content).map_err(|e| format!("Failed to write config file: {}", e))?;

    println!("Config saved successfully to: {:?}", config_path);
    Ok(())
}
