use crate::api::ytdlp::YtdlpWrapper;
use std::path::Path;
use tokio::sync::mpsc;

pub struct Downloader;

#[derive(Debug, Clone)]
pub enum DownloadProgress {
    Starting,
    Downloading { percent: u8, speed: String },
    Completed { path: String },
    Failed { error: String },
}

impl Downloader {
    pub fn download(
        video_id: &str,
        output_path: &Path,
        format_spec: Option<&str>,
    ) -> mpsc::Receiver<DownloadProgress> {
        let (tx, rx) = mpsc::channel(32);

        let video_id = video_id.to_string();
        let output_path_str = output_path.to_string_lossy().to_string();
        let format = format_spec
            .unwrap_or("bestvideo+bestaudio/best")
            .to_string();

        std::thread::spawn(move || {
            let _ = tx.blocking_send(DownloadProgress::Starting);

            let mut cmd = std::process::Command::new("yt-dlp");
            cmd.arg(format!("https://youtube.com/watch?v={}", video_id))
                .arg("-f")
                .arg(&format)
                .arg("-o")
                .arg(&output_path_str)
                .arg("--no-warnings")
                .arg("--progress")
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            match cmd.spawn() {
                Ok(mut child) => {
                    use std::io::{BufRead, BufReader};

                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines().map_while(Result::ok) {
                            if line.contains("%") && line.contains("download") {
                                if let Some(start) = line.find('%') {
                                    let before = &line[..start];
                                    if let Some(percent_start) = before.rfind(' ') {
                                        if let Ok(percent) =
                                            before[percent_start + 1..].trim().parse::<f64>()
                                        {
                                            let pct = percent as u8;
                                            let speed = if let Some(speed_start) = line.find("at ")
                                            {
                                                let after_at = &line[speed_start + 3..];
                                                if let Some(speed_end) = after_at.find(" ETA") {
                                                    after_at[..speed_end].to_string()
                                                } else {
                                                    "calculating...".to_string()
                                                }
                                            } else {
                                                "...".to_string()
                                            };

                                            let _ =
                                                tx.blocking_send(DownloadProgress::Downloading {
                                                    percent: pct.min(100),
                                                    speed,
                                                });
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(stderr) = child.stderr.take() {
                        let reader = std::io::BufReader::new(stderr);
                        for line in reader.lines().map_while(Result::ok) {
                            tracing::debug!("[yt-dlp stderr] {}", line);
                        }
                    }

                    match child.wait() {
                        Ok(status) if status.success() => {
                            let _ = tx.blocking_send(DownloadProgress::Completed {
                                path: output_path_str,
                            });
                        }
                        Ok(_) => {
                            let _ = tx.blocking_send(DownloadProgress::Failed {
                                error: "Download failed".to_string(),
                            });
                        }
                        Err(e) => {
                            let _ = tx.blocking_send(DownloadProgress::Failed {
                                error: e.to_string(),
                            });
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.blocking_send(DownloadProgress::Failed {
                        error: e.to_string(),
                    });
                }
            }
        });

        rx
    }

    pub fn get_formats(video_id: &str) -> Result<Vec<FormatInfo>, String> {
        let formats = YtdlpWrapper::get_formats(video_id).map_err(|e| e.to_string())?;
        Ok(formats.into_iter().map(FormatInfo::from).collect())
    }
}

#[derive(Debug, Clone)]
pub struct FormatInfo {
    pub format_id: String,
    pub label: String,
    pub extension: String,
    pub resolution: Option<String>,
    pub filesize_mb: Option<f64>,
}

impl From<crate::api::ytdlp::YtdlpFormat> for FormatInfo {
    fn from(f: crate::api::ytdlp::YtdlpFormat) -> Self {
        let label = if let Some(ref res) = f.resolution {
            format!("{} ({})", res, f.ext)
        } else {
            f.ext.clone()
        };

        Self {
            format_id: f.format_id,
            label,
            extension: f.ext,
            resolution: f.resolution,
            filesize_mb: f.filesize.map(|b| b as f64 / 1_048_576.0),
        }
    }
}
