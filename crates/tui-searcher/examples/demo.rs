use ratatui::layout::Constraint;
use tui_searcher::{FileRow, ListRow, ModeTexts, SearchData, Searcher};

fn main() -> anyhow::Result<()> {
    // Build sample data
    let primary_rows = vec![
        ListRow {
            label: "frontend".into(),
            count: 3,
        },
        ListRow {
            label: "backend".into(),
            count: 2,
        },
    ];
    let files = vec![
        FileRow::new("src/main.rs".into(), vec!["frontend".into()]),
        FileRow::new("src/lib.rs".into(), vec!["backend".into()]),
    ];

    let data = SearchData {
        repo_display: "example/repo".into(),
        context_value: "".into(),
        primary_rows,
        files,
    };

    let primary_mode = ModeTexts {
        title: "Category search".into(),
        hint: "Type to filter categories. Press Tab to inspect files.".into(),
        table_title: "Matching categories".into(),
        count_label: "Categories".into(),
        detail_label: None,
    };
    let secondary_mode = ModeTexts {
        title: "File inspection".into(),
        hint: "Type to search files or labels. Press Tab to return to categories.".into(),
        table_title: "Matching files".into(),
        count_label: "Files".into(),
        detail_label: Some("Labels".into()),
    };

    // Configure a fzf-like searcher
    let searcher = Searcher::new(data)
        .with_input_title("Search repo")
        .with_primary_headers(vec!["Category", "Count", "Score"])
        .with_secondary_headers(vec!["Path", "Labels", "Score"])
        .with_primary_widths(vec![
            Constraint::Percentage(60),
            Constraint::Length(8),
            Constraint::Length(8),
        ])
        .with_secondary_widths(vec![
            Constraint::Percentage(60),
            Constraint::Percentage(30),
            Constraint::Length(8),
        ])
        .with_branding("tui-searcher demo")
        .with_context_label("Active filter")
        .with_primary_mode_texts(primary_mode)
        .with_secondary_mode_texts(secondary_mode)
        .with_detail_title("File details")
        .with_detail_empty_message("No file selected")
        .with_no_results_message("No matches found");

    let outcome = searcher.run()?;
    println!("Accepted? {}", outcome.accepted);
    Ok(())
}
