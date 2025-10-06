use ratatui::layout::Constraint;
use tui_searcher::{FileRow, SearchData, Searcher, TagRow};

fn main() -> anyhow::Result<()> {
    // Build sample data
    let tags = vec![
        TagRow {
            name: "frontend".into(),
            count: 3,
        },
        TagRow {
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
        tags,
        files,
    };

    // Configure a fzf-like searcher
    let searcher = Searcher::new(data)
        .with_input_title("Search repo")
        .with_tag_headers(vec!["Tag", "Count", "Score"])
        .with_file_headers(vec!["Path", "Tags", "Score"])
        .with_tag_widths(vec![
            Constraint::Percentage(60),
            Constraint::Length(8),
            Constraint::Length(8),
        ])
        .with_file_widths(vec![
            Constraint::Percentage(60),
            Constraint::Percentage(30),
            Constraint::Length(8),
        ]);

    let outcome = searcher.run()?;
    println!("Accepted? {}", outcome.accepted);
    Ok(())
}
