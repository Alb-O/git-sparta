use crate::theme::Theme;
use ratatui::style::Color;

pub const LIGHT: Theme = Theme {
    header_fg: Color::Rgb(15, 23, 42),
    header_bg: Color::Rgb(226, 232, 240),
    row_highlight_bg: Color::Rgb(200, 200, 200),
    row_highlight_fg: Color::Rgb(120, 120, 0),
    prompt_fg: Color::Rgb(0, 102, 153),
    empty_fg: Color::Rgb(100, 100, 100),
    highlight_fg: Color::Rgb(120, 120, 0),
};
