use std::path::PathBuf;
use tracing::info;

pub struct PlayerInfo {
    pub name: String,
    pub path: PathBuf,
}

static PLAYER_SEARCH_PATHS: &[&str] = &[
    "/usr/bin",
    "/usr/local/bin",
    "/opt/local/bin",
    "/home/linuxbrew/.linuxbrew/bin",
];

pub fn get_player_path(player_name: &str) -> Option<PathBuf> {
    // Try PATH first
    if let Ok(output) = std::process::Command::new("which")
        .arg(player_name)
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }

    // Try common locations
    for dir in PLAYER_SEARCH_PATHS {
        let path = PathBuf::from(dir).join(player_name);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

pub fn get_best_player() -> Option<PlayerInfo> {
    for name in &["mpv", "vlc", "mplayer"] {
        if let Some(path) = get_player_path(name) {
            return Some(PlayerInfo {
                name: name.to_string(),
                path,
            });
        }
    }
    None
}

pub fn print_detection_report() {
    for player in &["mpv", "vlc", "mplayer", "yt-dlp"] {
        match get_player_path(player) {
            Some(p) => info!("  [FOUND] {} -> {:?}", player, p),
            None => info!("  [MISSING] {}", player),
        }
    }
}
