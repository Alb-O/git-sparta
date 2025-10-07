use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Cell;
use std::mem;

#[derive(Debug, Clone)]
pub struct FacetRow {
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

    /// Return the search_text (path plus display tags) used by the UI matcher.
    pub(crate) fn search_text(&self) -> &str {
        &self.search_text
    }
}

#[derive(Debug, Clone)]
pub struct PaneUiConfig {
    pub mode_title: String,
    pub hint: String,
    pub table_title: String,
    pub count_label: String,
}

impl PaneUiConfig {
    #[must_use]
    pub fn new(
        mode_title: impl Into<String>,
        hint: impl Into<String>,
        table_title: impl Into<String>,
        count_label: impl Into<String>,
    ) -> Self {
        Self {
            mode_title: mode_title.into(),
            hint: hint.into(),
            table_title: table_title.into(),
            count_label: count_label.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UiConfig {
    pub filter_label: String,
    pub facets: PaneUiConfig,
    pub files: PaneUiConfig,
    pub detail_panel_title: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            filter_label: "Filter facets".to_string(),
            facets: PaneUiConfig::new(
                "Facet search",
                "Type to filter facets. Press Tab to view files.",
                "Matching facets",
                "Facets",
            ),
            files: PaneUiConfig::new(
                "File search",
                "Type to filter files. Press Tab to view facets.",
                "Matching files",
                "Files",
            ),
            detail_panel_title: "Selection details".to_string(),
        }
    }
}

impl UiConfig {
    #[must_use]
    pub fn tags_and_files() -> Self {
        Self {
            filter_label: "Filter tag".to_string(),
            facets: PaneUiConfig::new(
                "Tag search",
                "Type to filter tags. Press Tab to view files.",
                "Matching tags",
                "Tags",
            ),
            files: PaneUiConfig::new(
                "File search",
                "Type to filter files by path or tag. Press Tab to view tags.",
                "Matching files",
                "Files",
            ),
            detail_panel_title: "Selection details".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Facets,
    Files,
}

impl SearchMode {
    pub fn title(self, ui: &UiConfig) -> &str {
        match self {
            SearchMode::Facets => ui.facets.mode_title.as_str(),
            SearchMode::Files => ui.files.mode_title.as_str(),
        }
    }

    pub fn hint(self, ui: &UiConfig) -> &str {
        match self {
            SearchMode::Facets => ui.facets.hint.as_str(),
            SearchMode::Files => ui.files.hint.as_str(),
        }
    }

    pub fn table_title(self, ui: &UiConfig) -> &str {
        match self {
            SearchMode::Facets => ui.facets.table_title.as_str(),
            SearchMode::Files => ui.files.table_title.as_str(),
        }
    }

    pub fn count_label(self, ui: &UiConfig) -> &str {
        match self {
            SearchMode::Facets => ui.facets.count_label.as_str(),
            SearchMode::Files => ui.files.count_label.as_str(),
        }
    }
}

pub struct SearchData {
    pub repo_display: String,
    pub user_filter: String,
    pub facets: Vec<FacetRow>,
    pub files: Vec<FileRow>,
}

pub struct SearchOutcome {
    pub accepted: bool,
}

pub(crate) fn highlight_cell(text: &str, indices: Option<Vec<usize>>) -> Cell<'_> {
    let Some(mut sorted_indices) = indices.filter(|indices| !indices.is_empty()) else {
        return Cell::from(text.to_string());
    };
    sorted_indices.sort_unstable();
    let mut next = sorted_indices.into_iter().peekable();
    let mut buffer = String::new();
    let mut highlighted = false;
    let mut spans = Vec::new();
    let theme = crate::theme::Theme::default();
    let highlight_style = Style::new()
        .fg(theme.highlight_fg)
        .add_modifier(Modifier::BOLD);

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
