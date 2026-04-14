pub mod api;
pub mod config;
pub mod db;
pub mod download;
pub mod error;
pub mod models;
pub mod player;
pub mod ui;
pub mod utils;

pub use config::settings::Settings;
pub use config::{set_debug_mode, is_debug_enabled};
pub use error::AppError;
