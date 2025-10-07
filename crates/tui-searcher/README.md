# tui-searcher

A small, configurable TUI searcher (fzf-like) with filesystem search helpers.

Features
- Interactive TUI built on `ratatui`.
- Uses `frizbee` fuzzy matching for typo-tolerant search.
- Builder-style API to configure prompts, column headers and widths.

Quick example

```rust
use tui_searcher::{FacetRow, FileRow, SearchData, SearchMode, Searcher, UiConfig};

let data = SearchData::new()
    .with_facets(vec![FacetRow::new("backend", 4)])
    .with_files(vec![FileRow::new("src/lib.rs".into(), vec!["backend".into()])]);

let outcome = Searcher::new(data)
    .with_ui_config(UiConfig::tags_and_files())
    .with_input_title("Search repo")
    .with_headers_for(SearchMode::Facets, vec!["Tag", "Count", "Score"])
    .run()?;

if let Some(selection) = outcome.selection() {
    println!("Selection: {selection:?}");
}
```

Filesystem search convenience:

```rust
use tui_searcher::{SearchSelection, Searcher};

let outcome = Searcher::filesystem_current_dir()?
    .with_input_title("Files")
    .run()?;

if let Some(SearchSelection::File(file)) = outcome.selection() {
    println!("Selected: {}", file.path);
}
```

Run the example

```bash
cargo run -p tui_searcher --example demo
cargo run -p tui_searcher --example filesystem
```

Notes
- This crate is meant to be a reusable component. You can integrate it into your own CLIs and customize headers/column widths using the builder API.
- The underlying matching behavior is provided by `frizbee`.
