use std::{mem, time::Duration};

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState, Wrap,
    },
};

use frizbee::{Config, match_indices, match_list};

#[derive(Debug, Clone)]
pub struct TagRow {
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct FileRow {
    pub path: String,
    pub tags: Vec<String>,
    pub display_tags: String,
    search_text: String,
}

impl FileRow {
    #[must_use]
    pub fn new(path: String, tags: Vec<String>) -> Self {
        let mut tags_sorted = tags;
        tags_sorted.sort();
        let display_tags = tags_sorted.join(", ");
        let search_text = if display_tags.is_empty() {
            path.clone()
        } else {
            format!("{path} {display_tags}")
        };
        Self {
            path,
            tags: tags_sorted,
            display_tags,
            search_text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> SearchData {
        SearchData {
            repo_display: "example/repo".to_string(),
            user_filter: "".to_string(),
            tags: vec![
                TagRow {
                    name: "docs".to_string(),
                    count: 4,
                },
                TagRow {
                    name: "tests".to_string(),
                    count: 2,
                },
            ],
            files: vec![FileRow::new("ui.rs".to_string(), vec!["ui".to_string()])],
        }
    }

    #[test]
    fn config_allows_typos_for_short_queries() {
        let app = App::new(sample_data());
        let config = app.config_for_query("uo");
        assert_eq!(config.max_typos, Some(1));
    }

    #[test]
    fn subsequence_query_matches_tags() {
        let mut app = App::new(sample_data());
        app.input = "cs".to_string();
        app.refresh_tags();
        let names: Vec<&str> = app
            .filtered_tags
            .iter()
            .map(|&idx| app.data.tags[idx].name.as_str())
            .collect();
        assert!(names.contains(&"docs"));
    }

    #[test]
    fn substitution_query_matches_files() {
        let mut app = App::new(sample_data());
        app.mode = SearchMode::Files;
        app.input = "uo".to_string();
        app.refresh_files();
        let paths: Vec<&str> = app
            .filtered_files
            .iter()
            .map(|&idx| app.data.files[idx].path.as_str())
            .collect();
        assert!(paths.contains(&"ui.rs"));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Tags,
    Files,
}

impl SearchMode {
    fn title(self) -> &'static str {
        match self {
            SearchMode::Tags => "Tag search",
            SearchMode::Files => "File search",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            SearchMode::Tags => "Type to filter tags. Press Tab to view files.",
            SearchMode::Files => "Type to filter files by path or tag. Press Tab to view tags.",
        }
    }
}

pub struct SearchData {
    pub repo_display: String,
    pub user_filter: String,
    pub tags: Vec<TagRow>,
    pub files: Vec<FileRow>,
}

pub struct SearchOutcome {
    pub accepted: bool,
}

pub fn run(data: SearchData) -> Result<SearchOutcome> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let mut app = App::new(data);

    let result = loop {
        terminal.draw(|frame| app.draw(frame))?;

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if let Some(outcome) = app.handle_key(key)? {
                        break outcome;
                    }
                }
                Event::Resize(_, _) => {
                    // Redraw handled at start of next loop iteration.
                }
                _ => {}
            }
        }
    };

    ratatui::restore();
    Ok(result)
}

struct App {
    data: SearchData,
    mode: SearchMode,
    input: String,
    table_state: TableState,
    filtered_tags: Vec<usize>,
    filtered_files: Vec<usize>,
    tag_scores: Vec<u16>,
    file_scores: Vec<u16>,
    matcher_config: Config,
}

