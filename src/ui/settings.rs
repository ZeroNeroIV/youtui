use crate::ui::app::{App, AppMode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use tracing::info;

pub fn render_in_panel(f: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &mut App) {
    render(f, area, app);
}

pub fn render(f: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new("Settings")
        .style(
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(app.theme.border),
        );

    f.render_widget(title, chunks[0]);

    let settings_items: Vec<ListItem> = vec![
        ListItem::new(format!(
            "Loop Playback: {}",
            if app.settings.loop_playback {
                "On"
            } else {
                "Off"
            }
        )),
        ListItem::new(format!(
            "Instance Mode: {}",
            match app.settings.player_instance_mode {
                crate::config::settings::PlayerInstanceMode::Single => "Single",
                crate::config::settings::PlayerInstanceMode::Multiple => "Multiple",
            }
        )),
        ListItem::new(format!("Quality: {}", app.settings.default_quality)),
        ListItem::new(format!("Format: {}", app.settings.default_format)),
        ListItem::new(format!(
            "Download Path: {}",
            app.settings.download_path.display()
        )),
        ListItem::new(format!("Player: {}", app.settings.player)),
        ListItem::new(format!(
            "Invidious API: {}",
            app.settings.api_instance_invidious
        )),
        ListItem::new(format!("Piped API: {}", app.settings.api_instance_piped)),
        ListItem::new(format!("Theme: {}", app.settings.theme)),
        ListItem::new(format!(
            "Auto Play: {}",
            if app.settings.auto_play { "On" } else { "Off" }
        )),
        ListItem::new(format!("Log Level: {}", app.settings.log_level)),
    ];

    let settings_list = List::new(settings_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(app.theme.border),
        )
        .highlight_style(
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(settings_list, chunks[1], &mut app.settings_state);

    let help = Paragraph::new("Enter: Edit | Esc: Back")
        .style(Style::default().fg(app.theme.secondary))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(app.theme.border),
        );

    f.render_widget(help, chunks[2]);
}

pub fn handle_events(app: &mut App, key: crossterm::event::KeyCode) {
    match key {
        crossterm::event::KeyCode::Esc => {
            app.mode = AppMode::Main;
        }
        crossterm::event::KeyCode::Up => {
            let i = match app.settings_state.selected() {
                Some(i) => {
                    if i == 0 {
                        10
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            app.settings_state.select(Some(i));
        }
        crossterm::event::KeyCode::Down => {
            let i = match app.settings_state.selected() {
                Some(i) => (i + 1) % 11,
                None => 0,
            };
            app.settings_state.select(Some(i));
        }
        crossterm::event::KeyCode::Enter => {
            edit_setting(app);
        }
        _ => {}
    }
}

fn edit_setting(app: &mut App) {
    let selected = app.settings_state.selected().unwrap_or(0);
    info!("DEBUG: Selected index on Enter -> {}", selected);
    match selected {
        0 => {
            app.settings.loop_playback = !app.settings.loop_playback;
        }
        1 => {
            app.settings.player_instance_mode = match app.settings.player_instance_mode {
                crate::config::settings::PlayerInstanceMode::Single => {
                    crate::config::settings::PlayerInstanceMode::Multiple
                }
                crate::config::settings::PlayerInstanceMode::Multiple => {
                    crate::config::settings::PlayerInstanceMode::Single
                }
            };
        }
        2 => {
            let qualities = ["144", "240", "360", "480", "720", "1080", "1440", "2160"];
            let current_idx = qualities
                .iter()
                .position(|&q| q == app.settings.default_quality)
                .unwrap_or(0);
            app.settings.default_quality =
                qualities[(current_idx + 1) % qualities.len()].to_string();
        }
        3 => {
            let formats = ["mp4", "mkv", "webm", "mp3"];
            let current_idx = formats
                .iter()
                .position(|&f| f == app.settings.default_format)
                .unwrap_or(0);
            app.settings.default_format = formats[(current_idx + 1) % formats.len()].to_string();
        }
        4 => {
            if let Some(home) = dirs::home_dir() {
                let paths = vec![
                    home.join("Downloads"),
                    home.join("Videos"),
                    home.join("Videos/youtui"),
                    dirs::data_local_dir()
                        .map(|p| p.join("youtui-rs/downloads"))
                        .unwrap_or_else(|| home.join("Downloads")),
                ];
                let current_str = app.settings.download_path.to_string_lossy().to_string();
                let current_idx = paths
                    .iter()
                    .position(|p| p.to_string_lossy() == current_str)
                    .unwrap_or(0);
                app.settings.download_path = paths[(current_idx + 1) % paths.len()].clone();
            }
        }
        5 => {
            let players = ["mpv", "vlc", "mplayer"];
            let current_idx = players
                .iter()
                .position(|&p| p == app.settings.player)
                .unwrap_or(0);
            app.settings.player = players[(current_idx + 1) % players.len()].to_string();
        }
        6 => {
            // API instances might need a text input, but for now we keep it simple or leave it
        }
        7 => {
            // API instances might need a text input, but for now we keep it simple or leave it
        }
        8 => {
            let themes = crate::ui::theme::Theme::all_themes();
            let current_idx = themes
                .iter()
                .position(|&t| t == app.settings.theme)
                .unwrap_or(0);
            let next_theme = themes[(current_idx + 1) % themes.len()];
            app.settings.theme = next_theme.to_string();
            if let Some(new_theme) = crate::ui::theme::Theme::from_name(next_theme) {
                app.theme = new_theme;
            }
        }
        9 => {
            app.settings.auto_play = !app.settings.auto_play;
        }
        10 => {
            let levels = ["trace", "debug", "info", "warn", "error"];
            let current_idx = levels
                .iter()
                .position(|&l| l == app.settings.log_level)
                .unwrap_or(2); // Default to "info"
            app.settings.log_level = levels[(current_idx + 1) % levels.len()].to_string();
            crate::utils::logger::update_log_level(&app.settings.log_level);
        }
        _ => {}
    }
    let _ = app.settings.save();
}
