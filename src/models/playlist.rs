use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub videos: Vec<super::video::Video>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistVideo {
    pub playlist_id: i64,
    pub video_id: String,
    pub position: i32,
}
