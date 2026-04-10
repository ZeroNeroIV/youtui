pub mod health {
    pub async fn check_invidious(_url: &str) -> bool {
        true
    }
    pub async fn check_piped(_url: &str) -> bool {
        true
    }
}