use frizbee::Config;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Cell, Row, Table};

use crate::types::UiConfig;
use crate::utils::{build_facet_rows, build_file_rows};

/// Render the facet table. This returns a stateful Table widget rendered into `area` using `table_state`.
pub fn render_facet_table(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    filtered_facets: &[usize],
    facet_scores: &[u16],
    facets: &[crate::types::FacetRow],
    facet_headers: &Option<Vec<String>>,
    facet_widths: &Option<Vec<Constraint>>,
    table_state: &mut ratatui::widgets::TableState,
    _ui: &UiConfig,
    highlight_state: Option<(&str, &Config)>,
) {
    let rows = build_facet_rows(filtered_facets, facet_scores, facets, highlight_state);

    let widths = facet_widths.clone().unwrap_or_else(|| {
        vec![
            Constraint::Percentage(50),
            Constraint::Length(8),
            Constraint::Length(8),
        ]
    });
    let header_cells = facet_headers
        .clone()
        .unwrap_or_else(|| vec!["Facet".into(), "Count".into(), "Score".into()])
        .into_iter()
        .map(Cell::from)
        .collect::<Vec<_>>();
    let header = Row::new(header_cells)
        .style(
            Style::new()
                .fg(Color::Rgb(226, 232, 240))
                .bg(Color::Rgb(15, 23, 42)),
        )
        .height(1)
        .bottom_margin(1);

    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(1)
        .row_highlight_style(
            Style::new()
                .bg(Color::Rgb(30, 41, 59))
                .fg(Color::Rgb(250, 204, 21)),
        )
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(table, area, table_state);
}

pub fn render_file_view(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    filtered_files: &[usize],
    file_scores: &[u16],
    files: &[crate::types::FileRow],
    file_headers: &Option<Vec<String>>,
    file_widths: &Option<Vec<Constraint>>,
    table_state: &mut ratatui::widgets::TableState,
    _ui: &UiConfig,
    highlight_state: Option<(&str, &Config)>,
) {
    let rows = build_file_rows(filtered_files, file_scores, files, highlight_state);

    let widths = file_widths.clone().unwrap_or_else(|| {
        vec![
            Constraint::Percentage(60),
            Constraint::Percentage(30),
            Constraint::Length(8),
        ]
    });
    let header_cells = file_headers
        .clone()
        .unwrap_or_else(|| vec!["Path".into(), "Tags".into(), "Score".into()])
        .into_iter()
        .map(Cell::from)
        .collect::<Vec<_>>();
    let header = Row::new(header_cells)
        .style(
            Style::new()
                .fg(Color::Rgb(226, 232, 240))
                .bg(Color::Rgb(15, 23, 42)),
        )
        .height(1)
        .bottom_margin(1);

    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(1)
        .row_highlight_style(
            Style::new()
                .bg(Color::Rgb(30, 41, 59))
                .fg(Color::Rgb(250, 204, 21)),
        )
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(table, area, table_state);
}
