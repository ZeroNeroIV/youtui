use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[allow(dead_code)]
const DEFAULT_TIMEOUT_SECS: u64 = 10;
const MAX_RETRIES: u32 = 3;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum PipedError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Video not found: {0}")]
    NotFound(String),
}

impl serde::Serialize for PipedError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// ============================================================================
// Client
// ============================================================================

pub struct PipedClient {
    base_url: String,
    client: Client,
}

impl PipedClient {
    /// Create a new PipedClient with the default instance
    pub fn new(base_url: &str) -> Self {
        Self::with_client(base_url, Client::new())
    }

    /// Create a new PipedClient with a custom reqwest Client
    pub fn with_client(base_url: &str, client: Client) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    /// Create a new PipedClient with custom timeout
    pub fn with_timeout(base_url: &str, timeout_secs: u64) -> Result<Self, PipedError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;
        Ok(Self::with_client(base_url, client))
    }

    /// Build the API URL for a given endpoint
    fn api_url(&self, endpoint: &str) -> String {
        format!("{}/{}", self.base_url, endpoint.trim_start_matches('/'))
    }

    /// Perform a GET request with retry logic
    async fn get(&self, url: &str) -> Result<String, PipedError> {
        let mut retries = 0;
        let mut last_error = None;

        while retries <= MAX_RETRIES {
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response.text().await?);
                    }
                    // Retry on server errors (5xx)
                    if response.status().is_server_error() {
                        last_error = Some(PipedError::ApiError(format!(
                            "Server error: {}",
                            response.status()
                        )));
                        retries += 1;
                        continue;
                    }
                    // Not found is a clear error
                    if response.status() == reqwest::StatusCode::NOT_FOUND {
                        return Err(PipedError::NotFound(url.to_string()));
                    }
                    return Err(PipedError::ApiError(format!(
                        "HTTP error: {}",
                        response.status()
                    )));
                }
                Err(e) => {
                    last_error = Some(PipedError::RequestFailed(e));
                    retries += 1;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| PipedError::ApiError("Max retries exceeded".to_string())))
    }
}

// ============================================================================
// Data Models (Piped-specific)
// ============================================================================

/// Stream quality info
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stream {
    pub url: String,
    #[serde(rename = "quality")]
    pub quality: String,
    pub mime_type: String,
    #[serde(rename = "codecs", default)]
    pub codecs: Option<String>,
    pub bitrate: Option<i64>,
    pub fps: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    #[serde(rename = "videoOnly", default)]
    pub video_only: Option<bool>,
}

/// Audio stream info
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioStream {
    pub url: String,
    #[serde(rename = "quality")]
    pub quality: String,
    pub mime_type: String,
    #[serde(rename = "codec", default)]
    pub codec: Option<String>,
    pub bitrate: Option<i64>,
    #[serde(rename = "videoOnly", default)]
    pub video_only: Option<bool>,
}

/// Video stream info
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoStream {
    pub url: String,
    #[serde(rename = "quality")]
    pub quality: String,
    pub mime_type: String,
    #[serde(rename = "codec", default)]
    pub codec: Option<String>,
    pub bitrate: Option<i64>,
    pub fps: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    #[serde(rename = "videoOnly", default)]
    pub video_only: Option<bool>,
}

/// Subtitle info
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subtitle {
    pub url: String,
    pub mime_type: String,
    pub name: String,
    pub code: String,
    #[serde(rename = "autoGenerated", default)]
    pub auto_generated: Option<bool>,
}

/// Trending video item from Piped API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrendingVideo {
    pub title: String,
    #[serde(rename = "thumbnail")]
    pub thumbnail: String,
    #[serde(rename = "duration")]
    pub duration: Option<i32>,
    #[serde(rename = "views")]
    pub views: Option<i64>,
    #[serde(rename = "uploader")]
    pub uploader: Option<String>,
    #[serde(rename = "uploaderUrl")]
    pub uploader_url: Option<String>,
    #[serde(rename = "uploaderAvatar")]
    pub uploader_avatar: Option<String>,
    #[serde(rename = "uploaderVerified")]
    pub uploader_verified: Option<bool>,
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "uploadedDate")]
    pub uploaded_date: Option<String>,
}

/// Full video info from streams endpoint
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Video {
    pub title: String,
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
    pub description: String,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: Option<String>,
    pub uploader: Option<String>,
    #[serde(rename = "uploaderUrl")]
    pub uploader_url: Option<String>,
    #[serde(rename = "uploaderAvatar")]
    pub uploader_avatar: Option<String>,
    #[serde(rename = "uploaderVerified")]
    pub uploader_verified: Option<bool>,
    #[serde(rename = "uploadDate")]
    pub upload_date: Option<String>,
    pub views: Option<i64>,
    pub likes: Option<i64>,
    pub dislikes: Option<i64>,
    pub duration: Option<i32>,
    #[serde(rename = "livestream")]
    pub livestream: Option<bool>,
    #[serde(rename = "audioStreams", default)]
    pub audio_streams: Option<Vec<AudioStream>>,
    #[serde(rename = "videoStreams", default)]
    pub video_streams: Option<Vec<VideoStream>>,
    #[serde(rename = "relatedStreams", default)]
    pub related_streams: Option<Vec<TrendingVideo>>,
    #[serde(rename = "subtitles", default)]
    pub subtitles: Option<Vec<Subtitle>>,
    #[serde(rename = "hls", default)]
    pub hls: Option<String>,
    #[serde(rename = "dash", default)]
    pub dash: Option<String>,
    #[serde(rename = "lbryId", default)]
    pub lbry_id: Option<String>,
    #[serde(rename = "proxyUrl", default)]
    pub proxy_url: Option<String>,
}

