use std::time::Duration;

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Style},
    widgets::{Clear, Paragraph, TableState},
};

use crate::input::SearchInput;
use crate::tables;
use crate::tabs;
use crate::types::{SearchData, SearchMode, SearchOutcome, UiConfig};
use frizbee::{Config, match_list};

const PREFILTER_ENABLE_THRESHOLD: usize = 1_000;
pub fn run(data: SearchData) -> Result<SearchOutcome> {
    let mut app: App = App::new(data);
    app.run()
}

pub struct App<'a> {
    pub data: SearchData,
    pub mode: SearchMode,
    pub search_input: SearchInput<'a>,
    pub table_state: TableState,
    pub filtered_facets: Vec<usize>,
    pub filtered_files: Vec<usize>,
    pub facet_scores: Vec<u16>,
    pub file_scores: Vec<u16>,
    pub matcher_config: Config,
    // Customization points for the fzf-like API
    pub(crate) input_title: Option<String>,
    pub(crate) facet_headers: Option<Vec<String>>,
    pub(crate) file_headers: Option<Vec<String>>,
    pub(crate) facet_widths: Option<Vec<Constraint>>,
    pub(crate) file_widths: Option<Vec<Constraint>>,
    pub(crate) ui: UiConfig,
}

impl<'a> App<'a> {
    pub fn new(data: SearchData) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        let matcher_config = Config {
            prefilter: false,
            ..Config::default()
        };
        let mut app = Self {
            data,
            mode: SearchMode::Facets,
            search_input: SearchInput::default(),
            table_state,
            filtered_facets: Vec::new(),
            filtered_files: Vec::new(),
            facet_scores: Vec::new(),
            file_scores: Vec::new(),
            matcher_config,
            input_title: None,
            facet_headers: None,
            file_headers: None,
            facet_widths: None,
            file_widths: None,
            ui: UiConfig::default(),
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
            vertical: 0,
            horizontal: 1,
        });

        // Input/tabs row (top line) and results below
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);

        // Delegate input + tabs rendering
        tabs::render_input_with_tabs(
            &self.search_input,
            &self.input_title,
            self.mode,
            &self.ui,
            frame,
            layout[0],
        );
        self.render_results(frame, layout[1]);

        // Minimal empty state
        if self.filtered_len() == 0 {
            let empty = Paragraph::new("No results")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(Clear, layout[1]);
            frame.render_widget(empty, layout[1]);
        }
    }

    fn render_results(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        match self.mode {
            SearchMode::Facets => {
                let query = self.search_input.text().trim();
                let highlight_owned = if query.is_empty() {
                    None
                } else {
                    Some((query.to_string(), self.config_for_query(query)))
                };
                let highlight_state: Option<(&str, &Config)> =
                    highlight_owned.as_ref().map(|(s, c)| (s.as_str(), c));
                tables::render_facet_table(
                    frame,
                    area,
                    &self.filtered_facets,
                    &self.facet_scores,
                    &self.data.facets,
                    &self.facet_headers,
                    &self.facet_widths,
                    &mut self.table_state,
                    &self.ui,
                    highlight_state,
                )
            }
            SearchMode::Files => {
                let query = self.search_input.text().trim();
                let highlight_owned = if query.is_empty() {
                    None
                } else {
                    Some((query.to_string(), self.config_for_query(query)))
                };
                let highlight_state: Option<(&str, &Config)> =
                    highlight_owned.as_ref().map(|(s, c)| (s.as_str(), c));
                tables::render_file_view(
                    frame,
                    area,
                    &self.filtered_files,
                    &self.file_scores,
                    &self.data.files,
                    &self.file_headers,
                    &self.file_widths,
                    &mut self.table_state,
                    &self.ui,
                    highlight_state,
                )
            }
        }
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
            KeyCode::Up => {
                self.move_selection_up();
            }
            KeyCode::Down => {
                self.move_selection_down();
            }
            _ => {
                // Let SearchInput handle all keys including arrow keys (for cursor movement), typing, backspace, etc.
                if self.search_input.input(key) {
                    self.refresh();
                }
            }
        }
        Ok(None)
    }

    fn switch_mode(&mut self) {
        self.mode = match self.mode {
            SearchMode::Facets => SearchMode::Files,
            SearchMode::Files => SearchMode::Facets,
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
            SearchMode::Facets => self.filtered_facets.len(),
            SearchMode::Files => self.filtered_files.len(),
        }
    }

    fn refresh(&mut self) {
        match self.mode {
            SearchMode::Facets => self.refresh_facets(),
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

    pub(crate) fn refresh_facets(&mut self) {
        let query = self.search_input.text().trim();
        if query.is_empty() {
            self.filtered_facets = (0..self.data.facets.len()).collect();
            self.facet_scores = vec![0; self.data.facets.len()];
            self.filtered_facets
                .sort_by(|&a, &b| self.data.facets[a].name.cmp(&self.data.facets[b].name));
            return;
        }

        let config = self.config_for_query(query);
        let haystacks: Vec<&str> = self
            .data
            .facets
            .iter()
            .map(|facet| facet.name.as_str())
            .collect();
        let ranked = match_list(query, &haystacks, &config);
        self.filtered_facets = Vec::new();
        self.facet_scores = Vec::new();
        for entry in ranked {
            if entry.score == 0 {
                continue;
            }
            self.filtered_facets.push(entry.index as usize);
            self.facet_scores.push(entry.score);
        }
    }

    pub(crate) fn refresh_files(&mut self) {
        let query = self.search_input.text().trim();
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
            SearchMode::Files => self.data.files.len(),
            SearchMode::Facets => self.data.facets.len(),
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
}
