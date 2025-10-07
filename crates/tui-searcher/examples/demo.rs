use tui_searcher::{FacetRow, FileRow, SearchData, SearchSelection, Searcher};

fn main() -> anyhow::Result<()> {
    // Build sample data
    let data = SearchData::new()
        .with_facets(vec![
            FacetRow::new("frontend", 3),
            FacetRow::new("backend", 2),
        ])
        .with_files(vec![
            FileRow::new("src/main.rs".into(), vec!["frontend".into()]),
            FileRow::new("src/lib.rs".into(), vec!["backend".into()]),
        ]);

    // Minimal searcher configuration with prompt
    let searcher = Searcher::new(data).with_input_title("workspace-prototype");
    let outcome = searcher.run()?;
    match (outcome.is_accepted(), outcome.selection()) {
        (true, Some(SearchSelection::Facet(facet))) => {
            println!("Accepted facet: {}", facet.name);
        }
        (true, Some(SearchSelection::File(file))) => {
            println!("Accepted file: {}", file.path);
        }
        (true, None) => println!("Accepted with no selection"),
        (false, _) => println!("Search cancelled"),
    }
    Ok(())
}
