use std::env;
use std::path::PathBuf;

use tui_searcher::{SearchSelection, Searcher};

fn main() -> anyhow::Result<()> {
    let root = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("failed to resolve current dir"));

    let title = root
        .file_name()
        .and_then(|name| name.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| root.to_string_lossy().into_owned());

    let searcher = Searcher::filesystem(&root)?.with_input_title(title);

    let outcome = searcher.run()?;

    if !outcome.accepted {
        println!("Search cancelled (query: '{}')", outcome.query);
        return Ok(());
    }

    match outcome.selection {
        Some(SearchSelection::File(file)) => println!("{}", file.path),
        Some(SearchSelection::Facet(facet)) => println!("Facet: {}", facet.name),
        None => println!("No selection"),
    }

    Ok(())
}
