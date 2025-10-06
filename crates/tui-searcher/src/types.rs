use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Cell;
use std::mem;

#[derive(Debug, Clone)]
pub struct ListRow {
    pub label: String,
    pub count: usize,
}

/// Backwards-compatible alias for callers using the old, tag-focused name.
pub type TagRow = ListRow;

#[derive(Debug, Clone)]
pub struct FileRow {
    pub path: String,
    pub labels: Vec<String>,
    pub display_labels: String,
    search_text: String,
}

impl FileRow {
    #[must_use]
    pub fn new(path: String, labels: Vec<String>) -> Self {
        let mut labels_sorted = labels;
        labels_sorted.sort();
        let display_labels = labels_sorted.join(", ");
        let search_text = if display_labels.is_empty() {
            path.clone()
        } else {
            format!("{path} {display_labels}")
        };
        Self {
            path,
            labels: labels_sorted,
            display_labels,
            search_text,
        }
    }

    /// Return the search_text (path plus display labels) used by the UI matcher.
    pub(crate) fn search_text(&self) -> &str {
        &self.search_text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Primary,
    Secondary,
}

#[derive(Debug, Clone)]
pub struct ModeTexts {
    pub title: String,
    pub hint: String,
    pub table_title: String,
    pub count_label: String,
    pub detail_label: Option<String>,
}

impl ModeTexts {
    #[must_use]
    pub fn for_primary() -> Self {
        Self {
            title: "Tag search".to_string(),
            hint: "Type to filter tags. Press Tab to view files.".to_string(),
            table_title: "Matching tags".to_string(),
            count_label: "Tags".to_string(),
            detail_label: None,
        }
    }

    #[must_use]
    pub fn for_secondary() -> Self {
        Self {
            title: "File search".to_string(),
            hint: "Type to filter files by path or tag. Press Tab to view tags.".to_string(),
            table_title: "Matching files".to_string(),
            count_label: "Files".to_string(),
            detail_label: Some("Tags".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UiConfig {
    pub brand: String,
    pub context_label: String,
    pub detail_title: String,
    pub detail_empty_message: String,
    pub no_results_message: String,
    pub primary_mode: ModeTexts,
    pub secondary_mode: ModeTexts,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            brand: "git-sparta".to_string(),
            context_label: "Filter tag".to_string(),
            detail_title: "Selection details".to_string(),
            detail_empty_message: "No selection".to_string(),
            no_results_message: "No results".to_string(),
            primary_mode: ModeTexts::for_primary(),
            secondary_mode: ModeTexts::for_secondary(),
        }
    }
}

pub struct SearchData {
    pub repo_display: String,
    pub context_value: String,
    pub primary_rows: Vec<ListRow>,
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
