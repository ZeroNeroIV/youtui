pub struct PipedClient {
    base_url: String,
    client: reqwest::Client,
}

impl PipedClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }
}
