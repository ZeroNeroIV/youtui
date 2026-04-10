use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application version - used for startup banner
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// State persisted across sessions (last screen, scroll position, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    /// Last active sidebar index (screen)
    pub last_sidebar_index: usize,
    /// Last scroll position in content list
    pub last_content_index: usize,
    /// Last scroll position in settings list
    pub last_settings_index: usize,
}

impl AppState {
    /// Load app state from config directory
    pub fn load() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let state_path = config_dir.join("youtui-rs").join("state.json");
            if state_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&state_path) {
                    if let Ok(state) = serde_json::from_str(&content) {
                        return state;
                    }
                }
            }
        }
        Self::default()
    }

    /// Save app state to config directory
    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(config_dir) = dirs::config_dir() {
            let config_dir = config_dir.join("youtui-rs");
            std::fs::create_dir_all(&config_dir)?;
            let state_path = config_dir.join("state.json");
            let content = serde_json::to_string_pretty(self)?;
            std::fs::write(state_path, content)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_quality: String,
    pub default_format: String,
    pub download_path: PathBuf,
    pub player: String,
    pub api_instance_invidious: String,
    pub api_instance_piped: String,
    pub theme: String,
    pub auto_play: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_quality: "720".to_string(),
            default_format: "mp4".to_string(),
            download_path: dirs::video_dir().unwrap_or_else(|| PathBuf::from(".")),
            player: "mpv".to_string(),
            api_instance_invidious: "https://invidious.snopyta.org".to_string(),
            api_instance_piped: "https://pipedapi.kavin.rocks".to_string(),
            theme: "dark".to_string(),
            auto_play: true,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("youtui-rs").join("config.json");
            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(settings) = serde_json::from_str(&content) {
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(config_dir) = dirs::config_dir() {
            let config_dir = config_dir.join("youtui-rs");
            std::fs::create_dir_all(&config_dir)?;
            let config_path = config_dir.join("config.json");
            let content = serde_json::to_string_pretty(self)?;
            std::fs::write(config_path, content)?;
        }
        Ok(())
    }
}
