use ratatui::style::{Color, Style};

// Declare available theme modules and a single source-of-truth for their
// canonical names in one place. The macro expands to `pub mod` and `pub use`
// declarations, the `NAMES` constant, and a `by_name` helper.
macro_rules! declare_themes {
    ( $( ($mod:ident, $const:ident) ),* $(,)? ) => {
        $( pub mod $mod; )*
        $( pub use $mod::$const; )*

        /// Canonical theme names supported by the UI.
        pub const NAMES: &[&str] = &[ $( stringify!($mod) ),* ];

        /// Lookup a Theme by case-insensitive name.
        pub fn by_name(name: &str) -> Option<Theme> {
            match name.to_lowercase().as_str() {
                $( stringify!($mod) => Some($const), )*
                _ => None,
            }
        }
    };
}

// List themes once here (module, exported-const). Ordering here defines the
// order exposed in `NAMES`.
declare_themes!((slate, SLATE), (solarized, SOLARIZED), (light, LIGHT),);

/// Core Theme struct
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub header_fg: Color,
    pub header_bg: Color,
    pub row_highlight_bg: Color,
    pub row_highlight_fg: Color,
    pub prompt_fg: Color,
    pub empty_fg: Color,
    pub highlight_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        SLATE
    }
}

impl Theme {
    #[must_use]
    pub fn header_style(&self) -> Style {
        Style::new().fg(self.header_fg).bg(self.header_bg)
    }

    #[must_use]
    pub fn row_highlight_style(&self) -> Style {
        Style::new()
            .bg(self.row_highlight_bg)
            .fg(self.row_highlight_fg)
    }

    #[must_use]
    pub fn prompt_style(&self) -> Style {
        Style::new().fg(self.prompt_fg)
    }

    #[must_use]
    pub fn empty_style(&self) -> Style {
        Style::new().fg(self.empty_fg)
    }

    #[must_use]
    pub fn highlight_fg(&self) -> Color {
        self.highlight_fg
    }
}
