use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

static PATH_CACHE: LazyLock<Mutex<HashMap<String, PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Searches for a binary named `name` in the system PATH and common locations.
pub fn find_player_binary(name: &str) -> Option<PathBuf> {
    // 1. Check PATH environment variable
    if let Some(path_var) = env::var_os("PATH") {
        for path in env::split_paths(&path_var) {
            let full_path = path.join(name);
            if full_path.exists() && full_path.is_file() {
                return Some(full_path);
            }
        }
    }

    // 2. Common system locations (Unix)
    #[cfg(unix)]
    {
        let common_paths = ["/usr/bin", "/usr/local/bin"];
        for path in &common_paths {
            let full_path = Path::new(path).join(name);
            if full_path.exists() && full_path.is_file() {
                return Some(full_path);
            }
        }

        // ~/.local/bin
        if let Some(home) = dirs::home_dir() {
            let local_bin = home.join(".local/bin").join(name);
            if local_bin.exists() && local_bin.is_file() {
                return Some(local_bin);
            }
        }
    }

    None
}

/// Returns the path to the player binary, using a cache to avoid repeated searches.
pub fn get_player_path(name: &str) -> Option<PathBuf> {
    let mut cache = PATH_CACHE.lock().ok()?;
    if let Some(path) = cache.get(name) {
        return Some(path.clone());
    }

    if let Some(path) = find_player_binary(name) {
        cache.insert(name.to_string(), path.clone());
        Some(path)
    } else {
        None
    }
}
