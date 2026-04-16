/*
 * UI REDESIGN AUDIT REPORT
 * -----------------------
 * Border Usage Audit (src/ui/):
 * - Borders::ALL: 15 occurrences -> REMOVE
 * - Block::default().borders(): 4 occurrences -> REDUCE
 *
 * MODERN/MINIMAL DESIGN RULES:
 * 1. No Borders::ALL on content blocks
 * 2. Use PADDING_MD for item interiors
 * 3. Use ITEM_GAP between list items
 * 4. Accent color for selection/focus only
 * 5. Secondary color for metadata/timestamps
 */

pub struct DesignTokens;
impl DesignTokens {
    pub const PADDING_SM: u16 = 1;
    pub const PADDING_MD: u16 = 2;
    pub const PADDING_LG: u16 = 3;
    pub const ITEM_GAP: u16 = 1;
    pub const SIDEBAR_WIDTH: u16 = 15;
    pub const TRUNCATE_LEN: usize = 40;
    pub const MIN_TERMINAL_WIDTH: u16 = 80;
    pub const MIN_TERMINAL_HEIGHT: u16 = 20;
}

/// Represents an item in the sidebar navigation
pub struct SidebarItem<'a> {
    pub icon: &'a str,
    pub label: &'a str,
}

use crate::ui::theme::Theme;
use ratatui::{
    layout::{Direction, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
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

/// Display a modern header with title and optional subtitle
pub fn render_header(f: &mut Frame, area: Rect, title: &str, subtitle: &str, theme: &Theme) {
    let mut lines = vec![Line::from(vec![Span::styled(
        title,
        Style::default().fg(theme.accent).bold(),
    )])];

    if !subtitle.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            subtitle,
            Style::default().fg(theme.secondary),
        )]));
    }

    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(theme.foreground))
        .block(Block::default().padding(Padding::uniform(DesignTokens::PADDING_MD)));

    f.render_widget(paragraph, area);
}

/// Display a modern item card for lists
pub fn render_item_card(
    f: &mut Frame,
    area: Rect,
    title: &str,
    meta: &str,
    theme: &Theme,
    is_selected: bool,
    is_focused: bool,
) {
    let title_truncated: String = if title.chars().count() > DesignTokens::TRUNCATE_LEN {
        title
            .chars()
            .take(DesignTokens::TRUNCATE_LEN - 3)
            .collect::<String>()
            + "..."
    } else {
        title.to_string()
    };

    let content = vec![
        Line::from(vec![Span::styled(
            &title_truncated,
            Style::default().fg(theme.foreground).bold(),
        )]),
        Line::from(vec![Span::styled(
            meta,
            Style::default().fg(theme.secondary),
        )]),
    ];

    let mut style = Style::default().fg(theme.foreground);
    if is_selected {
        style = style.bg(theme.highlight);
    }

    let mut borders = Borders::NONE;
    let mut border_style = Style::default().fg(theme.border);
    if is_focused {
        borders = Borders::LEFT;
        border_style = Style::default().fg(theme.accent);
    }

    let block = Block::default()
        .borders(borders)
        .border_style(border_style)
        .padding(Padding::uniform(DesignTokens::PADDING_MD));

    let paragraph = Paragraph::new(content).block(block).style(style);

    f.render_widget(paragraph, area);
}

/// Display a modern divider line
pub fn render_divider(f: &mut Frame, area: Rect, theme: &Theme, direction: Direction) {
    let horizontal = direction == Direction::Horizontal;
    let symbol = if horizontal { "─" } else { "│" };
    let line = Line::from(symbol.repeat(if horizontal {
        area.width as usize
    } else {
        area.height as usize
    }))
    .style(Style::default().fg(theme.border));

    let paragraph = Paragraph::new(line);
    f.render_widget(paragraph, area);
}

