use std::time::Duration;

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState, Wrap,
    },
};

use crate::types::{ModeTexts, SearchData, SearchMode, SearchOutcome, UiConfig};
use crate::utils::{build_file_rows, build_list_rows};
use frizbee::{Config, match_list};

const PREFILTER_ENABLE_THRESHOLD: usize = 1_000;
pub fn run(data: SearchData) -> Result<SearchOutcome> {
    let mut app = App::new(data, UiConfig::default());
    app.run()
}

pub struct App {
    pub data: SearchData,
    pub mode: SearchMode,
    pub input: String,
    pub table_state: TableState,
    pub filtered_primary: Vec<usize>,
    pub filtered_files: Vec<usize>,
    pub primary_scores: Vec<u16>,
    pub file_scores: Vec<u16>,
    pub matcher_config: Config,
    // Customization points for the fzf-like API
    pub(crate) input_title: Option<String>,
    pub(crate) primary_headers: Option<Vec<String>>,
    pub(crate) secondary_headers: Option<Vec<String>>,
    pub(crate) primary_widths: Option<Vec<Constraint>>,
    pub(crate) secondary_widths: Option<Vec<Constraint>>,
    pub(crate) ui_config: UiConfig,
}

impl App {
    pub fn new(data: SearchData, ui_config: UiConfig) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        let matcher_config = Config {
            prefilter: false,
            ..Config::default()
        };
        let mut app = Self {
            data,
            mode: SearchMode::Primary,
            input: String::new(),
            table_state,
            filtered_primary: Vec::new(),
            filtered_files: Vec::new(),
            primary_scores: Vec::new(),
            file_scores: Vec::new(),
            matcher_config,
            input_title: None,
            primary_headers: None,
            secondary_headers: None,
            primary_widths: None,
            secondary_widths: None,
            ui_config,
        };
        app.refresh();
        app
    }

    /// Run the interactive application. This is a method so callers can
    /// customize `App` fields before launching (used by the `Searcher`
    /// builder in the crate root).
    pub fn run(&mut self) -> Result<SearchOutcome> {
        let mut terminal = ratatui::init();
        terminal.clear()?;

        let result = loop {
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(250))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if let Some(outcome) = self.handle_key(key)? {
                            break outcome;
                        }
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
        };

        ratatui::restore();
        Ok(result)
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let area = area.inner(Margin {
            vertical: 1,
            horizontal: 2,
        });

        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(2),
            ])
            .split(area);

        let header = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::styled(
                    &self.ui_config.brand,
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  •  "),
                Span::styled(&self.data.repo_display, Style::new().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::raw(format!("{}: ", self.ui_config.context_label)),
                Span::styled(&self.data.context_value, Style::new().fg(Color::Yellow)),
                Span::raw("  •  "),
                Span::raw(format!(
                    "{}: ",
                    self.ui_config.primary_mode.count_label.as_str()
                )),
                Span::styled(
                    self.data.primary_rows.len().to_string(),
                    Style::new().fg(Color::Green),
                ),
                Span::raw("  •  "),
                Span::raw(format!(
                    "{}: ",
                    self.ui_config.secondary_mode.count_label.as_str()
                )),
                Span::styled(
                    self.data.files.len().to_string(),
                    Style::new().fg(Color::Green),
                ),
            ]),
        ]))
        .alignment(Alignment::Left);

        frame.render_widget(header, outer_layout[0]);

        let hint = Paragraph::new(self.mode_texts().hint.as_str())
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::BOTTOM),
            )
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(hint, outer_layout[1]);

        let body_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(3)])
            .split(outer_layout[2]);

        self.render_input(frame, body_layout[0]);
        self.render_results(frame, body_layout[1]);

        let footer = Paragraph::new(Text::from(vec![Line::from(vec![
            Span::styled(
                "Enter",
                Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" accept  •  "),
            Span::styled(
                "Esc",
                Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" cancel  •  "),
            Span::styled(
                "Tab",
                Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" switch mode"),
        ])]))
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(footer, outer_layout[3]);

        if self.filtered_len() == 0 {
            let empty = Paragraph::new(self.ui_config.no_results_message.as_str())
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .border_type(BorderType::Rounded)
                        .title(self.mode_texts().title.as_str())
                        .borders(Borders::ALL)
                        .border_style(Style::new().fg(Color::DarkGray)),
                );
            frame.render_widget(Clear, body_layout[1]);
            frame.render_widget(empty, body_layout[1]);
        }
    }

    fn render_input(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let title = self
            .input_title
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.mode_texts().title.clone());
        let input = Paragraph::new(self.input.as_str())
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::new().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White));
        frame.render_widget(input, area);
    }

    fn render_results(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        match self.mode {
            SearchMode::Primary => self.render_primary_table(frame, area),
            SearchMode::Secondary => self.render_file_view(frame, area),
        }
    }

    fn render_primary_table(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let query = self.input.trim();
        let highlight_owned = if query.is_empty() {
            None
        } else {
            Some((query.to_string(), self.config_for_query(query)))
        };
        let highlight_state = highlight_owned.as_ref().map(|(s, c)| (s.as_str(), c));
        let rows = build_list_rows(
            &self.filtered_primary,
            &self.primary_scores,
            &self.data.primary_rows,
            highlight_state,
        );

        let widths = self.primary_widths.clone().unwrap_or_else(|| {
            vec![
                Constraint::Percentage(50),
                Constraint::Length(8),
                Constraint::Length(8),
            ]
        });
        let header_cells = self
            .primary_headers
            .clone()
            .unwrap_or_else(|| {
                vec![
                    self.ui_config.primary_mode.count_label.clone(),
                    "Count".into(),
                    "Score".into(),
                ]
            })
            .into_iter()
            .map(Cell::from)
            .collect::<Vec<_>>();
        let header = Row::new(header_cells)
            .style(Style::new().fg(Color::Green))
            .height(1);

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .title(self.ui_config.primary_mode.table_title.as_str())
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(Color::Green)),
            )
            .row_highlight_style(Style::new().bg(Color::DarkGray).fg(Color::Yellow))
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn render_file_view(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(area);
        let table_area = areas[0];
        let detail_area = areas[1];

        let query = self.input.trim();
        let highlight_owned = if query.is_empty() {
            None
        } else {
            Some((query.to_string(), self.config_for_query(query)))
        };
        let highlight_state = highlight_owned.as_ref().map(|(s, c)| (s.as_str(), c));
        let rows = build_file_rows(
            &self.filtered_files,
            &self.file_scores,
            &self.data.files,
            highlight_state,
        );

        let widths = self.secondary_widths.clone().unwrap_or_else(|| {
            vec![
                Constraint::Percentage(55),
                Constraint::Percentage(35),
                Constraint::Length(8),
            ]
        });
        let header_cells = self
            .secondary_headers
            .clone()
            .unwrap_or_else(|| {
                vec![
                    "Path".into(),
                    self.ui_config
                        .secondary_mode
                        .detail_label
                        .clone()
                        .unwrap_or_else(|| "Labels".into()),
                    "Score".into(),
                ]
            })
            .into_iter()
            .map(Cell::from)
            .collect::<Vec<_>>();
        let header = Row::new(header_cells)
            .style(Style::new().fg(Color::Magenta))
            .height(1);

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .title(self.ui_config.secondary_mode.table_title.as_str())
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(Color::Magenta)),
            )
            .row_highlight_style(Style::new().bg(Color::DarkGray).fg(Color::Yellow))
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(table, table_area, &mut self.table_state);

        if let Some(selected) = self
            .table_state
            .selected()
            .and_then(|idx| self.filtered_files.get(idx))
        {
            let entry = &self.data.files[*selected];
            let mut lines = vec![Line::from(vec![Span::styled(
                "Path",
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )])];
            lines.push(Line::from(vec![Span::raw(entry.path.clone())]));
            lines.push(Line::from(""));
            if let Some(detail_label) = self.ui_config.secondary_mode.detail_label.as_ref() {
                lines.push(Line::from(vec![Span::styled(
                    detail_label,
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )]));
            }
            if entry.labels.is_empty() {
                lines.push(Line::from("<none>"));
            } else {
                for label in &entry.labels {
                    lines.push(Line::from(Span::raw(label.clone())));
                }
            }

            let detail = Paragraph::new(Text::from(lines))
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .border_type(BorderType::Rounded)
                        .title(self.ui_config.detail_title.as_str())
                        .borders(Borders::ALL)
                        .border_style(Style::new().fg(Color::Gray)),
                );
            frame.render_widget(detail, detail_area);
        } else {
            let detail = Paragraph::new(self.ui_config.detail_empty_message.as_str())
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .border_type(BorderType::Rounded)
                        .title(self.ui_config.detail_title.as_str())
                        .borders(Borders::ALL)
                        .border_style(Style::new().fg(Color::Gray)),
                );
            frame.render_widget(detail, detail_area);
        }

        let mut scrollbar_state = ScrollbarState::default()
            .content_length(self.filtered_files.len())
            .position(self.table_state.selected().unwrap_or(0));
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::new().bg(Color::Yellow));
        frame.render_stateful_widget(scrollbar, table_area, &mut scrollbar_state);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<Option<SearchOutcome>> {
        match key.code {
            KeyCode::Esc => {
                return Ok(Some(SearchOutcome { accepted: false }));
            }
            KeyCode::Enter => {
                return Ok(Some(SearchOutcome { accepted: true }));
            }
            KeyCode::Tab => {
                self.switch_mode();
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.refresh();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input.clear();
                self.refresh();
            }
            KeyCode::Char(ch) => {
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                {
                    self.input.push(ch);
                    self.refresh();
                }
            }
            KeyCode::Delete => {
                self.input.clear();
                self.refresh();
            }
            KeyCode::Up => {
                self.move_selection_up();
            }
            KeyCode::Down => {
                self.move_selection_down();
            }
            _ => {}
        }
        Ok(None)
    }

    fn switch_mode(&mut self) {
        self.mode = match self.mode {
            SearchMode::Primary => SearchMode::Secondary,
            SearchMode::Secondary => SearchMode::Primary,
        };
        self.table_state.select(Some(0));
        self.refresh();
    }

    fn move_selection_up(&mut self) {
        if let Some(selected) = self.table_state.selected()
            && selected > 0
        {
            self.table_state.select(Some(selected - 1));
        }
    }

    fn move_selection_down(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            let len = self.filtered_len();
            if selected + 1 < len {
                self.table_state.select(Some(selected + 1));
            }
        }
    }

    fn filtered_len(&self) -> usize {
        match self.mode {
            SearchMode::Primary => self.filtered_primary.len(),
            SearchMode::Secondary => self.filtered_files.len(),
        }
    }

    fn refresh(&mut self) {
        match self.mode {
            SearchMode::Primary => self.refresh_primary(),
            SearchMode::Secondary => self.refresh_files(),
        }
        if self.filtered_len() == 0 {
            self.table_state.select(None);
        } else if self.table_state.selected().is_none() {
            self.table_state.select(Some(0));
        } else if let Some(selected) = self.table_state.selected() {
            let len = self.filtered_len();
            if selected >= len {
                self.table_state.select(Some(len.saturating_sub(1)));
            }
        }
    }

    pub(crate) fn refresh_primary(&mut self) {
        let query = self.input.trim();
        if query.is_empty() {
            self.filtered_primary = (0..self.data.primary_rows.len()).collect();
            self.primary_scores = vec![0; self.data.primary_rows.len()];
            self.filtered_primary.sort_by(|&a, &b| {
                self.data.primary_rows[a]
                    .label
                    .cmp(&self.data.primary_rows[b].label)
            });
            return;
        }

        let config = self.config_for_query(query);
        let haystacks: Vec<&str> = self
            .data
            .primary_rows
            .iter()
            .map(|row| row.label.as_str())
            .collect();
        let ranked = match_list(query, &haystacks, &config);
        self.filtered_primary = Vec::new();
        self.primary_scores = Vec::new();
        for entry in ranked {
            if entry.score == 0 {
                continue;
            }
            self.filtered_primary.push(entry.index as usize);
            self.primary_scores.push(entry.score);
        }
    }

    pub(crate) fn refresh_files(&mut self) {
        let query = self.input.trim();
        if query.is_empty() {
            self.filtered_files = (0..self.data.files.len()).collect();
            self.file_scores = vec![0; self.data.files.len()];
            self.filtered_files
                .sort_by(|&a, &b| self.data.files[a].path.cmp(&self.data.files[b].path));
            return;
        }

        let config = self.config_for_query(query);
        let haystacks: Vec<&str> = self
            .data
            .files
            .iter()
            .map(|file| file.search_text())
            .collect();
        let ranked = match_list(query, &haystacks, &config);
        self.filtered_files = Vec::new();
        self.file_scores = Vec::new();
        for entry in ranked {
            if entry.score == 0 {
                continue;
            }
            self.filtered_files.push(entry.index as usize);
            self.file_scores.push(entry.score);
        }
    }

    pub(crate) fn config_for_query(&self, query: &str) -> Config {
        let mut config = self.matcher_config.clone();

        let length = query.chars().count();
        let mut allowed_typos: u16 = match length {
            0 => 0,
            1 => 0,
            2..=4 => 1,
            5..=7 => 2,
            8..=12 => 3,
            _ => 4,
        };
        if let Ok(max_reasonable) = u16::try_from(length.saturating_sub(1)) {
            allowed_typos = allowed_typos.min(max_reasonable);
        }

        let dataset_len = match self.mode {
            SearchMode::Secondary => self.data.files.len(),
            SearchMode::Primary => self.data.primary_rows.len(),
        };

        if dataset_len >= PREFILTER_ENABLE_THRESHOLD {
            config.prefilter = true;
            config.max_typos = Some(allowed_typos);
        } else {
            config.prefilter = false;
            config.max_typos = None;
        }

        config
    }

    fn mode_texts(&self) -> &ModeTexts {
        match self.mode {
            SearchMode::Primary => &self.ui_config.primary_mode,
            SearchMode::Secondary => &self.ui_config.secondary_mode,
        }
    }
}
