use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub foreground: Color,
    pub background: Color,
    pub accent: Color,
    pub secondary: Color,
    pub selection_fg: Color,
    pub selection_bg: Color,
}

impl Theme {
    pub fn default_theme() -> Self {
        Self {
            name: "default".to_string(),
            foreground: Color::Reset,
            background: Color::Reset,
            accent: Color::Reset,
            secondary: Color::Reset,
            selection_fg: Color::Reset,
            selection_bg: Color::Reset,
        }
    }

    pub fn from_name(_name: &str) -> Option<Self> {
        Some(Self::default_theme())
    }
}
