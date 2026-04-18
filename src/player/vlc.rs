use std::process::Stdio;
use tracing::{debug, error, info, warn};
use tokio::process::Child;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{Mutex, broadcast};
use crate::player::path;
use crate::player::Player;

#[derive(Debug)]
pub struct VlcPlayerInner {
    process: Mutex<Option<Child>>,
    playback_ended_tx: tokio::sync::mpsc::Sender<()>,
    notification_tx: broadcast::Sender<crate::player::mpv::PlaybackNotification>,
}

#[derive(Debug, Clone)]
pub struct VlcPlayer {
    inner: std::sync::Arc<VlcPlayerInner>,
}

#[async_trait::async_trait]
impl crate::player::Player for VlcPlayer {
    async fn play(&self, url: &str, _quality: &str, _format: &str, _loop_playback: bool, _extra_args: &[&str]) -> Result<(), String> {
        self.play_vlc(url).await
    }

    async fn play_audio(&self, url: &str, _quality: &str, _loop_playback: bool, _extra_args: &[&str]) -> Result<(), String> {
        self.play_vlc(url).await
    }

    async fn stop(&self) {
        let mut process_lock = self.inner.process.lock().await;
        if let Some(mut child) = process_lock.take() {
            let _ = child.kill().await;
        }
    }

    async fn is_playing(&self) -> bool {
        let process_lock = self.inner.process.lock().await;
        process_lock.is_some()
    }
}

impl VlcPlayer {
    pub fn new(
        playback_ended_tx: tokio::sync::mpsc::Sender<()>,
        notification_tx: broadcast::Sender<crate::player::mpv::PlaybackNotification>,
    ) -> Self {
        info!("Creating VlcPlayer");
        Self {
            inner: std::sync::Arc::new(VlcPlayerInner {
                process: Mutex::new(None),
                playback_ended_tx,
                notification_tx,
            }),
        }
    }

    pub async fn is_available() -> bool {
        info!("Detecting vlc...");
        if let Some(path) = path::get_player_path("vlc") {
            info!("Found vlc at: {:?}", path);
            let result = tokio::process::Command::new(&path)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            
            match result {
                Ok(_) => {
                    info!("vlc is ready");
                    true
                }
                Err(e) => {
                    warn!("vlc failed to start: {}", e);
                    false
                }
            }
        } else {
            error!("vlc not found in any location!");
            false
        }
    }

    async fn play_vlc(&self, url: &str) -> Result<(), String> {
        let path = path::get_player_path("vlc")
            .ok_or_else(|| "vlc binary not found".to_string())?;

        self.stop().await;

        let resolved_url = if url.contains("youtube.com") || url.contains("youtu.be") {
            let output = tokio::process::Command::new("yt-dlp")
                .arg("-g")
                .arg("-f")
                .arg("best")
                .arg(url)
                .output()
                .await;
            
            match output {
                Ok(o) if o.status.success() => {
                    String::from_utf8_lossy(&o.stdout)
                        .lines()
                        .next()
                        .unwrap_or(url)
                        .to_string()
                }
                _ => url.to_string(),
            }
        } else {
            url.to_string()
        };

        let mut cmd = tokio::process::Command::new(&path);
        cmd.arg("--play-and-exit");
        cmd.arg("--no-video-title-show");
        cmd.arg(&resolved_url);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        info!("Playing URL: {} with {:?}", resolved_url, path);
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start vlc: {}. Is vlc installed?", e))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let playback_ended_tx = self.inner.playback_ended_tx.clone();
        let notification_tx = self.inner.notification_tx.clone();

        tokio::spawn(async move {
            if let Some(stdout) = stdout {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(_line)) = reader.next_line().await {}
            }
        });

        tokio::spawn(async move {
            if let Some(stderr) = stderr {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    debug!("[vlc stderr] {}", line);
                    if line.contains("ERROR") || line.contains("failed") {
                        let _ = notification_tx.send(crate::player::mpv::PlaybackNotification::Failure(line));
                    }
                }
            }
        });

        let inner = self.inner.clone();
        let tx = playback_ended_tx.clone();
        tokio::spawn(async move {
            loop {
                if let Some(ref mut child) = *inner.process.lock().await {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            info!("vlc exited with code {:?} - playback ended", status.code());
                            let _ = tx.send(()).await;
                            break;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            debug!("Error waiting for vlc: {}", e);
                            let _ = tx.send(()).await;
                            break;
                        }
                    }
                } else {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        });

        let mut process_lock = self.inner.process.lock().await;
        *process_lock = Some(child);

        Ok(())
    }
}
