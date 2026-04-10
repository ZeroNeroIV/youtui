use super::video::Video;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub videos: Vec<Video>,
    pub next_page_token: Option<String>,
}
