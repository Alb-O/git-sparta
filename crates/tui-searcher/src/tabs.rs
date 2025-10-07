use crate::input::SearchInput;
use crate::types::SearchMode;
use crate::types::UiConfig;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::Tabs;

/// Render the input row with tabs at the right. This mirrors the behaviour
/// previously implemented inside `app.rs`.
pub fn render_input_with_tabs(
    search_input: &SearchInput<'_>,
    input_title: &Option<String>,
    mode: SearchMode,
    ui: &UiConfig,
    frame: &mut ratatui::Frame,
    area: Rect,
) {
    // Calculate tabs width: " Tags " + " Files " + extra padding = about 16 chars
    let tabs_width = 16u16;

    // Get prompt for calculating textarea width
    let prompt = input_title
        .as_deref()
        .or(Some(ui.facets.mode_title.as_str()))
        .unwrap_or("");
    let prompt_width = if prompt.is_empty() {
        0
    } else {
        prompt.len() as u16 + 3
    }; // " > "

    // Split area: prompt (if any), textarea, tabs on right
    let constraints = if prompt.is_empty() {
        vec![
            ratatui::layout::Constraint::Min(1),
            ratatui::layout::Constraint::Length(tabs_width),
        ]
    } else {
        vec![
            ratatui::layout::Constraint::Length(prompt_width),
            ratatui::layout::Constraint::Min(1),
            ratatui::layout::Constraint::Length(tabs_width),
        ]
    };

    let horizontal = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    // Render prompt if present
    if !prompt.is_empty() {
        let prompt_text = format!("{} > ", prompt);
        let prompt_widget = ratatui::widgets::Paragraph::new(prompt_text)
            .style(Style::default().fg(Color::LightCyan));
        frame.render_widget(prompt_widget, horizontal[0]);

        // Render textarea in the middle section
        search_input.render_textarea(frame, horizontal[1]);
    } else {
        // No prompt, render textarea in first section
        search_input.render_textarea(frame, horizontal[0]);
    }

    // Render tabs on the right (last section)
    let tabs_area = horizontal[horizontal.len() - 1];
    let selected = match mode {
        SearchMode::Facets => 0,
        SearchMode::Files => 1,
    };

    // Add extra padding to rightmost tab to prevent cutoff
    let tab_titles = vec![
        Line::from(format!(" {} ", "Tags"))
            .fg(Color::Rgb(226, 232, 240))
            .bg(if selected == 0 {
                Color::Rgb(15, 23, 42)
            } else {
                Color::Rgb(30, 41, 59)
            }),
        Line::from(format!(" {} ", "Files "))
            .fg(Color::Rgb(226, 232, 240))
            .bg(if selected == 1 {
                Color::Rgb(15, 23, 42)
            } else {
                Color::Rgb(30, 41, 59)
            }),
    ];

    let tabs = Tabs::new(tab_titles)
        .select(selected)
        .divider("")
        .highlight_style(Style::default().bg(Color::Rgb(15, 23, 42)));

    frame.render_widget(tabs, tabs_area);
}
