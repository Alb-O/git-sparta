use std::path::PathBuf;

use tui_searcher::{SearchSelection, Searcher};

fn main() -> anyhow::Result<()> {
    let root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?);

    let display = root.display().to_string();
    let outcome = Searcher::filesystem(&root)?
        .with_input_title(format!("search: {display}"))
        .run()?;

    if outcome.is_accepted() {
        match outcome.selection() {
            Some(SearchSelection::File(file)) => println!("{}", file.path),
            _ => println!("No file selected"),
        }
    } else {
        println!("Search cancelled");
    }

    Ok(())
}
