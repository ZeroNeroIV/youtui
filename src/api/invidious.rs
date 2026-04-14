use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[allow(dead_code)]
const DEFAULT_TIMEOUT_SECS: u64 = 10;
const MAX_RETRIES: u32 = 3;

/// Client for interacting with the Invidious API
#[derive(Clone)]
pub struct InvidiousClient {
    base_url: String,
    client: Client,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum InvidiousError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Instance is bad (returns HTML or 403)")]
    BadInstance,
    #[error("Video not found: {0}")]
    NotFound(String),
}

impl serde::Serialize for InvidiousError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvidiousStreams {
    pub url: String,
    #[serde(rename = "adaptiveFormats")]
    pub adaptive_formats: Vec<InvidiousFormat>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvidiousFormat {
    pub url: String,
    pub quality: String,
    pub mime_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Video {
    #[serde(rename = "type")]
    pub video_type: String,
    pub title: String,
    #[serde(rename = "videoId")]
    pub video_id: String,
    pub author: Option<String>,
    #[serde(rename = "authorId")]
    pub author_id: Option<String>,
    #[serde(rename = "authorUrl")]
    pub author_url: Option<String>,
    #[serde(rename = "authorThumbnails")]
    pub author_thumbnails: Option<Vec<Thumbnail>>,
    #[serde(rename = "videoThumbnails")]
    pub video_thumbnails: Vec<Thumbnail>,
    pub description: Option<String>,
    #[serde(rename = "descriptionHtml")]
    pub description_html: Option<String>,
    #[serde(rename = "viewCount")]
    pub view_count: Option<i64>,
    #[serde(rename = "likeCount")]
    pub like_count: Option<i32>,
    #[serde(rename = "dislikeCount")]
    pub dislike_count: Option<i32>,
    pub published: Option<i64>,
    #[serde(rename = "publishedText")]
    pub published_text: Option<String>,
    #[serde(rename = "lengthSeconds")]
    pub length_seconds: Option<i32>,
    pub live_now: Option<bool>,
    pub paid: Option<bool>,
    pub premium: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Thumbnail {
    pub quality: Option<String>,
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    #[serde(rename = "type")]
    pub channel_type: String,
    pub author: String,
    #[serde(rename = "authorId")]
    pub author_id: String,
    #[serde(rename = "authorUrl")]
    pub author_url: String,
    #[serde(rename = "authorThumbnails")]
    pub author_thumbnails: Option<Vec<Thumbnail>>,
    pub auto_generated: Option<bool>,
    pub sub_count: Option<i32>,
    #[serde(rename = "videoCount")]
    pub video_count: Option<i32>,
    pub description: Option<String>,
    #[serde(rename = "descriptionHtml")]
    pub description_html: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Playlist {
    #[serde(rename = "type")]
    pub playlist_type: String,
    pub title: String,
    #[serde(rename = "playlistId")]
    pub playlist_id: String,
    #[serde(rename = "playlistThumbnail")]
    pub playlist_thumbnail: Option<String>,
    pub author: Option<String>,
    #[serde(rename = "authorId")]
    pub author_id: Option<String>,
    #[serde(rename = "authorUrl")]
    pub author_url: Option<String>,
    #[serde(rename = "authorVerified")]
    pub author_verified: Option<bool>,
    #[serde(rename = "videoCount")]
    pub video_count: Option<i32>,
    pub videos: Option<Vec<PlaylistVideo>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistVideo {
    pub title: String,
    #[serde(rename = "videoId")]
    pub video_id: String,
    pub author: Option<String>,
    #[serde(rename = "authorId")]
    pub author_id: Option<String>,
    #[serde(rename = "authorUrl")]
    pub author_url: Option<String>,
    #[serde(rename = "videoThumbnails")]
    pub video_thumbnails: Option<Vec<Thumbnail>>,
    pub index: Option<i32>,
    #[serde(rename = "lengthSeconds")]
    pub length_seconds: Option<i32>,
}

/// Detailed playlist response from Invidious API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistDetails {
    pub title: String,
    #[serde(rename = "playlistId")]
    pub playlist_id: String,
    pub author: Option<String>,
    #[serde(rename = "authorId")]
    pub author_id: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "videoCount")]
    pub video_count: i32,
    #[serde(rename = "viewCount")]
    pub view_count: Option<i64>,
    pub videos: Vec<PlaylistVideo>,
}

/// Search result enum - can be video, channel, playlist, or hashtag
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum SearchResult {
    Video(Video),
    Channel(Channel),
    Playlist(Playlist),
    Hashtag {
        #[serde(rename = "type")]
        hashtag_type: String,
        title: String,
        url: String,
    },
}

impl SearchResult {
    pub fn as_video(&self) -> Option<&Video> {
        match self {
            SearchResult::Video(v) => Some(v),
            _ => None,
        }
    }
}

// ============================================================================
// InvidiousClient Implementation
// ============================================================================

impl InvidiousClient {
    /// Create a new InvidiousClient with the default Invidious instance
    pub fn new(base_url: &str) -> Self {
        Self::with_client(base_url, Client::new())
    }

    /// Create a new InvidiousClient with a custom reqwest Client
    pub fn with_client(base_url: &str, client: Client) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    /// Create a new InvidiousClient with custom timeout
    pub fn with_timeout(base_url: &str, timeout_secs: u64) -> Result<Self, InvidiousError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;
        Ok(Self::with_client(base_url, client))
    }

    /// Build the API URL for a given endpoint
    fn api_url(&self, endpoint: &str) -> String {
        format!(
            "{}/api/v1/{}",
            self.base_url,
            endpoint.trim_start_matches('/')
        )
    }

    /// Perform a GET request with retry logic
    async fn get(&self, url: &str) -> Result<String, InvidiousError> {
        let mut retries = 0;
        let mut last_error = None;

        while retries <= MAX_RETRIES {
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::FORBIDDEN {
                        return Err(InvidiousError::BadInstance);
                    }

                    if let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE) {
                        if let Ok(ct_str) = content_type.to_str() {
                            if ct_str.contains("text/html") {
                                return Err(InvidiousError::BadInstance);
                            }
                        }
                    }

                    if response.status().is_success() {
                        return Ok(response.text().await?);
                    }
                    // Retry on server errors (5xx)
                    if response.status().is_server_error() {
                        last_error = Some(InvidiousError::ApiError(format!(
                            "Server error: {}",
                            response.status()
                        )));
                        retries += 1;
                        continue;
                    }
                    // Not found is a clear error
                    if response.status() == reqwest::StatusCode::NOT_FOUND {
                        return Err(InvidiousError::NotFound(url.to_string()));
                    }
                    return Err(InvidiousError::ApiError(format!(
                        "HTTP error: {}",
                        response.status()
                    )));
                }
                Err(e) => {
                    last_error = Some(InvidiousError::RequestFailed(e));
                    retries += 1;
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| InvidiousError::ApiError("Max retries exceeded".to_string())))
    }

    /// Search for videos
    pub async fn search(&self, query: &str) -> Result<Vec<Video>, InvidiousError> {
        let url = format!(
            "{}?q={}",
            self.api_url("/search"),
            urlencoding::encode(query)
        );
        let response = self.get(&url).await?;

        let results: Vec<SearchResult> = serde_json::from_str(&response)?;
        let videos: Vec<Video> = results
            .into_iter()
            .filter_map(|r| r.as_video().cloned())
            .collect();

        Ok(videos)
    }

    /// Search for videos with optional filters
    pub async fn search_with_options(
        &self,
        query: &str,
        search_type: Option<&str>,
        region: Option<&str>,
        sort: Option<&str>,
    ) -> Result<Vec<Video>, InvidiousError> {
        let mut url = format!(
            "{}?q={}",
            self.api_url("/search"),
            urlencoding::encode(query)
        );

        if let Some(t) = search_type {
            url.push_str(&format!("&type={}", t));
        }
        if let Some(r) = region {
            url.push_str(&format!("&region={}", r));
        }
        if let Some(s) = sort {
            url.push_str(&format!("&sort={}", s));
        }

        let response = self.get(&url).await?;

        let results: Vec<SearchResult> = serde_json::from_str(&response)?;
        let videos: Vec<Video> = results
            .into_iter()
            .filter_map(|r| r.as_video().cloned())
            .collect();

        Ok(videos)
    }

    /// Get video details by ID
    pub async fn get_video(&self, video_id: &str) -> Result<Video, InvidiousError> {
        let url = self.api_url(&format!("/videos/{}", video_id));
        let response = self.get(&url).await?;

        let video: Video = serde_json::from_str(&response)?;
        Ok(video)
    }

    /// Get trending videos
    pub async fn get_trending(&self) -> Result<Vec<Video>, InvidiousError> {
        let url = self.api_url("/trending");
        let response = self.get(&url).await?;

        let results: Vec<SearchResult> = serde_json::from_str(&response)?;
        let videos: Vec<Video> = results
            .into_iter()
            .filter_map(|r| r.as_video().cloned())
            .collect();

        Ok(videos)
    }

    /// Get popular videos
    pub async fn get_popular(&self) -> Result<Vec<Video>, InvidiousError> {
        let url = self.api_url("/popular");
        let response = self.get(&url).await?;

        let results: Vec<SearchResult> = serde_json::from_str(&response)?;
        let videos: Vec<Video> = results
            .into_iter()
            .filter_map(|r| r.as_video().cloned())
            .collect();

        Ok(videos)
    }

    /// Get playlist details by playlist ID
    pub async fn get_playlist(&self, playlist_id: &str) -> Result<PlaylistDetails, InvidiousError> {
        let url = self.api_url(&format!("/playlists/{}", playlist_id));
        let response = self.get(&url).await?;

        let playlist: PlaylistDetails = serde_json::from_str(&response)?;
        Ok(playlist)
    }

    pub async fn get_stream_url(&self, video_id: &str) -> Result<String, InvidiousError> {
        let url = self.api_url(&format!("/videos/{}/streams", video_id));
        let response = self.get(&url).await?;
        let streams: InvidiousStreams = serde_json::from_str(&response)?;
        Ok(streams.url)
    }
}

// ============================================================================
// URL Encoding Helper (simple implementation)
// ============================================================================

mod urlencoding {
    pub fn encode(input: &str) -> String {
        let mut encoded = String::new();
        for byte in input.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(byte as char);
                }
                _ => {
                    encoded.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        encoded
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encoding() {
        assert_eq!(urlencoding::encode("hello world"), "hello%20world");
        assert_eq!(urlencoding::encode("test?query=1"), "test%3Fquery%3D1");
    }

    #[test]
    fn test_client_creation() {
        let client = InvidiousClient::new("https://invidious.snopyta.org");
        assert_eq!(client.base_url, "https://invidious.snopyta.org");
    }

    #[test]
    fn test_api_url_building() {
        let client = InvidiousClient::new("https://invidious.snopyta.org");
        assert_eq!(
            client.api_url("/search"),
            "https://invidious.snopyta.org/api/v1/search"
        );
        assert_eq!(
            client.api_url("search"),
            "https://invidious.snopyta.org/api/v1/search"
        );
    }
}
