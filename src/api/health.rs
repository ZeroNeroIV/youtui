use reqwest::Client;
use std::time::Duration;

pub static INVIDIOUS_INSTANCES: &[&str] = &[
    "https://invidious.flokinet.to",
    "https://invidious.privacydev.net",
    "https://yt.artemislena.eu",
    "https://iv.melmac.space",
    "https://invidious.namazso.eu",
    "https://inv.riverside.rocks",
    "https://invidious.kavin.rocks",
];

pub fn instances() -> &'static [&'static str] {
    INVIDIOUS_INSTANCES
}

/// Round-robin to the next instance after `current` — instant, no network.
/// Used on retry so each attempt hits a genuinely different instance even
/// when the network is busy (e.g. during a download).
pub fn next_instance(current: &str) -> &'static str {
    let cur = current.trim_end_matches('/');
    let idx = INVIDIOUS_INSTANCES
        .iter()
        .position(|i| i.trim_end_matches('/') == cur)
        .unwrap_or(0);
    INVIDIOUS_INSTANCES[(idx + 1) % INVIDIOUS_INSTANCES.len()]
}

/// Probe instances and return the first that answers. Kept for explicit
/// health checks (not used on the hot retry path, since under load the
/// probes themselves stall).
pub async fn rotate_to_healthy_invidious() -> &'static str {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    for instance in INVIDIOUS_INSTANCES {
        let url = format!("{}/api/v1/stats", instance);
        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                return instance;
            }
        }
    }
    INVIDIOUS_INSTANCES[0]
}
