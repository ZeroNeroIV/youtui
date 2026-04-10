use std::process::Stdio;
use tokio::process::Child;
use tokio::sync::Mutex;

pub struct MpvPlayer {
    process: Mutex<Option<Child>>,
}

impl MpvPlayer {
    pub fn new() -> Self {
        Self {
            process: Mutex::new(None),
        }
    }

    pub async fn is_available() -> bool {
        tokio::process::Command::new("mpv")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok()
    }

    pub async fn version() -> Option<String> {
        let output = tokio::process::Command::new("mpv")
            .arg("--version")
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            version.lines().next().map(|s| s.to_string())
        } else {
            None
        }
    }

    pub async fn play(&self, url: &str, extra_args: &[&str]) -> Result<(), String> {
        self.run_mpv(url, false, extra_args).await
    }

    pub async fn play_audio(&self, url: &str, extra_args: &[&str]) -> Result<(), String> {
        self.run_mpv(url, true, extra_args).await
    }

    async fn run_mpv(
        &self,
        url: &str,
        audio_only: bool,
        extra_args: &[&str],
    ) -> Result<(), String> {
        if !Self::is_available().await {
            return Err("mpv is not installed or not in PATH".to_string());
        }

        self.stop().await;

        let mut cmd = tokio::process::Command::new("mpv");

        if audio_only {
            cmd.arg("--video=no");
        }

        for arg in extra_args {
            cmd.arg(arg);
        }

        cmd.arg(url);
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        let child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start mpv: {}", e))?;

        let mut process = self.process.lock().await;
        *process = Some(child);

        Ok(())
    }

    pub async fn stop(&self) {
        let mut process = self.process.lock().await;

        if let Some(mut child) = process.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
    }

    pub async fn is_playing(&self) -> bool {
        let process = self.process.lock().await;

        if let Some(child) = process.as_ref() {
            child.id().is_some()
        } else {
            false
        }
    }
}

impl Default for MpvPlayer {
    fn default() -> Self {
        Self::new()
    }
}
