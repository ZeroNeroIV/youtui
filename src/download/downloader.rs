use crate::api::ytdlp::YtdlpWrapper;
use std::path::Path;
use tokio::sync::mpsc;

pub struct Downloader;

#[derive(Debug, Clone)]
pub enum DownloadProgress {
    Starting { title: String },
    Downloading { title: String, percent: u8, speed: String, eta: String },
    Completed { title: String, path: String },
    Failed { error: String },
}

impl Downloader {
    pub fn download(
        video_id: &str,
        output_dir: &Path,
        audio_only: bool,
    ) -> mpsc::Receiver<DownloadProgress> {
        let (tx, rx) = mpsc::channel(64);

        let video_id = video_id.to_string();
        // yt-dlp needs a template, not a bare directory
        let output_template = output_dir
            .join("%(title)s.%(ext)s")
            .to_string_lossy()
            .to_string();

        std::thread::spawn(move || {
            let _ = tx.blocking_send(DownloadProgress::Starting {
                title: video_id.clone(),
            });

            let mut cmd = std::process::Command::new("yt-dlp");
            cmd.arg(format!("https://youtube.com/watch?v={}", video_id))
                .arg("-o").arg(&output_template)
                .arg("--newline")      // one progress line per update
                .arg("--no-part");     // no .part files

            if audio_only {
                cmd.arg("-f").arg("bestaudio/best")
                    .arg("-x")
                    .arg("--audio-format").arg("mp3");
            } else {
                cmd.arg("-f").arg("bestvideo+bestaudio/best")
                    .arg("--merge-output-format").arg("mp4");
            }

            cmd.stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            match cmd.spawn() {
                Ok(mut child) => {
                    use std::io::{BufRead, BufReader};

                    // Spawn stderr reader thread so it doesn't fill/block
                    let stderr_lines: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
                        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
                    let stderr_lines_clone = stderr_lines.clone();

                    if let Some(stderr) = child.stderr.take() {
                        std::thread::spawn(move || {
                            let reader = BufReader::new(stderr);
                            for line in reader.lines().map_while(Result::ok) {
                                tracing::debug!("[yt-dlp stderr] {}", line);
                                stderr_lines_clone.lock().unwrap().push(line);
                            }
                        });
                    }

                    let mut current_title = video_id.clone();

                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines().map_while(Result::ok) {
                            // Extract destination filename
                            if line.starts_with("[download] Destination:") {
                                let path = line
                                    .trim_start_matches("[download] Destination:")
                                    .trim()
                                    .to_string();
                                // Use the stem of the filename as the title
                                if let Some(name) = std::path::Path::new(&path)
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                {
                                    current_title = name.to_string();
                                }
                                continue;
                            }

                            // Parse progress lines: "[download]  12.3% of 1.24MiB at 2.93MiB/s ETA 00:10"
                            if line.starts_with("[download]") && line.contains('%') {
                                let (percent, speed, eta) = parse_progress(&line);
                                let _ = tx.blocking_send(DownloadProgress::Downloading {
                                    title: current_title.clone(),
                                    percent,
                                    speed,
                                    eta,
                                });
                            }
                        }
                    }

                    match child.wait() {
                        Ok(status) if status.success() => {
                            let _ = tx.blocking_send(DownloadProgress::Completed {
                                title: current_title,
                                path: output_template,
                            });
                        }
                        Ok(status) => {
                            let stderr_msg = stderr_lines.lock().unwrap().join(" | ");
                            let err = if stderr_msg.is_empty() {
                                format!("yt-dlp exited with status {}", status)
                            } else {
                                stderr_msg
                            };
                            let _ = tx.blocking_send(DownloadProgress::Failed { error: err });
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
                        error: format!("Failed to start yt-dlp: {}", e),
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

fn parse_progress(line: &str) -> (u8, String, String) {
    let percent = line
        .find('%')
        .and_then(|idx| {
            line[..idx]
                .rsplit_once(' ')
                .and_then(|(_, num)| num.trim().parse::<f64>().ok())
        })
        .map(|p| p.clamp(0.0, 100.0) as u8)
        .unwrap_or(0);

    let speed = line
        .find(" at ")
        .map(|idx| {
            let after = &line[idx + 4..];
            after
                .find(" ETA")
                .map(|end| after[..end].trim().to_string())
                .unwrap_or_else(|| after.split_whitespace().next().unwrap_or("").to_string())
        })
        .unwrap_or_default();

    let eta = line
        .find("ETA ")
        .map(|idx| line[idx + 4..].split_whitespace().next().unwrap_or("").to_string())
        .unwrap_or_default();

    (percent, speed, eta)
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
