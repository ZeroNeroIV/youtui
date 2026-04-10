use crate::ui::app::{App, AppMode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();

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
                        7
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
                Some(i) => (i + 1) % 8,
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
    match selected {
        0 => {
            let qualities = ["144", "240", "360", "480", "720", "1080", "1440", "2160"];
            let current_idx = qualities
                .iter()
                .position(|&q| q == app.settings.default_quality)
                .unwrap_or(0);
            app.settings.default_quality =
                qualities[(current_idx + 1) % qualities.len()].to_string();
        }
        1 => {
            let formats = ["mp4", "mkv", "webm", "mp3"];
            let current_idx = formats
                .iter()
                .position(|&f| f == app.settings.default_format)
                .unwrap_or(0);
            app.settings.default_format = formats[(current_idx + 1) % formats.len()].to_string();
        }
        2 => {}
        3 => {
            let players = ["mpv", "vlc", "mplayer"];
            let current_idx = players
                .iter()
                .position(|&p| p == app.settings.player)
                .unwrap_or(0);
            app.settings.player = players[(current_idx + 1) % players.len()].to_string();
        }
        4 => {}
        5 => {}
        6 => {
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
        7 => {
            app.settings.auto_play = !app.settings.auto_play;
        }
        _ => {}
    }
    let _ = app.settings.save();
}