impl App {
    fn new(data: SearchData) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        let mut matcher_config = Config::default();
        matcher_config.prefilter = true;
        let mut app = Self {
            data,
            mode: SearchMode::Tags,
            input: String::new(),
            table_state,
            filtered_tags: Vec::new(),
            filtered_files: Vec::new(),
            tag_scores: Vec::new(),
            file_scores: Vec::new(),
            matcher_config,
        };
        app.refresh();
        app
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
                    "git-sparta",
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  •  "),
                Span::styled(&self.data.repo_display, Style::new().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::raw("Filter tag: "),
                Span::styled(&self.data.user_filter, Style::new().fg(Color::Yellow)),
                Span::raw("  •  Tags: "),
                Span::styled(
                    self.data.tags.len().to_string(),
                    Style::new().fg(Color::Green),
                ),
                Span::raw("  •  Files: "),
                Span::styled(
                    self.data.files.len().to_string(),
                    Style::new().fg(Color::Green),
                ),
            ]),
        ]))
        .alignment(Alignment::Left);

        frame.render_widget(header, outer_layout[0]);

        let hint = Paragraph::new(self.mode.hint())
            .block(Block::default().borders(Borders::BOTTOM))
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
            let empty = Paragraph::new("No results")
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title(self.mode.title())
                        .borders(Borders::ALL)
                        .border_style(Style::new().fg(Color::DarkGray)),
                );
            frame.render_widget(Clear, body_layout[1]);
            frame.render_widget(empty, body_layout[1]);
        }
    }

    fn render_input(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let title = format!("{}", self.mode.title());
        let input = Paragraph::new(self.input.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::new().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White));
        frame.render_widget(input, area);
    }

    fn render_results(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        match self.mode {
            SearchMode::Tags => self.render_tag_table(frame, area),
            SearchMode::Files => self.render_file_view(frame, area),
        }
    }

    fn render_tag_table(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let highlight_state = {
            let query = self.input.trim();
            if query.is_empty() {
                None
            } else {
                Some((query.to_string(), self.config_for_query(query)))
            }
        };

        let rows: Vec<Row> = self
            .filtered_tags
            .iter()
            .enumerate()
            .map(|(idx, &actual_index)| {
                let tag = &self.data.tags[actual_index];
                let score = self.tag_scores.get(idx).copied().unwrap_or_default();
                let highlight = highlight_state
                    .as_ref()
                    .and_then(|(needle, config)| Self::highlight_for(needle, config, &tag.name));
                Row::new([
                    Self::highlight_cell(&tag.name, highlight),
                    Cell::from(tag.count.to_string()),
                    Cell::from(score.to_string()),
                ])
            })
            .collect();

        let widths = [
            Constraint::Percentage(50),
            Constraint::Length(8),
            Constraint::Length(8),
        ];
        let table = Table::new(rows, widths)
            .block(
                Block::default()
                    .title("Matching tags")
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

        let highlight_state = {
            let query = self.input.trim();
            if query.is_empty() {
                None
            } else {
                Some((query.to_string(), self.config_for_query(query)))
            }
        };

        let rows: Vec<Row> = self
            .filtered_files
            .iter()
            .enumerate()
            .map(|(idx, &actual_index)| {
                let entry = &self.data.files[actual_index];
                let score = self.file_scores.get(idx).copied().unwrap_or_default();
                let path_highlight = highlight_state
                    .as_ref()
                    .and_then(|(needle, config)| Self::highlight_for(needle, config, &entry.path));
                let tag_highlight = highlight_state.as_ref().and_then(|(needle, config)| {
                    Self::highlight_for(needle, config, &entry.display_tags)
                });
                Row::new([
                    Self::highlight_cell(&entry.path, path_highlight),
                    Self::highlight_cell(&entry.display_tags, tag_highlight),
                    Cell::from(score.to_string()),
                ])
            })
            .collect();

        let widths = [
            Constraint::Percentage(55),
            Constraint::Percentage(35),
            Constraint::Length(8),
        ];
        let table = Table::new(rows, widths)
            .block(
                Block::default()
                    .title("Matching files")
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
            lines.push(Line::from(vec![Span::styled(
                "Tags",
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )]));
            if entry.tags.is_empty() {
                lines.push(Line::from("<none>"));
            } else {
                for tag in &entry.tags {
                    lines.push(Line::from(Span::raw(tag.clone())));
                }
            }

            let detail = Paragraph::new(Text::from(lines))
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .title("Selection details")
                        .borders(Borders::ALL)
                        .border_style(Style::new().fg(Color::Gray)),
                );
            frame.render_widget(detail, detail_area);
        } else {
            let detail = Paragraph::new("No selection")
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title("Selection details")
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
            KeyCode::Char('q') if key.modifiers.is_empty() => {
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
            SearchMode::Tags => SearchMode::Files,
            SearchMode::Files => SearchMode::Tags,
        };
        self.table_state.select(Some(0));
        self.refresh();
    }

    fn move_selection_up(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            if selected > 0 {
                self.table_state.select(Some(selected - 1));
            }
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
            SearchMode::Tags => self.filtered_tags.len(),
            SearchMode::Files => self.filtered_files.len(),
        }
    }

    fn refresh(&mut self) {
        match self.mode {
            SearchMode::Tags => self.refresh_tags(),
            SearchMode::Files => self.refresh_files(),
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

    fn refresh_tags(&mut self) {
        let query = self.input.trim();
        if query.is_empty() {
            self.filtered_tags = (0..self.data.tags.len()).collect();
            self.tag_scores = vec![0; self.data.tags.len()];
            self.filtered_tags
                .sort_by(|&a, &b| self.data.tags[a].name.cmp(&self.data.tags[b].name));
            return;
        }

        let config = self.config_for_query(query);
        let haystacks: Vec<&str> = self.data.tags.iter().map(|tag| tag.name.as_str()).collect();
        let ranked = match_list(query, &haystacks, &config);
        self.filtered_tags = Vec::new();
        self.tag_scores = Vec::new();
        for entry in ranked {
            if entry.score == 0 {
                continue;
            }
            self.filtered_tags.push(entry.index as usize);
            self.tag_scores.push(entry.score);
        }
    }

    fn refresh_files(&mut self) {
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
            .map(|file| file.search_text.as_str())
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

    fn config_for_query(&self, query: &str) -> Config {
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
        config.max_typos = Some(allowed_typos);
        config
    }

    fn highlight_for(query: &str, config: &Config, text: &str) -> Option<Vec<usize>> {
        match_indices(query, text, config).map(|m| m.indices)
    }

    fn highlight_cell(text: &str, indices: Option<Vec<usize>>) -> Cell<'_> {
        let Some(mut sorted_indices) = indices.filter(|indices| !indices.is_empty()) else {
            return Cell::from(text.to_string());
        };
        sorted_indices.sort_unstable();
        let mut next = sorted_indices.into_iter().peekable();
        let mut buffer = String::new();
        let mut highlighted = false;
        let mut spans = Vec::new();
        let highlight_style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);

        for (idx, ch) in text.chars().enumerate() {
            let should_highlight = next.peek().copied() == Some(idx);
            if should_highlight {
                next.next();
            }
            if should_highlight != highlighted {
                if !buffer.is_empty() {
                    let style = if highlighted {
                        highlight_style
                    } else {
                        Style::default()
                    };
                    spans.push(Span::styled(mem::take(&mut buffer), style));
                }
                highlighted = should_highlight;
            }
            buffer.push(ch);
        }

        if !buffer.is_empty() {
            let style = if highlighted {
                highlight_style
            } else {
                Style::default()
            };
            spans.push(Span::styled(buffer, style));
        }

        Cell::from(Text::from(Line::from(spans)))
    }
}
