use crate::theme::Theme;
use ratatui::style::Color;

pub const SOLARIZED: Theme = Theme {
    header_fg: Color::Rgb(253, 246, 227),
    header_bg: Color::Rgb(7, 54, 66),
    row_highlight_bg: Color::Rgb(0, 43, 54),
    row_highlight_fg: Color::Rgb(181, 137, 0),
    prompt_fg: Color::Rgb(38, 139, 210),
    empty_fg: Color::Rgb(88, 110, 117),
    highlight_fg: Color::Rgb(181, 137, 0),
};
