use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{LazyLock, Mutex};
use tracing::debug;

const MEDIA_PLAYERS: &[&str] = &["mpv", "vlc", "smplayer", "gmplayer", "kitty"];
const EXTENDED_PATHS: &[&str] = &[
    "/usr/bin",
    "/usr/local/bin",
    "/snap/bin",
    "/opt/bin",
    "/sw/bin",
    ".local/bin",
];

static PLAYER_CACHE: LazyLock<Mutex<HashMap<String, PlayerInfo>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub path: PathBuf,
    pub name: String,
    pub version: Option<String>,
    pub working: bool,
}

pub fn detect_players() -> Vec<PlayerInfo> {
    let mut results = Vec::new();
    for name in MEDIA_PLAYERS {
        if let Some(info) = detect_player(name) {
            if info.working {
                results.push(info);
            }
        }
    }
    results
}

fn detect_player(name: &str) -> Option<PlayerInfo> {
    let mut cache = PLAYER_CACHE.lock().ok()?;
    if let Some(cached) = cache.get(name) {
        return Some(cached.clone());
    }
    let path = find_player_binary(name)?;
    let (version, working) = test_player(&path);
    let info = PlayerInfo {
        path: path.clone(),
        name: name.to_string(),
        version,
        working,
    };
    cache.insert(name.to_string(), info.clone());
    Some(info)
}

fn find_player_binary(name: &str) -> Option<PathBuf> {
    if let Some(path_var) = env::var_os("PATH") {
        for dir in env::split_paths(&path_var) {
            let full_path = dir.join(name);
            if full_path.exists() && is_executable(&full_path) {
                return Some(full_path);
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let local_bin = home.join(".local/bin").join(name);
        if local_bin.exists() && is_executable(&local_bin) {
            return Some(local_bin);
        }
    }
    for base_path in EXTENDED_PATHS {
        let full_path = Path::new(base_path).join(name);
        if full_path.exists() && is_executable(&full_path) {
            return Some(full_path);
        }
    }
    if let Ok(output) = std::process::Command::new("which").arg(name).output() {
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() {
            let path_buf = PathBuf::from(path_str);
            if path_buf.exists() && is_executable(&path_buf) {
                return Some(path_buf);
            }
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if !path.exists() {
        return false;
    }
    if let Ok(m) = path.metadata() {
        return (m.permissions().mode() & 0o111) != 0;
    }
    false
}

fn test_player(path: &PathBuf) -> (Option<String>, bool) {
    let output = std::process::Command::new(path)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok();
    match output {
        Some(o) if o.status.success() => {
            let v = String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .unwrap_or("?")
                .to_string();
            (Some(v), true)
        }
        _ => (None, false),
    }
}

pub fn get_best_player() -> Option<PlayerInfo> {
    detect_players()
        .into_iter()
        .find(|p| p.name == "mpv" && p.working)
}

pub fn print_detection_report() {
    let players = detect_players();
    if players.is_empty() {
        return;
    }
    for player in &players {
        debug!("Detected {} at {:?}", player.name, player.path);
    }
}

pub fn clear_cache() {
    if let Ok(mut cache) = PLAYER_CACHE.lock() {
        cache.clear();
    }
}
