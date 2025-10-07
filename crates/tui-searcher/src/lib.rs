pub mod app;
pub mod input;
pub mod types;
pub mod utils;

pub use app::run;
pub use input::SearchInput;
pub use types::{FacetRow, FileRow, PaneUiConfig, SearchData, SearchMode, UiConfig};

use ratatui::layout::Constraint;

/// A small, ergonomic builder for configuring the TUI searcher.
/// This presents a tiny fzf-like API for setting prompts, column
/// headings and column widths before running the interactive picker.
pub struct Searcher {
    data: SearchData,
    input_title: Option<String>,
    facet_headers: Option<Vec<String>>,
    file_headers: Option<Vec<String>>,
    facet_widths: Option<Vec<Constraint>>,
    file_widths: Option<Vec<Constraint>>,
    ui_config: Option<UiConfig>,
}

impl Searcher {
    /// Create a new Searcher for the provided data.
    pub fn new(data: SearchData) -> Self {
        Self {
            data,
            input_title: None,
            facet_headers: None,
            file_headers: None,
            facet_widths: None,
            file_widths: None,
            ui_config: None,
        }
    }

    pub fn with_input_title(mut self, title: impl Into<String>) -> Self {
        self.input_title = Some(title.into());
        self
    }

    pub fn with_headers_for(mut self, mode: SearchMode, headers: Vec<&str>) -> Self {
        let headers = headers.into_iter().map(|s| s.to_string()).collect();
        match mode {
            SearchMode::Facets => self.facet_headers = Some(headers),
            SearchMode::Files => self.file_headers = Some(headers),
        }
        self
    }

    pub fn with_widths_for(mut self, mode: SearchMode, widths: Vec<Constraint>) -> Self {
        match mode {
            SearchMode::Facets => self.facet_widths = Some(widths),
            SearchMode::Files => self.file_widths = Some(widths),
        }
        self
    }

    pub fn with_ui_config(mut self, config: UiConfig) -> Self {
        self.ui_config = Some(config);
        self
    }

    /// Run the interactive searcher with the configured options.
    pub fn run(self) -> anyhow::Result<crate::types::SearchOutcome> {
        // Build an App and apply optional customizations, then run it.
        let mut app = crate::app::App::new(self.data);
        if let Some(title) = self.input_title {
            app.input_title = Some(title);
        }
        if let Some(headers) = self.facet_headers {
            app.facet_headers = Some(headers);
        }
        if let Some(headers) = self.file_headers {
            app.file_headers = Some(headers);
        }
        if let Some(widths) = self.facet_widths {
            app.facet_widths = Some(widths);
        }
        if let Some(widths) = self.file_widths {
            app.file_widths = Some(widths);
        }
        if let Some(ui) = self.ui_config {
            app.ui = ui;
        }

        app.run()
    }
}
