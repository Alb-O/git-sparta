use frizbee::Config;
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Cell, Paragraph, Row, Table};

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
    theme: &crate::theme::Theme,
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
                .style(theme.header_style())
                .height(1)
                .bottom_margin(1);

            let table = Table::new(rows, widths_owned)
                .header(header)
                .column_spacing(1)
                .row_highlight_style(theme.row_highlight_style())
                .highlight_symbol("▶ ");
            frame.render_stateful_widget(table, area, table_state);

            // Draw a horizontal separator under the header to replace the
            // previous blank line. We render a Paragraph filled with box
            // drawing characters across the table width and overlay it.
            let header_height = 1u16; // header.height() was set to 1 above
            if header_height < area.height {
                let sep_y = area.y + header_height;
                if sep_y < area.y + area.height {
                    let width = area.width as usize;
                    if width == 0 {
                        // nothing to draw
                    } else if width <= 2 {
                        let line = " ".repeat(width);
                        let para = Paragraph::new(line).style(Style::new().bg(theme.header_bg));
                        let sep_rect = Rect {
                            x: area.x,
                            y: sep_y,
                            width: area.width,
                            height: 1,
                        };
                        frame.render_widget(para, sep_rect);
                    } else {
                        let middle = "─".repeat(width - 2);
                        let spans = vec![
                            Span::styled(" ", Style::new().bg(theme.header_bg)),
                            Span::styled(
                                &middle,
                                Style::new().bg(theme.header_bg).fg(theme.header_fg),
                            ),
                            Span::styled(" ", Style::new().bg(theme.header_bg)),
                        ];
                        let para = Paragraph::new(Text::from(Line::from(spans)));
                        let sep_rect = Rect {
                            x: area.x,
                            y: sep_y,
                            width: area.width,
                            height: 1,
                        };
                        frame.render_widget(para, sep_rect);
                    }
                }
            }
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
                .style(theme.header_style())
                .height(1)
                .bottom_margin(1);

            let table = Table::new(rows, widths_owned)
                .header(header)
                .column_spacing(1)
                .row_highlight_style(theme.row_highlight_style())
                .highlight_symbol("▶ ");
            frame.render_stateful_widget(table, area, table_state);

            let header_height = 1u16;
            if header_height < area.height {
                let sep_y = area.y + header_height;
                if sep_y < area.y + area.height {
                    let width = area.width as usize;
                    if width == 0 {
                        // nothing
                    } else if width <= 2 {
                        let line = " ".repeat(width);
                        let para = Paragraph::new(line).style(Style::new().bg(theme.header_bg));
                        let sep_rect = Rect {
                            x: area.x,
                            y: sep_y,
                            width: area.width,
                            height: 1,
                        };
                        frame.render_widget(para, sep_rect);
                    } else {
                        let middle = "─".repeat(width - 2);
                        let spans = vec![
                            Span::styled(" ", Style::new().bg(theme.header_bg)),
                            Span::styled(
                                &middle,
                                Style::new().bg(theme.header_bg).fg(theme.header_fg),
                            ),
                            Span::styled(" ", Style::new().bg(theme.header_bg)),
                        ];
                        let para = Paragraph::new(Text::from(Line::from(spans)));
                        let sep_rect = Rect {
                            x: area.x,
                            y: sep_y,
                            width: area.width,
                            height: 1,
                        };
                        frame.render_widget(para, sep_rect);
                    }
                }
            }
        }
    }
}