/// Streams response - contains video info AND stream URLs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Streams {
    pub title: String,
    pub description: String,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: Option<String>,
    pub uploader: Option<String>,
    #[serde(rename = "uploaderUrl")]
    pub uploader_url: Option<String>,
    #[serde(rename = "uploaderVerified")]
    pub uploader_verified: Option<bool>,
    #[serde(rename = "uploadDate")]
    pub upload_date: Option<String>,
    pub views: Option<i64>,
    pub likes: Option<i64>,
    pub dislikes: Option<i64>,
    pub duration: Option<i32>,
    #[serde(rename = "livestream")]
    pub livestream: Option<bool>,
    #[serde(rename = "audioStreams", default)]
    pub audio_streams: Vec<AudioStream>,
    #[serde(rename = "videoStreams", default)]
    pub video_streams: Vec<VideoStream>,
    #[serde(rename = "relatedStreams", default)]
    pub related_streams: Vec<TrendingVideo>,
    #[serde(rename = "subtitles", default)]
    pub subtitles: Vec<Subtitle>,
    #[serde(rename = "hls", default)]
    pub hls: Option<String>,
    #[serde(rename = "dash", default)]
    pub dash: Option<String>,
    #[serde(rename = "lbryId", default)]
    pub lbry_id: Option<String>,
    #[serde(rename = "proxyUrl", default)]
    pub proxy_url: Option<String>,
}

impl Streams {
    /// Extract video ID from the URL in related streams or return None
    pub fn video_id(&self) -> Option<String> {
        // Try to get from first related stream URL
        if let Some(first) = self.related_streams.first() {
            return extract_video_id(&first.url);
        }
        None
    }
}

/// Search result - simplified for Piped (uses channel/playlist data)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum PipedSearchResult {
    Video(TrendingVideo),
    Channel {
        #[serde(rename = "name")]
        name: String,
        #[serde(rename = "channelId")]
        channel_id: String,
        #[serde(rename = "avatar")]
        avatar: Option<String>,
        #[serde(rename = "verified")]
        verified: Option<bool>,
    },
    Playlist {
        #[serde(rename = "title")]
        title: String,
        #[serde(rename = "playlistId")]
        playlist_id: String,
        #[serde(rename = "thumbnail")]
        thumbnail: Option<String>,
        #[serde(rename = "uploader")]
        uploader: Option<String>,
    },
}

impl PipedSearchResult {
    pub fn as_video(&self) -> Option<&TrendingVideo> {
        match self {
            PipedSearchResult::Video(v) => Some(v),
            _ => None,
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract video ID from a YouTube URL like /watch?v=abcd123
fn extract_video_id(url: &str) -> Option<String> {
    if url.contains("v=") {
        let parts: Vec<&str> = url.split("v=").collect();
        if let Some(id) = parts.get(1) {
            // Handle & (e.g., /watch?v=abc&list=...)
            let id = if let Some(pos) = id.find('&') {
                &id[..pos]
            } else {
                id
            };
            return Some(id.to_string());
        }
    }
    // Handle /video/abc123 format
    if url.contains("/video/") {
        let parts: Vec<&str> = url.split("/video/").collect();
        if let Some(id) = parts.get(1) {
            return Some(id.to_string());
        }
    }
    None
}

// ============================================================================
// PipedClient Methods
// ============================================================================

impl PipedClient {
    pub async fn search(&self, _query: &str) -> Result<Vec<Video>, PipedError> {
        Ok(vec![])
    }

    pub async fn get_streams(&self, video_id: &str) -> Result<Streams, PipedError> {
        let url = self.api_url(&format!("/streams/{}", video_id));
        let response = self.get(&url).await?;

        let streams: Streams = serde_json::from_str(&response)?;
        Ok(streams)
    }

    pub async fn get_trending(&self) -> Result<Vec<TrendingVideo>, PipedError> {
        let url = self.api_url("/trending");
        let response = self.get(&url).await?;

        let videos: Vec<TrendingVideo> = serde_json::from_str(&response)?;
        Ok(videos)
    }

    pub async fn get_trending_region(
        &self,
        region: &str,
    ) -> Result<Vec<TrendingVideo>, PipedError> {
        let url = format!("{}?region={}", self.api_url("/trending"), region);
        let response = self.get(&url).await?;

        let videos: Vec<TrendingVideo> = serde_json::from_str(&response)?;
        Ok(videos)
    }
}

// ============================================================================
// Allow unused fields
// ============================================================================

#[allow(dead_code)]
// Keep these to avoid warnings on unused fields that may be used in future versions
mod unused {
    use super::*;

    pub fn _mark_unused_video_fields(_v: &Video) {}
    pub fn _mark_unused_streams_fields(_s: &Streams) {}
    pub fn _mark_unused_trending_fields(_t: &TrendingVideo) {}
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = PipedClient::new("https://pipedapi.kavin.rocks");
        assert_eq!(client.base_url, "https://pipedapi.kavin.rocks");
    }

    #[test]
    fn test_api_url_building() {
        let client = PipedClient::new("https://pipedapi.kavin.rocks");
        assert_eq!(
            client.api_url("/streams/abc123"),
            "https://pipedapi.kavin.rocks/streams/abc123"
        );
        assert_eq!(
            client.api_url("trending"),
            "https://pipedapi.kavin.rocks/trending"
        );
    }

    #[test]
    fn test_extract_video_id() {
        assert_eq!(
            extract_video_id("/watch?v=abcd123"),
            Some("abcd123".to_string())
        );
        assert_eq!(
            extract_video_id("/watch?v=abcd123&list=xyz"),
            Some("abcd123".to_string())
        );
        assert_eq!(
            extract_video_id("/video/xyz789"),
            Some("xyz789".to_string())
        );
    }
}
