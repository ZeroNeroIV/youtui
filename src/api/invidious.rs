use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Debug, thiserror::Error)]
pub enum InvidiousError {
    #[error("Bad or unreachable Invidious instance")]
    BadInstance,
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Video not found: {0}")]
    NotFound(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

impl From<reqwest::Error> for InvidiousError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_connect() || e.is_timeout() {
            InvidiousError::BadInstance
        } else {
            InvidiousError::RequestFailed(e.to_string())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    #[serde(rename = "videoId")]
    pub video_id: String,
    pub title: String,
    pub author: Option<String>,
    #[serde(rename = "viewCount")]
    pub view_count: Option<u64>,
    #[serde(rename = "lengthSeconds")]
    pub length_seconds: Option<u64>,
    #[serde(rename = "publishedText")]
    pub published_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistVideo {
    #[serde(rename = "videoId")]
    pub video_id: String,
    pub title: String,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistDetails {
    pub title: String,
    pub videos: Vec<PlaylistVideo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdaptiveFormat {
    url: String,
    #[serde(rename = "type")]
    mime_type: String,
    #[serde(rename = "resolution", default)]
    resolution: Option<String>,
    quality: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VideoDetails {
    #[serde(rename = "adaptiveFormats", default)]
    adaptive_formats: Vec<AdaptiveFormat>,
    #[serde(rename = "formatStreams", default)]
    format_streams: Vec<AdaptiveFormat>,
}

pub struct InvidiousClient {
    base_url: String,
    client: Client,
}

impl InvidiousClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .connect_timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Video>, InvidiousError> {
        let url = format!("{}/api/v1/search?q={}&type=video", self.base_url, urlencoding(query));
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            return Err(InvidiousError::RequestFailed(format!("HTTP {}", resp.status())));
        }

        let items: serde_json::Value = resp.json().await
            .map_err(|e| InvidiousError::RequestFailed(e.to_string()))?;

        let videos = items.as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter(|v| v.get("type").and_then(|t| t.as_str()) == Some("video"))
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect();

        Ok(videos)
    }

    pub async fn get_playlist(&self, playlist_id: &str) -> Result<PlaylistDetails, InvidiousError> {
        let url = format!("{}/api/v1/playlists/{}", self.base_url, playlist_id);
        let resp = self.client.get(&url).send().await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(InvidiousError::NotFound(playlist_id.to_string()));
        }
        if !resp.status().is_success() {
            return Err(InvidiousError::RequestFailed(format!("HTTP {}", resp.status())));
        }

        let details: PlaylistDetails = resp.json().await
            .map_err(|e| InvidiousError::RequestFailed(e.to_string()))?;
        Ok(details)
    }

    pub async fn get_stream_url(&self, video_id: &str) -> Result<String, InvidiousError> {
        let url = format!("{}/api/v1/videos/{}", self.base_url, video_id);
        let resp = self.client.get(&url).send().await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(InvidiousError::NotFound(video_id.to_string()));
        }
        if !resp.status().is_success() {
            return Err(InvidiousError::RequestFailed(format!("HTTP {}", resp.status())));
        }

        let details: VideoDetails = resp.json().await
            .map_err(|e| InvidiousError::RequestFailed(e.to_string()))?;

        if let Some(f) = details.format_streams.first() {
            return Ok(f.url.clone());
        }
        if let Some(f) = details.adaptive_formats.first() {
            return Ok(f.url.clone());
        }

        Err(InvidiousError::NotFound(video_id.to_string()))
    }
}

fn urlencoding(s: &str) -> String {
    s.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        ' ' => '+'.to_string(),
        c => format!("%{:02X}", c as u32),
    }).collect()
}
