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
            background: Color::Rgb(13, 17, 23),
            foreground: Color::Rgb(201, 209, 217),
            accent: Color::Rgb(63, 185, 80),
            secondary: Color::Rgb(139, 148, 158),
            highlight: Color::Rgb(35, 134, 54),
            border: Color::Rgb(33, 38, 45),
            focused_border: Color::Rgb(63, 185, 80),
            error: Color::Rgb(248, 81, 73),
            success: Color::Rgb(63, 185, 80),
        }
    }

    /// Tokyo Night - popular dark theme with blue/purple tones
    pub fn tokyo_night() -> Self {
        Self {
            name: "tokyo_night".to_string(),
            background: Color::Rgb(26, 27, 38),
            foreground: Color::Rgb(192, 202, 245),
            accent: Color::Rgb(122, 162, 247),
            secondary: Color::Rgb(86, 95, 137),
            highlight: Color::Rgb(40, 52, 87),
            border: Color::Rgb(41, 46, 66),
            focused_border: Color::Rgb(122, 162, 247),
            error: Color::Rgb(247, 118, 142),
            success: Color::Rgb(158, 206, 106),
        }
    }

    /// Monokai Pro - warm colors on dark background
    pub fn monokai_pro() -> Self {
        Self {
            name: "monokai_pro".to_string(),
            background: Color::Rgb(45, 42, 46),
            foreground: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(166, 226, 46),
            secondary: Color::Rgb(117, 113, 94),
            highlight: Color::Rgb(73, 72, 62),
            border: Color::Rgb(62, 61, 50),
            focused_border: Color::Rgb(166, 226, 46),
            error: Color::Rgb(249, 38, 114),
            success: Color::Rgb(166, 226, 46),
        }
    }

    /// Nord - arctic, north-bluish color palette
    pub fn nord() -> Self {
        Self {
            name: "nord".to_string(),
            background: Color::Rgb(46, 52, 64),
            foreground: Color::Rgb(236, 239, 244),
            accent: Color::Rgb(136, 192, 208),
            secondary: Color::Rgb(76, 86, 106),
            highlight: Color::Rgb(59, 66, 82),
            border: Color::Rgb(67, 76, 94),
            focused_border: Color::Rgb(136, 192, 208),
            error: Color::Rgb(191, 97, 106),
            success: Color::Rgb(163, 190, 140),
        }
    }

    /// Catppuccin Mocha - soothing, pastel dark theme
    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "catppuccin_mocha".to_string(),
            background: Color::Rgb(30, 30, 46),
            foreground: Color::Rgb(205, 214, 244),
            accent: Color::Rgb(203, 166, 247),
            secondary: Color::Rgb(108, 112, 134),
            highlight: Color::Rgb(49, 50, 68),
            border: Color::Rgb(69, 71, 90),
            focused_border: Color::Rgb(203, 166, 247),
            error: Color::Rgb(243, 139, 168),
            success: Color::Rgb(166, 227, 161),
        }
    }

    /// Gruvbox - retro groove, earthy tones
    pub fn gruvbox() -> Self {
        Self {
            name: "gruvbox".to_string(),
            background: Color::Rgb(40, 40, 40),
            foreground: Color::Rgb(235, 219, 178),
            accent: Color::Rgb(250, 189, 47),
            secondary: Color::Rgb(146, 131, 116),
            highlight: Color::Rgb(60, 56, 54),
            border: Color::Rgb(80, 73, 69),
            focused_border: Color::Rgb(250, 189, 47),
            error: Color::Rgb(251, 73, 52),
            success: Color::Rgb(184, 187, 38),
        }
    }

    /// Get a theme by name (case-insensitive)
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "terminal" => Some(Self::terminal()),
            "tokyo_night" => Some(Self::tokyo_night()),
            "monokai_pro" => Some(Self::monokai_pro()),
            "nord" => Some(Self::nord()),
            "catppuccin_mocha" => Some(Self::catppuccin_mocha()),
            "gruvbox" => Some(Self::gruvbox()),
            _ => None,
        }
    }

    /// Get a list of all available theme names
    pub fn all_themes() -> Vec<&'static str> {
        vec![
            "terminal",
            "tokyo_night",
            "monokai_pro",
            "nord",
            "catppuccin_mocha",
            "gruvbox",
        ]
    }

    /// Get the default theme
    pub fn default_theme() -> Self {
        Self::catppuccin_mocha()
    }
}
