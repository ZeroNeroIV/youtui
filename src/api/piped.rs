use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum PipedError {
    #[error("Video not found: {0}")]
    NotFound(String),
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

impl From<reqwest::Error> for PipedError {
    fn from(e: reqwest::Error) -> Self {
        PipedError::RequestFailed(e.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipedVideoStream {
    pub url: String,
    pub quality: Option<String>,
    #[serde(rename = "mimeType", default)]
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipedStreams {
    #[serde(rename = "videoStreams", default)]
    pub video_streams: Vec<PipedVideoStream>,
    #[serde(rename = "audioStreams", default)]
    pub audio_streams: Vec<PipedVideoStream>,
    pub hls: Option<String>,
    pub dash: Option<String>,
}

pub struct PipedClient {
    base_url: String,
    client: Client,
}

impl PipedClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn get_streams(&self, video_id: &str) -> Result<PipedStreams, PipedError> {
        let url = format!("{}/streams/{}", self.base_url, video_id);
        let resp = self.client.get(&url).send().await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(PipedError::NotFound(video_id.to_string()));
        }
        if !resp.status().is_success() {
            return Err(PipedError::RequestFailed(format!("HTTP {}", resp.status())));
        }

        let streams: PipedStreams = resp.json().await
            .map_err(|e| PipedError::Parse(e.to_string()))?;
        Ok(streams)
    }
}
