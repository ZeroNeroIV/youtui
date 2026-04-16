use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use crate::player::detector;

static PATH_CACHE: LazyLock<Mutex<HashMap<String, PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn find_player_binary(name: &str) -> Option<PathBuf> {
    if let Some(cached) = get_cached(name) {
        return Some(cached);
    }

    // Use the comprehensive detector
    let players = detector::detect_players();

    for player in players {
        if player.name == name && player.working {
            let path = player.path;
            cache_player(name, path.clone());
            return Some(path);
        }
    }

    // Also search common locations directly
    if let Some(path) = search_common_locations(name) {
        cache_player(name, path.clone());
        return Some(path);
    }

    None
}

fn get_cached(name: &str) -> Option<PathBuf> {
    let cache = PATH_CACHE.lock().ok()?;
    cache.get(name).cloned()
}

fn cache_player(name: &str, path: PathBuf) {
    if let Ok(mut cache) = PATH_CACHE.lock() {
        cache.insert(name.to_string(), path);
    }
}

fn search_common_locations(name: &str) -> Option<PathBuf> {
    // Check PATH
    if let Some(path_var) = env::var_os("PATH") {
        for dir in env::split_paths(&path_var) {
            let full_path = dir.join(name);
            if full_path.exists() && is_executable(&full_path) {
                return Some(full_path);
            }
        }
    }

    // Check ~/.local/bin
    if let Some(home) = dirs::home_dir() {
        let local_bin = home.join(".local/bin").join(name);
        if local_bin.exists() && is_executable(&local_bin) {
            return Some(local_bin);
        }
    }

    // Check system locations
    let system_paths = ["/usr/bin", "/usr/local/bin", "/snap/bin", "/opt/bin"];
    for base in &system_paths {
        let full_path = Path::new(base).join(name);
        if full_path.exists() && is_executable(&full_path) {
            return Some(full_path);
        }
    }

    // Try which command
    if let Ok(output) = std::process::Command::new("which").arg(name).output() {
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() {
            let path = PathBuf::from(&path_str);
            if path.exists() && is_executable(&path) {
                return Some(path);
            }
        }
    }

    None
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = path.metadata() {
        let mode = metadata.permissions().mode();
        (mode & 0o111) != 0
    } else {
        false
    }
}

pub fn get_player_path(name: &str) -> Option<PathBuf> {
    find_player_binary(name)
}

pub fn get_best_player() -> Option<detector::PlayerInfo> {
    detector::get_best_player()
}

pub fn print_detection_report() {
    detector::print_detection_report();
}
