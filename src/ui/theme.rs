use ratatui::style::Color;

#[derive(Clone)]
pub struct Theme {
    pub focused_border: Color,
    pub name: String,
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub secondary: Color,
    pub highlight: Color,
    pub border: Color,
    pub error: Color,
    pub success: Color,
}

impl Theme {
    /// Classic green terminal - old school CRT monitor feel
    pub fn terminal() -> Self {
        Self {
            name: "terminal".to_string(),
            background: Color::Black,
            foreground: Color::Green,
            accent: Color::LightGreen,
            secondary: Color::DarkGray,
            highlight: Color::Yellow,
            border: Color::Green,
            focused_border: Color::Magenta,
            error: Color::Red,
            success: Color::LightGreen,
        }
    }

    /// Tokyo Night - popular dark theme with blue/purple tones
    pub fn tokyo() -> Self {
        Self {
            name: "tokyo".to_string(),
            background: Color::Rgb(26, 27, 38),
            foreground: Color::Rgb(192, 202, 245),
            accent: Color::Rgb(137, 180, 250),
            secondary: Color::Rgb(98, 114, 164),
            highlight: Color::Rgb(203, 166, 247),
            border: Color::Rgb(69, 71, 90),
            focused_border: Color::Rgb(137, 180, 250),
            error: Color::Rgb(243, 139, 168),
            success: Color::Rgb(166, 227, 161),
        }
    }

    /// Monokai - warm colors on dark background
    pub fn monokai() -> Self {
        Self {
            name: "monokai".to_string(),
            background: Color::Rgb(39, 40, 34),
            foreground: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(255, 102, 102),
            secondary: Color::Rgb(117, 113, 97),
            highlight: Color::Rgb(230, 219, 116),
            border: Color::Rgb(102, 102, 102),
            focused_border: Color::Rgb(248, 248, 242),
            error: Color::Rgb(255, 102, 102),
            success: Color::Rgb(166, 226, 46),
        }
    }

    /// Light theme - clean white background
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            background: Color::White,
            foreground: Color::Black,
            accent: Color::Blue,
            secondary: Color::Rgb(117, 113, 113),
            highlight: Color::Magenta,
            border: Color::DarkGray,
            focused_border: Color::Cyan,
            error: Color::Red,
            success: Color::Green,
        }
    }

    /// Dark theme - standard dark mode
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            background: Color::Black,
            foreground: Color::White,
            accent: Color::Blue,
            secondary: Color::Gray,
            highlight: Color::Yellow,
            border: Color::DarkGray,
            focused_border: Color::Magenta,
            error: Color::Red,
            success: Color::Green,
        }
    }

    /// Retro/Vaporwave - nostalgic 80s aesthetic
    pub fn retro() -> Self {
        Self {
            name: "retro".to_string(),
            background: Color::Rgb(25, 20, 40),
            foreground: Color::Rgb(255, 170, 204),
            accent: Color::Rgb(0, 255, 255),
            secondary: Color::Rgb(128, 85, 170),
            highlight: Color::Rgb(255, 255, 170),
            border: Color::Rgb(170, 85, 170),
            focused_border: Color::Rgb(255, 255, 170),
            error: Color::Rgb(255, 85, 170),
            success: Color::Rgb(0, 255, 170),
        }
    }

    /// Get a theme by name (case-insensitive)
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "terminal" => Some(Self::terminal()),
            "tokyo" => Some(Self::tokyo()),
            "monokai" => Some(Self::monokai()),
            "light" => Some(Self::light()),
            "dark" => Some(Self::dark()),
            "retro" => Some(Self::retro()),
            _ => None,
        }
    }

    /// Get a list of all available theme names
    pub fn all_themes() -> Vec<&'static str> {
        vec!["terminal", "tokyo", "monokai", "light", "dark", "retro"]
    }

    /// Get the default theme
    pub fn default_theme() -> Self {
        Self::dark()
    }
}
