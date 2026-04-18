use std::process::Stdio;
use tracing::{debug, error, info, warn};
use tokio::process::Child;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{Mutex, broadcast};
use crate::player::path;

#[derive(Debug, Clone)]
pub enum PlaybackNotification {
    FallbackAttempt(String),
    Failure(String),
    Success(String),
}

#[derive(Debug)]
pub struct MpvPlayerInner {
    process: Mutex<Option<Child>>,
    playback_ended_tx: tokio::sync::mpsc::Sender<()>,
    notification_tx: broadcast::Sender<PlaybackNotification>,
}

#[derive(Debug, Clone)]
pub struct MpvPlayer {
    inner: std::sync::Arc<MpvPlayerInner>,
}

#[async_trait::async_trait]
impl crate::player::Player for MpvPlayer {
    async fn play(&self, url: &str, quality: &str, loop_playback: bool, extra_args: &[&str]) -> Result<(), String> {
        self.play(url, quality, loop_playback, extra_args).await
    }

    async fn play_audio(&self, url: &str, quality: &str, loop_playback: bool, extra_args: &[&str]) -> Result<(), String> {
        self.play_audio(url, quality, loop_playback, extra_args).await
    }

    async fn stop(&self) {
        self.stop().await
    }

    async fn is_playing(&self) -> bool {
        self.is_playing().await
    }
}

impl MpvPlayer {
    pub fn new(
        playback_ended_tx: tokio::sync::mpsc::Sender<()>,
        notification_tx: broadcast::Sender<PlaybackNotification>,
    ) -> Self {
        info!("Creating MpvPlayer");
        Self {
            inner: std::sync::Arc::new(MpvPlayerInner {
                process: Mutex::new(None),
                playback_ended_tx,
                notification_tx,
            }),
        }
    }

    pub async fn is_available() -> bool {
        info!("Detecting media player...");
        path::print_detection_report();
        
        if let Some(path) = path::get_player_path("mpv") {
            info!("Found mpv at: {:?}", path);
            let result = tokio::process::Command::new(&path)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            
            match result {
                Ok(_) => {
                    info!("mpv is ready");
                    true
                }
                Err(e) => {
                    warn!("mpv failed to start: {}", e);
                    false
                }
            }
        } else {
            error!("mpv not found in any location!");
            info!("Trying to use best available player...");
            if let Some(player) = path::get_best_player() {
                info!("Best available player: {} at {:?}", player.name, player.path);
            }
            false
        }
    }

