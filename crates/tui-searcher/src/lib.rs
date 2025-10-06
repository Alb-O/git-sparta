pub mod app;
pub mod types;
pub mod utils;

pub use app::run;
pub use types::{FileRow, SearchData, SearchMode, TagRow};

use ratatui::layout::Constraint;

/// A small, ergonomic builder for configuring the TUI searcher.
/// This presents a tiny fzf-like API for setting prompts, column
/// headings and column widths before running the interactive picker.
pub struct Searcher {
    data: SearchData,
    input_title: Option<String>,
    tag_headers: Option<Vec<String>>,
    file_headers: Option<Vec<String>>,
    tag_widths: Option<Vec<Constraint>>,
    file_widths: Option<Vec<Constraint>>,
}

impl Searcher {
    /// Create a new Searcher for the provided data.
    pub fn new(data: SearchData) -> Self {
        Self {
            data,
            input_title: None,
            tag_headers: None,
            file_headers: None,
            tag_widths: None,
            file_widths: None,
        }
    }

    pub fn with_input_title(mut self, title: impl Into<String>) -> Self {
        self.input_title = Some(title.into());
        self
    }

    pub fn with_tag_headers(mut self, headers: Vec<&str>) -> Self {
        self.tag_headers = Some(headers.into_iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn with_file_headers(mut self, headers: Vec<&str>) -> Self {
        self.file_headers = Some(headers.into_iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn with_tag_widths(mut self, widths: Vec<Constraint>) -> Self {
        self.tag_widths = Some(widths);
        self
    }

    pub fn with_file_widths(mut self, widths: Vec<Constraint>) -> Self {
        self.file_widths = Some(widths);
        self
    }

    /// Run the interactive searcher with the configured options.
    pub fn run(self) -> anyhow::Result<crate::types::SearchOutcome> {
        // Build an App and apply optional customizations, then run it.
        let mut app = crate::app::App::new(self.data);
        if let Some(title) = self.input_title {
            app.input_title = Some(title);
        }
        if let Some(headers) = self.tag_headers {
            app.tag_headers = Some(headers);
        }
        if let Some(headers) = self.file_headers {
            app.file_headers = Some(headers);
        }
        if let Some(widths) = self.tag_widths {
            app.tag_widths = Some(widths);
        }
        if let Some(widths) = self.file_widths {
            app.file_widths = Some(widths);
        }

        app.run()
    }
}
