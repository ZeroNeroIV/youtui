pub mod config;
pub mod db;
pub mod error;
pub mod ui;
pub mod api;
pub mod models;
pub mod player;
pub mod download;
pub mod utils;

pub use config::settings::Settings;
pub use error::AppError;