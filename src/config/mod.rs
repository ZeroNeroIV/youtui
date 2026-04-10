pub mod settings;

use std::env;

pub fn is_development() -> bool {
    env::var("ENV").unwrap_or_else(|_| "development".to_string()) == "development"
}

pub const B_DEBUG_FLAG: bool = cfg!(debug_assertions);

pub fn log_to_file() -> bool {
    env::var("LOG_TO_FILE")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false)
}

pub fn log_to_terminal() -> bool {
    env::var("LOG_TO_TERMINAL")
        .map(|v| v.to_lowercase() != "false")
        .unwrap_or(true)
}
