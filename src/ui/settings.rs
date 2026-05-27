use crate::ui::app::{App, AppMode};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum SettingRow {
    ApiInvidious,
    ApiPiped,
    Player,
    Quality,
    Format,
    DownloadPath,
    LogLevel,
    AutoPlay,
    LoopPlayback,
}

const ROWS: &[SettingRow] = &[
    SettingRow::ApiInvidious,
    SettingRow::ApiPiped,
    SettingRow::Player,
    SettingRow::Quality,
    SettingRow::Format,
    SettingRow::DownloadPath,
    SettingRow::LogLevel,
    SettingRow::AutoPlay,
    SettingRow::LoopPlayback,
];

fn row_label(row: SettingRow) -> &'static str {
    match row {
        SettingRow::ApiInvidious => "API Instance (Invidious)",
        SettingRow::ApiPiped    => "API Instance (Piped)",
        SettingRow::Player      => "Player  [Enter to cycle: mpv / vlc / mplayer]",
        SettingRow::Quality     => "Default Quality  [Enter to cycle: 1080 / 720 / 480 / 360]",
        SettingRow::Format      => "Default Format  [Enter to cycle: mp4 / webm / audio-only]",
        SettingRow::DownloadPath => "Download Path  [Enter to open Yazi folder picker]",
        SettingRow::LogLevel    => "Log Level  [Enter to cycle: info / debug / warn / error]",
        SettingRow::AutoPlay    => "Auto-play next  [Enter to toggle]",
        SettingRow::LoopPlayback => "Loop playback  [Enter to toggle]",
    }
}

fn row_value(row: SettingRow, app: &App) -> String {
    match row {
        SettingRow::ApiInvidious  => app.settings.api_instance_invidious.clone(),
        SettingRow::ApiPiped      => app.settings.api_instance_piped.clone(),
        SettingRow::Player        => app.settings.player.clone(),
        SettingRow::Quality       => app.settings.default_quality.clone(),
        SettingRow::Format        => app.settings.default_format.clone(),
        SettingRow::DownloadPath  => app.settings.download_path.display().to_string(),
        SettingRow::LogLevel      => app.settings.log_level.clone(),
        SettingRow::AutoPlay      => if app.settings.auto_play { "on".into() } else { "off".into() },
        SettingRow::LoopPlayback  => if app.settings.loop_playback { "on".into() } else { "off".into() },
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let header = Paragraph::new(Line::from(vec![
        Span::styled("Settings", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  — ↑/↓ navigate  •  Enter to change  •  Esc back"),
    ]))
    .block(Block::default().borders(Borders::BOTTOM).padding(Padding::horizontal(1)));
    f.render_widget(header, chunks[0]);

    let selected = app.settings_state.selected().unwrap_or(0);
    let inner = chunks[1];

    for (i, &row) in ROWS.iter().enumerate() {
        let row_h = 2u16;
        let row_y = inner.y + i as u16 * row_h;
        if row_y + row_h > inner.y + inner.height {
            break;
        }
        let rect = Rect::new(inner.x, row_y, inner.width, row_h);
        let is_selected = i == selected;
        let val_style = if is_selected {
            Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
        } else {
            Style::default()
        };

        let content = vec![
            Line::from(Span::styled(
                format!(" {}", row_label(row)),
                Style::default().add_modifier(Modifier::DIM),
            )),
            Line::from(Span::styled(format!("   {}", row_value(row, app)), val_style)),
        ];
        let para = Paragraph::new(content)
            .block(Block::default().borders(Borders::NONE))
            .wrap(Wrap { trim: true });
        f.render_widget(para, rect);
    }

    let footer = Paragraph::new("  ↑/↓ Navigate  •  Enter Change  •  Esc Back  •  Changes saved automatically")
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}

pub fn handle_events(app: &mut App, key: KeyCode) {
    let max = ROWS.len().saturating_sub(1);
    match key {
        KeyCode::Up => {
            let i = app.settings_state.selected().unwrap_or(0).saturating_sub(1);
            app.settings_state.select(Some(i));
        }
        KeyCode::Down => {
            let i = (app.settings_state.selected().unwrap_or(0) + 1).min(max);
            app.settings_state.select(Some(i));
        }
        KeyCode::Enter => {
            let idx = app.settings_state.selected().unwrap_or(0);
            if let Some(&row) = ROWS.get(idx) {
                cycle_setting(app, row);
            }
        }
        KeyCode::Esc => {
            app.mode = AppMode::Main;
        }
        _ => {}
    }
}

fn cycle_setting(app: &mut App, row: SettingRow) {
    match row {
        SettingRow::Player => {
            let players = ["mpv", "vlc", "mplayer", "ffplay"];
            let cur = players.iter().position(|&p| p == app.settings.player.as_str()).unwrap_or(0);
            let next = players[(cur + 1) % players.len()];
            app.settings.player = next.to_string();
            app.recreate_player(next);
        }
        SettingRow::Quality => {
            let opts = ["1080", "720", "480", "360", "240"];
            let cur = opts.iter().position(|&q| q == app.settings.default_quality.as_str()).unwrap_or(0);
            app.settings.default_quality = opts[(cur + 1) % opts.len()].to_string();
        }
        SettingRow::Format => {
            let opts = ["mp4", "webm", "mkv", "mp3"];
            let cur = opts.iter().position(|&f| f == app.settings.default_format.as_str()).unwrap_or(0);
            app.settings.default_format = opts[(cur + 1) % opts.len()].to_string();
        }
        SettingRow::LogLevel => {
            let opts = ["info", "debug", "warn", "error"];
            let cur = opts.iter().position(|&l| l == app.settings.log_level.as_str()).unwrap_or(0);
            app.settings.log_level = opts[(cur + 1) % opts.len()].to_string();
        }
        SettingRow::AutoPlay => {
            app.settings.auto_play = !app.settings.auto_play;
            app.autoplay_enabled = app.settings.auto_play;
        }
        SettingRow::LoopPlayback => {
            app.settings.loop_playback = !app.settings.loop_playback;
        }
        SettingRow::DownloadPath => {
            // Suspend the TUI, open yazi, resume
            suspend_tui_and_run(app, |a| a.pick_download_path_with_yazi());
        }
        _ => {}
    }
    let _ = app.settings.save();
}

fn suspend_tui_and_run<F: FnOnce(&mut App)>(app: &mut App, f: F) {
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen, EnterAlternateScreen},
        event::{DisableMouseCapture, EnableMouseCapture},
    };
    let mut stdout = std::io::stdout();

    // Suspend TUI
    let _ = disable_raw_mode();
    let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);

    f(app);

    // Resume TUI
    let _ = enable_raw_mode();
    let _ = execute!(stdout, EnterAlternateScreen, EnableMouseCapture);
}
