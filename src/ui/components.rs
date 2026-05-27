use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Direction, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};

pub struct DesignTokens;

impl DesignTokens {
    pub const ITEM_GAP: u16 = 1;
    pub const PADDING_MD: u16 = 1;
}

pub struct SidebarItem<'a> {
    pub icon: &'a str,
    pub label: &'a str,
}

pub enum ErrorCategory {
    EmptyHistory,
    EmptySaved,
    EmptyPlaylists,
    NoResults,
}

impl ErrorCategory {
    pub fn message(&self) -> &'static str {
        match self {
            Self::EmptyHistory => "Start watching videos to build your history",
            Self::EmptySaved => "Press 's' on a video to save it here",
            Self::EmptyPlaylists => "Press 'n' to create a playlist",
            Self::NoResults => "Enter a search term to find videos",
        }
    }
}

pub fn render_sidebar(
    f: &mut Frame,
    area: Rect,
    items: &[SidebarItem],
    selected: usize,
    _theme: &Theme,
    is_focused: bool,
) {
    let border_style = if is_focused {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title("Menu");
    let inner = block.inner(area);
    f.render_widget(block, area);

    for (i, item) in items.iter().enumerate() {
        if i as u16 >= inner.height {
            break;
        }
        let y = inner.y + i as u16;
        let row = Rect::new(inner.x, y, inner.width, 1);

        let label = format!(" {} {}", item.icon, item.label);
        let style = if i == selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let para = Paragraph::new(label).style(style);
        f.render_widget(para, row);
    }
}

pub fn render_header(
    f: &mut Frame,
    area: Rect,
    title: &str,
    subtitle: &str,
    _theme: &Theme,
) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .padding(Padding::horizontal(1));

    let content = Line::from(vec![
        Span::styled(title, Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::raw(subtitle),
    ]);
    let para = Paragraph::new(content).block(block);
    f.render_widget(para, area);
}

pub fn render_info_bar(
    f: &mut Frame,
    area: Rect,
    items: &[(&str, &str)],
    _theme: &Theme,
) {
    let spans: Vec<Span> = items
        .iter()
        .flat_map(|(key, val)| {
            vec![
                Span::styled(format!(" {}: ", key), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(val.to_string()),
                Span::raw("  "),
            ]
        })
        .collect();

    let block = Block::default().borders(Borders::TOP);
    let para = Paragraph::new(Line::from(spans)).block(block);
    f.render_widget(para, area);
}

pub fn render_item_card(
    f: &mut Frame,
    area: Rect,
    title: &str,
    meta: &str,
    _theme: &Theme,
    is_selected: bool,
    is_focused: bool,
) {
    let border_style = if is_selected && is_focused {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::horizontal(1));

    let title_style = if is_selected && is_focused {
        Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
    } else if is_selected {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let content = vec![
        Line::from(Span::styled(title, title_style)),
        Line::from(Span::raw(meta)),
    ];
    let para = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

pub fn render_empty_state(
    f: &mut Frame,
    area: Rect,
    _theme: &Theme,
    title: &str,
    message: &str,
    icon: Option<&str>,
) {
    let icon_str = icon.unwrap_or("·");
    let content = vec![
        Line::from(""),
        Line::from(Span::raw(icon_str)),
        Line::from(""),
        Line::from(Span::styled(title, Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::raw(message)),
    ];
    let para = Paragraph::new(content).alignment(Alignment::Center);
    f.render_widget(para, area);
}

pub fn render_loading(f: &mut Frame, area: Rect, message: &str) {
    let content = vec![
        Line::from(""),
        Line::from(Span::styled("⟳ Loading...", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::raw(message)),
    ];
    let para = Paragraph::new(content).alignment(Alignment::Center);
    f.render_widget(para, area);
}

pub fn render_error(f: &mut Frame, area: Rect, message: &str, suggestion: Option<&str>) {
    let mut content = vec![
        Line::from(""),
        Line::from(Span::styled("Error", Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))),
        Line::from(""),
        Line::from(Span::raw(message)),
    ];
    if let Some(s) = suggestion {
        content.push(Line::from(""));
        content.push(Line::from(Span::raw(format!("Tip: {}", s))));
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Error")
        .border_style(Style::default().add_modifier(Modifier::BOLD));
    let para = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    let popup = centered_rect(60, 40, area);
    f.render_widget(ratatui::widgets::Clear, popup);
    f.render_widget(para, popup);
}

pub fn render_divider(f: &mut Frame, area: Rect, _theme: &Theme, _direction: Direction) {
    let block = Block::default().borders(Borders::BOTTOM);
    f.render_widget(block, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let w = r.width * percent_x / 100;
    let h = r.height * percent_y / 100;
    Rect {
        x: r.x + (r.width - w) / 2,
        y: r.y + (r.height - h) / 2,
        width: w,
        height: h,
    }
}
