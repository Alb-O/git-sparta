use crate::theme::Theme;
use ratatui::style::Color;

pub const SLATE: Theme = Theme {
    header_fg: Color::Rgb(226, 232, 240),
    header_bg: Color::Rgb(15, 23, 42),
    row_highlight_bg: Color::Rgb(30, 41, 59),
    row_highlight_fg: Color::Rgb(250, 204, 21),
    prompt_fg: Color::LightCyan,
    empty_fg: Color::DarkGray,
    highlight_fg: Color::Yellow,
};