/// Display a modern tab bar
pub fn render_tab_bar(
    f: &mut Frame,
    area: Rect,
    tabs: &[&str],
    selected_index: usize,
    theme: &Theme,
) {
    let mut spans = vec![];
    for (i, tab) in tabs.iter().enumerate() {
        let style = if i == selected_index {
            Style::default().fg(theme.accent).bold()
        } else {
            Style::default().fg(theme.secondary)
        };
        spans.push(Span::styled(*tab, style));
        if i < tabs.len() - 1 {
            spans.push(Span::raw("  "));
        }
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .block(Block::default().padding(Padding::uniform(DesignTokens::PADDING_SM)));

    f.render_widget(paragraph, area);
}

/// Renders a static sidebar with navigation items
pub fn render_sidebar(
    f: &mut Frame,
    area: Rect,
    items: &[SidebarItem],
    selected_index: usize,
    theme: &Theme,
    is_focused: bool,
) {
    let title_height = if is_focused { 3 } else { 0 };
    if title_height > 0 {
        let title_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: title_height,
        };
        let title = Paragraph::new(Line::from(vec![Span::styled(
            "Youtui",
            Style::default().fg(theme.accent).bold(),
        )]))
        .block(Block::default().padding(Padding::uniform(DesignTokens::PADDING_SM)));
        f.render_widget(title, title_area);
    }

    let item_height = if is_focused { 3 } else { 2 };
    let mut current_y = area.y + title_height + DesignTokens::ITEM_GAP;
    for (i, item) in items.iter().enumerate() {
        if current_y + item_height > area.y + area.height {
            break;
        }

        let item_area = Rect {
            x: area.x,
            y: current_y,
            width: area.width,
            height: item_height,
        };

        let is_selected = i == selected_index;
        let style = if is_selected {
            if is_focused {
                Style::default().fg(theme.foreground).bg(theme.highlight)
            } else {
                Style::default().fg(theme.accent)
            }
        } else {
            Style::default().fg(theme.secondary)
        };

        let mut borders = Borders::NONE;
        let mut border_style = Style::default().fg(theme.border);
        if is_selected {
            borders = Borders::LEFT;
            border_style = Style::default().fg(theme.accent);
        }

        let block = Block::default()
            .borders(borders)
            .border_style(border_style)
            .padding(Padding {
                left: DesignTokens::PADDING_MD,
                right: DesignTokens::PADDING_MD,
                top: 1,
                bottom: 1,
            });

        let content = if is_focused {
            Line::from(vec![Span::raw(format!("{} {}", item.icon, item.label))])
        } else {
            Line::from(vec![Span::raw(item.icon)])
        };

        let paragraph = Paragraph::new(content).block(block).style(style);
        f.render_widget(paragraph, item_area);

        current_y += item_height + DesignTokens::ITEM_GAP;
    }
}

/// Display a modern progress bar
pub fn render_progress_bar(f: &mut Frame, area: Rect, current: u64, total: u64, theme: &Theme) {
    use ratatui::widgets::Gauge;

    let ratio = if total > 0 {
        current as f64 / total as f64
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(theme.accent).bg(theme.border))
        .ratio(ratio);

    f.render_widget(gauge, area);
}

/// Display a modern info bar with label-value pairs
pub fn render_info_bar(f: &mut Frame, area: Rect, items: &[(&str, &str)], theme: &Theme) {
    let mut lines = vec![];
    for (label, value) in items {
        lines.push(Line::from(vec![
            Span::styled(format!("{}: ", label), Style::default().fg(theme.secondary)),
            Span::styled(*value, Style::default().fg(theme.foreground)),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().padding(Padding::uniform(DesignTokens::PADDING_MD)));

    f.render_widget(paragraph, area);
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
        .borders(Borders::NONE)
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
    theme: &Theme,
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
            Span::styled(title, Style::default().bold().fg(theme.accent)),
        ]),
        Line::from(""),
        Line::from(message),
    ];

    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(theme.secondary))
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

    let block = Block::default().borders(Borders::TOP).border_style(style);

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
