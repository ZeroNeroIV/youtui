pub mod detector;
pub mod ipc;
pub mod mpv;
pub mod vlc;
pub mod path;

use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct VideoQueue {
    urls: Vec<String>,
}

impl VideoQueue {
    pub fn push(&mut self, url: String) {
        self.urls.push(url);
    }

    pub fn pop(&mut self) -> Option<String> {
        if self.urls.is_empty() {
            None
        } else {
            Some(self.urls.remove(0))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.urls.is_empty()
    }

    pub fn len(&self) -> usize {
        self.urls.len()
    }

    pub fn clear(&mut self) {
        self.urls.clear();
    }
}

#[async_trait::async_trait]
pub trait Player: Send + Sync {
    async fn play(&self, url: &str, quality: &str, format: &str, loop_playback: bool, extra_args: &[&str]) -> Result<(), String>;
    async fn play_audio(&self, url: &str, quality: &str, loop_playback: bool, extra_args: &[&str]) -> Result<(), String>;
    async fn stop(&self);
    async fn is_playing(&self) -> bool;
    async fn queue_video(&self, url: &str) -> Result<(), String>;
}

pub fn create_player(
    name: &str,
    playback_ended_tx: tokio::sync::mpsc::Sender<()>,
    notification_tx: tokio::sync::broadcast::Sender<mpv::PlaybackNotification>,
) -> Option<Arc<dyn Player>> {
    match name.to_lowercase().as_str() {
        "mpv" => Some(Arc::new(mpv::MpvPlayer::new(playback_ended_tx, notification_tx)) as Arc<dyn Player>),
        "vlc" => Some(Arc::new(vlc::VlcPlayer::new(playback_ended_tx, notification_tx)) as Arc<dyn Player>),
        _ => None,
    }
}