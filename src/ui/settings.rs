use crate::ui::app::{App, AppMode};

pub fn render_in_panel(f: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &mut App) {
    render(f, area, app);
}

pub fn render(f: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &mut App) {
    use crate::ui::components::{
        render_divider, render_header, render_info_bar, render_progress_bar, DesignTokens,
    };
    use ratatui::layout::{Constraint, Direction, Layout};
    use ratatui::style::Style;
    use ratatui::widgets::{Block, Padding, Paragraph};

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(f, chunks[0], "Settings", "", &app.theme);

    let content_area = chunks[1];
    let selected_idx = app.settings_state.selected().unwrap_or(0);

    let highlight_color = app.theme.highlight;
    let secondary_color = app.theme.secondary;
    let foreground_color = app.theme.foreground;

    let mut current_y = 0;
    let total_height = content_area.height;

    let render_section = |f: &mut ratatui::Frame, y: &mut u16, title: &str| {
        let section_area = ratatui::layout::Rect {
            x: content_area.x,
            y: content_area.y + *y,
            width: content_area.width,
            height: 2,
        };
        render_header(f, section_area, title, "", &app.theme);
        *y += 2;

        let divider_area = ratatui::layout::Rect {
            x: content_area.x,
            y: content_area.y + *y,
            width: content_area.width,
            height: 1,
        };
        render_divider(f, divider_area, &app.theme, Direction::Horizontal);
        *y += 1;
    };

    let render_row =
        |f: &mut ratatui::Frame, y: &mut u16, label: &str, value: &str, is_selected: bool| {
            if *y >= total_height {
                return;
            }

            let row_area = ratatui::layout::Rect {
                x: content_area.x,
                y: content_area.y + *y,
                width: content_area.width,
                height: 1,
            };

            let row_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(row_area);

            let label_style = if is_selected {
                Style::default()
                    .fg(secondary_color)
                    .bg(highlight_color)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default().fg(secondary_color)
            };
            let label_para = Paragraph::new(label)
                .style(label_style)
                .block(Block::default().padding(Padding::horizontal(DesignTokens::PADDING_MD)));

            let value_style = if is_selected {
                Style::default()
                    .fg(foreground_color)
                    .bg(highlight_color)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default().fg(foreground_color)
            };
            let value_para = Paragraph::new(value)
                .style(value_style)
                .block(Block::default().padding(Padding::horizontal(DesignTokens::PADDING_MD)));

            f.render_widget(label_para, row_layout[0]);
            f.render_widget(value_para, row_layout[1]);

            *y += 1;
        };

    let render_volume_row = |f: &mut ratatui::Frame, y: &mut u16, is_selected: bool| {
        if *y >= total_height {
            return;
        }

        let row_area = ratatui::layout::Rect {
            x: content_area.x,
            y: content_area.y + *y,
            width: content_area.width,
            height: 1,
        };

        let row_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(row_area);

        let label_style = if is_selected {
            Style::default()
                .fg(secondary_color)
                .bg(highlight_color)
                .add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            Style::default().fg(secondary_color)
        };
        let label_para = Paragraph::new("Volume")
            .style(label_style)
            .block(Block::default().padding(Padding::horizontal(DesignTokens::PADDING_MD)));

        f.render_widget(label_para, row_layout[0]);
        render_progress_bar(f, row_layout[1], 70, 100, &app.theme);

        *y += 1;
    };

    render_section(f, &mut current_y, "Playback");
    render_row(
        f,
        &mut current_y,
        "Loop Playback",
        if app.settings.loop_playback {
            "On"
        } else {
            "Off"
        },
        selected_idx == 0,
    );
    render_row(
        f,
        &mut current_y,
        "Auto Play",
        if app.settings.auto_play { "On" } else { "Off" },
        selected_idx == 1,
    );
    render_row(
        f,
        &mut current_y,
        "Instance Mode",
        match app.settings.player_instance_mode {
            crate::config::settings::PlayerInstanceMode::Single => "Single",
            crate::config::settings::PlayerInstanceMode::Multiple => "Multiple",
        },
        selected_idx == 2,
    );
    render_row(
        f,
        &mut current_y,
        "Quality",
        &app.settings.default_quality,
        selected_idx == 3,
    );
    render_row(
        f,
        &mut current_y,
        "Format",
        &app.settings.default_format,
        selected_idx == 4,
    );
    render_volume_row(f, &mut current_y, selected_idx == 5);

    render_section(f, &mut current_y, "General");
    render_row(
        f,
        &mut current_y,
        "Theme",
        &app.settings.theme,
        selected_idx == 6,
    );
    render_row(
        f,
        &mut current_y,
        "Log Level",
        &app.settings.log_level,
        selected_idx == 7,
    );
    let download_path = app.settings.download_path.to_string_lossy();
    render_row(
        f,
        &mut current_y,
        "Download Path",
        &download_path,
        selected_idx == 8,
    );
    render_row(
        f,
        &mut current_y,
        "Player",
        &app.settings.player,
        selected_idx == 9,
    );

    render_section(f, &mut current_y, "API");
    render_row(
        f,
        &mut current_y,
        "Invidious API",
        &app.settings.api_instance_invidious,
        selected_idx == 10,
    );
    render_row(
        f,
        &mut current_y,
        "Piped API",
        &app.settings.api_instance_piped,
        selected_idx == 11,
    );

    render_info_bar(
        f,
        chunks[2],
        &[("Navigate", "↑↓"), ("Select", "Enter"), ("Back", "Esc")],
        &app.theme,
    );
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
                        11
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
                Some(i) => (i + 1) % 12,
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
            app.settings.loop_playback = !app.settings.loop_playback;
        }
        1 => {
            app.settings.auto_play = !app.settings.auto_play;
        }
        2 => {
            app.settings.player_instance_mode = match app.settings.player_instance_mode {
                crate::config::settings::PlayerInstanceMode::Single => {
                    crate::config::settings::PlayerInstanceMode::Multiple
                }
                crate::config::settings::PlayerInstanceMode::Multiple => {
                    crate::config::settings::PlayerInstanceMode::Single
                }
            };
        }
        3 => {
            let qualities = ["144", "240", "360", "480", "720", "1080", "1440", "2160"];
            let current_idx = qualities
                .iter()
                .position(|&q| q == app.settings.default_quality)
                .unwrap_or(0);
            app.settings.default_quality =
                qualities[(current_idx + 1) % qualities.len()].to_string();
        }
        4 => {
            let formats = ["mp4", "mkv", "webm", "mp3"];
            let current_idx = formats
                .iter()
                .position(|&f| f == app.settings.default_format)
                .unwrap_or(0);
            app.settings.default_format = formats[(current_idx + 1) % formats.len()].to_string();
        }
        5 => {
            // Volume is not yet in Settings struct, so we do nothing here
        }
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
            let levels = ["trace", "debug", "info", "warn", "error"];
            let current_idx = levels
                .iter()
                .position(|&l| l == app.settings.log_level)
                .unwrap_or(2); // Default to "info"
            app.settings.log_level = levels[(current_idx + 1) % levels.len()].to_string();
            crate::utils::logger::update_log_level(&app.settings.log_level);
        }
        8 => {
            if let Some(home) = dirs::home_dir() {
                let paths = [
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
        9 => {
            let players = ["mpv", "vlc", "mplayer"];
            let current_idx = players
                .iter()
                .position(|&p| p == app.settings.player)
                .unwrap_or(0);
            app.settings.player = players[(current_idx + 1) % players.len()].to_string();
        }
        10 => {
            // API instances might need a text input, but for now we keep it simple or leave it
        }
        11 => {
            // API instances might need a text input, but for now we keep it simple or leave it
        }
        _ => {}
    }
    let _ = app.settings.save();
}
