use crate::db::connection::Filterable;
use crate::download::downloader::{DownloadProgress, Downloader};
use crate::error::AppError;
use crate::ui::components;
use crate::ui::theme::Theme;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, ListState, Padding, Paragraph, Wrap},
    Terminal,
};
use std::io;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum AppMode {
    Main,
    Settings,
    Search,
    History,
    Saved,
    Playlist,
    Downloads,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PlaylistPromptMode {
    New,
    Import,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SearchFocus {
    Input,
    List,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ListFocus {
    Input,
    List,
}

pub enum SearchResponse {
    Success(Vec<crate::api::invidious::Video>),
    Error(String),
}

pub enum SavedResponse {
    Success(String),
    Error(String),
}

pub enum PlaylistAddResponse {
    Success(String),
    Error(String),
}

pub enum PlaylistResponse {
    Imported {
        id: i64,
        videos: Vec<crate::api::invidious::PlaylistVideo>,
    },
    Refreshed {
        id: i64,
        videos: Vec<crate::api::invidious::PlaylistVideo>,
    },
    Error(String),
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ActiveBlock {
    Sidebar,
    Content,
}

#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub enum DownloadBarState {
    #[default]
    Idle,
    Active,
    Done,
    Failed,
}

#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub enum PlaybackState {
    #[default]
    Idle,
    Loading,
    Playing,
    Error,
}


pub struct App {
    pub mode: AppMode,
    pub settings: crate::config::settings::Settings,
    pub app_state: crate::config::settings::AppState,
    pub settings_state: ListState,
    pub theme: Theme,
    pub items: Vec<String>,
    pub list_state: ListState,
    pub sidebar_items: Vec<String>,
    pub sidebar_state: ListState,
    pub active_block: ActiveBlock,
    pub should_quit: bool,
    pub show_context_menu: bool,
    pub context_menu_pos: (u16, u16),
    pub show_keybinds_popup: bool,
    pub show_download_menu: bool,
    pub pending_download_id: Option<String>,
    pub pending_download_title: Option<String>,
    pub anim_tick: u64,
    pub download_is_audio: bool,
    pub downloads: Vec<DownloadRecord>,
    pub downloads_state: ListState,
    pub sidebar_area: Rect,
    pub content_area: Rect,
    pub current_error: Option<String>,
    pub current_suggestion: Option<String>,
    pub is_loading: bool,
    pub is_playing: bool,
    pub is_paused: bool,
    pub loading_message: String,
    pub startup_warnings: Vec<String>,
    pub search_query: String,
    pub search_results: Vec<crate::api::invidious::Video>,
    pub is_searching: bool,
    pub search_error: Option<String>,
    pub search_focus: SearchFocus,
    pub history_focus: ListFocus,
    pub saved_focus: ListFocus,
    pub playlist_focus: ListFocus,
    pub search_tx: tokio::sync::mpsc::Sender<SearchResponse>,
    pub search_rx: tokio::sync::mpsc::Receiver<SearchResponse>,
    pub saved_tx: tokio::sync::mpsc::Sender<SavedResponse>,
    pub saved_rx: tokio::sync::mpsc::Receiver<SavedResponse>,
    pub playlist_tx: tokio::sync::mpsc::Sender<PlaylistResponse>,
    pub playlist_rx: tokio::sync::mpsc::Receiver<PlaylistResponse>,
    pub playlist_add_tx: tokio::sync::mpsc::Sender<PlaylistAddResponse>,
    pub playlist_add_rx: tokio::sync::mpsc::Receiver<PlaylistAddResponse>,
    pub settings_tx: tokio::sync::mpsc::Sender<String>,
    pub settings_rx: tokio::sync::mpsc::Receiver<String>,
    pub download_tx: tokio::sync::mpsc::Sender<DownloadProgress>,
    pub download_rx: tokio::sync::mpsc::Receiver<DownloadProgress>,
    pub download_bar_state: DownloadBarState,
    pub download_bar_title: String,
    pub download_bar_percent: u8,
    pub download_bar_speed: String,
    pub download_bar_eta: String,
    pub playback_state: PlaybackState,
    pub playback_title: String,
    pub playback_started: Option<std::time::Instant>,
    pub playback_error: Option<String>,
    pub playback_is_audio: bool,
    pub playback_notif_rx: tokio::sync::broadcast::Receiver<crate::player::mpv::PlaybackNotification>,
    pub db: std::sync::Arc<crate::db::connection::Database>,
    pub player: std::sync::Arc<dyn crate::player::Player>,
    pub history_results: Vec<crate::db::connection::HistoryEntry>,
    pub history_state: ListState,
    pub saved_results: Vec<crate::db::connection::SavedVideo>,
    pub saved_state: ListState,
    pub playlist_results: Vec<crate::db::connection::Playlist>,
    pub playlist_videos: Vec<crate::db::connection::PlaylistVideo>,
    pub playlist_state: ListState,
    pub playlist_videos_state: ListState,
    pub playlist_prompt: String,
    pub playlist_prompt_mode: Option<PlaylistPromptMode>,
    pub pending_playlist_add: Option<(String, String, Option<String>)>,
    pub playback_ended_rx: tokio::sync::mpsc::Receiver<()>,
    pub playback_ended_tx: tokio::sync::mpsc::Sender<()>,
    pub notification_tx: tokio::sync::broadcast::Sender<crate::player::mpv::PlaybackNotification>,
    pub autoplay_enabled: bool,
    pub last_played_category: Option<String>,
    pub last_played_index: Option<usize>,
}

impl App {
    pub async fn new() -> Result<Self, AppError> {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut sidebar_state = ListState::default();
        sidebar_state.select(Some(0));

        let mut settings_state = ListState::default();
        settings_state.select(Some(0));

        let settings = crate::config::settings::Settings::load();
        let theme = Theme::from_name(&settings.theme).unwrap_or_else(Theme::default_theme);
        let app_state = crate::config::settings::AppState::load();

        sidebar_state.select(Some(app_state.last_sidebar_index));
        list_state.select(Some(app_state.last_content_index));
        settings_state.select(Some(app_state.last_settings_index));

        let mut startup_warnings = Vec::new();

        let mpv_available = crate::player::mpv::MpvPlayer::is_available().await;
        if !mpv_available {
            startup_warnings.push("Warning: mpv not found. Playback will not work.".to_string());
        }

        let api_ok = true;
        if !api_ok {
            startup_warnings.push(
                "Warning: API instance not reachable. Some features may not work.".to_string(),
            );
        }

        if !startup_warnings.is_empty() {
            for warning in &startup_warnings {
                eprintln!("{}", warning);
            }
        }

        let (search_tx, search_rx) = tokio::sync::mpsc::channel(10);
        let (saved_tx, saved_rx) = tokio::sync::mpsc::channel(10);
        let (playlist_tx, playlist_rx) = tokio::sync::mpsc::channel(10);
        let (playlist_add_tx, playlist_add_rx) = tokio::sync::mpsc::channel(10);
        let (settings_tx, settings_rx) = tokio::sync::mpsc::channel(10);
        let (download_tx, download_rx) = tokio::sync::mpsc::channel(10);
        let (playback_ended_tx, playback_ended_rx) = tokio::sync::mpsc::channel(1);
        let (notification_tx, _notification_rx) = tokio::sync::broadcast::channel(16);

        Ok(Self {
            mode: AppMode::Main,
            settings: settings.clone(),
            app_state,
            settings_state,
            theme,
            items: vec![
                "Video 1: Rust for Beginners".to_string(),
                "Video 2: Advanced Ratatui Patterns".to_string(),
                "Video 3: Async Programming in Rust".to_string(),
                "Video 4: Building a TUI with Mouse Support".to_string(),
                "Video 5: YouTube API Integration".to_string(),
                "Video 6: SQLite Persistence".to_string(),
                "Video 7: Theme Systems in TUIs".to_string(),
                "Video 8: Error Handling Best Practices".to_string(),
            ],
            list_state,
            sidebar_items: vec![
                "Search".to_string(),
                "History".to_string(),
                "Saved".to_string(),
                "Playlists".to_string(),
                "Downloads".to_string(),
                "Settings".to_string(),
            ],
            sidebar_state,
            active_block: ActiveBlock::Sidebar,
            should_quit: false,
            show_context_menu: false,
            context_menu_pos: (0, 0),
            show_keybinds_popup: false,
            show_download_menu: false,
            pending_download_id: None,
            pending_download_title: None,
            anim_tick: 0,
            download_is_audio: false,
            downloads: load_downloads(),
            downloads_state: ListState::default(),
            sidebar_area: Rect::default(),
            content_area: Rect::default(),
            current_error: None,
            current_suggestion: None,
            is_loading: false,
        is_playing: false,
            is_paused: false,
            loading_message: String::new(),
            startup_warnings,
            search_query: String::new(),
            search_results: Vec::new(),
            is_searching: false,
            search_error: None,
            search_focus: SearchFocus::Input,
            history_focus: ListFocus::Input,
            saved_focus: ListFocus::Input,
            playlist_focus: ListFocus::Input,
            search_tx,
            search_rx,
            saved_tx,
            saved_rx,
            playlist_tx,
            playlist_rx,
            playlist_add_tx,
            playlist_add_rx,
            settings_tx,
            settings_rx,
            db: std::sync::Arc::new(crate::db::connection::Database::new()?),
            player: crate::player::create_player(
                &settings.player,
                playback_ended_tx.clone(),
                notification_tx.clone(),
                Some(&settings.api_instance_invidious),
                Some(&settings.api_instance_piped),
            )
            .unwrap_or_else(|| {
                std::sync::Arc::new(crate::player::mpv::MpvPlayer::new(
                    playback_ended_tx.clone(),
                    notification_tx.clone(),
                    Some(&settings.api_instance_invidious),
                    Some(&settings.api_instance_piped),
                )) as std::sync::Arc<dyn crate::player::Player>
            }),
            history_results: Vec::new(),
            history_state: ListState::default(),
            saved_results: Vec::new(),
            saved_state: ListState::default(),
            playlist_results: Vec::new(),
            playlist_videos: Vec::new(),
            playlist_state: ListState::default(),
            playlist_videos_state: ListState::default(),
            playlist_prompt: String::new(),
            playlist_prompt_mode: None,
            pending_playlist_add: None,
            download_tx,
            download_rx,
            download_bar_state: DownloadBarState::Idle,
            download_bar_title: String::new(),
            download_bar_percent: 0,
            download_bar_speed: String::new(),
            download_bar_eta: String::new(),
            playback_state: PlaybackState::Idle,
            playback_title: String::new(),
            playback_started: None,
            playback_error: None,
            playback_is_audio: false,
            playback_notif_rx: notification_tx.subscribe(),
            playback_ended_rx,
            playback_ended_tx,
            notification_tx,
            autoplay_enabled: settings.auto_play,
            last_played_category: None,
            last_played_index: None,
        })
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        self.show_startup_banner();

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        while !self.should_quit {
            self.update();
            terminal.draw(|f| self.render(f))?;
            self.handle_events()?;
        }

        self.save_state();

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn update(&mut self) {
        self.anim_tick = self.anim_tick.wrapping_add(1);
        if self.playback_ended_rx.try_recv().is_ok() {
            self.is_playing = false;
            self.is_paused = false;
            self.playback_state = PlaybackState::Idle;
            self.playback_started = None;
        }

        // Drain mpv playback notifications
        while let Ok(notif) = self.playback_notif_rx.try_recv() {
            use crate::player::mpv::PlaybackNotification;
            match notif {
                PlaybackNotification::Failure(e) => {
                    if self.playback_state != PlaybackState::Idle {
                        self.playback_state = PlaybackState::Error;
                        self.playback_error = Some(e);
                    }
                }
                PlaybackNotification::Success(_) => {
                    self.playback_state = PlaybackState::Playing;
                    if self.playback_started.is_none() {
                        self.playback_started = Some(std::time::Instant::now());
                    }
                }
                PlaybackNotification::FallbackAttempt(_) => {
                    self.playback_state = PlaybackState::Loading;
                }
            }
        }

        // Auto-promote Loading -> Playing after a short grace (normal play emits
        // no Success notification unless a fallback provider kicks in).
        if self.playback_state == PlaybackState::Loading {
            if let Some(started) = self.playback_started {
                if started.elapsed().as_millis() > 1200 {
                    self.playback_state = PlaybackState::Playing;
                }
            }
        }

        while let Ok(response) = self.search_rx.try_recv() {
            self.is_searching = false;
            match response {
                SearchResponse::Success(results) => {
                    self.search_results = results;
                    self.search_error = None;
                    if !self.search_results.is_empty() {
                        self.search_focus = SearchFocus::List;
                    }
                }
                SearchResponse::Error(e) => {
                    self.search_error = Some(e);
                    self.search_results.clear();
                }
            }
        }

        while let Ok(response) = self.saved_rx.try_recv() {
            match response {
                SavedResponse::Success(video_id) => {
                    self.saved_results = self.db.get_saved_videos().unwrap_or_default();
                    self.saved_state = ListState::default();
                    if !self.saved_results.is_empty() {
                        self.saved_state.select(Some(0));
                        self.saved_focus = ListFocus::List;
                    }
                    self.set_error(format!("Video {} saved successfully!", video_id));
                }
                SavedResponse::Error(e) => {
                    self.set_error(format!("Failed to save video: {}", e));
                }
            }
        }

        while let Ok(new_url) = self.settings_rx.try_recv() {
            self.settings.api_instance_invidious = new_url;
        }

        while let Ok(response) = self.playlist_rx.try_recv() {
            match response {
                PlaylistResponse::Imported { id, .. } => {
                    self.playlist_results = self.db.get_playlists().unwrap_or_default();
                    if let Ok(videos) = self.db.get_playlist_videos(id) {
                        self.playlist_videos = videos;
                        self.playlist_videos_state = ListState::default();
                        if !self.playlist_videos.is_empty() {
                            self.playlist_videos_state.select(Some(0));
                            self.playlist_focus = ListFocus::List;
                        }
                    }
                }
                PlaylistResponse::Refreshed { id, .. } => {
                    if let Ok(videos) = self.db.get_playlist_videos(id) {
                        self.playlist_videos = videos;
                        self.playlist_videos_state = ListState::default();
                        if !self.playlist_videos.is_empty() {
                            self.playlist_videos_state.select(Some(0));
                            self.playlist_focus = ListFocus::List;
                        }
                    }
                }
                PlaylistResponse::Error(e) => {
                    self.set_error(e);
                }
            }
        }

        while let Ok(response) = self.playlist_add_rx.try_recv() {
            match response {
                PlaylistAddResponse::Success(video_id) => {
                    self.set_error(format!(
                        "Video {} added to playlist successfully!",
                        video_id
                    ));
                }
                PlaylistAddResponse::Error(e) => {
                    self.set_error(format!("Failed to add video to playlist: {}", e));
                }
            }
        }

        while let Ok(progress) = self.download_rx.try_recv() {
            match progress {
                DownloadProgress::Starting { title } => {
                    self.download_bar_title = title;
                    self.download_bar_percent = 0;
                    self.download_bar_speed = String::new();
                    self.download_bar_eta = String::new();
                    self.download_bar_state = DownloadBarState::Active;
                }
                DownloadProgress::Downloading { title, percent, speed, eta } => {
                    self.download_bar_title = title;
                    self.download_bar_percent = percent;
                    self.download_bar_speed = speed;
                    self.download_bar_eta = eta;
                    self.download_bar_state = DownloadBarState::Active;
                }
                DownloadProgress::Completed { title, path: _ } => {
                    self.download_bar_title = title.clone();
                    self.download_bar_percent = 100;
                    self.download_bar_speed = String::new();
                    self.download_bar_eta = String::new();
                    self.download_bar_state = DownloadBarState::Done;

                    let ext = if self.download_is_audio { "mp3" } else { "mp4" };
                    let full = self.settings.download_path.join(format!("{}.{}", title, ext));
                    let rec = DownloadRecord {
                        title,
                        path: full.to_string_lossy().to_string(),
                        kind: if self.download_is_audio { "audio".into() } else { "video".into() },
                        when: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                    };
                    self.loading_message = format!("Saved to: {}", rec.path);
                    self.downloads.insert(0, rec);
                    let _ = save_downloads(&self.downloads);
                    if self.downloads_state.selected().is_none() {
                        self.downloads_state.select(Some(0));
                    }
                }
                DownloadProgress::Failed { error } => {
                    self.download_bar_state = DownloadBarState::Failed;
                    self.set_error(format!("Download failed: {}", error));
                }
            }
        }
    }

    fn show_startup_banner(&self) {
        let version = crate::config::settings::APP_VERSION;
        println!("╔══════════════════════════════════════════╗");
        println!("║           Youtui-rs v{}               ║", version);
        println!("║      YouTube Terminal Client            ║");
        println!("╚══════════════════════════════════════════╝");
        println!();
    }

    fn save_state(&mut self) {
        if let Some(sidebar_idx) = self.sidebar_state.selected() {
            self.app_state.last_sidebar_index = sidebar_idx;
        }
        if let Some(content_idx) = self.list_state.selected() {
            self.app_state.last_content_index = content_idx;
        }
        if let Some(settings_idx) = self.settings_state.selected() {
            self.app_state.last_settings_index = settings_idx;
        }
        if let Err(e) = self.app_state.save() {
            eprintln!("Warning: Failed to save app state: {}", e);
        }
    }

    pub fn recreate_player(&mut self, player_name: &str) {
        let player = self.player.clone();
        tokio::spawn(async move {
            let _ = player.stop().await;
        });

        if let Some(new_player) = crate::player::create_player(
            player_name,
            self.playback_ended_tx.clone(),
            self.notification_tx.clone(),
            Some(&self.settings.api_instance_invidious),
            Some(&self.settings.api_instance_piped),
        ) {
            self.player = new_player;
        }
    }

    fn filter_by_query<T: Filterable + Clone>(&self, items: &[T]) -> Vec<T> {
        if self.search_query.is_empty() {
            return items.to_vec();
        }
        items.iter().filter(|i| i.matches_query(&self.search_query)).cloned().collect()
    }

    #[allow(dead_code)]
    fn play_next_video(&mut self) {
        if self.is_playing {
            return;
        }
        
        let Some(category) = self.last_played_category.clone() else { return };
        let Some(idx) = self.last_played_index else { return };

        match category.as_str() {
            "history" => {
                if idx + 1 < self.history_results.len() {
                    let next_idx = idx + 1;
                    self.history_state.select(Some(next_idx));
                    self.last_played_index = Some(next_idx);
                    if let Some(entry) = self.history_results.get(next_idx).cloned() {
                        self.play_history_video(&entry);
                    }
                }
            }
            "saved" => {
                if idx + 1 < self.saved_results.len() {
                    let next_idx = idx + 1;
                    self.saved_state.select(Some(next_idx));
                    self.last_played_index = Some(next_idx);
                    if let Some(video) = self.saved_results.get(next_idx).cloned() {
                        self.play_saved_video(&video);
                    }
                }
            }
            "search" => {
                if idx + 1 < self.search_results.len() {
                    let next_idx = idx + 1;
                    self.list_state.select(Some(next_idx));
                    self.last_played_index = Some(next_idx);
                    if let Some(video) = self.search_results.get(next_idx).cloned() {
                        self.play_search_video(&video);
                    }
                }
            }
            "main" => {
                if idx + 1 < self.items.len() {
                    let next_idx = idx + 1;
                    self.list_state.select(Some(next_idx));
                    self.last_played_index = Some(next_idx);
                    if let Some(title) = self.items.get(next_idx).cloned() {
                        self.play_main_video(&title);
                    }
                }
            }
            "playlist"
                if self.mode == AppMode::Playlist
                    && !self.playlist_videos.is_empty()
                    && idx + 1 < self.playlist_videos.len() =>
            {
                let next_idx = idx + 1;
                self.playlist_videos_state.select(Some(next_idx));
                self.last_played_index = Some(next_idx);
                if let Some(video) = self.playlist_videos.get(next_idx).cloned() {
                    self.play_playlist_video(&video);
                }
            }
            _ => {}
        }
    }

    /// Manually skip to the next/previous item in the last-played list
    /// (delta = +1 next, -1 previous). Unlike play_next_video this ignores the
    /// is_playing guard so it works while something is already playing.
    fn skip_relative(&mut self, delta: i64) {
        let Some(category) = self.last_played_category.clone() else { return };
        let Some(idx) = self.last_played_index else { return };
        let target = idx as i64 + delta;
        if target < 0 { return; }
        let t = target as usize;
        match category.as_str() {
            "history" => {
                if t < self.history_results.len() {
                    self.history_state.select(Some(t));
                    self.last_played_index = Some(t);
                    if let Some(e) = self.history_results.get(t).cloned() { self.play_history_video(&e); }
                }
            }
            "saved" => {
                if t < self.saved_results.len() {
                    self.saved_state.select(Some(t));
                    self.last_played_index = Some(t);
                    if let Some(v) = self.saved_results.get(t).cloned() { self.play_saved_video(&v); }
                }
            }
            "search" => {
                if t < self.search_results.len() {
                    self.list_state.select(Some(t));
                    self.last_played_index = Some(t);
                    if let Some(v) = self.search_results.get(t).cloned() { self.play_search_video(&v); }
                }
            }
            "main" => {
                if t < self.items.len() {
                    self.list_state.select(Some(t));
                    self.last_played_index = Some(t);
                    if let Some(title) = self.items.get(t).cloned() { self.play_main_video(&title); }
                }
            }
            "playlist" if t < self.playlist_videos.len() => {
                self.playlist_videos_state.select(Some(t));
                self.last_played_index = Some(t);
                if let Some(v) = self.playlist_videos.get(t).cloned() { self.play_playlist_video(&v); }
            }
            _ => {}
        }
    }

    fn is_typing_context(&self) -> bool {
        match self.mode {
            AppMode::Search => self.search_focus == SearchFocus::Input,
            AppMode::Playlist => self.playlist_prompt_mode.is_some() || self.playlist_focus == ListFocus::Input,
            AppMode::History => self.history_focus == ListFocus::Input,
            AppMode::Saved => self.saved_focus == ListFocus::Input,
            _ => false,
        }
    }

    fn media_toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
        let player = self.player.clone();
        tokio::spawn(async move { player.toggle_pause().await; });
    }

    fn media_seek(&mut self, secs: i64) {
        let player = self.player.clone();
        tokio::spawn(async move { player.seek(secs).await; });
    }

    fn render(&mut self, f: &mut ratatui::Frame) {
        let dl_active = self.download_bar_state == DownloadBarState::Active;
        let dl_lingering = matches!(self.download_bar_state, DownloadBarState::Done | DownloadBarState::Failed);
        let pb_active = self.playback_state != PlaybackState::Idle;
        let show_bottom_bar = dl_active || dl_lingering || pb_active;
        let outer = if show_bottom_bar {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(2)])
                .split(f.area())
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0)])
                .split(f.area())
        };
        let main_area = outer[0];

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(15), Constraint::Percentage(85)])
            .split(main_area);

        self.sidebar_area = chunks[0];
        self.content_area = chunks[1];

        let sidebar_items = [
            components::SidebarItem { icon: "🔍", label: "Search" },
            components::SidebarItem { icon: "📜", label: "History" },
            components::SidebarItem { icon: "🔖", label: "Saved" },
            components::SidebarItem { icon: "📋", label: "Playlists" },
            components::SidebarItem { icon: "⬇️", label: "Downloads" },
            components::SidebarItem { icon: "⚙️", label: "Settings" },
        ];

        components::render_sidebar(
            f,
            self.sidebar_area,
            &sidebar_items,
            self.sidebar_state.selected().unwrap_or(0),
            &self.theme,
            self.active_block == ActiveBlock::Sidebar,
        );

        match self.mode {
            AppMode::Main => self.render_main_content(f),
            AppMode::Settings => crate::ui::settings::render(f, self.content_area, self),
            AppMode::Search => self.render_search(f, self.content_area),
            AppMode::History => self.render_history(f, self.content_area),
            AppMode::Saved => self.render_saved(f, self.content_area),
            AppMode::Playlist => self.render_playlist(f, self.content_area),
            AppMode::Downloads => self.render_downloads(f),
        }
        
        if self.show_keybinds_popup {
            self.render_keybinds_popup(f);
        }

        if self.show_download_menu {
            self.render_download_menu(f);
        }

        if show_bottom_bar {
            // Active download takes priority on the bar; otherwise show playback;
            // otherwise the lingering download result.
            if dl_active {
                self.render_download_bar(f, outer[1]);
            } else if pb_active {
                self.render_playback_bar(f, outer[1]);
            } else {
                self.render_download_bar(f, outer[1]);
            }
        }
    }

    fn render_downloads(&mut self, f: &mut ratatui::Frame) {
        use ratatui::style::Modifier;
        let area = self.content_area;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        components::render_header(
            f, chunks[0],
            "Downloads",
            &format!("• {} files", self.downloads.len()),
            &self.theme,
        );

        if self.downloads.is_empty() {
            components::render_empty_state(
                f, chunks[1], &self.theme,
                "No downloads yet",
                "Press 'd' on any video to download it",
                Some("⬇"),
            );
        } else {
            let list_area = chunks[1];
            let row_h: u16 = 2;
            let visible = (list_area.height / row_h) as usize;
            let sel = self.downloads_state.selected().unwrap_or(0);
            let offset = if sel >= visible { sel + 1 - visible } else { 0 };
            let maxw = list_area.width.saturating_sub(2) as usize;

            for (i, rec) in self.downloads.iter().enumerate().skip(offset).take(visible) {
                let y = list_area.y + (i - offset) as u16 * row_h;
                if y + row_h > list_area.y + list_area.height { break; }
                let is_sel = sel == i;
                let icon = if rec.kind == "audio" { "🎵" } else { "🎬" };
                let title_line = format!("  {} {}", icon, rec.title);
                let meta_line = format!("       {}  ·  {}  ·  {}", rec.kind, rec.when, rec.path);
                let tstyle = if is_sel {
                    Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
                } else {
                    Style::default()
                };
                f.render_widget(
                    Paragraph::new(title_line.chars().take(maxw).collect::<String>()).style(tstyle),
                    Rect::new(list_area.x, y, list_area.width, 1),
                );
                f.render_widget(
                    Paragraph::new(meta_line.chars().take(maxw).collect::<String>())
                        .style(Style::default().add_modifier(Modifier::DIM)),
                    Rect::new(list_area.x, y + 1, list_area.width, 1),
                );
            }
        }

        let foot = Paragraph::new("  ↑/↓ navigate  ·  Enter play  ·  Esc back")
            .style(Style::default().add_modifier(Modifier::DIM));
        f.render_widget(foot, chunks[2]);
    }
    
    fn render_keybinds_popup(&mut self, f: &mut ratatui::Frame) {
        let popup_area = Rect {
            x: self.content_area.width / 4,
            y: self.content_area.height / 6,
            width: self.content_area.width / 2,
            height: self.content_area.height * 2 / 3,
        };
        
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent))
            .title("Keybindings");
        
        let keybinds = [
            ("/", "Show/hide keybindings"),
            ("Esc", "Close popup / Clear input"),
            ("Tab", "Toggle focus (sidebar/list)"),
            ("Enter", "Select / Play video"),
            ("↑/↓", "Navigate list"),
            ("s", "Search mode"),
            ("h", "History"),
            ("v", "Saved videos"),
            ("p", "Playlists"),
            ("d", "Download video"),
            ("q", "Quit"),
        ];
        
        let content: Vec<Line> = keybinds.iter()
            .map(|(key, desc)| {
                Line::from(vec![
                    Span::styled(format!("{:8}", key), Style::default().fg(self.theme.accent).bold()),
                    Span::raw(*desc),
                ])
            })
            .collect();
        
        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: true });
        
        f.render_widget(Clear, popup_area);
        f.render_widget(paragraph, popup_area);
    }

    fn render_main_content(&mut self, f: &mut ratatui::Frame) {
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(self.content_area);

        let ascii_art = r##"

  ██╗   ██╗ ██████╗ ██╗   ██╗████████╗██╗   ██╗██╗
  ╚██╗ ██╔╝██╔═══██╗██║   ██║╚══██╔══╝██║   ██║██║
   ╚████╔╝ ██║   ██║██║   ██║   ██║   ██║   ██║██║
    ╚██╔╝  ██║   ██║██║   ██║   ██║   ██║   ██║██║
     ██║   ╚██████╔╝╚██████╔╝   ██║   ╚██████╔╝██║
     ╚═╝    ╚═════╝  ╚═════╝    ╚═╝    ╚═════╝ ╚═╝

  ╔══════════════════════════════════════════════════════╗
  ║   ▶  YouTube Terminal Interface — no browser needed  ║
  ║                                                      ║
  ║  [s] Search   [h] History   [v] Saved                ║
  ║  [p] Playlists              [q] Quit                 ║
  ╚══════════════════════════════════════════════════════╝
"##;

        let art_paragraph = Paragraph::new(ascii_art)
            .style(Style::default().fg(self.theme.accent))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(art_paragraph, content_chunks[0]);

        components::render_info_bar(
            f,
            content_chunks[1],
            &[("Theme", &self.theme.name)],
            &self.theme,
        );

        if self.show_context_menu {
            let area = Rect::new(self.context_menu_pos.0, self.context_menu_pos.1, 20, 6);
            let menu =
                Paragraph::new("1. Play\n2. Save\n3. Download\n4. Add to Playlist\n5. Cancel")
                    .block(
                        Block::default()
                            .title("Options")
                            .borders(Borders::LEFT)
                            .border_style(self.theme.accent),
                    );
            f.render_widget(ratatui::widgets::Clear, area);
            f.render_widget(menu, area);
        }

        if self.current_error.is_some() {
            self.render_error_overlay(f);
        } else if self.is_loading {
            self.render_loading_overlay(f);
        }
    }

    fn render_history(&mut self, f: &mut ratatui::Frame, area: Rect) {
        let filtered = self.filter_by_query(&self.history_results);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        components::render_header(
            f,
            chunks[0],
            "History",
            &format!("• {} items", filtered.len()),
            &self.theme,
        );

        let display_text = if self.search_query.is_empty() {
            "Filter history...".to_string()
        } else {
            format!("{}▌", self.search_query)
        };
        let input = Paragraph::new(display_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .padding(Padding::uniform(1)),
            )
            .style(Style::default().fg(Color::Yellow).bg(Color::Blue));
        f.render_widget(input, chunks[1]);

        if filtered.is_empty() {
            components::render_empty_state(
                f,
                chunks[2],
                &self.theme,
                "History",
                components::ErrorCategory::EmptyHistory.message(),
                Some("🕒"),
            );
        } else {
            let item_height = 6 + components::DesignTokens::ITEM_GAP;
            let offset = self.history_state.selected().unwrap_or(0);
            let visible_count = (chunks[2].height / item_height) as usize;

            for (i, entry) in filtered
                .iter()
                .enumerate()
                .skip(offset)
                .take(visible_count)
            {
                let rect = Rect::new(
                    chunks[2].x,
                    chunks[2].y + (i - offset) as u16 * item_height,
                    chunks[2].width,
                    6,
                );
                let channel = entry.channel.as_deref().unwrap_or("Unknown");
                let meta = format!("{} • Watched {}", channel, entry.watched_at);
                let is_selected = self.history_state.selected() == Some(i);
                let is_focused = self.history_focus == ListFocus::List;

                components::render_item_card(
                    f,
                    rect,
                    &entry.title,
                    &meta,
                    &self.theme,
                    is_selected,
                    is_focused,
                );
            }
        }

        components::render_info_bar(f, chunks[3], &[("Theme", &self.theme.name), ("Filter", &self.search_query)], &self.theme);
    }

    fn render_saved(&mut self, f: &mut ratatui::Frame, area: Rect) {
        let filtered = self.filter_by_query(&self.saved_results);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        components::render_header(
            f,
            chunks[0],
            "Saved",
            &format!("• {} items", filtered.len()),
            &self.theme,
        );

        let display_text = if self.search_query.is_empty() {
            "Filter saved...".to_string()
        } else {
            format!("{}▌", self.search_query)
        };
        let input = Paragraph::new(display_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .padding(Padding::uniform(1)),
            )
            .style(Style::default().fg(Color::Yellow).bg(Color::Blue));
        f.render_widget(input, chunks[1]);

        if filtered.is_empty() {
            components::render_empty_state(
                f,
                chunks[2],
                &self.theme,
                "Saved",
                components::ErrorCategory::EmptySaved.message(),
                Some("🔖"),
            );
} else {
            let item_height = 6 + components::DesignTokens::ITEM_GAP;
            let offset = self.saved_state.selected().unwrap_or(0);
            let visible_count = (chunks[2].height / item_height) as usize;

            for (i, entry) in filtered
                .iter()
                .enumerate()
                .skip(offset)
                .take(visible_count)
            {
                let rect = Rect::new(
                    chunks[2].x,
                    chunks[2].y + (i - offset) as u16 * item_height,
                    chunks[2].width,
                    6,
                );
                let channel = entry.channel.as_deref().unwrap_or("Unknown");
                let meta = format!("{} • Saved {}", channel, entry.saved_at);
                let is_selected = self.saved_state.selected() == Some(i);
                let is_focused = self.saved_focus == ListFocus::List;

                components::render_item_card(
                    f,
                    rect,
                    &entry.title,
                    &meta,
                    &self.theme,
                    is_selected,
                    is_focused,
                );
            }
        }

        components::render_info_bar(f, chunks[3], &[("Theme", &self.theme.name), ("Filter", &self.search_query)], &self.theme);
    }

    fn render_playlist(&mut self, f: &mut ratatui::Frame, area: Rect) {
        if let Some(prompt_mode) = self.playlist_prompt_mode {
            let title = match prompt_mode {
                PlaylistPromptMode::New => "Create New Playlist",
                PlaylistPromptMode::Import => "Import YouTube Playlist",
            };
            let subtitle = match prompt_mode {
                PlaylistPromptMode::New => "Enter name and press Enter to create, Esc to cancel",
                PlaylistPromptMode::Import => "Enter URL and press Enter to import, Esc to cancel",
            };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(area);

            components::render_header(f, chunks[0], title, subtitle, &self.theme);

            let input = Paragraph::new(self.playlist_prompt.as_str())
                .block(
                    Block::default()
                        .borders(Borders::NONE)
                        .padding(Padding::uniform(components::DesignTokens::PADDING_MD)),
                )
                .style(Style::default().fg(self.theme.foreground));
            f.render_widget(input, chunks[1]);

            components::render_info_bar(
                f,
                chunks[2],
                &[("Enter", "Confirm"), ("Esc", "Cancel")],
                &self.theme,
            );
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        if !self.playlist_videos.is_empty() {
            let playlist_name = self
                .playlist_results
                .get(self.playlist_state.selected().unwrap_or(0))
                .map(|p| p.name.as_str())
                .unwrap_or("Unknown Playlist");

            components::render_header(
                f,
                chunks[0],
                &format!("Playlist: {}", playlist_name),
                "Videos in this playlist",
                &self.theme,
            );

            components::render_divider(f, chunks[1], &self.theme, Direction::Horizontal);

            let item_height = 6 + components::DesignTokens::ITEM_GAP;
            let offset = self.playlist_videos_state.selected().unwrap_or(0);
            let visible_count = (chunks[2].height / item_height) as usize;

            for (i, video) in self
                .playlist_videos
                .iter()
                .enumerate()
                .skip(offset)
                .take(visible_count)
            {
                let rect = Rect::new(
                    chunks[2].x,
                    chunks[2].y + (i - offset) as u16 * item_height,
                    chunks[2].width,
                    6,
                );
                let channel = video.channel.as_deref().unwrap_or("Unknown");
                let meta = format!("{} • Position {}", channel, video.position);
                let is_selected = self.playlist_videos_state.selected() == Some(i);
                let is_focused = self.playlist_focus == ListFocus::List;

                components::render_item_card(
                    f,
                    rect,
                    &video.title,
                    &meta,
                    &self.theme,
                    is_selected,
                    is_focused,
                );
            }

            components::render_info_bar(
                f,
                chunks[3],
                &[
                    ("Enter", "Play"),
                    ("d", "Download"),
                    ("x", "Remove"),
                    ("Esc", "Back"),
                    ("Ctrl+Shift+X", if self.autoplay_enabled { "Auto ON" } else { "Auto OFF" }),
                ],
                &self.theme,
            );
        } else {
            components::render_header(
                f,
                chunks[0],
                "Playlists",
                &format!("• {} playlists", self.playlist_results.len()),
                &self.theme,
            );

            components::render_divider(f, chunks[1], &self.theme, Direction::Horizontal);

            let item_height = 6 + components::DesignTokens::ITEM_GAP;
            let offset = self.playlist_state.selected().unwrap_or(0);
            let visible_count = (chunks[2].height / item_height) as usize;

            for (i, playlist) in self
                .playlist_results
                .iter()
                .enumerate()
                .skip(offset)
                .take(visible_count)
            {
                let rect = Rect::new(
                    chunks[2].x,
                    chunks[2].y + (i - offset) as u16 * item_height,
                    chunks[2].width,
                    6,
                );
                let meta = if playlist.is_imported { "Imported" } else { "" };
                let is_selected = self.playlist_state.selected() == Some(i);
                let is_focused = self.playlist_focus == ListFocus::List;

                components::render_item_card(
                    f,
                    rect,
                    &playlist.name,
                    meta,
                    &self.theme,
                    is_selected,
                    is_focused,
                );
            }

            components::render_info_bar(
                f,
                chunks[3],
                &[
                    ("Enter", "View"),
                    ("n", "New"),
                    ("i", "Import"),
                    ("r", "Refresh"),
                    ("Shift+D", "Delete"),
                    ("Esc", "Back"),
                ],
                &self.theme,
            );
        }
    }

    fn render_search(&mut self, f: &mut ratatui::Frame, area: Rect) {
        use ratatui::style::Modifier;

        // Layout: search bar | status strip | results | keybind strip
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // search bar
                Constraint::Length(1),  // status strip
                Constraint::Min(0),     // results
                Constraint::Length(1),  // keybind strip
            ])
            .split(area);

        // ── Search bar ──────────────────────────────────────────────────────
        let is_input_focused = self.search_focus == SearchFocus::Input;
        let bar_border = if is_input_focused {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prompt = if self.search_query.is_empty() {
            "  🔍  Search YouTube...".to_string()
        } else {
            format!("  🔍  {}▌", self.search_query)
        };
        let search_bar = Paragraph::new(prompt)
            .style(if is_input_focused {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(bar_border),
            );
        f.render_widget(search_bar, chunks[0]);

        // ── Status strip (one line) ──────────────────────────────────────────
        let status_line = if self.is_searching {
            "  ⟳  Searching…".to_string()
        } else if !self.search_results.is_empty() {
            let n = self.search_results.len();
            let sel = self.list_state.selected().map(|i| i + 1).unwrap_or(0);
            format!("  {} of {} results  ·  Tab: switch focus  ·  Enter: play  ·  s: save  ·  d: download", sel, n)
        } else if self.search_error.is_some() {
            "  ✗  Search failed".to_string()
        } else {
            "  Press Enter to search".to_string()
        };
        let status = Paragraph::new(status_line)
            .style(Style::default().add_modifier(Modifier::DIM));
        f.render_widget(status, chunks[1]);

        // ── Results area ────────────────────────────────────────────────────
        if self.is_searching {
            components::render_loading(f, chunks[2], "Querying Invidious…");
        } else if let Some(ref error) = self.search_error.clone() {
            components::render_error(f, chunks[2], error, Some("Try a different Invidious instance in Settings"));
        } else if self.search_results.is_empty() {
            components::render_empty_state(
                f, chunks[2], &self.theme,
                "No results",
                "Type a query above and press Enter",
                Some("🔍"),
            );
        } else {
            let row_h: u16 = 3; // title + meta + blank
            let list_area = chunks[2];
            let visible = (list_area.height / row_h) as usize;
            let selected = self.list_state.selected().unwrap_or(0);
            // Keep selected in view
            let offset = if selected >= visible { selected + 1 - visible } else { 0 };
            let is_list_focused = self.search_focus == SearchFocus::List;

            for (i, video) in self.search_results.iter().enumerate().skip(offset).take(visible) {
                let y = list_area.y + (i - offset) as u16 * row_h;
                if y + row_h > list_area.y + list_area.height { break; }

                let is_sel = selected == i;

                // Format metadata
                let channel  = video.author.as_deref().unwrap_or("Unknown");
                let duration = video.length_seconds
                    .map(|s| {
                        let h = s / 3600;
                        let m = (s % 3600) / 60;
                        let sec = s % 60;
                        if h > 0 { format!("{}:{:02}:{:02}", h, m, sec) }
                        else      { format!("{}:{:02}", m, sec) }
                    })
                    .unwrap_or_else(|| "?:??".to_string());
                let views = video.view_count
                    .map(|v| {
                        if v >= 1_000_000 { format!("{:.1}M views", v as f64 / 1_000_000.0) }
                        else if v >= 1_000 { format!("{:.0}K views", v as f64 / 1_000.0) }
                        else              { format!("{} views", v) }
                    })
                    .unwrap_or_default();
                let date = video.published_text.as_deref().unwrap_or("");

                // Title row
                let idx_label = format!("{:>2}. ", i + 1);
                let title_prefix = if is_sel && is_list_focused { "▶ " } else { "  " };
                let title_text = format!("{}{}{}", idx_label, title_prefix, video.title);
                // Truncate to width
                let max_w = list_area.width.saturating_sub(2) as usize;
                let title_display: String = title_text.chars().take(max_w).collect();

                let title_style = if is_sel && is_list_focused {
                    Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
                } else if is_sel {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                // Meta row — dimmed, indented
                let meta_parts: Vec<&str> = [channel, &duration, &views, date]
                    .iter().filter(|s| !s.is_empty()).copied().collect();
                let meta_text = format!("      {}", meta_parts.join("  ·  "));
                let meta_display: String = meta_text.chars().take(max_w).collect();

                let title_row = Rect::new(list_area.x, y,     list_area.width, 1);
                let meta_row  = Rect::new(list_area.x, y + 1, list_area.width, 1);

                f.render_widget(
                    Paragraph::new(title_display).style(title_style),
                    title_row,
                );
                f.render_widget(
                    Paragraph::new(meta_display)
                        .style(Style::default().add_modifier(Modifier::DIM)),
                    meta_row,
                );
            }

            // Scrollbar indicator (right edge)
            if self.search_results.len() > visible {
                let total = self.search_results.len();
                let bar_h = (list_area.height as usize * visible / total).max(1) as u16;
                let bar_y = list_area.y + (list_area.height as usize * offset / total) as u16;
                for dy in 0..list_area.height {
                    let in_thumb = dy >= (bar_y - list_area.y) && dy < (bar_y - list_area.y + bar_h);
                    let ch = if in_thumb { "█" } else { "░" };
                    let r = Rect::new(list_area.x + list_area.width - 1, list_area.y + dy, 1, 1);
                    f.render_widget(Paragraph::new(ch), r);
                }
            }
        }

        // ── Keybind strip ────────────────────────────────────────────────────
        let keys = Paragraph::new(
            "  Enter play  ·  Tab focus  ·  s save  ·  d download  ·  a playlist  ·  Esc back"
        )
        .style(Style::default().add_modifier(Modifier::DIM))
        .block(Block::default().borders(Borders::TOP));
        f.render_widget(keys, chunks[3]);
    }

    fn handle_events(&mut self) -> Result<(), AppError> {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if self.show_download_menu => {
                    match key.code {
                        KeyCode::Char('1') | KeyCode::Char('v') | KeyCode::Char('V') => {
                            self.show_download_menu = false;
                            if let Some(id) = self.pending_download_id.take() {
                                let title = self.pending_download_title.take().unwrap_or_else(|| id.clone());
                                self.trigger_download(id, title, false);
                            }
                        }
                        KeyCode::Char('2') | KeyCode::Char('a') | KeyCode::Char('A') => {
                            self.show_download_menu = false;
                            if let Some(id) = self.pending_download_id.take() {
                                let title = self.pending_download_title.take().unwrap_or_else(|| id.clone());
                                self.trigger_download(id, title, true);
                            }
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            self.show_download_menu = false;
                            self.pending_download_id = None;
                            self.pending_download_title = None;
                        }
                        _ => {}
                    }
                }
                Event::Key(key)
                    if self.is_playing
                        && !self.is_typing_context()
                        && matches!(
                            key.code,
                            KeyCode::Char(' ')
                                | KeyCode::Left
                                | KeyCode::Right
                                | KeyCode::Char('>')
                                | KeyCode::Char('<')
                        ) =>
                {
                    match key.code {
                        KeyCode::Char(' ') => self.media_toggle_pause(),
                        KeyCode::Left => self.media_seek(-10),
                        KeyCode::Right => self.media_seek(10),
                        KeyCode::Char('>') => self.skip_relative(1),
                        KeyCode::Char('<') => self.skip_relative(-1),
                        _ => {}
                    }
                }
                Event::Key(key) => match self.mode {
                    AppMode::Main => {
                        if key.code == KeyCode::Char('q') {
                            let player = self.player.clone();
                            self.should_quit = true;
                            tokio::spawn(async move {
                                let _ = player.stop().await;
                            });
                        } else if key.code == KeyCode::Char('s') {
                            self.mode = AppMode::Search;
                            self.search_query.clear();
                            self.search_results.clear();
                            self.search_error = None;
                        } else if key.code == KeyCode::Char('h') {
                            self.mode = AppMode::History;
                            self.history_results = self.db.get_history(100).unwrap_or_default();
                            self.history_state = ListState::default();
                            if !self.history_results.is_empty() {
                                self.history_state.select(Some(0));
                                self.history_focus = ListFocus::List;
                            }
                        } else if key.code == KeyCode::Char('v') {
                            self.mode = AppMode::Saved;
                            self.saved_results = self.db.get_saved_videos().unwrap_or_default();
                            self.saved_state = ListState::default();
                            if !self.saved_results.is_empty() {
                                self.saved_state.select(Some(0));
                                self.saved_focus = ListFocus::List;
                            }
                        } else if key.code == KeyCode::Char('p') {
                            self.mode = AppMode::Playlist;
                            self.playlist_results = self.db.get_playlists().unwrap_or_default();
                            self.playlist_state = ListState::default();
                            if !self.playlist_results.is_empty() {
                                self.playlist_state.select(Some(0));
                                self.playlist_focus = ListFocus::List;
                            }
                            self.playlist_videos.clear();
                            self.playlist_videos_state = ListState::default();
                            self.playlist_prompt.clear();
                            self.playlist_prompt_mode = None;
                        } else if key.code == KeyCode::Esc {
                            self.show_context_menu = false;
                            self.show_keybinds_popup = false;
                        } else if key.code == KeyCode::Char('/') {
                            self.show_keybinds_popup = !self.show_keybinds_popup;
                        } else if key.code == KeyCode::Enter {
                            if self.active_block == ActiveBlock::Sidebar {
                                if let Some(idx) = self.sidebar_state.selected() {
                                    match self.sidebar_items[idx].as_str() {
                                        "Search" => {
                                            self.mode = AppMode::Search;
                                            self.search_query.clear();
                                            self.search_results.clear();
                                            self.search_error = None;
                                        }
                                        "History" => {
                                            self.mode = AppMode::History;
                                            self.history_results = self.db.get_history(100).unwrap_or_default();
                                            self.history_state = ListState::default();
                                            if !self.history_results.is_empty() {
                                                self.history_state.select(Some(0));
                                                self.history_focus = ListFocus::List;
                                            }
                                        }
                                        "Saved" => {
                                            self.mode = AppMode::Saved;
                                            self.saved_results = self.db.get_saved_videos().unwrap_or_default();
                                            self.saved_state = ListState::default();
                                            if !self.saved_results.is_empty() {
                                                self.saved_state.select(Some(0));
                                                self.saved_focus = ListFocus::List;
                                            }
                                        }
                                        "Playlists" => {
                                            self.mode = AppMode::Playlist;
                                            self.playlist_results = self.db.get_playlists().unwrap_or_default();
                                            self.playlist_state = ListState::default();
                                            if !self.playlist_results.is_empty() {
                                                self.playlist_state.select(Some(0));
                                                self.playlist_focus = ListFocus::List;
                                            }
                                            self.playlist_videos.clear();
                                            self.playlist_videos_state = ListState::default();
                                            self.playlist_prompt.clear();
                                            self.playlist_prompt_mode = None;
                                        }
                                        "Downloads" => {
                                            self.mode = AppMode::Downloads;
                                            self.downloads = load_downloads();
                                            if !self.downloads.is_empty() && self.downloads_state.selected().is_none() {
                                                self.downloads_state.select(Some(0));
                                            }
                                        }
                                        "Settings" => self.mode = AppMode::Settings,
                                        _ => {}
                                    }
                                }
                            } else if self.active_block == ActiveBlock::Content {
                                if let Some(idx) = self.list_state.selected() {
                                    if let Some(video_title) = self.items.get(idx).cloned() {
                                        self.play_main_video(&video_title);
                                    }
                                }
                            }
                        } else if key.code == KeyCode::Up {
                            self.scroll_up();
                        } else if key.code == KeyCode::Down {
                            self.scroll_down();
                        } else if key.code == KeyCode::Tab {
                            if self.mode == AppMode::Search || self.mode == AppMode::History || self.mode == AppMode::Saved || self.mode == AppMode::Playlist {
                                self.search_focus = match self.search_focus {
                                    SearchFocus::Input => SearchFocus::List,
                                    SearchFocus::List => SearchFocus::Input,
                                };
                            } else if self.active_block == ActiveBlock::Sidebar {
                                self.active_block = ActiveBlock::Content;
                            } else {
                                self.active_block = ActiveBlock::Sidebar;
                            }
                        }
                    }
                    AppMode::Settings => {
                        crate::ui::settings::handle_events(self, key.code);
                    }
                    AppMode::Downloads => {
                        if key.code == KeyCode::Esc {
                            self.show_keybinds_popup = false;
                            self.mode = AppMode::Main;
                        } else if key.code == KeyCode::Char('/') {
                            self.show_keybinds_popup = !self.show_keybinds_popup;
                        } else if key.code == KeyCode::Up {
                            if !self.downloads.is_empty() {
                                let i = match self.downloads_state.selected() {
                                    Some(0) | None => self.downloads.len() - 1,
                                    Some(i) => i - 1,
                                };
                                self.downloads_state.select(Some(i));
                            }
                        } else if key.code == KeyCode::Down {
                            if !self.downloads.is_empty() {
                                let i = match self.downloads_state.selected() {
                                    Some(i) => (i + 1) % self.downloads.len(),
                                    None => 0,
                                };
                                self.downloads_state.select(Some(i));
                            }
                        } else if key.code == KeyCode::Enter {
                            if let Some(idx) = self.downloads_state.selected() {
                                if let Some(rec) = self.downloads.get(idx).cloned() {
                                    self.play_downloaded(&rec);
                                }
                            }
                        }
                    }
                    AppMode::Search => match key.code {
                        KeyCode::Char('d') if self.search_focus == SearchFocus::List => {
                            self.handle_download_shortcut();
                        }
                        KeyCode::Char('s') if self.search_focus == SearchFocus::List => {
                            self.trigger_save();
                        }
                        KeyCode::Char('a') if self.search_focus == SearchFocus::List => {
                            self.trigger_add_to_playlist();
                        }
                        KeyCode::Char(c) => {
                            if self.search_focus == SearchFocus::Input {
                                self.search_query.push(c);
                                self.search_results.clear();
                                self.list_state = ListState::default();
                                self.list_state.select(Some(0));
                            }
                        }
                        KeyCode::Backspace => {
                            if self.search_focus == SearchFocus::Input {
                                self.search_query.pop();
                                self.search_results.clear();
                                self.list_state = ListState::default();
                                self.list_state.select(Some(0));
                            }
                        }
                        KeyCode::Up => {
                            if self.search_focus == SearchFocus::List {
                                let i = match self.list_state.selected() {
                                    Some(i) => {
                                        if i == 0 {
                                            self.search_results.len().saturating_sub(1)
                                        } else {
                                            i - 1
                                        }
                                    }
                                    None => 0,
                                };
                                self.list_state.select(Some(i));
                            }
                        }
                        KeyCode::Down => {
                            if self.search_focus == SearchFocus::List {
                                let i = match self.list_state.selected() {
                                    Some(i) => (i + 1) % self.search_results.len().max(1),
                                    None => 0,
                                };
                                self.list_state.select(Some(i));
                            }
                        }
                        KeyCode::Esc => {
                            self.mode = AppMode::Main;
                        }
KeyCode::Enter => {
                            if self.search_focus == SearchFocus::Input {
                                if !self.search_query.is_empty() {
                                    self.trigger_search();
                                }
                            } else if !self.search_results.is_empty() {
                                if let Some(idx) = self.list_state.selected() {
                                    if let Some(video) = self.search_results.get(idx).cloned() {
                                        self.play_search_video(&video);
                                    }
                                }
                            }

                        }
                        KeyCode::Tab => {
                            self.search_focus = match self.search_focus {
                                SearchFocus::Input => SearchFocus::List,
                                SearchFocus::List => SearchFocus::Input,
                            };
                        }
                        _ => {}
                    },
                    AppMode::History => match key.code {
                        KeyCode::Char('c') => {
                            if let Err(e) = self.db.clear_history() {
                                self.set_error(format!("Failed to clear history: {}", e));
                            } else {
                                self.history_results.clear();
                                self.history_state = ListState::default();
                            }
                        }
                        KeyCode::Tab => {
                            self.history_focus = match self.history_focus {
                                ListFocus::Input => ListFocus::List,
                                ListFocus::List => ListFocus::Input,
                            };
                        }
                        KeyCode::Char(c) => {
                            if self.history_focus == ListFocus::Input {
                                self.search_query.push(c);
                            } else if c == 'd' {
                                self.handle_download_shortcut();
                            }
                        }
                        KeyCode::Backspace => {
                            if self.history_focus == ListFocus::Input {
                                self.search_query.pop();
                            }
                        }
                        KeyCode::Up => self.scroll_history_up(),
                        KeyCode::Down => self.scroll_history_down(),
                        KeyCode::Enter => {
                            if let Some(idx) = self.history_state.selected() {
                                let entry = self.history_results.get(idx).cloned();
                                if let Some(e) = entry {
                                    self.play_history_video(&e);
                                }
                            }
                        }
                        KeyCode::Esc => {
                            self.mode = AppMode::Main;
                            self.search_query.clear();
                        }
                        _ => {}
                    },
                    AppMode::Saved => match key.code {
                        KeyCode::Char('s') => {
                            if let Some(idx) = self.saved_state.selected() {
                                if let Some(video) = self.saved_results.get(idx) {
                                    if let Err(e) = self.db.unsave_video(&video.video_id) {
                                        self.set_error(format!("Failed to unsave video: {}", e));
                                    } else {
                                        self.saved_results =
                                            self.db.get_saved_videos().unwrap_or_default();
                                        self.saved_state = ListState::default();
                                        self.saved_state.select(Some(0));
                                    }
                                }
                            }
                        }
                        KeyCode::Tab => {
                            self.saved_focus = match self.saved_focus {
                                ListFocus::Input => ListFocus::List,
                                ListFocus::List => ListFocus::Input,
                            };
                        }
                        KeyCode::Char(c) => {
                            if self.saved_focus == ListFocus::Input {
                                self.search_query.push(c);
                            } else if c == 'd' {
                                self.handle_download_shortcut();
                            }
                        }
                        KeyCode::Backspace => {
                            if self.saved_focus == ListFocus::Input {
                                self.search_query.pop();
                            }
                        }
                        KeyCode::Up => {
                            let i = match self.saved_state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        self.saved_results.len().saturating_sub(1)
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            self.saved_state.select(Some(i));
                        }
                        KeyCode::Down => {
                            let i = match self.saved_state.selected() {
                                Some(i) => (i + 1) % self.saved_results.len().max(1),
                                None => 0,
                            };
                            self.saved_state.select(Some(i));
                        }
                        KeyCode::Enter => {
                            if let Some(idx) = self.saved_state.selected() {
                                let video = self.saved_results.get(idx).cloned();
                                if let Some(v) = video {
                                    self.play_saved_video(&v);
                                }
                            }
                        }
                        KeyCode::Esc => {
                            self.mode = AppMode::Main;
                            self.search_query.clear();
                        }
                        _ => {}
                    },
                    AppMode::Playlist => {
                        if let Some(prompt_mode) = self.playlist_prompt_mode {
                            match key.code {
                                KeyCode::Char(c) => {
                                    self.playlist_prompt.push(c);
                                }
                                KeyCode::Backspace => {
                                    self.playlist_prompt.pop();
                                }
                                KeyCode::Enter => {
                                    let prompt = self.playlist_prompt.clone();
                                    self.playlist_prompt_mode = None;
                                    self.playlist_prompt.clear();
                                    match prompt_mode {
                                        PlaylistPromptMode::New => {
                                            self.handle_create_playlist(prompt)
                                        }
                                        PlaylistPromptMode::Import => {
                                            self.handle_import_playlist(prompt)
                                        }
                                    }
                                }
                                KeyCode::Esc => {
                                    self.playlist_prompt_mode = None;
                                    self.playlist_prompt.clear();
                                }
                                _ => {
                                    let _ = key;
                                }
                            }
                        } else {
                            match key.code {
                                KeyCode::Char('n') => {
                                    self.playlist_prompt_mode = Some(PlaylistPromptMode::New);
                                    self.playlist_prompt.clear();
                                }
                                KeyCode::Char('i') => {
                                    self.playlist_prompt_mode = Some(PlaylistPromptMode::Import);
                                    self.playlist_prompt.clear();
                                }
                                KeyCode::Char('r') => {
                                    if let Some(idx) = self.playlist_state.selected() {
                                        if let Some(playlist) =
                                            self.playlist_results.get(idx).cloned()
                                        {
                                            if playlist.is_imported {
                                                self.handle_refresh_playlist(&playlist);
                                            }
                                        }
                                    }
                                }
                                KeyCode::Tab => {
                                    self.playlist_focus = match self.playlist_focus {
                                        ListFocus::Input => ListFocus::List,
                                        ListFocus::List => ListFocus::Input,
                                    };
                                }

                                KeyCode::Char('d') => self.handle_download_shortcut(),
                                _ if key.modifiers.contains(KeyModifiers::SHIFT | KeyModifiers::CONTROL)
                                    && key.code == KeyCode::Char('X') =>
                                {
                                    self.autoplay_enabled = !self.autoplay_enabled;
                                    if !self.autoplay_enabled {
                                        self.is_playing = false;
                                    }
                                }
                                KeyCode::Char('x') => {
                                    if !self.playlist_videos.is_empty() {
                                        if let Some(idx) = self.playlist_videos_state.selected() {
                                            if let Some(video) =
                                                self.playlist_videos.get(idx).cloned()
                                            {
                                                self.handle_remove_from_playlist(&video);
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char('D') => {
                                    if self.playlist_videos.is_empty() {
                                        if let Some(idx) = self.playlist_state.selected() {
                                            if let Some(playlist) =
                                                self.playlist_results.get(idx).cloned()
                                            {
                                                self.handle_delete_playlist(&playlist);
                                            }
                                        }
                                    }
                                }

                                KeyCode::Char(c) => {
                                    if self.search_focus == SearchFocus::Input {
                                        self.search_query.push(c);
                                    }
                                }
                                KeyCode::Backspace => {
                                    if self.search_focus == SearchFocus::Input {
                                        self.search_query.pop();
                                    }
                                }
                                KeyCode::Up => {
                                    if !self.playlist_videos.is_empty() {
                                        self.scroll_playlist_videos_up();
                                    } else {
                                        self.scroll_playlist_up();
                                    }
                                }
                                KeyCode::Down => {
                                    if !self.playlist_videos.is_empty() {
                                        self.scroll_playlist_videos_down();
                                    } else {
                                        self.scroll_playlist_down();
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Some((video_id, title, channel)) =
                                        self.pending_playlist_add.take()
                                    {
                                        if let Some(idx) = self.playlist_state.selected() {
                                            if let Some(playlist) = self.playlist_results.get(idx) {
                                                let db = self.db.clone();
                                                let tx = self.playlist_add_tx.clone();
                                                let video_id = video_id.clone();
                                                let title = title.clone();
                                                let channel = channel.clone();
                                                let playlist_id = playlist.id;
                                                tokio::spawn(async move {
                                                    match db.add_to_playlist(
                                                        playlist_id,
                                                        &video_id,
                                                        &title,
                                                        channel.as_deref(),
                                                    ) {
                                                        Ok(_) => {
                                                            let _ = tx
                                                                .send(PlaylistAddResponse::Success(
                                                                    video_id,
                                                                ))
                                                                .await;
                                                        }
                                                        Err(e) => {
                                                            let _ = tx
                                                                .send(PlaylistAddResponse::Error(
                                                                    e.to_string(),
                                                                ))
                                                                .await;
                                                        }
                                                    }
                                                });
                                            }
                                        } else {
                                            self.set_error("No playlist selected");
                                        }
                                    } else if !self.playlist_videos.is_empty() {
                                        if let Some(idx) = self.playlist_videos_state.selected() {
                                            let video = self.playlist_videos.get(idx).cloned();
                                            if let Some(v) = video {
                                                self.play_playlist_video(&v);
                                            }
                                        }
                                    } else if let Some(idx) = self.playlist_state.selected() {
                                        self.load_playlist_videos(idx);
                                    }
                                }

                                KeyCode::Esc => {
                                    if !self.playlist_videos.is_empty() {
                                        self.playlist_videos.clear();
                                        self.playlist_videos_state = ListState::default();
                                    } else {
                                        self.mode = AppMode::Main;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                },
                Event::Mouse(mouse) if self.mode == AppMode::Main => {
                    self.handle_mouse_event(mouse);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn scroll_history_up(&mut self) {
        let i = match self.history_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.history_results.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.history_state.select(Some(i));
    }

    fn scroll_history_down(&mut self) {
        let i = match self.history_state.selected() {
            Some(i) => (i + 1) % self.history_results.len().max(1),
            None => 0,
        };
        self.history_state.select(Some(i));
    }

    fn play_history_video(&mut self, entry: &crate::db::connection::HistoryEntry) {
        self.begin_playback(&entry.title);
        
        if let Some(idx) = self.history_state.selected() {
            self.last_played_category = Some("history".to_string());
            self.last_played_index = Some(idx);
            if idx + 1 < self.history_results.len() {
                let next_entry = &self.history_results[idx + 1];
                let next_url = format!("https://www.youtube.com/watch?v={}", next_entry.video_id);
                let player = self.player.clone();
                tokio::spawn(async move {
                    let _ = player.queue_video(&next_url).await;
                });
            }
        }
        
        let player = self.player.clone();
        let db = self.db.clone();
        let url = format!("https://www.youtube.com/watch?v={}", entry.video_id);
        let video_id = entry.video_id.clone();
        let title = entry.title.clone();
        let channel = entry.channel.clone();
        let quality = self.settings.default_quality.clone();
        let format = self.settings.default_format.clone();
        let loop_playback = self.settings.loop_playback;
        tokio::spawn(async move {
            let _ = db.add_to_history(&video_id, &title, channel.as_deref());
            let _ = player.play(&url, &quality, &format, loop_playback, &[]).await;
        });
    }

    fn play_saved_video(&mut self, video: &crate::db::connection::SavedVideo) {
        self.begin_playback(&video.title);
        
        if let Some(idx) = self.saved_state.selected() {
            self.last_played_category = Some("saved".to_string());
            self.last_played_index = Some(idx);
            if idx + 1 < self.saved_results.len() {
                let next_video = &self.saved_results[idx + 1];
                let next_url = format!("https://www.youtube.com/watch?v={}", next_video.video_id);
                let player = self.player.clone();
                tokio::spawn(async move {
                    let _ = player.queue_video(&next_url).await;
                });
            }
        }
        
        let player = self.player.clone();
        let db = self.db.clone();
        let url = format!("https://www.youtube.com/watch?v={}", video.video_id);
        let video_id = video.video_id.clone();
        let title = video.title.clone();
        let channel = video.channel.clone();
        let quality = self.settings.default_quality.clone();
        let format = self.settings.default_format.clone();
        let loop_playback = self.settings.loop_playback;
        tokio::spawn(async move {
            let _ = db.add_to_history(&video_id, &title, channel.as_deref());
            let _ = player.play(&url, &quality, &format, loop_playback, &[]).await;
        });
    }

    fn play_search_video(&mut self, video: &crate::api::invidious::Video) {
        self.begin_playback(&video.title);
        
        if let Some(idx) = self.list_state.selected() {
            self.last_played_category = Some("search".to_string());
            self.last_played_index = Some(idx);
            if idx + 1 < self.search_results.len() {
                let next_video = &self.search_results[idx + 1];
                let next_url = format!("https://www.youtube.com/watch?v={}", next_video.video_id);
                let player = self.player.clone();
                tokio::spawn(async move {
                    let _ = player.queue_video(&next_url).await;
                });
            }
        }
        
        let player = self.player.clone();
        let db = self.db.clone();
        let url = format!("https://www.youtube.com/watch?v={}", video.video_id);
        let video_id = video.video_id.clone();
        let title = video.title.clone();
        let channel = video.author.clone();
        let quality = self.settings.default_quality.clone();
        let format = self.settings.default_format.clone();
        let loop_playback = self.settings.loop_playback;
        tokio::spawn(async move {
            let _ = db.add_to_history(&video_id, &title, channel.as_deref());
            let _ = player.play(&url, &quality, &format, loop_playback, &[]).await;
        });
    }

    fn play_main_video(&mut self, video_title: &str) {
        let player = self.player.clone();
        let db = self.db.clone();
        let title = video_title.to_string();
        let quality = self.settings.default_quality.clone();
        let format = self.settings.default_format.clone();
        let loop_playback = self.settings.loop_playback;
        
        if let Some(idx) = self.list_state.selected() {
            self.last_played_category = Some("main".to_string());
            self.last_played_index = Some(idx);
            if idx + 1 < self.items.len() {
                let next_title = &self.items[idx + 1];
                let next_id = match next_title.as_str() {
                    "Video 1: Rust for Beginners" => "S_S_S_S_S1_",
                    "Video 2: Advanced Ratatui Patterns" => "S_S_S_S_S2_",
                    "Video 3: Async Programming in Rust" => "S_S_S_S_S3_",
                    "Video 4: Building a TUI with Mouse Support" => "S_S_S_S_S4_",
                    "Video 5: YouTube API Integration" => "S_S_S_S_S5_",
                    "Video 6: SQLite Persistence" => "S_S_S_S_S6_",
                    "Video 7: Theme Systems in TUIs" => "S_S_S_S_S7_",
                    "Video 8: Error Handling Best Practices" => "S_S_S_S_S8_",
                    _ => "dQw4w9WgXcQ",
                };
                let next_url = format!("https://www.youtube.com/watch?v={}", next_id);
                let player = self.player.clone();
                tokio::spawn(async move {
                    let _ = player.queue_video(&next_url).await;
                });
            }
        }

        self.begin_playback(&title);

        let (video_id, channel) = match video_title {
            "Video 1: Rust for Beginners" => ("S_S_S_S_S1_", "Rust Lang"),
            "Video 2: Advanced Ratatui Patterns" => ("S_S_S_S_S2_", "Ratatui"),
            "Video 3: Async Programming in Rust" => ("S_S_S_S_S3_", "Tokio"),
            "Video 4: Building a TUI with Mouse Support" => ("S_S_S_S_S4_", "TUI Dev"),
            "Video 5: YouTube API Integration" => ("S_S_S_S_S5_", "API Guide"),
            "Video 6: SQLite Persistence" => ("S_S_S_S_S6_", "DB Tips"),
            "Video 7: Theme Systems in TUIs" => ("S_S_S_S_S7_", "Design"),
            "Video 8: Error Handling Best Practices" => ("S_S_S_S_S8_", "Rust Tips"),
            _ => ("dQw4w9WgXcQ", "Rick Astley"),
        };

        let url = format!("https://www.youtube.com/watch?v={}", video_id);
        let video_id = video_id.to_string();
        let channel = Some(channel.to_string());

        tokio::spawn(async move {
            let _ = db.add_to_history(&video_id, &title, channel.as_deref());
            let _ = player.play(&url, &quality, &format, loop_playback, &[]).await;
        });
    }

    fn handle_create_playlist(&mut self, prompt: String) {
        if prompt.trim().is_empty() {
            self.set_error("Playlist name cannot be empty");
            return;
        }
        match self.db.create_playlist(&prompt) {
            Ok(_) => {
                self.playlist_results = self.db.get_playlists().unwrap_or_default();
                self.playlist_state = ListState::default();
                self.playlist_state.select(Some(0));
            }
            Err(e) => self.set_error(format!("Failed to create playlist: {}", e)),
        }
    }

    fn handle_import_playlist(&mut self, prompt: String) {
        if prompt.trim().is_empty() {
            self.set_error("Playlist URL/ID cannot be empty");
            return;
        }

        let playlist_id = if prompt.contains("list=") {
            prompt
                .split("list=")
                .nth(1)
                .and_then(|s| s.split('&').next())
                .map(|s| s.to_string())
        } else {
            Some(prompt.clone())
        };

        let playlist_id = match playlist_id {
            Some(id) => id,
            None => {
                self.set_error("Invalid playlist URL or ID");
                return;
            }
        };

        let api_url = self.settings.api_instance_invidious.clone();
        let tx = self.playlist_tx.clone();
        let settings_tx = self.settings_tx.clone();
        let db = self.db.clone();

        tokio::spawn(async move {
            let mut current_url = api_url.clone();
            let mut attempts = 0;
            loop {
                let client = crate::api::invidious::InvidiousClient::new(&current_url);
                match client.get_playlist(&playlist_id).await {
                    Ok(details) => {
                        match db.create_imported_playlist(&details.title, &playlist_id) {
                            Ok(id) => {
                                for v in &details.videos {
                                    let _ = db.sync_imported_playlist(
                                        id,
                                        &v.video_id,
                                        &v.title,
                                        v.author.as_deref(),
                                    );
                                }
                                let _ = tx
                                    .send(PlaylistResponse::Imported {
                                        id,
                                        videos: details.videos,
                                    })
                                    .await;
                            }
                            Err(e) => {
                                let _ = tx
                                    .send(PlaylistResponse::Error(format!("DB Error: {}", e)))
                                    .await;
                            }
                        }
                        if current_url != api_url {
                            let _ = settings_tx.send(current_url).await;
                        }
                        break;
                    }
                    Err(crate::api::invidious::InvidiousError::BadInstance 
                        | crate::api::invidious::InvidiousError::RequestFailed(_))
                        if attempts < crate::api::health::instances().len() =>
                    {
                        current_url = crate::api::health::next_instance(&current_url).to_string();
                        attempts += 1;
                    }
                    Err(e) => {
                        let _ = tx.send(PlaylistResponse::Error(e.to_string())).await;
                        break;
                    }
                }
            }
        });
    }

    fn scroll_playlist_videos_up(&mut self) {
        let i = match self.playlist_videos_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.playlist_videos.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.playlist_videos_state.select(Some(i));
    }

    fn scroll_playlist_up(&mut self) {
        let i = match self.playlist_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.playlist_results.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.playlist_state.select(Some(i));
    }

    fn scroll_playlist_videos_down(&mut self) {
        let i = match self.playlist_videos_state.selected() {
            Some(i) => (i + 1) % self.playlist_videos.len().max(1),
            None => 0,
        };
        self.playlist_videos_state.select(Some(i));
    }

    fn scroll_playlist_down(&mut self) {
        let i = match self.playlist_state.selected() {
            Some(i) => (i + 1) % self.playlist_results.len().max(1),
            None => 0,
        };
        self.playlist_state.select(Some(i));
    }

    fn play_playlist_video(&mut self, video: &crate::db::connection::PlaylistVideo) {
        self.begin_playback(&video.title);
        
        if let Some(idx) = self.playlist_videos_state.selected() {
            self.last_played_category = Some("playlist".to_string());
            self.last_played_index = Some(idx);
            if idx + 1 < self.playlist_videos.len() {
                let next_video = &self.playlist_videos[idx + 1];
                let next_url = format!("https://www.youtube.com/watch?v={}", next_video.video_id);
                let player = self.player.clone();
                tokio::spawn(async move {
                    let _ = player.queue_video(&next_url).await;
                });
            }
        }
        
        let player = self.player.clone();
        let db = self.db.clone();
        let url = format!("https://www.youtube.com/watch?v={}", video.video_id);
        let video_id = video.video_id.clone();
        let title = video.title.clone();
        let channel = video.channel.clone();
        let quality = self.settings.default_quality.clone();
        let format = self.settings.default_format.clone();
        let loop_playback = self.settings.loop_playback;
        tokio::spawn(async move {
            let _ = db.add_to_history(&video_id, &title, channel.as_deref());
            let _ = player.play(&url, &quality, &format, loop_playback, &[]).await;
        });
    }

    fn load_playlist_videos(&mut self, idx: usize) {
        if let Some(playlist) = self.playlist_results.get(idx) {
            match self.db.get_playlist_videos(playlist.id) {
                Ok(videos) => {
                    self.playlist_videos = videos;
                    self.playlist_videos_state = ListState::default();
                    if !self.playlist_videos.is_empty() {
                        self.playlist_videos_state.select(Some(0));
                    }
                }
                Err(e) => self.set_error(format!("Failed to load playlist videos: {}", e)),
            }
        }
    }

    fn handle_refresh_playlist(&mut self, playlist: &crate::db::connection::Playlist) {
        if playlist.youtube_id.is_none() {
            self.set_error("Only imported playlists can be refreshed");
            return;
        }

        let youtube_id = match playlist.youtube_id.clone() {
            Some(id) => id,
            None => {
                self.set_error("Playlist has no YouTube ID");
                return;
            }
        };
        let playlist_id = playlist.id;
        let api_url = self.settings.api_instance_invidious.clone();
        let tx = self.playlist_tx.clone();
        let settings_tx = self.settings_tx.clone();
        let db = self.db.clone();

        tokio::spawn(async move {
            let mut current_url = api_url.clone();
            let mut attempts = 0;
            loop {
                let client = crate::api::invidious::InvidiousClient::new(&current_url);
                match client.get_playlist(&youtube_id).await {
                    Ok(details) => {
                        match db.clear_imported_playlist(playlist_id) {
                            Ok(_) => {
                                for v in &details.videos {
                                    let _ = db.sync_imported_playlist(
                                        playlist_id,
                                        &v.video_id,
                                        &v.title,
                                        v.author.as_deref(),
                                    );
                                }
                                let _ = tx
                                    .send(PlaylistResponse::Refreshed {
                                        id: playlist_id,
                                        videos: details.videos,
                                    })
                                    .await;
                            }
                            Err(e) => {
                                let _ = tx
                                    .send(PlaylistResponse::Error(format!("DB Error: {}", e)))
                                    .await;
                            }
                        }
                        if current_url != api_url {
                            let _ = settings_tx.send(current_url).await;
                        }
                        break;
                    }
                    Err(crate::api::invidious::InvidiousError::BadInstance 
                        | crate::api::invidious::InvidiousError::RequestFailed(_))
                        if attempts < crate::api::health::instances().len() =>
                    {
                        current_url = crate::api::health::next_instance(&current_url).to_string();
                        attempts += 1;
                    }
                    Err(e) => {
                        let _ = tx.send(PlaylistResponse::Error(e.to_string())).await;
                        break;
                    }
                }
            }
        });
    }

    fn handle_remove_from_playlist(&mut self, video: &crate::db::connection::PlaylistVideo) {
        match self.db.remove_from_playlist(video.playlist_id, video.id) {
            Ok(_) => {
                if let Ok(videos) = self.db.get_playlist_videos(video.playlist_id) {
                    self.playlist_videos = videos;
                    if let Some(selected) = self.playlist_videos_state.selected() {
                        if selected >= self.playlist_videos.len() {
                            self.playlist_videos_state
                                .select(Some(self.playlist_videos.len().saturating_sub(1)));
                        }
                    }
                }
            }
            Err(e) => self.set_error(format!("Failed to remove video from playlist: {}", e)),
        }
    }

    fn handle_delete_playlist(&mut self, playlist: &crate::db::connection::Playlist) {
        match self.db.delete_playlist(playlist.id) {
            Ok(_) => {
                self.playlist_results = self.db.get_playlists().unwrap_or_default();
                self.playlist_state = ListState::default();
                self.playlist_state.select(Some(0));
                self.playlist_videos.clear();
                self.playlist_videos_state = ListState::default();
            }
            Err(e) => self.set_error(format!("Failed to delete playlist: {}", e)),
        }
    }

    fn trigger_search(&mut self) {
        let query = self.search_query.clone();
        let api_url = self.settings.api_instance_invidious.clone();
        let tx = self.search_tx.clone();
        let settings_tx = self.settings_tx.clone();

        self.is_searching = true;
        self.search_error = None;

        tokio::spawn(async move {
            let mut current_url = api_url.clone();
            let mut attempts = 0;
            loop {
                let client = crate::api::invidious::InvidiousClient::new(&current_url);
                match client.search(&query).await {
                    Ok(results) => {
                        let _ = tx.send(SearchResponse::Success(results)).await;
                        if current_url != api_url {
                            let _ = settings_tx.send(current_url).await;
                        }
                        break;
                    }
                    Err(crate::api::invidious::InvidiousError::BadInstance 
                        | crate::api::invidious::InvidiousError::RequestFailed(_))
                        if attempts < crate::api::health::instances().len() =>
                    {
                        current_url = crate::api::health::next_instance(&current_url).to_string();
                        attempts += 1;
                    }
                    Err(e) => {
                        let _ = tx.send(SearchResponse::Error(e.to_string())).await;
                        break;
                    }
                }
            }
        });
    }

    #[allow(dead_code)]
    fn get_selected_video_id(&self) -> Option<String> {
        match self.mode {
            AppMode::Main => None,
            AppMode::Search => self
                .list_state
                .selected()
                .and_then(|idx| self.search_results.get(idx))
                .map(|v| v.video_id.clone()),
            AppMode::History => self
                .history_state
                .selected()
                .and_then(|idx| self.history_results.get(idx))
                .map(|v| v.video_id.clone()),
            AppMode::Saved => self
                .saved_state
                .selected()
                .and_then(|idx| self.saved_results.get(idx))
                .map(|v| v.video_id.clone()),
            AppMode::Playlist => {
                if !self.playlist_videos.is_empty() {
                    self.playlist_videos_state
                        .selected()
                        .and_then(|idx| self.playlist_videos.get(idx))
                        .map(|v| v.video_id.clone())
                } else {
                    None
                }
            }
            AppMode::Settings => None,
            AppMode::Downloads => None,
        }
    }

    fn get_selected_video_meta(&self) -> Option<(String, String)> {
        match self.mode {
            AppMode::Search => self
                .list_state
                .selected()
                .and_then(|idx| self.search_results.get(idx))
                .map(|v| (v.video_id.clone(), v.title.clone())),
            AppMode::History => self
                .history_state
                .selected()
                .and_then(|idx| self.history_results.get(idx))
                .map(|v| (v.video_id.clone(), v.title.clone())),
            AppMode::Saved => self
                .saved_state
                .selected()
                .and_then(|idx| self.saved_results.get(idx))
                .map(|v| (v.video_id.clone(), v.title.clone())),
            AppMode::Playlist => {
                if !self.playlist_videos.is_empty() {
                    self.playlist_videos_state
                        .selected()
                        .and_then(|idx| self.playlist_videos.get(idx))
                        .map(|v| (v.video_id.clone(), v.title.clone()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn trigger_download(&mut self, video_id: String, title: String, audio_only: bool) {
        self.current_error = None;
        self.download_is_audio = audio_only;
        self.download_bar_state = DownloadBarState::Active;
        self.download_bar_title = title;
        self.download_bar_percent = 0;
        self.download_bar_speed = String::new();
        self.download_bar_eta = String::new();

        let download_path = self.settings.download_path.clone();
        let tx = self.download_tx.clone();

        tokio::spawn(async move {
            let mut rx = Downloader::download(&video_id, &download_path, audio_only);
            while let Some(progress) = rx.recv().await {
                let _ = tx.send(progress).await;
            }
        });
    }

    fn handle_download_shortcut(&mut self) {
        if let Some((video_id, title)) = self.get_selected_video_meta() {
            self.pending_download_id = Some(video_id);
            self.pending_download_title = Some(title);
            self.show_download_menu = true;
        } else {
            self.set_error("No video selected for download");
        }
    }

    fn trigger_save(&mut self) {
        let metadata = match self.mode {
            AppMode::Search => self
                .list_state
                .selected()
                .and_then(|idx| self.search_results.get(idx))
                .map(|v| (v.video_id.clone(), v.title.clone(), v.author.clone())),
            AppMode::History => self
                .history_state
                .selected()
                .and_then(|idx| self.history_results.get(idx))
                .map(|v| (v.video_id.clone(), v.title.clone(), v.channel.clone())),
            AppMode::Saved => self
                .saved_state
                .selected()
                .and_then(|idx| self.saved_results.get(idx))
                .map(|v| (v.video_id.clone(), v.title.clone(), v.channel.clone())),
            AppMode::Playlist => {
                if !self.playlist_videos.is_empty() {
                    self.playlist_videos_state
                        .selected()
                        .and_then(|idx| self.playlist_videos.get(idx))
                        .map(|v| (v.video_id.clone(), v.title.clone(), v.channel.clone()))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some((video_id, title, channel)) = metadata {
            let db = self.db.clone();
            let tx = self.saved_tx.clone();
            tokio::spawn(async move {
                match db.save_video(&video_id, &title, channel.as_deref()) {
                    Ok(_) => {
                        let _ = tx.send(SavedResponse::Success(video_id)).await;
                    }
                    Err(e) => {
                        let _ = tx.send(SavedResponse::Error(e.to_string())).await;
                    }
                }
            });
        } else {
            self.set_error("No video selected to save");
        }
    }

    fn trigger_add_to_playlist(&mut self) {
        let metadata = match self.mode {
            AppMode::Search => self
                .list_state
                .selected()
                .and_then(|idx| self.search_results.get(idx))
                .map(|v| (v.video_id.clone(), v.title.clone(), v.author.clone())),
            AppMode::History => self
                .history_state
                .selected()
                .and_then(|idx| self.history_results.get(idx))
                .map(|v| (v.video_id.clone(), v.title.clone(), v.channel.clone())),
            AppMode::Saved => self
                .saved_state
                .selected()
                .and_then(|idx| self.saved_results.get(idx))
                .map(|v| (v.video_id.clone(), v.title.clone(), v.channel.clone())),
            AppMode::Playlist => {
                if !self.playlist_videos.is_empty() {
                    self.playlist_videos_state
                        .selected()
                        .and_then(|idx| self.playlist_videos.get(idx))
                        .map(|v| (v.video_id.clone(), v.title.clone(), v.channel.clone()))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some((video_id, title, channel)) = metadata {
            if self.mode == AppMode::Playlist && !self.playlist_videos.is_empty() {
                if let Some(playlist_idx) = self.playlist_state.selected() {
                    if let Some(playlist) = self.playlist_results.get(playlist_idx) {
                        let db = self.db.clone();
                        let tx = self.playlist_add_tx.clone();
                        let video_id = video_id.clone();
                        let title = title.clone();
                        let channel = channel.clone();
                        let playlist_id = playlist.id;
                        tokio::spawn(async move {
                            match db.add_to_playlist(
                                playlist_id,
                                &video_id,
                                &title,
                                channel.as_deref(),
                            ) {
                                Ok(_) => {
                                    let _ = tx.send(PlaylistAddResponse::Success(video_id)).await;
                                }
                                Err(e) => {
                                    let _ =
                                        tx.send(PlaylistAddResponse::Error(e.to_string())).await;
                                }
                            }
                        });
                        return;
                    }
                }
            }

            self.pending_playlist_add = Some((video_id, title, channel));
            self.mode = AppMode::Playlist;
            self.playlist_results = self.db.get_playlists().unwrap_or_default();
            self.playlist_state = ListState::default();
            self.playlist_state.select(Some(0));
            self.playlist_videos.clear();
            self.playlist_videos_state = ListState::default();
        } else {
            self.set_error("No video selected to add to playlist");
        }
    }

    fn handle_mouse_event(&mut self, mouse: event::MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let x = mouse.column;
                let y = mouse.row;

                if self.show_context_menu {
                    let menu_area =
                        Rect::new(self.context_menu_pos.0, self.context_menu_pos.1, 20, 5);
                    if menu_area.contains(ratatui::layout::Position::new(x, y)) {
                        let relative_y = y.saturating_sub(self.context_menu_pos.1 + 1);
                        match relative_y {
                            0 => { /* Play */ }
                            1 => {
                                self.trigger_save();
                            }
                            2 => {
                                if let Some((video_id, title)) = self.get_selected_video_meta() {
                                    self.pending_download_id = Some(video_id);
                                    self.pending_download_title = Some(title);
                                    self.show_download_menu = true;
                                } else {
                                    self.set_error("No video selected for download");
                                }
                            }
                            3 => {
                                self.trigger_add_to_playlist();
                            }
                            4 => { /* Cancel */ }
                            _ => {}
                        }

                        self.show_context_menu = false;
                        return;
                    }
                }
                self.show_context_menu = false;

                if self
                    .sidebar_area
                    .contains(ratatui::layout::Position::new(x, y))
                {
                    self.active_block = ActiveBlock::Sidebar;
                    let relative_y = y.saturating_sub(self.sidebar_area.y + 1);
                    if relative_y < self.sidebar_items.len() as u16 {
                        self.sidebar_state.select(Some(relative_y as usize));
                    }
                } else if self
                    .content_area
                    .contains(ratatui::layout::Position::new(x, y))
                {
                    self.active_block = ActiveBlock::Content;
                    let relative_y = y.saturating_sub(self.content_area.y + 1);
                    if relative_y < self.items.len() as u16 {
                        self.list_state.select(Some(relative_y as usize));
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Right) => {
                self.show_context_menu = true;
                self.context_menu_pos = (mouse.column, mouse.row);
            }
            MouseEventKind::ScrollUp => {
                self.scroll_up();
            }
            MouseEventKind::ScrollDown => {
                self.scroll_down();
            }
            _ => {}
        }
    }

    fn scroll_up(&mut self) {
        if self.active_block == ActiveBlock::Sidebar {
            let i = match self.sidebar_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.sidebar_items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.sidebar_state.select(Some(i));
        } else {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }

    fn scroll_down(&mut self) {
        if self.active_block == ActiveBlock::Sidebar {
            let i = match self.sidebar_state.selected() {
                Some(i) => (i + 1) % self.sidebar_items.len(),
                None => 0,
            };
            self.sidebar_state.select(Some(i));
        } else {
            let i = match self.list_state.selected() {
                Some(i) => (i + 1) % self.items.len(),
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }

    pub fn set_error(&mut self, error: impl Into<String>) {
        self.current_error = Some(error.into());
        self.current_suggestion = None;
    }

    pub fn set_error_with_suggestion(
        &mut self,
        error: impl Into<String>,
        suggestion: impl Into<String>,
    ) {
        self.current_error = Some(error.into());
        self.current_suggestion = Some(suggestion.into());
    }

    pub fn clear_error(&mut self) {
        self.current_error = None;
        self.current_suggestion = None;
    }

    pub fn set_loading(&mut self, message: impl Into<String>) {
        self.is_loading = true;
        self.loading_message = message.into();
    }

    pub fn clear_loading(&mut self) {
        self.is_loading = false;
        self.loading_message.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn current_view(&self) -> &str {
        self.sidebar_items
            .get(self.sidebar_state.selected().unwrap_or(0))
            .map(|s| s.as_str())
            .unwrap_or("Search")
    }

    fn render_error_overlay(&mut self, f: &mut ratatui::Frame) {
        if let Some(ref error) = self.current_error {
            let area = self.content_area;
            let suggestion = self.current_suggestion.as_deref();
            components::render_error(f, area, error, suggestion);
        }
    }

    #[allow(dead_code)]
    fn render_empty_state(&mut self, f: &mut ratatui::Frame) {
        if self.items.is_empty() && !self.is_loading && self.current_error.is_none() {
            let view = self.current_view();
            let (title, message, icon) = match view {
                "Search" => (
                    "No Results",
                    "Enter a search term to find videos",
                    Some("🔍"),
                ),
                "History" => (
                    "No History",
                    "Start watching videos to build your history",
                    Some("📜"),
                ),
                "Saved" => (
                    "No Saved Videos",
                    "Press 's' to save videos from search results",
                    Some("⭐"),
                ),
                "Playlists" => (
                    "No Playlists",
                    "Create a playlist to organize your videos",
                    Some("📋"),
                ),
                "Downloads" => (
                    "No Downloads",
                    "Download videos to watch offline",
                    Some("⬇️"),
                ),
                _ => ("Empty", "No items to display", None),
            };
            components::render_empty_state(f, self.content_area, &self.theme, title, message, icon);
        }
    }

    fn render_loading_overlay(&mut self, f: &mut ratatui::Frame) {
        if self.is_loading {
            components::render_loading(f, self.content_area, &self.loading_message);
        }
    }

    fn render_download_menu(&self, f: &mut ratatui::Frame) {
        use ratatui::style::Modifier;
        let area = self.content_area;
        let w = 44u16.min(area.width.saturating_sub(4));
        let h = 9u16;
        let popup = Rect {
            x: area.x + (area.width.saturating_sub(w)) / 2,
            y: area.y + (area.height.saturating_sub(h)) / 2,
            width: w,
            height: h,
        };

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled("  Download as", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("   [1] ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("🎬 Video   "),
                Span::styled("(mp4, best quality)", Style::default().add_modifier(Modifier::DIM)),
            ]),
            Line::from(vec![
                Span::styled("   [2] ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("🎵 Audio   "),
                Span::styled("(mp3, audio only)", Style::default().add_modifier(Modifier::DIM)),
            ]),
            Line::from(""),
            Line::from(Span::styled("   1/v video · 2/a audio · Esc cancel", Style::default().add_modifier(Modifier::DIM))),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().add_modifier(Modifier::BOLD))
            .title(" Download ");
        let para = Paragraph::new(lines).block(block);
        f.render_widget(Clear, popup);
        f.render_widget(para, popup);
    }

    fn play_downloaded(&mut self, rec: &DownloadRecord) {
        if !std::path::Path::new(&rec.path).exists() {
            self.set_error(format!("File not found: {}", rec.path));
            return;
        }
        let is_audio = rec.kind == "audio";
        self.begin_playback(&rec.title);
        self.playback_is_audio = is_audio;

        let player = self.player.clone();
        let path = rec.path.clone();
        let quality = self.settings.default_quality.clone();
        let loop_playback = self.settings.loop_playback;
        tokio::spawn(async move {
            if is_audio {
                let _ = player.play_audio(&path, &quality, loop_playback, &[]).await;
            } else {
                let _ = player.play(&path, &quality, "mp4", loop_playback, &[]).await;
            }
        });
    }

    fn begin_playback(&mut self, title: &str) {
        self.is_playing = true;
        self.is_paused = false;
        self.loading_message = format!("Playing: {}...", title);
        self.playback_title = title.to_string();
        self.playback_state = PlaybackState::Loading;
        self.playback_started = Some(std::time::Instant::now());
        self.playback_error = None;
        self.playback_is_audio = self.settings.default_format == "mp3";
    }

    fn render_playback_bar(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        use ratatui::style::Modifier;
        const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let spin = SPINNER[self.anim_tick as usize % SPINNER.len()];
        let media = if self.playback_is_audio { "♪" } else { "▶" };
        let title = marquee(&self.playback_title, 40, self.anim_tick);

        let text = match self.playback_state {
            PlaybackState::Loading => format!(" {} Loading {} {}…", spin, media, title),
            PlaybackState::Playing => {
                if self.is_paused {
                    format!(" ⏸ {} paused  ·  Space resume · ←/→ seek · </> prev/next", title)
                } else {
                    let secs = self.playback_started.map(|s| s.elapsed().as_secs()).unwrap_or(0);
                    format!(" {} {}  ·  {:02}:{:02}  ·  Space pause · ←/→ seek · </> prev/next",
                        media, title, secs / 60, secs % 60)
                }
            }
            PlaybackState::Error => {
                let e = self.playback_error.as_deref().unwrap_or("playback failed");
                let e = e.chars().take(50).collect::<String>();
                format!(" ✗ Playback error: {}", e)
            }
            PlaybackState::Idle => return,
        };

        let para = Paragraph::new(text)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(para, area);
    }

    fn render_download_bar(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::Gauge;

        const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let spin = SPINNER[self.anim_tick as usize % SPINNER.len()];

        let (label, pct) = match &self.download_bar_state {
            DownloadBarState::Active => {
                // Marquee-scroll the title if it is long, so the bar visibly "moves"
                let title = marquee(&self.download_bar_title, 40, self.anim_tick);
                let label = if self.download_bar_speed.is_empty() {
                    format!(" {} ▼ {}  ·  {}%", spin, title, self.download_bar_percent)
                } else {
                    format!(
                        " {} ▼ {}  ·  {}%  ·  {}  ·  ETA {}",
                        spin, title, self.download_bar_percent, self.download_bar_speed, self.download_bar_eta
                    )
                };
                (label, self.download_bar_percent as u16)
            }
            DownloadBarState::Done => {
                (format!(" ✓ {}  — saved", self.download_bar_title), 100)
            }
            DownloadBarState::Failed => {
                (" ✗ Download failed (see error)".to_string(), 0)
            }
            DownloadBarState::Idle => return,
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::TOP))
            .gauge_style(Style::default().add_modifier(ratatui::style::Modifier::BOLD))
            .percent(pct)
            .label(label);
        f.render_widget(gauge, area);
    }

    pub fn pick_download_path_with_yazi(&mut self) {
        // Launch yazi with --chooser-file to let user pick a directory
        let chooser_file = std::env::temp_dir().join("youtui_yazi_chooser");
        let _ = std::fs::remove_file(&chooser_file);

        let chooser_path = chooser_file.to_string_lossy().to_string();
        let status = std::process::Command::new("yazi")
            .arg("--chooser-file")
            .arg(&chooser_path)
            .status();

        match status {
            Ok(s) if s.success() => {
                if let Ok(chosen) = std::fs::read_to_string(&chooser_file) {
                    let path = std::path::PathBuf::from(chosen.trim());
                    if path.is_dir() {
                        self.settings.download_path = path.clone();
                        let _ = self.settings.save();
                        self.set_error(format!("Download path set to: {}", path.display()));
                    } else if path.parent().map(|p| p.is_dir()).unwrap_or(false) {
                        let dir = path.parent().unwrap().to_path_buf();
                        self.settings.download_path = dir.clone();
                        let _ = self.settings.save();
                        self.set_error(format!("Download path set to: {}", dir.display()));
                    }
                    let _ = std::fs::remove_file(&chooser_file);
                }
            }
            Ok(_) => {}
            Err(_) => {
                self.set_error("yazi not found. Install yazi to use the folder picker.");
            }
        }
    }
}

/// Scroll a string within a fixed width window. If the text fits, it is
/// returned padded; if longer, it scrolls left over time based on `tick`.
fn marquee(text: &str, width: usize, tick: u64) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= width {
        return text.to_string();
    }
    // Add a gap then wrap around for a continuous scroll.
    let gap = "   •   ";
    let mut ring: Vec<char> = chars.clone();
    ring.extend(gap.chars());
    let period = ring.len();
    // advance one char every 3 ticks (~ slower, readable)
    let start = (tick / 3) as usize % period;
    let window: String = (0..width)
        .map(|i| ring[(start + i) % period])
        .collect();
    window
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadRecord {
    pub title: String,
    pub path: String,
    pub kind: String,
    pub when: String,
}

fn downloads_file() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("youtui-rs").join("downloads.json"))
}

fn load_downloads() -> Vec<DownloadRecord> {
    if let Some(p) = downloads_file() {
        if let Ok(s) = std::fs::read_to_string(&p) {
            if let Ok(v) = serde_json::from_str(&s) {
                return v;
            }
        }
    }
    Vec::new()
}

fn save_downloads(records: &[DownloadRecord]) -> std::io::Result<()> {
    if let Some(p) = downloads_file() {
        if let Some(dir) = p.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let s = serde_json::to_string_pretty(records)?;
        std::fs::write(p, s)?;
    }
    Ok(())
}
