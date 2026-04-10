use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// Health check timeout in seconds
const HEALTH_CHECK_TIMEOUT_SECS: u64 = 5;

/// Invidious public instances (tested and working)
pub const INVIDIOUS_INSTANCES: &[&str] = &[
    "https://iv.melmac.space",
    "https://invidious.materialio.us",
    "https://invidious.snopyta.org",
    "https://invidious.kavin.rocks",
    "https://invidious.jingl.xyz",
    "https://invidious.namazso.eu",
    "https://invidious.projectsegfau.lt",
];

/// Piped public instances (tested and working)
pub const PIPED_INSTANCES: &[&str] = &[
    "https://pipedapi.kavin.rocks",
    "https://pipedapi-libre.kavin.rocks",
    "https://api.piped.yt",
    "https://pipedapi.moomoo.me",
    "https://api.piped.privacydev.net",
];

/// Atomic counter for instance rotation
static INVIDIOUS_INDEX: AtomicUsize = AtomicUsize::new(0);
static PIPED_INDEX: AtomicUsize = AtomicUsize::new(0);

/// Create a reqwest client for health checks with timeout
fn create_health_check_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(HEALTH_CHECK_TIMEOUT_SECS))
        .build()
        .unwrap_or_default()
}

/// Check if an Invidious instance is healthy
/// Returns true if the instance responds successfully
pub async fn check_invidious(url: &str) -> bool {
    let client = create_health_check_client();

    let test_url = format!("{}/api/v1/trending", url.trim_end_matches('/'));

    match client.get(&test_url).send().await {
        Ok(response) => {
            if let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE) {
                if let Ok(ct_str) = content_type.to_str() {
                    if ct_str.contains("text/html") {
                        return false;
                    }
                }
            }
            response.status().is_success() || response.status().is_redirection()
        }
        Err(_) => false,
    }
}

/// Check if a Piped instance is healthy
/// Returns true if the instance responds successfully
pub async fn check_piped(url: &str) -> bool {
    let client = create_health_check_client();

    // Piped API endpoint to check health
    let test_url = format!("{}/streams/pw4w9HwD1Fo", url.trim_end_matches('/'));

    match client.get(&test_url).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// Get the next Invidious instance in rotation
/// If the current instance fails, this will return the next one
pub fn rotate_invidious() -> &'static str {
    let index = INVIDIOUS_INDEX.fetch_add(1, Ordering::SeqCst);
    INVIDIOUS_INSTANCES[index % INVIDIOUS_INSTANCES.len()]
}

/// Get the next Piped instance in rotation
/// If the current instance fails, this will return the next one
pub fn rotate_piped() -> &'static str {
    let index = PIPED_INDEX.fetch_add(1, Ordering::SeqCst);
    PIPED_INSTANCES[index % PIPED_INSTANCES.len()]
}

/// Reset the rotation index (useful for testing or reinitialization)
pub fn reset_invidious_index() {
    INVIDIOUS_INDEX.store(0, Ordering::SeqCst);
}

pub fn reset_piped_index() {
    PIPED_INDEX.store(0, Ordering::SeqCst);
}

/// Get the current Invidious instance without rotating
pub fn current_invidious() -> &'static str {
    let index = INVIDIOUS_INDEX.load(Ordering::SeqCst);
    INVIDIOUS_INSTANCES[index % INVIDIOUS_INSTANCES.len()]
}

/// Get the current Piped instance without rotating
pub fn current_piped() -> &'static str {
    let index = PIPED_INDEX.load(Ordering::SeqCst);
    PIPED_INSTANCES[index % PIPED_INSTANCES.len()]
}

/// Perform a health check on a specific URL
/// Returns true if the URL is reachable and responds
pub async fn health_check(url: &str, api_type: ApiType) -> bool {
    match api_type {
        ApiType::Invidious => check_invidious(url).await,
        ApiType::Piped => check_piped(url).await,
    }
}

/// API type enum for health checking
#[derive(Debug, Clone, Copy)]
pub enum ApiType {
    Invidious,
    Piped,
}

/// Find the first healthy Invidious instance from the list
/// Returns the URL of the first healthy instance, or None if all fail
pub async fn find_healthy_invidious() -> Option<&'static str> {
    for &instance in INVIDIOUS_INSTANCES {
        if check_invidious(instance).await {
            return Some(instance);
        }
    }
    None
}

/// Find the first healthy Piped instance from the list
/// Returns the URL of the first healthy instance, or None if all fail
pub async fn find_healthy_piped() -> Option<&'static str> {
    for &instance in PIPED_INSTANCES {
        if check_piped(instance).await {
            return Some(instance);
        }
    }
    None
}

/// Rotate to the next instance until a healthy one is found
/// Returns the URL of a healthy instance, or rotates through all if none are healthy
pub async fn rotate_to_healthy_invidious() -> &'static str {
    let start_index = INVIDIOUS_INDEX.load(Ordering::SeqCst);
    let count = INVIDIOUS_INSTANCES.len();

    for i in 0..count {
        let index = (start_index + i) % count;
        let instance = INVIDIOUS_INSTANCES[index];

        if check_invidious(instance).await {
            INVIDIOUS_INDEX.store(index, Ordering::SeqCst);
            return instance;
        }
    }

    rotate_invidious()
}

/// Rotate to the next instance until a healthy one is found
/// Returns the URL of a healthy instance, or rotates through all if none are healthy
pub async fn rotate_to_healthy_piped() -> &'static str {
    let start_index = PIPED_INDEX.load(Ordering::SeqCst);
    let count = PIPED_INSTANCES.len();

    for i in 0..count {
        let index = (start_index + i) % count;
        let instance = PIPED_INSTANCES[index];

        if check_piped(instance).await {
            PIPED_INDEX.store(index, Ordering::SeqCst);
            return instance;
        }
    }

    rotate_piped()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_lists_not_empty() {
        assert!(!INVIDIOUS_INSTANCES.is_empty());
        assert!(!PIPED_INSTANCES.is_empty());
    }

    #[test]
    fn test_rotation_returns_valid_instance() {
        let invidious = rotate_invidious();
        assert!(INVIDIOUS_INSTANCES.contains(&invidious));

        let piped = rotate_piped();
        assert!(PIPED_INSTANCES.contains(&piped));
    }

    #[test]
    fn test_reset_index() {
        rotate_invidious();
        rotate_invidious();
        reset_invidious_index();
        let first = current_invidious();
        assert_eq!(first, INVIDIOUS_INSTANCES[0]);
    }
}
