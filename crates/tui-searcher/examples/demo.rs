use tui_searcher::{FacetRow, FileRow, SearchData, Searcher};

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

    // Minimal searcher configuration with prompt
    let searcher = Searcher::new(data).with_input_title("workspace-prototype");
    let outcome = searcher.run()?;
    println!("Accepted? {}", outcome.accepted);
    Ok(())
}
