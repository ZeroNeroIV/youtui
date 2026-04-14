use serde::{Deserialize, Serialize};
use std::process::Command;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum YtdlpError {
    #[error("yt-dlp not found in PATH")]
    NotInstalled,
    #[error("Failed to execute yt-dlp: {0}")]
    ExecutionFailed(String),
    #[error("Failed to parse output: {0}")]
    ParseError(String),
    #[error("Video not found: {0}")]
    NotFound(String),
    #[error("No results found")]
    NoResults,
}

impl serde::Serialize for YtdlpError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// ============================================================================
// Data Models
// ============================================================================

/// Simplified video info from yt-dlp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtdlpVideo {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default, alias = "duration_string")]
    pub duration: Option<String>,
    #[serde(default)]
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub view_count: Option<i64>,
    #[serde(default)]
    pub upload_date: Option<String>,
}

/// Format info from yt-dlp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtdlpFormat {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    #[serde(default)]
    pub filesize: Option<i64>,
    #[serde(default)]
    pub fps: Option<f64>,
    #[serde(default)]
    pub vcodec: Option<String>,
    #[serde(default)]
    pub acodec: Option<String>,
}

/// Full video info with formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtdlpVideoInfo {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub view_count: Option<i64>,
    #[serde(default)]
    pub upload_date: Option<String>,
    #[serde(default)]
    pub formats: Vec<YtdlpFormat>,
}

/// Search results container
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YtdlpSearchResults {
    pub videos: Vec<YtdlpVideo>,
    pub next_page_token: Option<String>,
}

// ============================================================================
// YtdlpWrapper Implementation
// ============================================================================

pub struct YtdlpWrapper;

