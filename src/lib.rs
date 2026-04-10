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
pub use error::AppError;
