pub mod app;
pub mod types;
pub mod utils;

pub use app::run;
pub use types::{FileRow, ListRow, ModeTexts, SearchData, SearchMode, TagRow, UiConfig};

use ratatui::layout::Constraint;

/// A small, ergonomic builder for configuring the TUI searcher.
/// This presents a tiny fzf-like API for setting prompts, column
/// headings and column widths before running the interactive picker.
pub struct Searcher {
    data: SearchData,
    input_title: Option<String>,
    primary_headers: Option<Vec<String>>,
    secondary_headers: Option<Vec<String>>,
    primary_widths: Option<Vec<Constraint>>,
    secondary_widths: Option<Vec<Constraint>>,
    ui_config: UiConfig,
}

impl Searcher {
    /// Create a new Searcher for the provided data.
    pub fn new(data: SearchData) -> Self {
        Self {
            data,
            input_title: None,
            primary_headers: None,
            secondary_headers: None,
            primary_widths: None,
            secondary_widths: None,
            ui_config: UiConfig::default(),
        }
    }

    pub fn with_input_title(mut self, title: impl Into<String>) -> Self {
        self.input_title = Some(title.into());
        self
    }

    pub fn with_primary_headers(mut self, headers: Vec<&str>) -> Self {
        self.primary_headers = Some(headers.into_iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn with_secondary_headers(mut self, headers: Vec<&str>) -> Self {
        self.secondary_headers = Some(headers.into_iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn with_primary_widths(mut self, widths: Vec<Constraint>) -> Self {
        self.primary_widths = Some(widths);
        self
    }

    pub fn with_secondary_widths(mut self, widths: Vec<Constraint>) -> Self {
        self.secondary_widths = Some(widths);
        self
    }

    pub fn with_ui_config(mut self, ui_config: UiConfig) -> Self {
        self.ui_config = ui_config;
        self
    }

    pub fn with_primary_mode_texts(mut self, config: ModeTexts) -> Self {
        self.ui_config.primary_mode = config;
        self
    }

    pub fn with_secondary_mode_texts(mut self, config: ModeTexts) -> Self {
        self.ui_config.secondary_mode = config;
        self
    }

    pub fn with_branding(mut self, brand: impl Into<String>) -> Self {
        self.ui_config.brand = brand.into();
        self
    }

    pub fn with_context_label(mut self, label: impl Into<String>) -> Self {
        self.ui_config.context_label = label.into();
        self
    }

    pub fn with_detail_title(mut self, title: impl Into<String>) -> Self {
        self.ui_config.detail_title = title.into();
        self
    }

    pub fn with_detail_empty_message(mut self, message: impl Into<String>) -> Self {
        self.ui_config.detail_empty_message = message.into();
        self
    }

    pub fn with_no_results_message(mut self, message: impl Into<String>) -> Self {
        self.ui_config.no_results_message = message.into();
        self
    }

    #[deprecated(note = "use with_primary_headers instead")]
    pub fn with_tag_headers(self, headers: Vec<&str>) -> Self {
        self.with_primary_headers(headers)
    }

    #[deprecated(note = "use with_secondary_headers instead")]
    pub fn with_file_headers(self, headers: Vec<&str>) -> Self {
        self.with_secondary_headers(headers)
    }

    #[deprecated(note = "use with_primary_widths instead")]
    pub fn with_tag_widths(self, widths: Vec<Constraint>) -> Self {
        self.with_primary_widths(widths)
    }

    #[deprecated(note = "use with_secondary_widths instead")]
    pub fn with_file_widths(self, widths: Vec<Constraint>) -> Self {
        self.with_secondary_widths(widths)
    }

    /// Run the interactive searcher with the configured options.
    pub fn run(self) -> anyhow::Result<crate::types::SearchOutcome> {
        // Build an App and apply optional customizations, then run it.
        let mut app = crate::app::App::new(self.data, self.ui_config);
        if let Some(title) = self.input_title {
            app.input_title = Some(title);
        }
        if let Some(headers) = self.primary_headers {
            app.primary_headers = Some(headers);
        }
        if let Some(headers) = self.secondary_headers {
            app.secondary_headers = Some(headers);
        }
        if let Some(widths) = self.primary_widths {
            app.primary_widths = Some(widths);
        }
        if let Some(widths) = self.secondary_widths {
            app.secondary_widths = Some(widths);
        }

        app.run()
    }
}
