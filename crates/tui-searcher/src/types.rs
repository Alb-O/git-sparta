use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Cell;
use std::mem;

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

    /// Return the search_text (path plus display tags) used by the UI matcher.
    pub(crate) fn search_text(&self) -> &str {
        &self.search_text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Tags,
    Files,
}

impl SearchMode {
    pub fn title(self) -> &'static str {
        match self {
            SearchMode::Tags => "Tag search",
            SearchMode::Files => "File search",
        }
    }

    pub fn hint(self) -> &'static str {
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
