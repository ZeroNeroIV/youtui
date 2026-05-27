use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum YtdlpError {
    #[error("yt-dlp not found in PATH")]
    NotInstalled,
    #[error("Video not found: {0}")]
    NotFound(String),
    #[error("yt-dlp error: {0}")]
    Process(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtdlpFormat {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub filesize: Option<u64>,
    pub url: Option<String>,
}

pub struct YtdlpWrapper;

impl YtdlpWrapper {
    pub fn get_stream_url(video_id: &str) -> Result<String, YtdlpError> {
        let output = Command::new("yt-dlp")
            .args([
                "--get-url",
                "-f", "bestvideo+bestaudio/best",
                &format!("https://youtube.com/watch?v={}", video_id),
            ])
            .output()
            .map_err(|_| YtdlpError::NotInstalled)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if stderr.contains("Video unavailable") || stderr.contains("not available") {
                return Err(YtdlpError::NotFound(video_id.to_string()));
            }
            return Err(YtdlpError::Process(stderr));
        }

        let url = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();

        if url.is_empty() {
            return Err(YtdlpError::NotFound(video_id.to_string()));
        }

        Ok(url)
    }

    pub fn get_formats(video_id: &str) -> Result<Vec<YtdlpFormat>, YtdlpError> {
        let output = Command::new("yt-dlp")
            .args([
                "-J",
                "--no-warnings",
                &format!("https://youtube.com/watch?v={}", video_id),
            ])
            .output()
            .map_err(|_| YtdlpError::NotInstalled)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(YtdlpError::Process(stderr));
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| YtdlpError::Parse(e.to_string()))?;

        let formats = json["formats"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|f| {
                let format_id = f["format_id"].as_str()?.to_string();
                let ext = f["ext"].as_str().unwrap_or("unknown").to_string();
                let resolution = f["resolution"].as_str().map(|s| s.to_string());
                let filesize = f["filesize"].as_u64();
                let url = f["url"].as_str().map(|s| s.to_string());
                Some(YtdlpFormat { format_id, ext, resolution, filesize, url })
            })
            .collect();

        Ok(formats)
    }
}
