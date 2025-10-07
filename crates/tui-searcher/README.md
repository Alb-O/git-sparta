# tui-searcher

A small, configurable TUI fuzzy finder inspired by `fzf`.

## Features
- Interactive TUI built on `ratatui`.
- Uses `frizbee` fuzzy matching for typo-tolerant search.
- Builder-style API to configure prompts, column headers and widths.
- Ready-to-use filesystem scanner (`Searcher::filesystem`) that walks directories recursively.
- Rich outcome information including which entry was selected and the final query string.

## Quick example

```rust
use tui_searcher::{SearchData, SearchMode, Searcher, UiConfig};

let data = SearchData::from_filesystem(".")?;
let outcome = Searcher::new(data)
    .with_ui_config(UiConfig::tags_and_files())
    .with_start_mode(SearchMode::Files)
    .run()?;

if let Some(file) = outcome.selected_file() {
    println!("Selected file: {}", file.path);
}
```

## Run the examples

```bash
cargo run -p tui_searcher --example demo
cargo run -p tui_searcher --example filesystem -- /path/to/project
```

## Notes
- This crate is meant to be a reusable component. You can integrate it into your own CLIs and customize headers/column widths using the builder API.
- The underlying matching behavior is provided by `frizbee`.
