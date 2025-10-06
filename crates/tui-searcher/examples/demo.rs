use ratatui::layout::Constraint;
use tui_searcher::{FacetRow, FileRow, SearchData, SearchMode, Searcher, UiConfig};

fn main() -> anyhow::Result<()> {
    // Build sample data
    let facets = vec![
        FacetRow {
            name: "frontend".into(),
            count: 3,
        },
        FacetRow {
            name: "backend".into(),
            count: 2,
        },
    ];
    let files = vec![
        FileRow::new("src/main.rs".into(), vec!["frontend".into()]),
        FileRow::new("src/lib.rs".into(), vec!["backend".into()]),
    ];

    let data = SearchData {
        repo_display: "example/repo".into(),
        user_filter: "".into(),
        facets,
        files,
    };

    // Configure a fzf-like searcher
    let searcher = Searcher::new(data)
        .with_ui_config(UiConfig::tags_and_files())
        .with_input_title("Search repo")
        .with_headers_for(SearchMode::Facets, vec!["Tag", "Count", "Score"])
        .with_headers_for(SearchMode::Files, vec!["Path", "Tags", "Score"])
        .with_widths_for(
            SearchMode::Facets,
            vec![
                Constraint::Percentage(60),
                Constraint::Length(8),
                Constraint::Length(8),
            ],
        )
        .with_widths_for(
            SearchMode::Files,
            vec![
                Constraint::Percentage(60),
                Constraint::Percentage(30),
                Constraint::Length(8),
            ],
        );

    let outcome = searcher.run()?;
    println!("Accepted? {}", outcome.accepted);
    Ok(())
}
