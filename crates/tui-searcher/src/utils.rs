use frizbee::Config;
use frizbee::match_indices;
use ratatui::widgets::{Cell, Row};

use crate::types::{FileRow, TagRow, highlight_cell};

/// Create an Option containing references to the needle string and Config for highlighting.
/// Accepts an owned pair elsewhere and the caller should invoke `.as_ref().map(|(s,c)| (s.as_str(), c))`.
pub fn highlight_for_refs(needle: &str, config: &Config, text: &str) -> Option<Vec<usize>> {
    match_indices(needle, text, config).map(|m| m.indices)
}

pub fn build_tag_rows<'a>(
    filtered_tags: &'a [usize],
    tag_scores: &'a [u16],
    tags: &'a [TagRow],
    highlight_state: Option<(&'a str, &'a Config)>,
) -> Vec<Row<'a>> {
    filtered_tags
        .iter()
        .enumerate()
        .map(|(idx, &actual_index)| {
            let tag = &tags[actual_index];
            let score = tag_scores.get(idx).copied().unwrap_or_default();
            let highlight = highlight_state
                .and_then(|(needle, config)| highlight_for_refs(needle, config, &tag.name));
            Row::new([
                highlight_cell(&tag.name, highlight),
                Cell::from(tag.count.to_string()),
                Cell::from(score.to_string()),
            ])
        })
        .collect()
}

pub fn build_file_rows<'a>(
    filtered_files: &'a [usize],
    file_scores: &'a [u16],
    files: &'a [FileRow],
    highlight_state: Option<(&'a str, &'a Config)>,
) -> Vec<Row<'a>> {
    filtered_files
        .iter()
        .enumerate()
        .map(|(idx, &actual_index)| {
            let entry = &files[actual_index];
            let score = file_scores.get(idx).copied().unwrap_or_default();
            let path_highlight = highlight_state
                .and_then(|(needle, config)| highlight_for_refs(needle, config, &entry.path));
            let tag_highlight = highlight_state.and_then(|(needle, config)| {
                highlight_for_refs(needle, config, &entry.display_tags)
            });
            Row::new([
                highlight_cell(&entry.path, path_highlight),
                highlight_cell(&entry.display_tags, tag_highlight),
                Cell::from(score.to_string()),
            ])
        })
        .collect()
}
