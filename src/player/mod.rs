pub mod mpv;
pub mod path;

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

#[async_trait::async_trait]
pub trait Player: Send + Sync {
    async fn play(
        &self,
        url: &str,
        quality: &str,
        format: &str,
        loop_playback: bool,
        extra_args: &[&str],
    ) -> Result<(), String>;

    async fn play_audio(
        &self,
        url: &str,
        quality: &str,
        loop_playback: bool,
        extra_args: &[&str],
    ) -> Result<(), String>;

    async fn stop(&self);

    async fn is_playing(&self) -> bool;

    async fn queue_video(&self, url: &str) -> Result<(), String>;

    async fn toggle_pause(&self) {}

    async fn seek(&self, _secs: i64) {}

    async fn set_volume(&self, _delta: i64) {}
}

pub fn create_player(
    player_name: &str,
    playback_ended_tx: mpsc::Sender<()>,
    notification_tx: broadcast::Sender<mpv::PlaybackNotification>,
    invidious_url: Option<&str>,
    piped_url: Option<&str>,
) -> Option<Arc<dyn Player>> {
    match player_name.to_lowercase().as_str() {
        "mpv" | "" => {
            if path::get_player_path("mpv").is_some() {
                Some(Arc::new(mpv::MpvPlayer::new(
                    playback_ended_tx,
                    notification_tx,
                    invidious_url,
                    piped_url,
                )))
            } else {
                None
            }
        }
        _ => None,
    }
}
