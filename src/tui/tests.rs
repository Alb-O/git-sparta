use crate::tui::app::App;
use crate::tui::types::{FileRow, SearchData, TagRow};

fn sample_data() -> SearchData {
    SearchData {
        repo_display: "example/repo".to_string(),
        user_filter: "".to_string(),
        tags: vec![
            TagRow {
                name: "docs".to_string(),
                count: 4,
            },
            TagRow {
                name: "tests".to_string(),
                count: 2,
            },
        ],
        files: vec![FileRow::new("ui.rs".to_string(), vec!["ui".to_string()])],
    }
}

#[test]
fn config_allows_typos_for_short_queries() {
    let app = App::new(sample_data());
    let config = app.config_for_query("uo");
    // Prefilter is disabled by default; max_typos will be None so that
    // Smith-Waterman alignment always runs and substitution typos are handled.
    assert!(config.max_typos.is_none());
}

#[test]
fn subsequence_query_matches_tags() {
    let mut app = App::new(sample_data());
    app.input = "cs".to_string();
    app.refresh_tags();
    let names: Vec<&str> = app
        .filtered_tags
        .iter()
        .map(|&idx| app.data.tags[idx].name.as_str())
        .collect();
    assert!(names.contains(&"docs"));
}

#[test]
fn substitution_query_matches_files() {
    let mut app = App::new(sample_data());
    app.mode = crate::tui::types::SearchMode::Files;
    app.input = "uo".to_string();
    app.refresh_files();
    let paths: Vec<&str> = app
        .filtered_files
        .iter()
        .map(|&idx| app.data.files[idx].path.as_str())
        .collect();
    assert!(paths.contains(&"ui.rs"));
}

#[test]
fn neighbor_substitution_matches_frontend() {
    // Verify a single-character neighbor substitution (m <-> n) is accepted
    // when prefilter is disabled and Smith-Waterman scoring runs.
    let mut data = sample_data();
    data.files = vec![FileRow::new("frontend".to_string(), vec![])]
        .into_iter()
        .chain(data.files.into_iter())
        .collect();

    let mut app = App::new(data);
    app.mode = crate::tui::types::SearchMode::Files;
    app.input = "fromt".to_string();
    app.refresh_files();
    let paths: Vec<&str> = app
        .filtered_files
        .iter()
        .map(|&idx| app.data.files[idx].path.as_str())
        .collect();

    assert!(
        paths.contains(&"frontend"),
        "expected 'frontend' to match 'fromt'"
    );
}
