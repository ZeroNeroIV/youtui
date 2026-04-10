use ratatui::style::Color;

pub struct Theme {
    pub name: String,
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub secondary: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            background: Color::Black,
            foreground: Color::White,
            accent: Color::Blue,
            secondary: Color::DarkGray,
        }
    }
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            background: Color::White,
            foreground: Color::Black,
            accent: Color::Blue,
            secondary: Color::DarkGray,
        }
    }
}
