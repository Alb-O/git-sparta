use anyhow::{Context, Result};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::mem;
use std::path::{Component, Path};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FacetRow {
    pub name: String,
    pub count: usize,
}

impl FacetRow {
    #[must_use]
    pub fn new(name: impl Into<String>, count: usize) -> Self {
        Self {
            name: name.into(),
            count,
        }
    }
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
    pub fn new<I, S>(path: impl Into<String>, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let path = path.into();
        let mut tags_sorted: Vec<String> = tags.into_iter().map(Into::into).collect();
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
    pub context_label: Option<String>,
    pub initial_query: String,
    pub facets: Vec<FacetRow>,
    pub files: Vec<FileRow>,
}

impl Default for SearchData {
    fn default() -> Self {
        Self {
            context_label: None,
            initial_query: String::new(),
            facets: Vec::new(),
            files: Vec::new(),
        }
    }
}

impl SearchData {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_context(mut self, label: impl Into<String>) -> Self {
        self.context_label = Some(label.into());
        self
    }

    #[must_use]
    pub fn with_initial_query(mut self, query: impl Into<String>) -> Self {
        self.initial_query = query.into();
        self
    }

    #[must_use]
    pub fn with_facets(mut self, facets: Vec<FacetRow>) -> Self {
        self.facets = facets;
        self
    }

    #[must_use]
    pub fn with_files(mut self, files: Vec<FileRow>) -> Self {
        self.files = files;
        self
    }

    pub fn from_filesystem(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let mut files = Vec::new();
        let mut facet_counts: BTreeMap<String, usize> = BTreeMap::new();

        for entry in WalkDir::new(root).into_iter() {
            let entry = entry.with_context(|| format!("failed to walk {}", root.display()))?;
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap_or(path);
            let mut tags: BTreeSet<String> = BTreeSet::new();

            if let Some(parent) = relative.parent() {
                for component in parent.components() {
                    if let Component::Normal(part) = component {
                        let value = part.to_string_lossy().to_string();
                        if !value.is_empty() {
                            tags.insert(value);
                        }
                    }
                }
            }

            if let Some(ext) = relative.extension().and_then(|ext| ext.to_str()) {
                if !ext.is_empty() {
                    tags.insert(format!("*.{ext}"));
                }
            }

            let tags_vec: Vec<String> = tags.into_iter().collect();
            for tag in &tags_vec {
                *facet_counts.entry(tag.clone()).or_default() += 1;
            }

            let relative_display = relative.to_string_lossy().replace("\\", "/");
            files.push(FileRow::new(relative_display, tags_vec));
        }

        files.sort_by(|a, b| a.path.cmp(&b.path));

        let facets = facet_counts
            .into_iter()
            .map(|(name, count)| FacetRow::new(name, count))
            .collect();

        Ok(Self {
            context_label: Some(root.display().to_string()),
            initial_query: String::new(),
            facets,
            files,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SearchOutcome {
    pub accepted: bool,
    pub selection: Option<SearchSelection>,
    pub query: String,
}

#[derive(Debug, Clone)]
pub enum SearchSelection {
    Facet(FacetRow),
    File(FileRow),
}

impl SearchOutcome {
    #[must_use]
    pub fn selected_file(&self) -> Option<&FileRow> {
        match self.selection {
            Some(SearchSelection::File(ref file)) => Some(file),
            _ => None,
        }
    }

    #[must_use]
    pub fn selected_facet(&self) -> Option<&FacetRow> {
        match self.selection {
            Some(SearchSelection::Facet(ref facet)) => Some(facet),
            _ => None,
        }
    }
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
