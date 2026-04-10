use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Represents an error category for user-friendly messages
#[derive(Debug, Clone)]
pub enum ErrorCategory {
    NetworkNoConnection,
    NetworkTimeout,
    NetworkRateLimited,
    InstanceDown,
    ApiFailure,
    EmptySearchResults,
    EmptyHistory,
    EmptySaved,
    EmptyPlaylists,
    DatabaseCorrupted,
    VideoUnavailable,
    PlayerNotFound,
    InvalidInput,
    Unknown,
}

impl ErrorCategory {
    /// Convert error category to user-friendly message
    pub fn message(&self) -> &'static str {
        match self {
            ErrorCategory::NetworkNoConnection => {
                "No internet connection. Please check your network."
            }
            ErrorCategory::NetworkTimeout => "Request timed out. Please try again.",
            ErrorCategory::NetworkRateLimited => "Too many requests. Please wait a moment.",
            ErrorCategory::InstanceDown => "Video service unavailable. Trying alternative...",
            ErrorCategory::ApiFailure => "Failed to fetch data. Please try again later.",
            ErrorCategory::EmptySearchResults => "No videos found. Try a different search term.",
            ErrorCategory::EmptyHistory => {
                "No watch history yet. Start watching to see videos here."
            }
            ErrorCategory::EmptySaved => "No saved videos. Press 's' to save a video.",
            ErrorCategory::EmptyPlaylists => "No playlists yet. Create one from the menu.",
            ErrorCategory::DatabaseCorrupted => "Data storage error. Some features may not work.",
            ErrorCategory::VideoUnavailable => "Video is unavailable or private.",
            ErrorCategory::PlayerNotFound => "mpv not found. Install mpv to play videos.",
            ErrorCategory::InvalidInput => "Invalid input. Please try again.",
            ErrorCategory::Unknown => "Something went wrong. Please try again.",
        }
    }

    /// Get suggestion text for user
    pub fn suggestion(&self) -> &'static str {
        match self {
            ErrorCategory::NetworkNoConnection => "Check your Wi-Fi or Ethernet connection.",
            ErrorCategory::NetworkTimeout => "Check your connection and try again.",
            ErrorCategory::NetworkRateLimited => "Wait 30 seconds before retrying.",
            ErrorCategory::InstanceDown => "Attempting to connect to backup service...",
            ErrorCategory::ApiFailure => "Try again in a few moments.",
            ErrorCategory::EmptySearchResults => "Enter different keywords to search.",
            ErrorCategory::EmptyHistory => "Search for videos to start watching.",
            ErrorCategory::EmptySaved => "Browse search results and save your favorites.",
            ErrorCategory::EmptyPlaylists => "Use the menu to create a new playlist.",
            ErrorCategory::DatabaseCorrupted => "Restart the app to attempt recovery.",
            ErrorCategory::VideoUnavailable => "This video may have been removed.",
            ErrorCategory::PlayerNotFound => "Run: sudo apt install mpv (Linux), brew install mpv (macOS), or choco install mpv (Windows).",
            ErrorCategory::InvalidInput => "Enter valid search terms.",
            ErrorCategory::Unknown => "If the problem persists, restart the app.",
        }
    }
}

/// Display an error message as an overlay in the UI
pub fn render_error(f: &mut Frame, area: Rect, error: &str, suggestion: Option<&str>) {
    let content = if let Some(s) = suggestion {
        vec![
            Line::from(vec![
                Span::styled("Error: ", Style::default().fg(Color::Red).bold()),
                Span::raw(error),
            ]),
            Line::from(vec![
                Span::styled("Hint: ", Style::default().fg(Color::Yellow)),
                Span::raw(s),
            ]),
        ]
    } else {
        vec![Line::from(vec![
            Span::styled("Error: ", Style::default().fg(Color::Red).bold()),
            Span::raw(error),
        ])]
    };

    let block = Block::default()
        .title(" Error ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

/// Display an empty state message in the UI
pub fn render_empty_state(
    f: &mut Frame,
    area: Rect,
    title: &str,
    message: &str,
    icon: Option<&str>,
) {
    let icon_str = icon.unwrap_or("📭");
    let content = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::raw(icon_str),
            Span::raw(" "),
            Span::styled(title, Style::default().bold().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(message),
    ];

    let block = Block::default().title(" ").borders(Borders::NONE);

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

/// Display a loading indicator
pub fn render_loading(f: &mut Frame, area: Rect, message: &str) {
    let content = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Loading ", Style::default().fg(Color::Green)),
            Span::styled(
                "...",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(message),
    ];

    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

/// Display a status bar message at the bottom of the screen
pub fn render_status_bar(f: &mut Frame, area: Rect, message: &str, is_error: bool) {
    let style = if is_error {
        Style::default().fg(Color::Red).bg(Color::Black)
    } else {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    };

    let content = Line::from(message).alignment(ratatui::layout::Alignment::Left);

    let block = Block::default().borders(Borders::ALL).border_style(style);

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(style)
        .alignment(ratatui::layout::Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Convert AppError to user-friendly error category
pub fn error_to_category(err: &crate::error::AppError) -> ErrorCategory {
    let msg = err.to_string().to_lowercase();
    if msg.contains("connection") || msg.contains("dns") || msg.contains("failed to connect") {
        ErrorCategory::NetworkNoConnection
    } else if msg.contains("timeout") || msg.contains("timed out") {
        ErrorCategory::NetworkTimeout
    } else if msg.contains("429") || msg.contains("rate limit") {
        ErrorCategory::NetworkRateLimited
    } else if msg.contains("404") || msg.contains("not found") || msg.contains("unavailable") {
        ErrorCategory::VideoUnavailable
    } else if msg.contains("database") || msg.contains("sqlite") || msg.contains("corrupted") {
        ErrorCategory::DatabaseCorrupted
    } else if msg.contains("mpv") || msg.contains("player") {
        ErrorCategory::PlayerNotFound
    } else if msg.contains("network") || msg.contains("reqwest") {
        ErrorCategory::NetworkNoConnection
    } else {
        ErrorCategory::ApiFailure
    }
}
