use std::mem;
use std::path::{Path, PathBuf};

use anyhow::Result;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Cell;
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

    #[must_use]
    pub fn from_path(path: impl Into<String>) -> Self {
        Self::new(path.into(), Vec::new())
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

#[derive(Debug, Clone, Default)]
pub struct SearchData {
    pub(crate) facets: Vec<FacetRow>,
    pub(crate) files: Vec<FileRow>,
}

impl SearchData {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_facets(mut self, facets: impl IntoIterator<Item = FacetRow>) -> Self {
        self.facets = facets.into_iter().collect();
        self
    }

    #[must_use]
    pub fn with_files(mut self, files: impl IntoIterator<Item = FileRow>) -> Self {
        self.files = files.into_iter().collect();
        self
    }

    pub fn push_facet(&mut self, facet: FacetRow) {
        self.facets.push(facet);
    }

    pub fn push_file(&mut self, file: FileRow) {
        self.files.push(file);
    }

    #[must_use]
    pub fn facets(&self) -> &[FacetRow] {
        &self.facets
    }

    #[must_use]
    pub fn files(&self) -> &[FileRow] {
        &self.files
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.facets.is_empty() && self.files.is_empty()
    }

    pub fn filesystem(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let mut files = Vec::new();

        for entry in WalkDir::new(root) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let mut display: PathBuf = if let Ok(relative) = entry.path().strip_prefix(root) {
                relative.to_path_buf()
            } else {
                entry.path().to_path_buf()
            };
            if display.as_os_str().is_empty() {
                if let Some(file_name) = entry.path().file_name() {
                    display = PathBuf::from(file_name);
                }
            }
            let display = display.to_string_lossy().replace("\\", "/");
            files.push(FileRow::from_path(display));
        }

        files.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(Self::default().with_files(files))
    }

    pub fn filesystem_current_dir() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        Self::filesystem(cwd)
    }
}

#[derive(Debug, Clone)]
pub enum SearchSelection {
    Facet(FacetRow),
    File(FileRow),
}

impl SearchSelection {
    #[must_use]
    pub fn as_facet(&self) -> Option<&FacetRow> {
        match self {
            SearchSelection::Facet(facet) => Some(facet),
            SearchSelection::File(_) => None,
        }
    }

    #[must_use]
    pub fn as_file(&self) -> Option<&FileRow> {
        match self {
            SearchSelection::Facet(_) => None,
            SearchSelection::File(file) => Some(file),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchOutcome {
    accepted: bool,
    selection: Option<SearchSelection>,
}

impl SearchOutcome {
    #[must_use]
    pub fn accepted(selection: Option<SearchSelection>) -> Self {
        Self {
            accepted: true,
            selection,
        }
    }

    #[must_use]
    pub fn cancelled() -> Self {
        Self {
            accepted: false,
            selection: None,
        }
    }

    #[must_use]
    pub fn is_accepted(&self) -> bool {
        self.accepted
    }

    #[must_use]
    pub fn selection(&self) -> Option<&SearchSelection> {
        self.selection.as_ref()
    }

    #[must_use]
    pub fn into_selection(self) -> Option<SearchSelection> {
        self.selection
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
