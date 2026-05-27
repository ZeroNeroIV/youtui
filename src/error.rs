pub use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("API error: {0}")]
    Api(String),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Playback error: {0}")]
    Playback(String),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
