use crate::download::downloader::{DownloadProgress, Downloader};
use crate::error::AppError;
use crate::ui::components;
use crate::ui::theme::Theme;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, ListState, Padding, Paragraph},
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
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PlaylistPromptMode {
    New,
    Import,
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
    pub sidebar_area: Rect,
    pub content_area: Rect,
    pub current_error: Option<String>,
    pub current_suggestion: Option<String>,
    pub is_loading: bool,
    pub loading_message: String,
    pub startup_warnings: Vec<String>,
    pub search_query: String,
    pub search_results: Vec<crate::api::invidious::Video>,
    pub is_searching: bool,
    pub search_error: Option<String>,
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
    pub db: std::sync::Arc<crate::db::connection::Database>,
    pub player: std::sync::Arc<crate::player::mpv::MpvPlayer>,
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
        let (playback_ended_tx, _playback_ended_rx) = tokio::sync::mpsc::channel(1);
        let (notification_tx, _notification_rx) = tokio::sync::broadcast::channel(16);

        Ok(Self {
            mode: AppMode::Main,
            settings,
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
            sidebar_area: Rect::default(),
            content_area: Rect::default(),
            current_error: None,
            current_suggestion: None,
            is_loading: false,
            loading_message: String::new(),
            startup_warnings,
            search_query: String::new(),
            search_results: Vec::new(),
            is_searching: false,
            search_error: None,
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
            player: std::sync::Arc::new(crate::player::mpv::MpvPlayer::new(
                playback_ended_tx,
                notification_tx,
            )),
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
        while let Ok(response) = self.search_rx.try_recv() {
            self.is_searching = false;
            match response {
                SearchResponse::Success(results) => {
                    self.search_results = results;
                    self.search_error = None;
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
                        }
                    }
                }
                PlaylistResponse::Refreshed { id, .. } => {
                    if let Ok(videos) = self.db.get_playlist_videos(id) {
                        self.playlist_videos = videos;
                        self.playlist_videos_state = ListState::default();
                        if !self.playlist_videos.is_empty() {
                            self.playlist_videos_state.select(Some(0));
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
                DownloadProgress::Starting => {
                    self.is_loading = true;
                    self.loading_message = "Starting download...".to_string();
                }
                DownloadProgress::Downloading { percent, speed } => {
                    self.is_loading = true;
                    self.loading_message = format!("Downloading: {}% ({})...", percent, speed);
                }
                DownloadProgress::Completed { path } => {
                    self.is_loading = false;
                    self.loading_message = format!("Download completed: {}", path);
                }
                DownloadProgress::Failed { error } => {
                    self.is_loading = false;
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

    fn render(&mut self, f: &mut ratatui::Frame) {
        match self.mode {
            AppMode::Main => {
                self.render_main_view(f);
            }
            AppMode::Settings => {
                crate::ui::settings::render(f, self.content_area, self);
            }
            AppMode::Search => {
                self.render_search(f);
            }
            AppMode::History => {
                self.render_history(f);
            }
            AppMode::Saved => {
                self.render_saved(f);
            }
            AppMode::Playlist => {
                self.render_playlist(f);
            }
        }
    }

    fn render_main_view(&mut self, f: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(15), Constraint::Percentage(85)])
            .split(f.area());

        self.sidebar_area = chunks[0];
        self.content_area = chunks[1];

        let item_height = 6 + components::DesignTokens::ITEM_GAP;
        let sidebar_offset = self.sidebar_state.selected().unwrap_or(0);
        let sidebar_visible_count = (self.sidebar_area.height / item_height) as usize;

        for (i, item) in self
            .sidebar_items
            .iter()
            .enumerate()
            .skip(sidebar_offset)
            .take(sidebar_visible_count)
        {
            let rect = Rect::new(
                self.sidebar_area.x,
                self.sidebar_area.y + (i - sidebar_offset) as u16 * item_height,
                self.sidebar_area.width,
                6,
            );
            let is_selected = self.sidebar_state.selected() == Some(i);
            components::render_item_card(f, rect, item, "", &self.theme, is_selected, false);
        }

        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(self.content_area);

        components::render_header(
            f,
            content_chunks[0],
            "Videos",
            &format!("• {} items", self.items.len()),
            &self.theme,
        );

        components::render_divider(f, content_chunks[1], &self.theme, Direction::Horizontal);

        if self.items.is_empty() {
            components::render_empty_state(
                f,
                content_chunks[2],
                &self.theme,
                "Videos",
                "No videos available",
                Some("📺"),
            );
        } else {
            let content_offset = self.list_state.selected().unwrap_or(0);
            let content_visible_count = (content_chunks[2].height / item_height) as usize;

            for (i, item) in self
                .items
                .iter()
                .enumerate()
                .skip(content_offset)
                .take(content_visible_count)
            {
                let rect = Rect::new(
                    content_chunks[2].x,
                    content_chunks[2].y + (i - content_offset) as u16 * item_height,
                    content_chunks[2].width,
                    6,
                );
                let is_selected = self.list_state.selected() == Some(i);
                components::render_item_card(f, rect, item, "", &self.theme, is_selected, false);
            }
        }

        components::render_info_bar(
            f,
            content_chunks[3],
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
        } else if self.items.is_empty() {
            self.render_empty_state(f);
        }
    }

    fn render_history(&mut self, f: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        components::render_header(
            f,
            chunks[0],
            "History",
            &format!("• {} items", self.history_results.len()),
            &self.theme,
        );

        components::render_divider(f, chunks[1], &self.theme, Direction::Horizontal);

        if self.history_results.is_empty() {
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

            for (i, entry) in self
                .history_results
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

                components::render_item_card(
                    f,
                    rect,
                    &entry.title,
                    &meta,
                    &self.theme,
                    is_selected,
                    false,
                );
            }
        }

        components::render_info_bar(f, chunks[3], &[("Theme", &self.theme.name)], &self.theme);
    }

    fn render_saved(&mut self, f: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        components::render_header(
            f,
            chunks[0],
            "Saved",
            &format!("• {} items", self.saved_results.len()),
            &self.theme,
        );

        components::render_divider(f, chunks[1], &self.theme, Direction::Horizontal);

        if self.saved_results.is_empty() {
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

            for (i, video) in self
                .saved_results
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
                let meta = format!("{} • Saved {}", channel, video.saved_at);
                let is_selected = self.saved_state.selected() == Some(i);

                components::render_item_card(
                    f,
                    rect,
                    &video.title,
                    &meta,
                    &self.theme,
                    is_selected,
                    false,
                );
            }
        }

        components::render_info_bar(f, chunks[3], &[("Theme", &self.theme.name)], &self.theme);
    }

    fn render_playlist(&mut self, f: &mut ratatui::Frame) {
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
                .split(f.area());

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
            .split(f.area());

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

                components::render_item_card(
                    f,
                    rect,
                    &video.title,
                    &meta,
                    &self.theme,
                    is_selected,
                    false,
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

                components::render_item_card(
                    f,
                    rect,
                    &playlist.name,
                    meta,
                    &self.theme,
                    is_selected,
                    false,
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

    fn render_search(&mut self, f: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        let subtitle = if self.search_query.is_empty() {
            "Enter a search term...".to_string()
        } else {
            format!("Query: {}", self.search_query)
        };

        components::render_header(f, chunks[0], "Search", &subtitle, &self.theme);

        let input = Paragraph::new(self.search_query.as_str())
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(self.theme.border)
                    .padding(Padding::uniform(components::DesignTokens::PADDING_MD)),
            )
            .style(Style::default().fg(self.theme.foreground));
        f.render_widget(input, chunks[1]);

        if self.is_searching {
            components::render_loading(f, chunks[2], "Searching Invidious...");
        } else if let Some(ref error) = self.search_error {
            components::render_error(f, chunks[2], error, None);
        } else if self.search_results.is_empty() {
            components::render_empty_state(
                f,
                chunks[2],
                &self.theme,
                "No Results",
                "Enter a search term to find videos",
                Some("🔍"),
            );
        } else {
            let item_height = 6 + components::DesignTokens::ITEM_GAP;
            let offset = self.list_state.selected().unwrap_or(0);
            let visible_count = (chunks[2].height / item_height) as usize;

            for (i, video) in self
                .search_results
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

                let channel = video.author.as_deref().unwrap_or("Unknown");
                let views = video
                    .view_count
                    .map(|v| format!("{} views", v))
                    .unwrap_or_default();
                let duration = video
                    .length_seconds
                    .map(|s| format!("{}:{:02}", s / 60, s % 60))
                    .unwrap_or_default();
                let date = video.published_text.as_deref().unwrap_or("Unknown date");

                let meta = format!("{} • {} • {} • {}", channel, views, duration, date);
                let is_selected = self.list_state.selected() == Some(i);

                components::render_item_card(
                    f,
                    rect,
                    &video.title,
                    &meta,
                    &self.theme,
                    is_selected,
                    false,
                );
            }
        }

        components::render_info_bar(
            f,
            chunks[3],
            &[
                ("Results", &format!("{}", self.search_results.len())),
                ("Quality", &self.settings.default_quality),
            ],
            &self.theme,
        );
    }

    fn handle_events(&mut self) -> Result<(), AppError> {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => match self.mode {
                    AppMode::Main => {
                        if key.code == KeyCode::Char('q') {
                            self.should_quit = true;
                        } else if key.code == KeyCode::Char('s') {
                            self.mode = AppMode::Search;
                            self.search_query.clear();
                            self.search_results.clear();
                            self.search_error = None;
                        } else if key.code == KeyCode::Char('h') {
                            self.mode = AppMode::History;
                            self.history_results = self.db.get_history(100).unwrap_or_default();
                            self.history_state = ListState::default();
                            self.history_state.select(Some(0));
                        } else if key.code == KeyCode::Char('v') {
                            self.mode = AppMode::Saved;
                            self.saved_results = self.db.get_saved_videos().unwrap_or_default();
                            self.saved_state = ListState::default();
                            self.saved_state.select(Some(0));
                        } else if key.code == KeyCode::Char('p') {
                            self.mode = AppMode::Playlist;
                            self.playlist_results = self.db.get_playlists().unwrap_or_default();
                            self.playlist_state = ListState::default();
                            self.playlist_state.select(Some(0));
                            self.playlist_videos.clear();
                            self.playlist_videos_state = ListState::default();
                            self.playlist_prompt.clear();
                            self.playlist_prompt_mode = None;
                        } else if key.code == KeyCode::Esc {
                            self.show_context_menu = false;
                        } else if key.code == KeyCode::Enter {
                            if self.active_block == ActiveBlock::Sidebar {
                                if let Some(idx) = self.sidebar_state.selected() {
                                    match self.sidebar_items[idx].as_str() {
                                        "Search" => self.mode = AppMode::Search,
                                        "History" => self.mode = AppMode::History,
                                        "Saved" => self.mode = AppMode::Saved,
                                        "Playlists" => self.mode = AppMode::Playlist,
                                        "Settings" => self.mode = AppMode::Settings,
                                        _ => {}
                                    }
                                }
                            } else if self.active_block == ActiveBlock::Content {
                                if let Some(idx) = self.list_state.selected() {
                                    if let Some(video_title) = self.items.get(idx) {
                                        self.play_main_video(video_title);
                                    }
                                }
                            }
                        } else if key.code == KeyCode::Up {
                            self.scroll_up();
                        } else if key.code == KeyCode::Down {
                            self.scroll_down();
                        }
                    }
                    AppMode::Settings => {
                        crate::ui::settings::handle_events(self, key.code);
                    }
                    AppMode::Search => match key.code {
                        KeyCode::Char(c) => {
                            self.search_query.push(c);
                            self.search_results.clear();
                            self.list_state = ListState::default();
                            self.list_state.select(Some(0));
                        }
                        KeyCode::Backspace => {
                            self.search_query.pop();
                            self.search_results.clear();
                            self.list_state = ListState::default();
                            self.list_state.select(Some(0));
                        }
                        KeyCode::Up => {
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
                        KeyCode::Down => {
                            let i = match self.list_state.selected() {
                                Some(i) => (i + 1) % self.search_results.len().max(1),
                                None => 0,
                            };
                            self.list_state.select(Some(i));
                        }
                        KeyCode::Esc => {
                            self.mode = AppMode::Main;
                        }
                        KeyCode::Enter => {
                            if !self.search_results.is_empty() {
                                if let Some(idx) = self.list_state.selected() {
                                    if let Some(video) = self.search_results.get(idx) {
                                        self.play_search_video(video);
                                    }
                                }
                            } else if !self.search_query.is_empty() {
                                self.trigger_search();
                            }
                        }
                        _ => {}
                    },
                    AppMode::History => match key.code {
                        KeyCode::Up => self.scroll_history_up(),
                        KeyCode::Down => self.scroll_history_down(),
                        KeyCode::Enter => {
                            if let Some(idx) = self.history_state.selected() {
                                if let Some(entry) = self.history_results.get(idx) {
                                    self.play_history_video(entry);
                                }
                            }
                        }
                        KeyCode::Char('c') => {
                            if let Err(e) = self.db.clear_history() {
                                self.set_error(format!("Failed to clear history: {}", e));
                            } else {
                                self.history_results.clear();
                                self.history_state = ListState::default();
                            }
                        }
                        KeyCode::Esc => {
                            self.mode = AppMode::Main;
                        }
                        KeyCode::Char('d') => self.handle_download_shortcut(),
                        _ => {}
                    },
                    AppMode::Saved => match key.code {
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
                                if let Some(video) = self.saved_results.get(idx) {
                                    self.play_saved_video(video);
                                }
                            }
                        }
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
                        KeyCode::Esc => {
                            self.mode = AppMode::Main;
                        }
                        KeyCode::Char('d') => self.handle_download_shortcut(),
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
                                _ => {}
                            }
                        } else {
                            match key.code {
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
                                            if let Some(video) = self.playlist_videos.get(idx) {
                                                self.play_playlist_video(video);
                                            }
                                        }
                                    } else if let Some(idx) = self.playlist_state.selected() {
                                        self.load_playlist_videos(idx);
                                    }
                                }

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

                                KeyCode::Char('d') => self.handle_download_shortcut(),
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
                Event::Mouse(mouse) => {
                    if self.mode == AppMode::Main {
                        self.handle_mouse_event(mouse);
                    }
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

    fn play_history_video(&self, entry: &crate::db::connection::HistoryEntry) {
        let player = self.player.clone();
        let db = self.db.clone();
        let url = format!("https://www.youtube.com/watch?v={}", entry.video_id);
        let video_id = entry.video_id.clone();
        let title = entry.title.clone();
        let channel = entry.channel.clone();
        let quality = self.settings.default_quality.clone();
        tokio::spawn(async move {
            let _ = db.add_to_history(&video_id, &title, channel.as_deref());
            let _ = player.play(&url, &quality, &[]).await;
        });
    }

    fn play_saved_video(&self, video: &crate::db::connection::SavedVideo) {
        let player = self.player.clone();
        let db = self.db.clone();
        let url = format!("https://www.youtube.com/watch?v={}", video.video_id);
        let video_id = video.video_id.clone();
        let title = video.title.clone();
        let channel = video.channel.clone();
        let quality = self.settings.default_quality.clone();
        tokio::spawn(async move {
            let _ = db.add_to_history(&video_id, &title, channel.as_deref());
            let _ = player.play(&url, &quality, &[]).await;
        });
    }

    fn play_search_video(&self, video: &crate::api::invidious::Video) {
        let player = self.player.clone();
        let db = self.db.clone();
        let url = format!("https://www.youtube.com/watch?v={}", video.video_id);
        let video_id = video.video_id.clone();
        let title = video.title.clone();
        let channel = video.author.clone();
        let quality = self.settings.default_quality.clone();
        tokio::spawn(async move {
            let _ = db.add_to_history(&video_id, &title, channel.as_deref());
            let _ = player.play(&url, &quality, &[]).await;
        });
    }

    fn play_main_video(&self, video_title: &str) {
        let player = self.player.clone();
        let db = self.db.clone();
        let title = video_title.to_string();
        let quality = self.settings.default_quality.clone();

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
            let _ = player.play(&url, &quality, &[]).await;
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
                    Err(crate::api::invidious::InvidiousError::BadInstance) if attempts < 3 => {
                        current_url = crate::api::health::rotate_to_healthy_invidious()
                            .await
                            .to_string();
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

    fn scroll_playlist_videos_up(&mut self) {}

    fn scroll_playlist_up(&mut self) {}

    fn scroll_playlist_videos_down(&mut self) {}

    fn scroll_playlist_down(&mut self) {}

    fn play_playlist_video(&self, video: &crate::db::connection::PlaylistVideo) {
        let player = self.player.clone();
        let db = self.db.clone();
        let url = format!("https://www.youtube.com/watch?v={}", video.video_id);
        let video_id = video.video_id.clone();
        let title = video.title.clone();
        let channel = video.channel.clone();
        let quality = self.settings.default_quality.clone();
        tokio::spawn(async move {
            let _ = db.add_to_history(&video_id, &title, channel.as_deref());
            let _ = player.play(&url, &quality, &[]).await;
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
                    Err(crate::api::invidious::InvidiousError::BadInstance) if attempts < 3 => {
                        current_url = crate::api::health::rotate_to_healthy_invidious()
                            .await
                            .to_string();
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
                    Err(crate::api::invidious::InvidiousError::BadInstance) if attempts < 3 => {
                        current_url = crate::api::health::rotate_to_healthy_invidious()
                            .await
                            .to_string();
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
        }
    }

    fn trigger_download(&mut self, video_id: String) {
        self.is_loading = true;
        self.loading_message = "Downloading video...".to_string();
        self.current_error = None;

        let download_path = self.settings.download_path.clone();
        let tx = self.download_tx.clone();

        tokio::spawn(async move {
            let mut rx = Downloader::download(&video_id, &download_path, None);
            while let Some(progress) = rx.recv().await {
                let _ = tx.send(progress).await;
            }
        });
    }

    fn handle_download_shortcut(&mut self) {
        if let Some(video_id) = self.get_selected_video_id() {
            self.trigger_download(video_id);
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
                                if let Some(video_id) = self.get_selected_video_id() {
                                    self.trigger_download(video_id);
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
}
