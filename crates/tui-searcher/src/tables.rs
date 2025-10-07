use frizbee::Config;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Cell, Row, Table};

use crate::types::UiConfig;
use crate::utils::{build_facet_rows, build_file_rows};

/// Description of a table pane to render.
pub enum TablePane<'a> {
    Facets {
        filtered: &'a [usize],
        scores: &'a [u16],
        facets: &'a [crate::types::FacetRow],
        headers: Option<&'a Vec<String>>,
        widths: Option<&'a Vec<Constraint>>,
    },
    Files {
        filtered: &'a [usize],
        scores: &'a [u16],
        files: &'a [crate::types::FileRow],
        headers: Option<&'a Vec<String>>,
        widths: Option<&'a Vec<Constraint>>,
    },
}

/// Unified renderer for both kinds of tables. Accepts a `TablePane` which
/// packages all pane-specific data.
pub fn render_table(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    table_state: &mut ratatui::widgets::TableState,
    _ui: &UiConfig,
    highlight_state: Option<(&str, &Config)>,
    pane: TablePane<'_>,
) {
    match pane {
        TablePane::Facets {
            filtered,
            scores,
            facets,
            headers,
            widths,
        } => {
            let rows = build_facet_rows(filtered, scores, facets, highlight_state);
            let widths_owned = widths.cloned().unwrap_or_else(|| {
                vec![
                    Constraint::Percentage(50),
                    Constraint::Length(8),
                    Constraint::Length(8),
                ]
            });
            let header_cells = headers
                .cloned()
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

            let table = Table::new(rows, widths_owned)
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
        TablePane::Files {
            filtered,
            scores,
            files,
            headers,
            widths,
        } => {
            let rows = build_file_rows(filtered, scores, files, highlight_state);
            let widths_owned = widths.cloned().unwrap_or_else(|| {
                vec![
                    Constraint::Percentage(60),
                    Constraint::Percentage(30),
                    Constraint::Length(8),
                ]
            });
            let header_cells = headers
                .cloned()
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

            let table = Table::new(rows, widths_owned)
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
    }
}