impl YtdlpWrapper {
    /// Check if yt-dlp is installed and available
    pub fn is_available() -> bool {
        std::process::Command::new("yt-dlp")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Run yt-dlp command and return output
    fn run_command(args: &[&str]) -> Result<String, YtdlpError> {
        if !Self::is_available() {
            return Err(YtdlpError::NotInstalled);
        }

        let output = Command::new("yt-dlp")
            .args(args)
            .output()
            .map_err(|e| YtdlpError::ExecutionFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Video not found") || stderr.contains("ID not found") {
                return Err(YtdlpError::NotFound("Video not found".to_string()));
            }
            return Err(YtdlpError::ExecutionFailed(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// Search for videos using yt-dlp
    /// Uses "ytsearchN:query" format to get search results
    pub fn search(query: &str, max_results: usize) -> Result<Vec<YtdlpVideo>, YtdlpError> {
        // Use --print to output JSON for each video result
        // The format specifier extracts key fields
        let args = [
            &format!("ytsearch{}:{}", max_results, query),
            "--print",
            "%(id)s|%(title)s|%(channel)s|%(duration)s|%(thumbnail)s|%(description)s",
            "--no-warnings",
            "--no-progress",
        ];

        let output = Self::run_command(&args)?;

        let videos: Vec<YtdlpVideo> = output
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 6 {
                    Some(YtdlpVideo {
                        id: parts[0].to_string(),
                        title: parts[1].to_string(),
                        channel: Some(parts[2].to_string()).filter(|s| !s.is_empty()),
                        duration: Some(parts[3].to_string()).filter(|s| !s.is_empty()),
                        thumbnail: Some(parts[4].to_string()).filter(|s| !s.is_empty()),
                        description: Some(parts[5].to_string()).filter(|s| !s.is_empty()),
                        view_count: None,
                        upload_date: None,
                    })
                } else {
                    None
                }
            })
            .collect();

        if videos.is_empty() {
            return Err(YtdlpError::NoResults);
        }

        Ok(videos)
    }

    /// Get video information using --dump-json
    pub fn get_video_info(video_id: &str) -> Result<YtdlpVideoInfo, YtdlpError> {
        let args = [
            &format!("https://youtube.com/watch?v={}", video_id),
            "--dump-json",
            "--no-warnings",
            "--no-progress",
        ];

        let output = Self::run_command(&args)?;

        let info: YtdlpVideoInfo =
            serde_json::from_str(&output).map_err(|e| YtdlpError::ParseError(e.to_string()))?;

        Ok(info)
    }

    /// Get available formats for a video
    /// Uses --print to get format information
    pub fn get_formats(video_id: &str) -> Result<Vec<YtdlpFormat>, YtdlpError> {
        // Use --print to output format info in a parseable way
        // Format: format_id|extension|resolution|filesize|fps|vcodec|acodec
        let args = [
            &format!("https://youtube.com/watch?v={}", video_id),
            "--print",
            "%(format_id)s|%(ext)s|%(resolution)s|%(filesize)s|%(fps)s|%(vcodec)s|%(acodec)s",
            "--no-warnings",
            "--no-progress",
            "--list-formats",
        ];

        let output = Self::run_command(&args)?;

        let formats: Vec<YtdlpFormat> = output
            .lines()
            .skip(1) // Skip header line
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 7 {
                    Some(YtdlpFormat {
                        format_id: parts[0].to_string(),
                        ext: parts[1].to_string(),
                        resolution: Some(parts[2].to_string())
                            .filter(|s| !s.is_empty() && s != "unknown"),
                        filesize: parts[3].parse().ok(),
                        fps: parts[4].parse().ok(),
                        vcodec: Some(parts[5].to_string()).filter(|s| !s.is_empty() && s != "none"),
                        acodec: Some(parts[6].to_string()).filter(|s| !s.is_empty() && s != "none"),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(formats)
    }

    /// Get the best available stream URL for a video
    /// Used for playback fallback
    pub fn get_stream_url(video_id: &str) -> Result<String, YtdlpError> {
        Self::get_stream_url_with_quality(video_id, "best")
    }

    /// Get stream URL with specific quality
    pub fn get_stream_url_with_quality(
        video_id: &str,
        quality: &str,
    ) -> Result<String, YtdlpError> {
        let quality_format = match quality {
            "1080p" => "bestvideo[height<=1080]+bestaudio/best[height<=1080]",
            "720p" => "bestvideo[height<=720]+bestaudio/best[height<=720]",
            "480p" => "bestvideo[height<=480]+bestaudio/best[height<=480]",
            "worst" => "worst+worst",
            "bestvideo+bestaudio" => "bestvideo+bestaudio/best",
            _ => "best",
        };

        let args = [
            &format!("https://youtube.com/watch?v={}", video_id),
            "-f",
            quality_format,
            "-g",
            "--no-warnings",
        ];

        let output = Self::run_command(&args)?;
        let url = output.trim().to_string();

        if url.is_empty() {
            return Err(YtdlpError::NotFound("No stream URL found".to_string()));
        }

        Ok(url)
    }

    /// Download video to specified path
    pub fn download(
        video_id: &str,
        output_path: &str,
        format_spec: Option<&str>,
    ) -> Result<(), YtdlpError> {
        let format = format_spec.unwrap_or("bestvideo+bestaudio/best");

        if !Self::is_available() {
            return Err(YtdlpError::NotInstalled);
        }

        let mut cmd = Command::new("yt-dlp");
        cmd.arg(format!("https://youtube.com/watch?v={}", video_id))
            .arg("-f")
            .arg(format)
            .arg("-o")
            .arg(output_path)
            .arg("--no-warnings");

        let output = cmd
            .output()
            .map_err(|e| YtdlpError::ExecutionFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(YtdlpError::ExecutionFailed(stderr.to_string()));
        }

        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_available() {
        // This will fail if yt-dlp is not installed
        // But the function handles that gracefully
        let _ = YtdlpWrapper::is_available();
    }

    #[test]
    fn test_search_with_empty_query() {
        // Should return error for empty query
        let result = YtdlpWrapper::search("", 5);
        assert!(result.is_err() || result.unwrap().is_empty());
    }
}

// ============================================================================
// Playlist Support (for Mix and regular playlists)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtdlpPlaylistVideo {
    pub id: String,
    pub title: String,
    pub channel: Option<String>,
    pub duration: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum YtdlpPlaylistError {
    #[error("yt-dlp not found in PATH")]
    NotInstalled,
    #[error("Failed to execute yt-dlp: {0}")]
    ExecutionFailed(String),
    #[error("Failed to parse output: {0}")]
    ParseError(String),
}

impl serde::Serialize for YtdlpPlaylistError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub struct YtdlpPlaylist;

impl YtdlpPlaylist {
    pub fn get_playlist(
        url: &str,
        max_videos: usize,
    ) -> Result<Vec<YtdlpPlaylistVideo>, YtdlpPlaylistError> {
        if !YtdlpWrapper::is_available() {
            return Err(YtdlpPlaylistError::NotInstalled);
        }

        let output = Command::new("yt-dlp")
            .arg(url)
            .arg("--flat-playlist")
            .arg("--print=%(id)s|%(title)s|%(channel)s|%(duration)s")
            .arg("--no-warnings")
            .output()
            .map_err(|e| YtdlpPlaylistError::ExecutionFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(YtdlpPlaylistError::ExecutionFailed(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut videos = Vec::new();

        for line in stdout.lines() {
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() >= 2 {
                let video = YtdlpPlaylistVideo {
                    id: parts[0].to_string(),
                    title: parts[1].to_string(),
                    channel: parts
                        .get(2)
                        .map(|s| s.to_string())
                        .filter(|s| !s.is_empty()),
                    duration: parts
                        .get(3)
                        .map(|s| s.to_string())
                        .filter(|s| !s.is_empty()),
                };
                videos.push(video);
            }
            if videos.len() >= max_videos {
                break;
            }
        }

        Ok(videos)
    }
}
