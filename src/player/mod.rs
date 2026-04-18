pub mod detector;
pub mod mpv;
pub mod vlc;
pub mod path;

use std::sync::Arc;

#[async_trait::async_trait]
pub trait Player: Send + Sync {
    async fn play(&self, url: &str, quality: &str, loop_playback: bool, extra_args: &[&str]) -> Result<(), String>;
    async fn play_audio(&self, url: &str, quality: &str, loop_playback: bool, extra_args: &[&str]) -> Result<(), String>;
    async fn stop(&self);
    async fn is_playing(&self) -> bool;
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