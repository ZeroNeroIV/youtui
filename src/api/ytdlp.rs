pub struct YtdlpWrapper;

impl YtdlpWrapper {
    pub fn is_available() -> bool {
        std::process::Command::new("yt-dlp")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