    pub async fn version() -> Option<String> {
        let path = path::get_player_path("mpv")?;
        let output = tokio::process::Command::new(path)
            .arg("--version")
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            version.lines().next().map(|s| s.to_string())
        } else {
            None
        }
    }

    pub async fn play(&self, url: &str, quality: &str, loop_playback: bool, extra_args: &[&str]) -> Result<(), String> {
        if url.is_empty() {
            return Err("Invalid or empty URL passed to play function".to_string());
        }
        self.play_with_fallback(url.to_string(), quality, loop_playback, extra_args.iter().map(|s| s.to_string()).collect()).await
    }

    pub async fn play_audio(&self, url: &str, quality: &str, loop_playback: bool, extra_args: &[&str]) -> Result<(), String> {
        let args: Vec<String> = extra_args.iter().copied().chain(std::iter::once("--video=no")).map(|s| s.to_string()).collect();
        if url.is_empty() {
            return Err("Invalid or empty URL passed to play_audio function".to_string());
        }
        info!("play_audio function received URL: {}", url);
        self.play_with_fallback(url.to_string(), quality, loop_playback, args).await
    }

    pub async fn play_with_fallback(
        &self,
        url: String,
        quality: &str,
        loop_playback: bool,
        extra_args: Vec<String>,
    ) -> Result<(), String> {
        let args_refs: Vec<&str> = extra_args.iter().map(|s| s.as_str()).collect();
        self.run_mpv(&url, quality, loop_playback, &args_refs).await?;

        let inner = self.inner.clone();
        let url_clone = url.clone();
        let args_clone = extra_args.clone();
        let quality_clone = quality.to_string();

        tokio::spawn(async move {
            info!("Starting fallback manager for URL: {}", url_clone);
            let mut rx = inner.notification_tx.subscribe();
            let mut provider_index = 0;
            let start = std::time::Instant::now();
            
            let providers: Vec<Box<dyn crate::api::providers::StreamProvider>> = vec![
                Box::new(crate::api::providers::YtdlpProvider),
                Box::new(crate::api::providers::InvidiousProvider { 
                    client: crate::api::invidious::InvidiousClient::new("https://invidious.snopyta.org") 
                }),
                Box::new(crate::api::providers::PipedProvider { 
                    client: crate::api::piped::PipedClient::new("https://pipedapi.kavin.rocks") 
                }),
            ];

            info!("Waiting for failure notification...");
            
            while let Ok(notification) = rx.recv().await {
                if let PlaybackNotification::Failure(err) = notification {
                    warn!("Playback failed: {}", err);

                    if provider_index >= providers.len() {
                        error!("All {} providers exhausted", providers.len());
                        break;
                    }

                    let provider = &providers[provider_index];
                    let provider_name = match provider_index {
                        0 => "Ytdlp",
                        1 => "Invidious",
                        2 => "Piped",
                        _ => "Unknown",
                    };

                    info!("Trying provider {} (#{})", provider_name, provider_index + 1);
                    let _ = inner.notification_tx.send(PlaybackNotification::FallbackAttempt(provider_name.to_string()));

                    if let Some(video_id) = extract_video_id(&url_clone) {
                        info!("Extracted video ID: {}", video_id);
                        match provider.get_stream_url(&video_id).await {
                            Ok(stream_url) => {
                                info!("Got stream URL from {}: {}", provider_name, stream_url);
                                let args_refs: Vec<&str> = args_clone.iter().map(|s| s.as_str()).collect();
                                let player = MpvPlayer { inner: inner.clone() };
                                if let Err(e) = player.run_mpv(&stream_url, &quality_clone, loop_playback, &args_refs).await {
                                    error!("Fallback failed: {}", e);
                                } else {
                                    info!("SUCCESS - playing with stream URL");
                                    let _ = inner.notification_tx.send(PlaybackNotification::Success(stream_url));
                                    break;
                                }
                            }
                            Err(e) => {
                                warn!("Provider {} failed: {}", provider_name, e);
                            }
                        }
                    } else {
                        warn!("Could not extract video ID from: {}", url_clone);
                    }

                    provider_index += 1;
                } else if let PlaybackNotification::FallbackAttempt(name) = notification {
                    info!("Received fallback attempt: {}", name);
                } else if let PlaybackNotification::Success(url) = notification {
                    info!("Received success: {}", url);
                    break;
                }
            }
            
            info!("Fallback manager finished after {:?}", start.elapsed());
        });

        Ok(())
    }

    async fn run_mpv(
        &self,
        url: &str,
        quality: &str,
        loop_playback: bool,
        extra_args: &[&str]) -> Result<(), String> {
        let path = path::get_player_path("mpv")
            .ok_or_else(|| "mpv binary not found. Run with --detect-players to see available players.".to_string())?;

        if !Self::is_available().await {
            return Err("mpv is not installed or not responding".to_string());
        }

        self.stop().await;

        let mut cmd = tokio::process::Command::new(&path);

        // Exit immediately when video ends (don't wait for user input)
        // Use --keep-open=no instead of --quit-after-end for better compatibility
        cmd.arg("--keep-open=no");
        
        if loop_playback {
            cmd.arg("--loop");
        }

        let quality_arg = format!("bestvideo[height<={}]+bestaudio/best[height<={}]", quality, quality);
        cmd.arg(format!("--ytdl-format={}", quality_arg));

        for arg in extra_args {
            cmd.arg(arg);
        }

        if url.is_empty() {
            return Err("Invalid or empty URL specified for playback".to_string());
        }

        info!("Playing URL: {} with {:?}", url, path);
        cmd.arg(url);
        
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start mpv at {:?}: {}. Is mpv installed?", path, e))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        if let Some(stdout) = stdout {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    debug!("[mpv stdout] {}", line);
                }
            });
        }

        if let Some(stderr) = stderr {
            let inner = self.inner.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    debug!("[mpv stderr] {}", line);
                    if line.contains("Failed to open") || line.contains("[ytdl_hook] ERROR") || line.contains("ERROR") {
                        warn!("Detected failure in mpv stderr");
                        let _ = inner.notification_tx.send(PlaybackNotification::Failure(line));
                    }
                }
            });
        }

        let mut process = self.inner.process.lock().await;
        *process = Some(child);
        drop(process);

        let inner = self.inner.clone();
        let tx = self.inner.playback_ended_tx.clone();
        tokio::spawn(async move {
            loop {
                {
                    let mut process = inner.process.lock().await;
                    if let Some(child) = process.as_mut() {
                        match child.try_wait() {
                            Ok(Some(status)) => {
                            // Exit code 2 is normal for mpv when playback completes (e.g., end of playlist)
                            // Treat it the same as other exit codes - send playback_ended signal
                            *process = None;
                            if status.code() == Some(2) {
                                debug!("mpv exited with code 2 (playback completed normally)");
                            }
                            let _ = tx.send(()).await;
                            break;
                        }
                            Ok(None) => {}
                            Err(e) => {
                                debug!("Error waiting for mpv: {}", e);
                                *process = None;
                                let _ = tx.send(()).await;
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        });

        Ok(())
    }

    pub async fn stop(&self) {
        let mut process = self.inner.process.lock().await;

        if let Some(mut child) = process.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
    }

    pub async fn is_playing(&self) -> bool {
        let process = self.inner.process.lock().await;

        if let Some(child) = process.as_ref() {
            child.id().is_some()
        } else {
            false
        }
    }
}

fn extract_video_id(url: &str) -> Option<String> {
    if url.contains("v=") {
        let parts: Vec<&str> = url.split("v=").collect();
        if let Some(id) = parts.get(1) {
            let id = if let Some(pos) = id.find('&') {
                &id[..pos]
            } else {
                id
            };
            return Some(id.to_string());
        }
    }
    if url.contains("/video/") {
        let parts: Vec<&str> = url.split("/video/").collect();
        if let Some(id) = parts.get(1) {
            return Some(id.to_string());
        }
    }
    None
}

